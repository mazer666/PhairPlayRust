//! A minimal HTTP/1.1 reader and writer for the UPnP control plane.
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
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_dlna::http::HttpLimits;
//! use std::time::Duration;
//!
//! let limits = HttpLimits {
//!     max_header_count: 64,
//!     max_header_bytes: 8192,
//!     max_body_bytes: 65536,
//!     read_timeout: Duration::from_secs(2),
//! };
//! assert!(limits.max_body_bytes > 0);
//! ```

use std::net::SocketAddr;
use std::time::Duration;

use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

/// The limits of the HTTP reader.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpLimits {
    /// The maximum number of header lines.
    pub max_header_count: usize,
    /// The maximum length of a single header line, in bytes.
    pub max_header_bytes: usize,
    /// The maximum length of the request body, in bytes.
    pub max_body_bytes: usize,
    /// The deadline of every read.
    pub read_timeout: Duration,
}

impl Default for HttpLimits {
    fn default() -> Self {
        Self {
            max_header_count: 64,
            max_header_bytes: 8 * 1024,
            max_body_bytes: 64 * 1024,
            read_timeout: Duration::from_secs(2),
        }
    }
}

/// The HTTP method of a request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Method {
    /// `GET`.
    Get,
    /// `HEAD`.
    Head,
    /// `POST`.
    Post,
    /// `SUBSCRIBE` — the GENA subscription method.
    Subscribe,
    /// `UNSUBSCRIBE` — the GENA cancellation method.
    Unsubscribe,
    /// Any other method, verbatim.
    Other(String),
}

impl Method {
    /// The method as it appears on the wire.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Get => "GET",
            Self::Head => "HEAD",
            Self::Post => "POST",
            Self::Subscribe => "SUBSCRIBE",
            Self::Unsubscribe => "UNSUBSCRIBE",
            Self::Other(name) => name,
        }
    }
}

/// One parsed request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    /// The method.
    pub method: Method,
    /// The path, without the query.
    pub path: String,
    /// The query, if the request carried one.
    pub query: Option<String>,
    /// The headers, with lower-case names.
    pub headers: Vec<(String, String)>,
    /// The body.
    pub body: Vec<u8>,
    /// The address of the client.
    pub peer: SocketAddr,
}

impl HttpRequest {
    /// The value of a header.
    #[must_use]
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    /// The body as text.
    #[must_use]
    pub fn body_text(&self) -> String {
        String::from_utf8_lossy(&self.body).to_string()
    }
}

/// The reasons a request was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpError {
    /// A line was longer than the limit.
    LineTooLong {
        /// The limit that was exceeded.
        limit: usize,
    },
    /// There were more headers than the limit allows.
    TooManyHeaders {
        /// The limit that was exceeded.
        limit: usize,
    },
    /// The header block was larger than the limit.
    HeadersTooLarge {
        /// The limit that was exceeded.
        limit: usize,
    },
    /// The body was larger than the limit.
    BodyTooLarge {
        /// The limit that was exceeded.
        limit: usize,
    },
    /// The request was malformed.
    Malformed(String),
    /// The client did not send anything in time.
    Timeout,
    /// The request used a transfer encoding the receiver does not support.
    UnsupportedTransferEncoding,
    /// The request did not announce a body length.
    LengthRequired,
}

impl HttpError {
    /// The HTTP status the error is answered with.
    #[must_use]
    pub fn status(&self) -> u16 {
        match self {
            Self::LineTooLong { .. } => 414,
            Self::TooManyHeaders { .. } | Self::HeadersTooLarge { .. } => 431,
            Self::BodyTooLarge { .. } => 413,
            Self::Malformed(_) => 400,
            Self::Timeout => 408,
            Self::UnsupportedTransferEncoding => 501,
            Self::LengthRequired => 411,
        }
    }

    /// The HTTP reason phrase the error is answered with.
    #[must_use]
    pub fn reason(&self) -> &'static str {
        match self {
            Self::LineTooLong { .. } => "URI Too Long",
            Self::TooManyHeaders { .. } | Self::HeadersTooLarge { .. } => {
                "Request Header Fields Too Large"
            }
            Self::BodyTooLarge { .. } => "Payload Too Large",
            Self::Malformed(_) => "Bad Request",
            Self::Timeout => "Request Timeout",
            Self::UnsupportedTransferEncoding => "Not Implemented",
            Self::LengthRequired => "Length Required",
        }
    }
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LineTooLong { limit } => {
                write!(f, "a line exceeded the limit of {limit} bytes")
            }
            Self::TooManyHeaders { limit } => {
                write!(f, "the request carried more than {limit} headers")
            }
            Self::HeadersTooLarge { limit } => {
                write!(f, "the header block exceeded the limit of {limit} bytes")
            }
            Self::BodyTooLarge { limit } => {
                write!(f, "the body exceeded the limit of {limit} bytes")
            }
            Self::Malformed(reason) => write!(f, "the request is malformed: {reason}"),
            Self::Timeout => write!(f, "the client did not answer in time"),
            Self::UnsupportedTransferEncoding => {
                write!(f, "the transfer encoding is not supported")
            }
            Self::LengthRequired => write!(f, "the request has no content length"),
        }
    }
}

impl std::error::Error for HttpError {}

mod reader;

pub use reader::read_request;

/// The answer to a request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    /// The status code.
    pub status: u16,
    /// The reason phrase.
    pub reason: &'static str,
    /// The content type.
    pub content_type: &'static str,
    /// The body.
    pub body: Vec<u8>,
    /// Additional headers, e.g. the `SID` of a subscription.
    pub extra_headers: Vec<(String, String)>,
}

impl HttpResponse {
    /// A response with a body.
    #[must_use]
    pub fn new(
        status: u16,
        reason: &'static str,
        content_type: &'static str,
        body: Vec<u8>,
    ) -> Self {
        Self {
            status,
            reason,
            content_type,
            body,
            extra_headers: Vec::new(),
        }
    }

    /// A response with a text body.
    #[must_use]
    pub fn text(status: u16, reason: &'static str, body: &str) -> Self {
        Self::new(
            status,
            reason,
            "text/plain; charset=\"utf-8\"",
            body.as_bytes().to_vec(),
        )
    }

    /// A response with an XML body.
    #[must_use]
    pub fn xml(status: u16, reason: &'static str, body: &str) -> Self {
        Self::new(
            status,
            reason,
            "text/xml; charset=\"utf-8\"",
            body.as_bytes().to_vec(),
        )
    }

    /// Adds a header.
    #[must_use]
    pub fn with_header(mut self, name: &str, value: &str) -> Self {
        self.extra_headers
            .push((name.to_string(), value.to_string()));
        self
    }
}

/// Writes a response to a client connection.
///
/// # Errors
/// Returns the I/O error of the write.
pub async fn write_response(
    stream: &mut TcpStream,
    response: &HttpResponse,
    include_body: bool,
) -> std::io::Result<()> {
    let mut head = format!(
        "HTTP/1.1 {} {}\r\nConnection: close\r\nContent-Type: {}\r\nContent-Length: {}\r\n",
        response.status,
        response.reason,
        response.content_type,
        response.body.len()
    );
    for (name, value) in &response.extra_headers {
        head.push_str(&format!("{name}: {value}\r\n"));
    }
    head.push_str("\r\n");
    stream.write_all(head.as_bytes()).await?;
    if include_body {
        stream.write_all(&response.body).await?;
    }
    stream.flush().await
}

#[cfg(test)]
mod tests;
