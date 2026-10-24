#![allow(clippy::expect_used, clippy::unwrap_used)]

use super::*;

use crate::error::AirPlayError;

fn parse(text: &str) -> Result<RtspRequest, AirPlayError> {
    RtspRequest::parse(text.as_bytes(), 64 * 1024)
}

#[test]
fn an_options_request_is_parsed_completely() {
    let request =
        parse("OPTIONS rtsp://10.0.0.1/live RTSP/1.0\r\nCSeq: 2\r\n\r\n").expect("valid request");
    assert_eq!(request.method, RtspMethod::Options);
    assert_eq!(request.uri, "rtsp://10.0.0.1/live");
    assert_eq!(request.protocol, "RTSP/1.0");
    assert_eq!(request.cseq(), Some(2));
    assert!(request.body.is_empty());
}

#[test]
fn headers_are_matched_case_insensitively() {
    let request =
        parse("GET /info HTTP/1.1\r\ncontent-type: text/plain\r\n\r\n").expect("valid request");
    assert_eq!(request.header("Content-Type"), Some("text/plain"));
    assert_eq!(request.content_type().as_deref(), Some("text/plain"));
}

#[test]
fn the_body_is_kept_as_raw_bytes() {
    let request =
        parse("POST /play HTTP/1.1\r\nContent-Length: 3\r\n\r\nabc").expect("valid request");
    assert_eq!(request.body, b"abc");
    assert_eq!(request.body_text().as_deref(), Some("abc"));
}

#[test]
fn a_query_parameter_is_extracted() {
    let request = parse("POST /rate?value=0.5 HTTP/1.1\r\n\r\n").expect("valid request");
    assert_eq!(request.path(), "/rate");
    assert_eq!(request.query_param("value"), Some("0.5"));
    assert_eq!(request.query_param("missing"), None);
}

#[test]
fn an_unknown_method_is_kept_instead_of_rejected() {
    let request = parse("FROBNICATE rtsp://x/y RTSP/1.0\r\n\r\n").expect("valid request");
    assert_eq!(request.method, RtspMethod::Other("FROBNICATE".to_string()));
    assert_eq!(request.method.as_str(), "FROBNICATE");
}

#[test]
fn a_request_without_a_terminator_is_rejected() {
    let error = parse("OPTIONS rtsp://x/y RTSP/1.0\r\nCSeq: 1\r\n").expect_err("must fail");
    assert!(matches!(error, AirPlayError::Parse { .. }));
}

#[test]
fn a_header_without_a_colon_is_rejected() {
    let error =
        parse("OPTIONS rtsp://x/y RTSP/1.0\r\nbroken-header\r\n\r\n").expect_err("must fail");
    assert!(matches!(error, AirPlayError::Parse { .. }));
}

#[test]
fn an_oversized_message_is_rejected() {
    let text = "OPTIONS rtsp://x/y RTSP/1.0\r\n\r\n";
    let error = RtspRequest::parse(text.as_bytes(), 4).expect_err("must fail");
    assert!(matches!(error, AirPlayError::LimitExceeded { .. }));
}

#[test]
fn an_http_request_answers_in_http() {
    let request = parse("GET /info HTTP/1.1\r\n\r\n").expect("valid request");
    assert_eq!(request.response_protocol(), "HTTP/1.1");
}

#[test]
fn an_rtsp_request_answers_in_rtsp() {
    let request = parse("OPTIONS rtsp://x/y RTSP/1.0\r\n\r\n").expect("valid request");
    assert_eq!(request.response_protocol(), "RTSP/1.0");
}

#[test]
fn a_binary_plist_body_is_recognised() {
    let mut request = parse("POST /pair-setup HTTP/1.1\r\n\r\n").expect("valid request");
    assert!(!request.is_binary_plist());
    request.body = b"bplist00....".to_vec();
    assert!(request.is_binary_plist());
}
