//! The end-to-end tests of the hardening measures of the Smart View API.
//!
//! WHY: every hardening measure has to be exercised over a real socket,
//! with a real request and a real answer. A reader that was never asked
//! for a request line of ten kilobytes is a reader that has not been
//! tested, and a guard that was never asked for a control character in a
//! device name is a guard that does not work.
//!
//! HOW: every test sends a request that a hostile device on the local
//! network could send, and asserts that the receiver refuses it — and that
//! the receiver is still alive afterwards.

#![allow(clippy::expect_used, clippy::unwrap_used)]

mod common;

use common::*;
use phairplay_core::protocol::ProtocolEngine;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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
    let mut raw = String::from("GET /api/v2/ HTTP/1.1\r\n");
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
        "GET /api/v2/ HTTP/1.1\r\nX-Big: {}\r\n\r\n",
        "x".repeat(9000)
    );
    let answer = request(port, &raw).await;
    assert!(status_line(&answer).contains("431"), "{answer}");
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn a_body_that_is_too_large_is_refused_before_it_is_read() {
    // WHY: the length is checked before the first byte is read. A body that
    // announces a hundred megabytes must not cost a hundred megabytes.
    let (engine, port) = start().await;
    let raw = "POST /api/v2/ HTTP/1.1\r\nHOST: 127.0.0.1\r\nCONTENT-LENGTH: 104857600\r\n\r\n";
    let answer = request(port, raw).await;
    assert!(status_line(&answer).contains("413"), "{answer}");
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn a_content_length_that_is_not_a_number_is_refused() {
    let (engine, port) = start().await;
    let raw = "POST /api/v2/ HTTP/1.1\r\nHOST: 127.0.0.1\r\nCONTENT-LENGTH: abc\r\n\r\n";
    let answer = request(port, raw).await;
    assert!(status_line(&answer).contains("400"), "{answer}");
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn a_chunked_body_is_refused() {
    // WHY: the receiver does not implement the chunked transfer encoding,
    // and a body that is read with the wrong framing is a body that is
    // read wrong.
    let (engine, port) = start().await;
    let raw = "POST /api/v2/ HTTP/1.1\r\nHOST: 127.0.0.1\r\n\
               TRANSFER-ENCODING: chunked\r\n\r\n5\r\nhello\r\n0\r\n\r\n";
    let answer = request(port, raw).await;
    assert!(status_line(&answer).contains("400"), "{answer}");
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn a_request_without_a_request_line_is_refused() {
    let (engine, port) = start().await;
    let answer = request(port, "\r\n\r\n").await;
    assert!(status_line(&answer).contains("400"), "{answer}");
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn a_control_character_in_the_name_of_the_sender_never_reaches_the_answer() {
    // WHY: the name of the sender ends up in the log and in the session. A
    // line break in it would forge a log line, and a NUL would truncate a
    // C string of a native shell.
    let (engine, port) = start().await;
    let payload = "{\"mac\":\"aabbccddeeff\",\"name\":\"Galaxy\\u0000\\u001bS23\\n\"}";
    let raw = format!(
        "POST /api/v2/ HTTP/1.1\r\nHOST: 127.0.0.1\r\nCONTENT-TYPE: application/json\r\n\
         CONTENT-LENGTH: {}\r\n\r\n{payload}",
        payload.len()
    );
    let answer = request(port, &raw).await;
    assert!(status_line(&answer).contains("200"), "{answer}");
    let document = body(&answer);
    assert!(!document.contains('\u{0}'), "{document}");
    assert!(!document.contains('\u{1b}'), "{document}");
    assert!(!document.contains('\n'), "{document}");
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn a_control_character_in_the_name_of_the_device_never_reaches_the_document() {
    // WHY: the name of the receiver comes from the configuration of the
    // user. A name with a control character must not produce a document
    // that a parser of the phone rejects.
    let engine = phairplay_smartview::engine::SmartViewEngine::new(test_config());
    let (sender, _receiver) = tokio::sync::mpsc::channel(8);
    let context = phairplay_core::protocol::EngineContext {
        identity: std::sync::Arc::new(phairplay_core::identity::DeviceIdentity::with_name(
            "Living\u{0} Room\u{1b}",
        )),
        capabilities: std::sync::Arc::new(phairplay_core::capabilities::MediaCapabilities::empty()),
        video_sink: std::sync::Arc::new(phairplay_core::media::NullVideoSink),
        audio_sink: std::sync::Arc::new(phairplay_core::media::NullAudioSink),
        events: sender,
    };
    engine.start(context).await.expect("the engine starts");
    let port = engine.status().tcp_port.expect("the API server has a port");
    let answer = request(
        port,
        "GET /description.xml HTTP/1.1\r\nHOST: 127.0.0.1\r\n\r\n",
    )
    .await;
    let document = body(&answer);
    assert!(!document.contains('\u{0}'), "{document}");
    assert!(!document.contains('\u{1b}'), "{document}");
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn an_unmasked_websocket_frame_breaks_the_channel() {
    // WHY: RFC 6455 requires every frame of a client to be masked. A frame
    // without a mask is refused — and the channel is closed, so that the
    // receiver does not keep a connection open that it does not understand.
    let (engine, port) = start().await;
    let (mut stream, head) = open_channel(port).await;
    assert!(head.contains("101"), "{head}");
    // A frame without the mask bit.
    let unmasked = [0x81u8, 0x02, 0x41, 0x42];
    stream
        .write_all(&unmasked)
        .await
        .expect("the frame is sent");
    stream.flush().await.expect("the frame is flushed");
    let mut answer = Vec::new();
    let read = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        stream.read_to_end(&mut answer),
    )
    .await
    .expect("the channel ends");
    assert!(read.is_ok(), "the channel is closed");
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn a_websocket_frame_that_announces_a_gigabyte_is_refused() {
    // WHY: the length of a frame is checked before the payload is read. A
    // frame that announces a gigabyte must not cost a gigabyte.
    let (engine, port) = start().await;
    let (mut stream, head) = open_channel(port).await;
    assert!(head.contains("101"), "{head}");
    // A masked frame with a 64-bit length of one gigabyte.
    let hostile = [
        0x82u8,
        0x80 | 127,
        0,
        0,
        0,
        0,
        0x40,
        0,
        0,
        0,
        0x12,
        0x34,
        0x56,
        0x78,
    ];
    stream.write_all(&hostile).await.expect("the frame is sent");
    stream.flush().await.expect("the frame is flushed");
    let mut answer = Vec::new();
    let read = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        stream.read_to_end(&mut answer),
    )
    .await
    .expect("the channel ends");
    assert!(read.is_ok(), "the channel is closed");
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn the_receiver_is_still_alive_after_every_refusal() {
    // WHY: a refusal has to cost the peer its request, not the receiver
    // its service.
    let (engine, port) = start().await;
    let hostile = format!(
        "GET /{} HTTP/1.1\r\nHOST: 127.0.0.1\r\n\r\n",
        "x".repeat(9000)
    );
    let _ = request(port, &hostile).await;
    let answer = request(port, "GET /api/v2/ HTTP/1.1\r\nHOST: 127.0.0.1\r\n\r\n").await;
    assert!(status_line(&answer).contains("200"), "{answer}");
    engine.stop().await.expect("the engine stops");
}
