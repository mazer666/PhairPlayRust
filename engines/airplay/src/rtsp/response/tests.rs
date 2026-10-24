#![allow(clippy::expect_used, clippy::unwrap_used)]

use super::*;

fn header_and_body(bytes: &[u8]) -> (String, Vec<u8>) {
    let split = bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|index| index + 4)
        .expect("header terminator");
    (
        String::from_utf8_lossy(&bytes[..split]).to_string(),
        bytes[split..].to_vec(),
    )
}

#[test]
fn an_ok_response_carries_the_cseq_and_the_server_token() {
    let bytes = RtspResponse::ok().to_bytes(12);
    let (head, body) = header_and_body(&bytes);
    assert!(head.starts_with("RTSP/1.0 200 OK\r\n"), "{head}");
    assert!(head.contains("CSeq: 12\r\n"), "{head}");
    assert!(head.contains("Server: AirTunes/220.68\r\n"), "{head}");
    assert!(body.is_empty());
}

#[test]
fn an_http_response_omits_the_cseq() {
    let bytes = RtspResponse::ok().with_protocol("HTTP/1.1").to_bytes(12);
    let (head, _) = header_and_body(&bytes);
    assert!(head.starts_with("HTTP/1.1 200 OK\r\n"), "{head}");
    assert!(!head.contains("CSeq"), "{head}");
}

#[test]
fn the_content_length_counts_bytes_and_not_characters() {
    let body = vec![0x00u8, 0xFF, 0xFE, 0x80];
    let bytes = RtspResponse::ok().with_body(body.clone()).to_bytes(1);
    let (head, payload) = header_and_body(&bytes);
    assert!(head.contains("Content-Length: 4\r\n"), "{head}");
    assert_eq!(payload, body);
}

#[test]
fn an_empty_body_carries_no_content_length() {
    let bytes = RtspResponse::ok().to_bytes(1);
    let (head, _) = header_and_body(&bytes);
    assert!(!head.contains("Content-Length"), "{head}");
}

#[test]
fn extra_headers_are_written_in_order() {
    let response = RtspResponse::ok()
        .with_header("Session", "PhairPlaySession")
        .with_header("Transport", "RTP/AVP/TCP;unicast;interleaved=0-1");
    let (head, _) = header_and_body(&response.to_bytes(1));
    let session = head.find("Session:").expect("session header");
    let transport = head.find("Transport:").expect("transport header");
    assert!(session < transport, "{head}");
}

#[test]
fn the_second_peer_is_answered_with_service_unavailable() {
    let bytes = RtspResponse::service_unavailable().to_bytes(0);
    let (head, _) = header_and_body(&bytes);
    assert!(
        head.starts_with("RTSP/1.0 503 Service Unavailable\r\n"),
        "{head}"
    );
}
