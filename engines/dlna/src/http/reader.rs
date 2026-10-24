//! The reader of the HTTP requests.
//!
//! WHY: a control point is an arbitrary device on the LAN. It may send a
//! request line of ten megabytes, ten thousand headers, or a body that
//! claims to be a gigabyte. Every one of those is a denial of service if
//! the server believes the length.
//!
//! HOW: the reader never materialises a line before it has checked its
//! length — it fills a buffer, looks for the newline, and gives up at the
//! limit. The body length is checked against the limit *before* a single
//! byte is read, and a body that is too large is rejected instead of being
//! truncated. A truncated body would be a request the control point never
//! sent.

use std::net::SocketAddr;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::net::TcpStream;

use super::{HttpError, HttpLimits, HttpRequest, Method};

/// The outcome of reading one line.
#[derive(Debug, Clone, PartialEq, Eq)]
struct BoundedLine {
    value: Option<String>,
    too_long: bool,
}

/// Reads one request from a client connection.
///
/// # Errors
/// Returns [`HttpError`] when the request violates a limit or is malformed.
pub async fn read_request(
    reader: &mut BufReader<TcpStream>,
    limits: &HttpLimits,
    peer: SocketAddr,
) -> Result<HttpRequest, HttpError> {
    let request_line =
        read_bounded_line(reader, limits.max_header_bytes, limits.read_timeout).await?;
    if request_line.too_long {
        return Err(HttpError::LineTooLong {
            limit: limits.max_header_bytes,
        });
    }
    let Some(request_line) = request_line.value else {
        return Err(HttpError::Malformed(
            "the client closed the connection before sending a request".to_string(),
        ));
    };

    let mut parts = request_line.split(' ');
    let method = parts.next().unwrap_or_default().to_string();
    let target = parts.next().unwrap_or_default().to_string();
    let version = parts.next().unwrap_or_default().to_string();
    if method.is_empty() || target.is_empty() {
        return Err(HttpError::Malformed(
            "the request line is incomplete".to_string(),
        ));
    }
    if !version.starts_with("HTTP/1.") {
        return Err(HttpError::Malformed(
            "the request line carries an unknown version".to_string(),
        ));
    }

    let (path, query) = match target.split_once('?') {
        Some((path, query)) => (path.to_string(), Some(query.to_string())),
        None => (target.clone(), None),
    };
    if path.len() > limits.max_header_bytes {
        return Err(HttpError::LineTooLong {
            limit: limits.max_header_bytes,
        });
    }

    let mut headers: Vec<(String, String)> = Vec::new();
    let mut total = 0usize;
    loop {
        let line = read_bounded_line(reader, limits.max_header_bytes, limits.read_timeout).await?;
        if line.too_long {
            return Err(HttpError::LineTooLong {
                limit: limits.max_header_bytes,
            });
        }
        let Some(line) = line.value else {
            return Err(HttpError::Malformed(
                "the header block is not terminated".to_string(),
            ));
        };
        if line.is_empty() {
            break;
        }
        if headers.len() >= limits.max_header_count {
            return Err(HttpError::TooManyHeaders {
                limit: limits.max_header_count,
            });
        }
        total += line.len();
        if total > limits.max_header_bytes * limits.max_header_count {
            return Err(HttpError::HeadersTooLarge {
                limit: limits.max_header_bytes * limits.max_header_count,
            });
        }
        let Some((key, value)) = line.split_once(':') else {
            return Err(HttpError::Malformed(
                "a header line has no colon".to_string(),
            ));
        };
        headers.push((key.trim().to_ascii_lowercase(), value.trim().to_string()));
    }

    let method = match method.as_str() {
        "GET" => Method::Get,
        "HEAD" => Method::Head,
        "POST" => Method::Post,
        "SUBSCRIBE" => Method::Subscribe,
        "UNSUBSCRIBE" => Method::Unsubscribe,
        other => Method::Other(other.to_string()),
    };

    let body = read_body(reader, &headers, &method, limits).await?;

    Ok(HttpRequest {
        method,
        path,
        query,
        headers,
        body,
        peer,
    })
}

/// Reads the body of a request.
async fn read_body(
    reader: &mut BufReader<TcpStream>,
    headers: &[(String, String)],
    method: &Method,
    limits: &HttpLimits,
) -> Result<Vec<u8>, HttpError> {
    let transfer_encoding = headers
        .iter()
        .find(|(key, _)| key == "transfer-encoding")
        .map(|(_, value)| value.to_ascii_lowercase());
    if let Some(encoding) = transfer_encoding {
        if encoding.contains("chunked") {
            return Err(HttpError::UnsupportedTransferEncoding);
        }
    }
    let Some(length) = headers
        .iter()
        .find(|(key, _)| key == "content-length")
        .map(|(_, value)| value.clone())
    else {
        if matches!(method, Method::Post) {
            return Err(HttpError::LengthRequired);
        }
        return Ok(Vec::new());
    };
    let length: usize = length
        .trim()
        .parse()
        .map_err(|_| HttpError::Malformed("the content length is not a number".to_string()))?;
    if length > limits.max_body_bytes {
        return Err(HttpError::BodyTooLarge {
            limit: limits.max_body_bytes,
        });
    }
    let mut body = vec![0u8; length];
    if length > 0 {
        tokio::time::timeout(limits.read_timeout, reader.read_exact(&mut body))
            .await
            .map_err(|_| HttpError::Timeout)?
            .map_err(|error| HttpError::Malformed(error.to_string()))?;
    }
    Ok(body)
}

/// Reads one line, with a hard limit on its length.
async fn read_bounded_line(
    reader: &mut BufReader<TcpStream>,
    limit: usize,
    timeout: Duration,
) -> Result<BoundedLine, HttpError> {
    let mut line: Vec<u8> = Vec::new();
    loop {
        let available = tokio::time::timeout(timeout, reader.fill_buf())
            .await
            .map_err(|_| HttpError::Timeout)?
            .map_err(|error| HttpError::Malformed(error.to_string()))?;
        if available.is_empty() {
            return Ok(BoundedLine {
                value: if line.is_empty() {
                    None
                } else {
                    Some(String::from_utf8_lossy(&line).to_string())
                },
                too_long: false,
            });
        }
        match available.iter().position(|byte| *byte == b'\n') {
            Some(index) => {
                if line.len() + index > limit {
                    return Ok(BoundedLine {
                        value: None,
                        too_long: true,
                    });
                }
                line.extend_from_slice(&available[..index]);
                reader.consume(index + 1);
                let mut text = String::from_utf8_lossy(&line).to_string();
                if text.ends_with('\r') {
                    text.pop();
                }
                return Ok(BoundedLine {
                    value: Some(text),
                    too_long: false,
                });
            }
            None => {
                if line.len() + available.len() > limit {
                    return Ok(BoundedLine {
                        value: None,
                        too_long: true,
                    });
                }
                let length = available.len();
                line.extend_from_slice(available);
                reader.consume(length);
            }
        }
    }
}
