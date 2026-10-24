//! End-to-end tests of the hardening measures of the DLNA control plane.
//!
//! WHY: every hardening measure has to be exercised over a real socket,
//! with a real request and a real answer. A parser that is never asked for
//! a DOCTYPE is a parser that has not been tested, and a guard that is
//! never asked for `169.254.169.254` is a guard that does not work.
//!
//! HOW: every test sends a request that a hostile device on the local
//! network could send, and asserts that the server refuses it.

#![allow(clippy::expect_used, clippy::unwrap_used)]

mod common;

use common::*;
use phairplay_core::protocol::ProtocolEngine;
#[tokio::test]
async fn a_doctype_in_the_soap_body_is_refused() {
    // This is the XXE attack, executed over a real socket.
    let (engine, port) = start().await;
    let payload = "<?xml version=\"1.0\"?><!DOCTYPE foo [ <!ENTITY xxe SYSTEM \"file:///etc/passwd\"> ]>\
                   <s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\"><s:Body>\
                   <u:Play xmlns:u=\"urn:schemas-upnp-org:service:AVTransport:1\"/></s:Body></s:Envelope>";
    let answer = request(port, &soap_raw(payload)).await;
    assert!(status_line(&answer).contains("400"), "{answer}");
    assert!(!body(&answer).contains("root:"), "{answer}");
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn an_external_entity_in_the_soap_body_is_refused() {
    let (engine, port) = start().await;
    let payload = "<s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\"><s:Body>\
                   <u:Play xmlns:u=\"urn:x\">&xxe;</u:Play></s:Body></s:Envelope>";
    let answer = request(port, &soap_raw(payload)).await;
    assert!(status_line(&answer).contains("400"), "{answer}");
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn a_gena_callback_to_the_metadata_service_is_refused() {
    // The SSRF attack, executed over a real socket.
    let (engine, port) = start().await;
    let raw = "SUBSCRIBE /upnp/event/AVTransport HTTP/1.1\r\nHOST: 127.0.0.1\r\n\
               CALLBACK: <http://169.254.169.254/latest/meta-data/>\r\nNT: upnp:event\r\n\r\n";
    let answer = request(port, raw).await;
    assert!(status_line(&answer).contains("412"), "{answer}");
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn a_gena_callback_to_the_loopback_address_is_refused() {
    let (engine, port) = start().await;
    let raw = "SUBSCRIBE /upnp/event/AVTransport HTTP/1.1\r\nHOST: 127.0.0.1\r\n\
               CALLBACK: <http://127.0.0.1:8080/events>\r\nNT: upnp:event\r\n\r\n";
    let answer = request(port, raw).await;
    assert!(status_line(&answer).contains("412"), "{answer}");
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn a_gena_callback_to_the_ipv6_loopback_is_refused() {
    let (engine, port) = start().await;
    let raw = "SUBSCRIBE /upnp/event/AVTransport HTTP/1.1\r\nHOST: 127.0.0.1\r\n\
               CALLBACK: <http://[::1]:8080/events>\r\nNT: upnp:event\r\n\r\n";
    let answer = request(port, raw).await;
    assert!(status_line(&answer).contains("412"), "{answer}");
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn a_gena_callback_to_a_host_name_is_refused() {
    let (engine, port) = start().await;
    let raw = "SUBSCRIBE /upnp/event/AVTransport HTTP/1.1\r\nHOST: 127.0.0.1\r\n\
               CALLBACK: <http://metadata.google.internal/>\r\nNT: upnp:event\r\n\r\n";
    let answer = request(port, raw).await;
    assert!(status_line(&answer).contains("412"), "{answer}");
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn an_over_long_request_line_is_refused() {
    let (engine, port) = start().await;
    let raw = format!(
        "GET /{} HTTP/1.1\r\nHOST: 127.0.0.1\r\n\r\n",
        "x".repeat(9000)
    );
    let answer = request(port, &raw).await;
    assert!(status_line(&answer).contains("414"), "{answer}");
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn too_many_headers_are_refused() {
    let (engine, port) = start().await;
    let mut raw = String::from("GET /description.xml HTTP/1.1\r\n");
    for index in 0..200 {
        raw.push_str(&format!("X-{index}: value\r\n"));
    }
    raw.push_str("\r\n");
    let answer = request(port, &raw).await;
    assert!(status_line(&answer).contains("431"), "{answer}");
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn an_over_long_header_line_is_refused() {
    let (engine, port) = start().await;
    let raw = format!(
        "GET /description.xml HTTP/1.1\r\nX-Big: {}\r\n\r\n",
        "x".repeat(9000)
    );
    let answer = request(port, &raw).await;
    assert!(status_line(&answer).contains("414"), "{answer}");
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn a_body_that_is_too_large_is_refused() {
    let (engine, port) = start().await;
    let raw = "POST /upnp/control/AVTransport HTTP/1.1\r\nHOST: 127.0.0.1\r\n\
               CONTENT-LENGTH: 1048576\r\n\r\n";
    let answer = request(port, raw).await;
    assert!(status_line(&answer).contains("413"), "{answer}");
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn a_chunked_body_is_refused() {
    let (engine, port) = start().await;
    let raw = "POST /upnp/control/AVTransport HTTP/1.1\r\nHOST: 127.0.0.1\r\n\
               TRANSFER-ENCODING: chunked\r\n\r\n0\r\n\r\n";
    let answer = request(port, raw).await;
    assert!(status_line(&answer).contains("501"), "{answer}");
    engine.stop().await.expect("the engine stops");
}
