//! The WFD RTSP control plane.
//!
//! WHY: a Miracast sender opens a RTSP connection to the sink and negotiates
//! the session there. The vocabulary is small — `OPTIONS`, `GET_PARAMETER`,
//! `SET_PARAMETER`, `SETUP`, `PLAY`, `PAUSE`, `TEARDOWN` — and the answers
//! are plain text. Getting the vocabulary exactly right is what makes a
//! Windows laptop show its picture on the television.
//!
//! HOW: the request is read by a bounded reader, routed by its method, and
//! answered by a response builder. The reader refuses a message that is
//! larger than the limit, a header that is malformed, and a request line
//! without a method — a peer on the local network must not be able to make
//! the receiver read forever.
//!
//! ## Example
//!
//! ```
//! use phairplay_miracast::rtsp::{RtspRequest, route};
//! use std::collections::HashMap;
//!
//! let request = RtspRequest {
//!     method: "OPTIONS".to_string(),
//!     uri: "rtsp://192.168.1.10:7236/".to_string(),
//!     cseq: 1,
//!     headers: HashMap::new(),
//! };
//!
//! let response = route(&request).expect("a supported method");
//! assert_eq!(response.status_code, 200);
//! assert!(response.headers.contains_key("Public"));
//! ```

use std::collections::HashMap;

use crate::error::MiracastError;

/// The largest RTSP message the engine accepts, in bytes.
pub const MAX_MESSAGE_BYTES: usize = 64 * 1024;

/// The largest number of headers one request may carry.
pub const MAX_HEADERS: usize = 64;

/// The largest length of a single header line, in bytes.
pub const MAX_HEADER_BYTES: usize = 8 * 1024;

/// The identifier of the WFD session.
pub const WFD_SESSION_ID: &str = "PhairPlayRustWfdSession";

/// The public methods the sink announces.
pub const WFD_PUBLIC_METHODS: &str = "org.wfa.wfd1.0, GET_PARAMETER, SET_PARAMETER";

/// A parsed RTSP request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RtspRequest {
    /// The method of the request.
    pub method: String,
    /// The URI of the request.
    pub uri: String,
    /// The sequence number of the request.
    pub cseq: u32,
    /// The headers of the request.
    pub headers: HashMap<String, String>,
}

/// The answer to an RTSP request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RtspResponse {
    /// The status code of the answer.
    pub status_code: u16,
    /// The reason phrase of the answer.
    pub status_message: &'static str,
    /// The headers of the answer.
    pub headers: HashMap<String, String>,
    /// The body of the answer.
    pub body: String,
}

impl RtspResponse {
    /// Builds an answer without a body.
    fn plain(status_code: u16, status_message: &'static str) -> Self {
        Self {
            status_code,
            status_message,
            headers: HashMap::new(),
            body: String::new(),
        }
    }

    /// Builds an answer with one header.
    fn with_header(status_code: u16, status_message: &'static str, key: &str, value: &str) -> Self {
        let mut headers = HashMap::new();
        headers.insert(key.to_string(), value.to_string());
        Self {
            status_code,
            status_message,
            headers,
            body: String::new(),
        }
    }

    /// Serialises the answer into the bytes that go on the wire.
    #[must_use]
    pub fn to_bytes(&self, cseq: u32) -> Vec<u8> {
        let mut out = format!("RTSP/1.0 {} {}\r\n", self.status_code, self.status_message);
        out.push_str(&format!("CSeq: {cseq}\r\n"));
        out.push_str("Server: PhairPlayRust/1.0\r\n");
        for (key, value) in &self.headers {
            out.push_str(&format!("{key}: {value}\r\n"));
        }
        if !self.body.is_empty() {
            out.push_str(&format!("Content-Length: {}\r\n", self.body.len()));
        }
        out.push_str("\r\n");
        out.push_str(&self.body);
        out.into_bytes()
    }
}

/// Reads one RTSP request from a buffer.
///
/// # Errors
/// Returns [`MiracastError::MessageTooLarge`] when the buffer holds more than
/// [`MAX_MESSAGE_BYTES`], [`MiracastError::TruncatedMessage`] when the
/// request line is incomplete, and [`MiracastError::MalformedHeader`] when a
/// header is empty, too long or too numerous.
pub fn read_request(buffer: &[u8]) -> Result<RtspRequest, MiracastError> {
    if buffer.len() > MAX_MESSAGE_BYTES {
        return Err(MiracastError::MessageTooLarge {
            protocol: "miracast",
            limit: MAX_MESSAGE_BYTES,
        });
    }

    let text = String::from_utf8_lossy(buffer);
    let mut lines = text.split("\r\n");

    let request_line = lines.next().unwrap_or_default();
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default().to_string();
    let uri = parts.next().unwrap_or_default().to_string();
    if method.is_empty() || uri.is_empty() {
        return Err(MiracastError::TruncatedMessage {
            protocol: "miracast",
        });
    }

    let mut headers = HashMap::new();
    let mut cseq = 0_u32;
    for line in lines {
        if line.is_empty() {
            break;
        }
        if headers.len() >= MAX_HEADERS {
            return Err(MiracastError::MalformedHeader {
                protocol: "miracast",
                reason: format!("more than {MAX_HEADERS} headers"),
            });
        }
        if line.len() > MAX_HEADER_BYTES {
            return Err(MiracastError::MalformedHeader {
                protocol: "miracast",
                reason: format!("a header line of {} bytes", line.len()),
            });
        }
        let Some((key, value)) = line.split_once(':') else {
            return Err(MiracastError::MalformedHeader {
                protocol: "miracast",
                reason: format!("the header `{line}` has no colon"),
            });
        };
        let key = key.trim().to_string();
        let value = value.trim().to_string();
        if key.is_empty() {
            return Err(MiracastError::MalformedHeader {
                protocol: "miracast",
                reason: "a header without a name".to_string(),
            });
        }
        if key.eq_ignore_ascii_case("CSeq") {
            cseq = value.parse::<u32>().unwrap_or(0);
        }
        headers.insert(key, value);
    }

    Ok(RtspRequest {
        method,
        uri,
        cseq,
        headers,
    })
}

/// Routes a request to the answer of the sink.
///
/// # Errors
/// Returns [`MiracastError::UnknownMethod`] for a method the WFD protocol
/// does not define.
pub fn route(request: &RtspRequest) -> Result<RtspResponse, MiracastError> {
    match request.method.as_str() {
        "OPTIONS" => Ok(RtspResponse::with_header(
            200,
            "OK",
            "Public",
            WFD_PUBLIC_METHODS,
        )),
        "GET_PARAMETER" => {
            let mut response =
                RtspResponse::with_header(200, "OK", "Content-Type", "text/parameters");
            response.body = sink_parameters();
            Ok(response)
        }
        "SET_PARAMETER" => Ok(RtspResponse::plain(200, "OK")),
        "SETUP" => {
            let mut response = RtspResponse::with_header(200, "OK", "Session", WFD_SESSION_ID);
            response.headers.insert(
                "Transport".to_string(),
                "RTP/AVP/TCP;unicast;interleaved=0-1".to_string(),
            );
            Ok(response)
        }
        "PLAY" | "PAUSE" | "TEARDOWN" => Ok(RtspResponse::with_header(
            200,
            "OK",
            "Session",
            WFD_SESSION_ID,
        )),
        other => Err(MiracastError::UnknownMethod {
            protocol: "miracast",
            method: other.to_string(),
        }),
    }
}

/// The parameters of the sink, as the sender asks for them with
/// `GET_PARAMETER`.
///
/// WHY: this body is the contract between the sink and the sender. It says
/// which audio codec the sink takes, which video format it understands, that
/// the transport is RTP over TCP and that the content is not protected.
#[must_use]
pub fn sink_parameters() -> String {
    [
        "wfd_audio_codecs: LPCM 00000003 00",
        "wfd_video_formats: 00 00 02 10 0001FFFF 00000000 00000000 00 0000 0000 00 none none",
        "wfd_client_rtp_ports: RTP/AVP/TCP;unicast 0 0 mode=play",
        "wfd_content_protection: none",
        "wfd_display_edid: none",
        "wfd_coupled_sink: none",
        "wfd_connector_type: 05",
    ]
    .join("\r\n")
        + "\r\n"
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    #[test]
    fn a_request_is_parsed() {
        let raw =
            b"OPTIONS rtsp://192.168.1.10:7236/ RTSP/1.0\r\nCSeq: 7\r\nUser-Agent: Test\r\n\r\n";
        let request = read_request(raw).expect("parse");
        assert_eq!(request.method, "OPTIONS");
        assert_eq!(request.uri, "rtsp://192.168.1.10:7236/");
        assert_eq!(request.cseq, 7);
        assert_eq!(
            request.headers.get("User-Agent").map(String::as_str),
            Some("Test")
        );
    }

    #[test]
    fn an_oversized_message_is_rejected() {
        let raw = vec![b'x'; MAX_MESSAGE_BYTES + 1];
        let error = read_request(&raw).expect_err("must fail");
        assert!(
            matches!(error, MiracastError::MessageTooLarge { .. }),
            "{error}"
        );
    }

    #[test]
    fn a_request_line_without_a_method_is_rejected() {
        let error = read_request(b"\r\n\r\n").expect_err("must fail");
        assert!(
            matches!(error, MiracastError::TruncatedMessage { .. }),
            "{error}"
        );
    }

    #[test]
    fn a_header_without_a_colon_is_rejected() {
        let raw = b"OPTIONS rtsp://x/ RTSP/1.0\r\nBrokenHeader\r\n\r\n";
        let error = read_request(raw).expect_err("must fail");
        assert!(
            matches!(error, MiracastError::MalformedHeader { .. }),
            "{error}"
        );
    }

    #[test]
    fn too_many_headers_are_rejected() {
        let mut raw = String::from("OPTIONS rtsp://x/ RTSP/1.0\r\n");
        for index in 0..=MAX_HEADERS {
            raw.push_str(&format!("X-{index}: v\r\n"));
        }
        let error = read_request(raw.as_bytes()).expect_err("must fail");
        assert!(
            matches!(error, MiracastError::MalformedHeader { .. }),
            "{error}"
        );
    }

    #[test]
    fn an_overlong_header_line_is_rejected() {
        let value = "x".repeat(MAX_HEADER_BYTES + 16);
        let raw = format!("OPTIONS rtsp://x/ RTSP/1.0\r\nX-Long: {value}\r\n\r\n");
        let error = read_request(raw.as_bytes()).expect_err("must fail");
        assert!(
            matches!(error, MiracastError::MalformedHeader { .. }),
            "{error}"
        );
    }

    #[test]
    fn options_announces_the_public_methods() {
        let request = RtspRequest {
            method: "OPTIONS".to_string(),
            uri: "rtsp://x/".to_string(),
            cseq: 1,
            headers: HashMap::new(),
        };
        let response = route(&request).expect("route");
        assert_eq!(response.status_code, 200);
        assert!(response.headers.contains_key("Public"));
    }

    #[test]
    fn get_parameter_answers_with_the_sink_parameters() {
        let request = RtspRequest {
            method: "GET_PARAMETER".to_string(),
            uri: "rtsp://x/".to_string(),
            cseq: 1,
            headers: HashMap::new(),
        };
        let response = route(&request).expect("route");
        assert!(response.body.contains("wfd_audio_codecs"));
        assert!(response.body.contains("wfd_client_rtp_ports"));
    }

    #[test]
    fn setup_answers_with_the_session_and_the_transport() {
        let request = RtspRequest {
            method: "SETUP".to_string(),
            uri: "rtsp://x/".to_string(),
            cseq: 1,
            headers: HashMap::new(),
        };
        let response = route(&request).expect("route");
        assert_eq!(
            response.headers.get("Session").map(String::as_str),
            Some(WFD_SESSION_ID)
        );
        assert!(response.headers.contains_key("Transport"));
    }

    #[test]
    fn an_unknown_method_is_refused() {
        let request = RtspRequest {
            method: "INVITE".to_string(),
            uri: "rtsp://x/".to_string(),
            cseq: 1,
            headers: HashMap::new(),
        };
        let error = route(&request).expect_err("must fail");
        assert!(
            matches!(error, MiracastError::UnknownMethod { .. }),
            "{error}"
        );
    }

    #[test]
    fn the_serialised_answer_carries_the_sequence_number() {
        let response = RtspResponse::plain(200, "OK");
        let bytes = response.to_bytes(42);
        let text = String::from_utf8(bytes).expect("utf8");
        assert!(text.starts_with("RTSP/1.0 200 OK\r\n"), "{text}");
        assert!(text.contains("CSeq: 42\r\n"), "{text}");
    }
}
