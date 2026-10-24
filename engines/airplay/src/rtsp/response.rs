//! The response builder of the AirPlay control channel.
//!
//! WHY: every answer the receiver sends is a *byte-exact* protocol artefact:
//! the `Content-Length` must count bytes, not characters, because the bodies
//! are binary property lists, FairPlay blobs and encrypted payloads. Building
//! the response as a typed value and serialising it in one place removes a
//! whole class of framing bugs.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_airplay::rtsp::RtspResponse;
//!
//! let response = RtspResponse::ok().with_text("volume: 0.000000\r\n");
//! let bytes = response.to_bytes(7);
//! let text = String::from_utf8(bytes).expect("ascii");
//! assert!(text.starts_with("RTSP/1.0 200 OK\r\n"));
//! assert!(text.contains("CSeq: 7\r\n"));
//! ```

/// The `Server` token the AirPlay stack identifies itself with.
pub const SERVER_TOKEN: &str = "AirTunes/220.68";

/// A response of the AirPlay control channel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RtspResponse {
    /// The status code.
    pub status: u16,
    /// The reason phrase.
    pub reason: String,
    /// The protocol token of the status line.
    pub protocol: String,
    /// Extra headers, in the order they are written.
    pub headers: Vec<(String, String)>,
    /// The response body.
    pub body: Vec<u8>,
}

impl RtspResponse {
    /// A `200 OK` response.
    #[must_use]
    pub fn ok() -> Self {
        Self::new(200, "OK")
    }

    /// A response with an explicit status code and reason phrase.
    #[must_use]
    pub fn new(status: u16, reason: &str) -> Self {
        Self {
            status,
            reason: reason.to_string(),
            protocol: "RTSP/1.0".to_string(),
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    /// A `400 Bad Request` response.
    #[must_use]
    pub fn bad_request() -> Self {
        Self::new(400, "Bad Request")
    }

    /// A `404 Not Found` response.
    #[must_use]
    pub fn not_found() -> Self {
        Self::new(404, "Not Found")
    }

    /// A `455 Method Not Valid in This State` response.
    #[must_use]
    pub fn method_not_valid() -> Self {
        Self::new(455, "Method Not Valid in This State")
    }

    /// A `470 Connection Authorization Required` response.
    #[must_use]
    pub fn authorization_required() -> Self {
        Self::new(470, "Connection Authorization Required")
    }

    /// A `501 Not Implemented` response.
    #[must_use]
    pub fn not_implemented() -> Self {
        Self::new(501, "Not Implemented")
    }

    /// A `503 Service Unavailable` response — the answer to a second peer.
    #[must_use]
    pub fn service_unavailable() -> Self {
        Self::new(503, "Service Unavailable")
    }

    /// Sets the protocol token of the status line.
    #[must_use]
    pub fn with_protocol(mut self, protocol: &str) -> Self {
        self.protocol = protocol.to_string();
        self
    }

    /// Adds a header.
    #[must_use]
    pub fn with_header(mut self, name: &str, value: impl Into<String>) -> Self {
        self.headers.push((name.to_string(), value.into()));
        self
    }

    /// Sets the body to raw bytes.
    #[must_use]
    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    /// Sets the body to a UTF-8 text.
    #[must_use]
    pub fn with_text(mut self, body: &str) -> Self {
        self.body = body.as_bytes().to_vec();
        self
    }

    /// Sets the body and the `Content-Type` header.
    #[must_use]
    pub fn with_typed_body(mut self, content_type: &str, body: Vec<u8>) -> Self {
        self.body = body;
        self.headers
            .push(("Content-Type".to_string(), content_type.to_string()));
        self
    }

    /// Serialises the response into the bytes that go on the wire.
    #[must_use]
    pub fn to_bytes(&self, cseq: u32) -> Vec<u8> {
        let mut head = String::with_capacity(160);
        head.push_str(&self.protocol);
        head.push(' ');
        head.push_str(&self.status.to_string());
        head.push(' ');
        head.push_str(&self.reason);
        head.push_str("\r\n");
        if self.protocol.starts_with("RTSP") {
            head.push_str(&format!("CSeq: {cseq}\r\n"));
        }
        head.push_str(&format!("Server: {SERVER_TOKEN}\r\n"));
        for (name, value) in &self.headers {
            head.push_str(name);
            head.push_str(": ");
            head.push_str(value);
            head.push_str("\r\n");
        }
        if !self.body.is_empty() {
            head.push_str(&format!("Content-Length: {}\r\n", self.body.len()));
        }
        head.push_str("\r\n");

        let mut bytes = head.into_bytes();
        bytes.extend_from_slice(&self.body);
        bytes
    }
}

impl Default for RtspResponse {
    fn default() -> Self {
        Self::ok()
    }
}

#[cfg(test)]
mod tests;
