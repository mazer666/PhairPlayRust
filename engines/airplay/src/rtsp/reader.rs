//! The bounded reader of the AirPlay control channel.
//!
//! WHY: this is the door through which every untrusted byte of the local
//! network enters the receiver. Three properties are non-negotiable here:
//!
//! 1. **Every read has a limit.** A request line, a header line, the number of
//!    headers, the `Content-Length` and the total message size are all
//!    bounded by [`AirPlayLimits`].
//! 2. **Every read has a deadline.** A peer that connects and then says
//!    nothing must not hold the connection slot forever.
//! 3. **No length field may allocate.** The body is only read after the
//!    declared length has been checked against the limit, so a peer that
//!    claims four gigabytes never makes the process allocate four gigabytes.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_airplay::limits::AirPlayLimits;
//! use phairplay_airplay::rtsp::RtspRequestReader;
//! use tokio::io::AsyncWriteExt;
//!
//! # async fn demo() {
//! let mut stream = tokio::io::duplex(1024);
//! let writer = stream.1;
//! tokio::spawn(async move {
//!     let mut writer = writer;
//!     writer.write_all(b"OPTIONS rtsp://x/y RTSP/1.0\r\nCSeq: 1\r\n\r\n").await.ok();
//! });
//!
//! let mut buffer = phairplay_airplay::rtsp::StreamBuffer::new(stream.0);
//! let reader = RtspRequestReader::new(AirPlayLimits::default());
//! let request = reader.read_request(&mut buffer).await.expect("read").expect("request");
//! assert_eq!(request.method.as_str(), "OPTIONS");
//! # }
//! ```

use std::time::Duration;

use tokio::time::Instant;

use tokio::io::{AsyncRead, AsyncReadExt};

use crate::error::AirPlayError;
use crate::limits::AirPlayLimits;
use crate::rtsp::request::{RtspMethod, RtspRequest};

/// The size of one socket read.
const CHUNK_BYTES: usize = 8 * 1024;

/// A byte buffer in front of a stream.
///
/// WHY: the AirPlay control connection changes its *protocol* mid-stream —
/// RTSP text first, then binary interleaved RTP on the same socket. A
/// buffered reader that hides the leftover bytes would lose the first RTP
/// frame. [`StreamBuffer`] therefore keeps the unconsumed bytes visible, so
/// the RTP reader can continue exactly where the RTSP reader stopped.
#[derive(Debug)]
pub struct StreamBuffer<R> {
    inner: R,
    buffer: Vec<u8>,
    start: usize,
}

impl<R> StreamBuffer<R> {
    /// Wraps a stream.
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            buffer: Vec::new(),
            start: 0,
        }
    }

    /// The bytes that have been read but not consumed yet.
    #[must_use]
    pub fn buffered(&self) -> &[u8] {
        &self.buffer[self.start..]
    }

    /// The wrapped stream.
    pub fn inner(&self) -> &R {
        &self.inner
    }

    /// The wrapped stream, mutably.
    pub fn inner_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    /// Unwraps the stream, returning the buffer that is still unread.
    #[must_use]
    pub fn into_parts(self) -> (R, Vec<u8>) {
        let rest = self.buffer[self.start..].to_vec();
        (self.inner, rest)
    }

    fn available(&self) -> usize {
        self.buffer.len() - self.start
    }

    /// Drops `count` bytes from the front of the buffer.
    pub fn consume(&mut self, count: usize) {
        self.start += count;
        if self.start == self.buffer.len() {
            self.buffer.clear();
            self.start = 0;
        }
    }
}

impl<R: AsyncRead + Unpin> StreamBuffer<R> {
    /// Reads more bytes from the stream into the buffer.
    ///
    /// # Errors
    /// Returns [`AirPlayError::Timeout`] when the peer sends nothing before
    /// the deadline and [`AirPlayError::Io`] on a socket failure.
    pub async fn fill_until(&mut self, deadline: Instant) -> Result<usize, AirPlayError> {
        let mut chunk = [0u8; CHUNK_BYTES];
        let read = tokio::time::timeout_at(deadline, self.inner.read(&mut chunk))
            .await
            .map_err(|_| AirPlayError::timeout("peer sent no data"))??;
        if read == 0 {
            return Ok(0);
        }
        self.buffer.extend_from_slice(&chunk[..read]);
        Ok(read)
    }

    /// Reads one line, without its terminator.
    ///
    /// Returns `Ok(None)` at a clean end of stream.
    ///
    /// # Errors
    /// Returns [`AirPlayError::LimitExceeded`] when the line is longer than
    /// `max_len`, [`AirPlayError::Timeout`] when the peer stalls and
    /// [`AirPlayError::Io`] on a socket failure.
    pub async fn read_line(
        &mut self,
        max_len: usize,
        timeout: Duration,
    ) -> Result<Option<String>, AirPlayError> {
        let deadline = Instant::now() + timeout;
        let mut line: Vec<u8> = Vec::new();
        loop {
            if let Some(position) = self.buffer[self.start..].iter().position(|byte| *byte == b'\n') {
                let end = self.start + position;
                line.extend_from_slice(&self.buffer[self.start..end]);
                self.consume(position + 1);
                if line.len() > max_len {
                    return Err(AirPlayError::limit_exceeded(format!(
                        "line of {} bytes exceeds the {max_len} byte limit",
                        line.len()
                    )));
                }
                if line.last() == Some(&b'\r') {
                    line.pop();
                }
                return Ok(Some(String::from_utf8_lossy(&line).into_owned()));
            }
            if self.available() + line.len() > max_len {
                return Err(AirPlayError::limit_exceeded(format!(
                    "line exceeds the {max_len} byte limit"
                )));
            }
            if self.fill_until(deadline).await? == 0 {
                if line.is_empty() {
                    return Ok(None);
                }
                if line.len() > max_len {
                    return Err(AirPlayError::limit_exceeded(format!(
                        "line of {} bytes exceeds the {max_len} byte limit",
                        line.len()
                    )));
                }
                return Ok(Some(String::from_utf8_lossy(&line).into_owned()));
            }
        }
    }

    /// Reads a single byte.
    ///
    /// Returns `Ok(None)` at a clean end of stream.
    ///
    /// # Errors
    /// Returns [`AirPlayError::Timeout`] when the peer stalls and
    /// [`AirPlayError::Io`] on a socket failure.
    pub async fn read_byte(&mut self, timeout: Duration) -> Result<Option<u8>, AirPlayError> {
        if let Some(byte) = self.buffered().first().copied() {
            self.consume(1);
            return Ok(Some(byte));
        }
        let deadline = Instant::now() + timeout;
        let mut chunk = [0u8; 1];
        loop {
            let read = tokio::time::timeout_at(deadline, self.inner.read(&mut chunk))
                .await
                .map_err(|_| AirPlayError::timeout("peer sent no data"))??;
            if read == 0 {
                return Ok(None);
            }
            self.buffer.extend_from_slice(&chunk[..read]);
            if let Some(byte) = self.buffered().first().copied() {
                self.consume(1);
                return Ok(Some(byte));
            }
        }
    }

    /// Reads exactly `len` bytes.
    ///
    /// # Errors
    /// Returns [`AirPlayError::Protocol`] when the peer closes the connection
    /// in the middle of the payload and [`AirPlayError::Timeout`] when it
    /// stalls.
    pub async fn read_exact(
        &mut self,
        len: usize,
        timeout: Duration,
    ) -> Result<Vec<u8>, AirPlayError> {
        let deadline = Instant::now() + timeout;
        while self.available() < len {
            if self.fill_until(deadline).await? == 0 {
                return Err(AirPlayError::protocol(
                    "peer closed the connection in the middle of a message",
                ));
            }
        }
        let out = self.buffer[self.start..self.start + len].to_vec();
        self.consume(len);
        Ok(out)
    }
}

/// The reader of AirPlay requests.
#[derive(Debug, Clone)]
pub struct RtspRequestReader {
    limits: AirPlayLimits,
}

impl RtspRequestReader {
    /// Creates a reader that enforces the given limits.
    #[must_use]
    pub fn new(limits: AirPlayLimits) -> Self {
        Self { limits }
    }

    /// The limits this reader enforces.
    #[must_use]
    pub fn limits(&self) -> &AirPlayLimits {
        &self.limits
    }

    /// Reads one request.
    ///
    /// Returns `Ok(None)` when the peer closed the connection cleanly.
    ///
    /// # Errors
    /// Returns [`AirPlayError::LimitExceeded`] when a limit is reached,
    /// [`AirPlayError::Parse`] when the request is malformed,
    /// [`AirPlayError::Timeout`] when the peer stalls and
    /// [`AirPlayError::Io`] on a socket failure.
    pub async fn read_request<R: AsyncRead + Unpin>(
        &self,
        stream: &mut StreamBuffer<R>,
    ) -> Result<Option<RtspRequest>, AirPlayError> {
        let request_line = loop {
            match stream
                .read_line(self.limits.max_request_line_bytes, self.limits.idle_timeout)
                .await?
            {
                None => return Ok(None),
                Some(line) if line.trim().is_empty() => continue,
                Some(line) => break line,
            }
        };

        let mut parts = request_line.split_whitespace();
        let method = parts
            .next()
            .ok_or_else(|| AirPlayError::parse("request line without a method"))?;
        let uri = parts
            .next()
            .ok_or_else(|| AirPlayError::parse("request line without a URI"))?;
        let protocol = parts.next().unwrap_or("RTSP/1.0");

        let (headers, header_bytes) = self.read_headers(stream).await?;
        let content_length = content_length_of(&headers)?;
        let body_limit = self.body_limit(method, uri);
        if content_length > body_limit {
            return Err(AirPlayError::limit_exceeded(format!(
                "body of {content_length} bytes exceeds the {body_limit} byte limit"
            )));
        }
        // A photo is allowed to be larger than a control message; the effective
        // message limit is therefore the larger of the two bounds.
        let message_limit = self.limits.max_message_bytes.max(body_limit);
        let total = request_line.len() + header_bytes + content_length;
        if total > message_limit {
            return Err(AirPlayError::limit_exceeded(format!(
                "message of {total} bytes exceeds the {message_limit} byte limit"
            )));
        }

        let body = if content_length == 0 {
            Vec::new()
        } else {
            stream
                .read_exact(content_length, self.limits.idle_timeout)
                .await?
        };

        Ok(Some(RtspRequest {
            method: RtspMethod::parse(method),
            uri: uri.to_string(),
            protocol: protocol.to_string(),
            headers,
            body,
        }))
    }

    async fn read_headers<R: AsyncRead + Unpin>(
        &self,
        stream: &mut StreamBuffer<R>,
    ) -> Result<(Vec<(String, String)>, usize), AirPlayError> {
        let mut headers: Vec<(String, String)> = Vec::new();
        let mut bytes = 0usize;
        loop {
            let line = stream
                .read_line(self.limits.max_header_bytes, self.limits.idle_timeout)
                .await?
                .ok_or_else(|| {
                    AirPlayError::protocol("peer closed the connection inside the header block")
                })?;
            if line.is_empty() {
                return Ok((headers, bytes));
            }
            bytes += line.len();
            if bytes > self.limits.max_message_bytes {
                return Err(AirPlayError::limit_exceeded(format!(
                    "header block of {bytes} bytes exceeds the {} byte limit",
                    self.limits.max_message_bytes
                )));
            }
            if headers.len() >= self.limits.max_header_count {
                return Err(AirPlayError::limit_exceeded(format!(
                    "more than {} headers",
                    self.limits.max_header_count
                )));
            }
            let Some((name, value)) = line.split_once(':') else {
                return Err(AirPlayError::parse(format!(
                    "header line without a colon: {line}"
                )));
            };
            let name = name.trim();
            if name.is_empty() {
                return Err(AirPlayError::parse("header with an empty name"));
            }
            headers.push((name.to_string(), value.trim().to_string()));
        }
    }

    fn body_limit(&self, method: &str, uri: &str) -> usize {
        let path = uri.split('?').next().unwrap_or(uri);
        if method.eq_ignore_ascii_case("PUT") && path == crate::photo::PHOTO_PATH {
            self.limits.max_photo_bytes
        } else {
            self.limits.max_message_bytes
        }
    }
}

/// The `Content-Length` of a header block.
///
/// # Errors
/// Returns [`AirPlayError::Parse`] when the header is present but not a
/// number — a malformed length is rejected, never guessed.
fn content_length_of(headers: &[(String, String)]) -> Result<usize, AirPlayError> {
    let Some((_, raw)) = headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("Content-Length"))
    else {
        return Ok(0);
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AirPlayError::parse("empty Content-Length"));
    }
    trimmed
        .parse::<usize>()
        .map_err(|_| AirPlayError::parse(format!("malformed Content-Length `{trimmed}`")))
}

#[cfg(test)]
mod tests;
