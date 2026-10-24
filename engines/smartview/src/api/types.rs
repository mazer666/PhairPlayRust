//! The types of the HTTP API of the Smart View receiver.
//!
//! WHY: the request, the answer and their limits are the vocabulary of the
//! API. They live in one module, so that the reader, the routes and the
//! server cannot drift apart: a route that forgets a limit is a route that
//! reads without a bound.
//!
//! HOW: every limit of the reader is a field of [`ApiLimits`] and not a
//! constant in the code — the engine hands the limits to the server, and
//! the tests hand smaller ones to the same code.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_smartview::api::{ApiLimits, Method};
//!
//! let limits = ApiLimits::default();
//! assert!(limits.max_header_bytes > 0);
//! assert_eq!(Method::parse("GET"), Method::Get);
//! ```

use std::net::SocketAddr;
use std::time::Duration;

use crate::api::API_ROOT_PATH;

/// The limits of the HTTP reader of the API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiLimits {
    /// The maximum number of header lines of one request.
    pub max_header_count: usize,
    /// The maximum length of a single header line, in bytes.
    pub max_header_bytes: usize,
    /// The maximum length of the body of one request, in bytes.
    pub max_body_bytes: usize,
    /// The maximum length of one frame of the channel, in bytes.
    pub max_frame_bytes: usize,
    /// The deadline of every read of a request.
    pub read_timeout: Duration,
    /// The deadline of every read of the channel.
    pub idle_timeout: Duration,
}

impl Default for ApiLimits {
    fn default() -> Self {
        Self {
            max_header_count: 64,
            max_header_bytes: 8 * 1024,
            max_body_bytes: 64 * 1024,
            max_frame_bytes: 1024 * 1024,
            read_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_secs(30),
        }
    }
}

/// The method of a request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Method {
    /// `GET`.
    Get,
    /// `HEAD`.
    Head,
    /// `POST`.
    Post,
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
            Self::Other(name) => name,
        }
    }

    /// Parses a method.
    #[must_use]
    pub fn parse(raw: &str) -> Self {
        match raw {
            "GET" => Self::Get,
            "HEAD" => Self::Head,
            "POST" => Self::Post,
            other => Self::Other(other.to_string()),
        }
    }
}

/// One request of the API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiRequest {
    /// The method of the request.
    pub method: Method,
    /// The path of the request, without the query.
    pub path: String,
    /// The query of the request, if it carried one.
    pub query: Option<String>,
    /// The headers, with lower-case names.
    pub headers: Vec<(String, String)>,
    /// The body of the request.
    pub body: Vec<u8>,
    /// The address of the sender.
    pub peer: SocketAddr,
}

impl ApiRequest {
    /// The value of a header.
    #[must_use]
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }
}

/// The answer to a request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiResponse {
    /// The status code.
    pub status: u16,
    /// The reason phrase.
    pub reason: &'static str,
    /// The content type.
    pub content_type: &'static str,
    /// The body.
    pub body: Vec<u8>,
    /// The headers that are not part of every answer.
    pub extra_headers: Vec<(String, String)>,
}

impl ApiResponse {
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

    /// A response with a JSON body.
    #[must_use]
    pub fn json(status: u16, reason: &'static str, body: &serde_json::Value) -> Self {
        let text = body.to_string();
        Self::new(
            status,
            reason,
            "application/json; charset=\"utf-8\"",
            text.into_bytes(),
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

    /// The answer to a request that the API does not know.
    #[must_use]
    pub fn not_found(path: &str) -> Self {
        Self::json(
            404,
            "Not Found",
            &serde_json::json!({
                "error": "unknown path",
                "path": path,
                "api": API_ROOT_PATH,
            }),
        )
    }

    /// The answer to a request that the API refuses.
    #[must_use]
    pub fn refused(reason: &str) -> Self {
        Self::json(409, "Conflict", &serde_json::json!({ "error": reason }))
    }

    /// Adds a header.
    #[must_use]
    pub fn with_header(mut self, name: &str, value: &str) -> Self {
        self.extra_headers
            .push((name.to_string(), value.to_string()));
        self
    }
}
