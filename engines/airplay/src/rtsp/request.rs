//! The request model of the AirPlay control channel.
//!
//! WHY: an AirPlay sender speaks two dialects on the same TCP port: classic
//! RTSP (`OPTIONS`, `ANNOUNCE`, `SETUP`, …) and an HTTP-like dialect used by
//! the AirPlay 2 handshake (`GET /info`, `POST /pair-setup`, `PUT /photo`).
//! Both are parsed into one typed request so that the router never has to
//! look at raw strings.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_airplay::rtsp::RtspRequest;
//!
//! let request = RtspRequest::parse(
//!     "OPTIONS rtsp://10.0.0.1/live RTSP/1.0\r\nCSeq: 2\r\n\r\n".as_bytes(),
//!     4096,
//! )
//! .expect("valid request");
//! assert_eq!(request.method.as_str(), "OPTIONS");
//! assert_eq!(request.cseq(), Some(2));
//! ```

use crate::error::AirPlayError;

/// The method of an AirPlay request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RtspMethod {
    /// `OPTIONS` — the sender asks which methods are supported.
    Options,
    /// `ANNOUNCE` — the sender describes the stream with an SDP body.
    Announce,
    /// `SETUP` — the sender allocates a media channel.
    Setup,
    /// `RECORD` — the sender starts delivering media.
    Record,
    /// `TEARDOWN` — the sender releases a channel or the whole session.
    Teardown,
    /// `GET_PARAMETER` — the sender queries receiver state (volume).
    GetParameter,
    /// `SET_PARAMETER` — the sender pushes state (volume, metadata, artwork).
    SetParameter,
    /// `FLUSH` — the sender asks the receiver to drop buffered media.
    Flush,
    /// `PAUSE` — the sender suspends media delivery.
    Pause,
    /// `GET` — an HTTP-like read request (`/info`, `/playback-info`, …).
    Get,
    /// `POST` — an HTTP-like write request (`/pair-setup`, `/play`, …).
    Post,
    /// `PUT` — an HTTP-like upload (`/photo`).
    Put,
    /// `DELETE` — an HTTP-like delete (`/photo`).
    Delete,
    /// `SETRATEANCHORTIME` — the AirPlay 2 buffered-audio anchor verb.
    SetRateAnchorTime,
    /// `SETPEERS` — the AirPlay 2 buffered-audio peer verb.
    SetPeers,
    /// `FLUSHBUFFERED` — the AirPlay 2 buffered-audio flush verb.
    FlushBuffered,
    /// Any method the engine does not know.
    Other(String),
}

impl RtspMethod {
    /// Parses a method token. Unknown methods are kept, never rejected.
    #[must_use]
    pub fn parse(raw: &str) -> Self {
        match raw {
            "OPTIONS" => Self::Options,
            "ANNOUNCE" => Self::Announce,
            "SETUP" => Self::Setup,
            "RECORD" => Self::Record,
            "TEARDOWN" => Self::Teardown,
            "GET_PARAMETER" => Self::GetParameter,
            "SET_PARAMETER" => Self::SetParameter,
            "FLUSH" => Self::Flush,
            "PAUSE" => Self::Pause,
            "GET" => Self::Get,
            "POST" => Self::Post,
            "PUT" => Self::Put,
            "DELETE" => Self::Delete,
            "SETRATEANCHORTIME" | "SETRATEANCHORTIM" => Self::SetRateAnchorTime,
            "SETPEERS" | "SETPEERSX" => Self::SetPeers,
            "FLUSHBUFFERED" => Self::FlushBuffered,
            other => Self::Other(other.to_string()),
        }
    }

    /// The wire token of the method.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Options => "OPTIONS",
            Self::Announce => "ANNOUNCE",
            Self::Setup => "SETUP",
            Self::Record => "RECORD",
            Self::Teardown => "TEARDOWN",
            Self::GetParameter => "GET_PARAMETER",
            Self::SetParameter => "SET_PARAMETER",
            Self::Flush => "FLUSH",
            Self::Pause => "PAUSE",
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
            Self::SetRateAnchorTime => "SETRATEANCHORTIME",
            Self::SetPeers => "SETPEERS",
            Self::FlushBuffered => "FLUSHBUFFERED",
            Self::Other(other) => other,
        }
    }
}

/// One parsed AirPlay request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RtspRequest {
    /// The request method.
    pub method: RtspMethod,
    /// The request URI, including the query string.
    pub uri: String,
    /// The protocol token, e.g. `RTSP/1.0` or `HTTP/1.1`.
    pub protocol: String,
    /// The headers in the order they arrived.
    pub headers: Vec<(String, String)>,
    /// The request body.
    pub body: Vec<u8>,
}

impl RtspRequest {
    /// Parses a complete request from a byte buffer.
    ///
    /// # Errors
    /// Returns [`AirPlayError::Parse`] when the request line or a header is
    /// malformed, and [`AirPlayError::LimitExceeded`] when the message is
    /// longer than `max_message_bytes`.
    pub fn parse(input: &[u8], max_message_bytes: usize) -> Result<Self, AirPlayError> {
        if input.len() > max_message_bytes {
            return Err(AirPlayError::limit_exceeded(format!(
                "message of {} bytes exceeds the {max_message_bytes} byte limit",
                input.len()
            )));
        }
        let split = find_header_end(input)?;
        let head = String::from_utf8(input[..split].to_vec())
            .map_err(|_| AirPlayError::parse("request head is not valid UTF-8"))?;
        let body = input[split..].to_vec();

        let mut lines = head.split("\r\n").filter(|line| !line.is_empty());
        let request_line = lines
            .next()
            .ok_or_else(|| AirPlayError::parse("empty request line"))?;
        let mut parts = request_line.split(' ').filter(|part| !part.is_empty());
        let method = parts
            .next()
            .ok_or_else(|| AirPlayError::parse("request line without a method"))?;
        let uri = parts
            .next()
            .ok_or_else(|| AirPlayError::parse("request line without a URI"))?;
        let protocol = parts.next().unwrap_or("RTSP/1.0");

        let mut headers = Vec::new();
        for line in lines {
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

        Ok(Self {
            method: RtspMethod::parse(method),
            uri: uri.to_string(),
            protocol: protocol.to_string(),
            headers,
            body,
        })
    }

    /// Looks a header up, ignoring the case of its name.
    #[must_use]
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    /// The `CSeq` header, if it carries a number.
    #[must_use]
    pub fn cseq(&self) -> Option<u32> {
        self.header("CSeq")?.trim().parse().ok()
    }

    /// The `Content-Type` header in lower case, if present.
    #[must_use]
    pub fn content_type(&self) -> Option<String> {
        self.header("Content-Type")
            .map(|value| value.trim().to_ascii_lowercase())
    }

    /// The URI path, without the query string.
    #[must_use]
    pub fn path(&self) -> &str {
        match self.uri.split_once('?') {
            Some((path, _)) => path,
            None => self.uri.as_str(),
        }
    }

    /// The URI query string, without the leading `?`.
    #[must_use]
    pub fn query(&self) -> &str {
        match self.uri.split_once('?') {
            Some((_, query)) => query,
            None => "",
        }
    }

    /// One query parameter of the request URI.
    #[must_use]
    pub fn query_param(&self, key: &str) -> Option<&str> {
        self.query()
            .split('&')
            .filter(|pair| !pair.is_empty())
            .find_map(|pair| pair.split_once('='))
            .filter(|(name, _)| *name == key)
            .map(|(_, value)| value)
    }

    /// Whether the body is an Apple binary property list.
    #[must_use]
    pub fn is_binary_plist(&self) -> bool {
        self.body.starts_with(b"bplist00")
    }

    /// The body decoded as UTF-8 text, if it is valid UTF-8.
    #[must_use]
    pub fn body_text(&self) -> Option<String> {
        String::from_utf8(self.body.clone()).ok()
    }

    /// The response protocol token: an HTTP request stays HTTP, everything else
    /// answers in RTSP.
    #[must_use]
    pub fn response_protocol(&self) -> &str {
        if self.protocol.starts_with("HTTP/") {
            &self.protocol
        } else {
            "RTSP/1.0"
        }
    }
}

/// Finds the offset just after the `\r\n\r\n` that ends the request head.
///
/// # Errors
/// Returns [`AirPlayError::Parse`] when the terminator is missing.
fn find_header_end(input: &[u8]) -> Result<usize, AirPlayError> {
    input
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|index| index + 4)
        .ok_or_else(|| AirPlayError::parse("request has no header terminator"))
}

#[cfg(test)]
mod tests;
