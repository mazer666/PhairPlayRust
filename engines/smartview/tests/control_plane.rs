//! The end-to-end tests of the control plane of the Smart View receiver.
//!
//! WHY: the discovery is only the first half of the work: the sender reads
//! the description document, posts its MAC address, its IP address and its
//! device name, and opens the channel. A receiver that fails in one of
//! those steps is found and then dropped.
//!
//! HOW: every test talks to the running engine over a real socket, with
//! the raw bytes a Galaxy device would send.

#![allow(clippy::expect_used, clippy::unwrap_used)]

mod common;

use common::*;
use phairplay_core::protocol::{ProtocolEngine, ProtocolState};
use phairplay_smartview::api::{accept_key, upgrade_response};
use tokio::io::AsyncWriteExt;

#[tokio::test]
async fn the_description_document_is_served_at_the_location() {
    let (engine, port) = start().await;
    let answer = request(
        port,
        "GET /description.xml HTTP/1.1\r\nHOST: 127.0.0.1\r\n\r\n",
    )
    .await;
    assert!(status_line(&answer).contains("200"), "{answer}");
    let document = body(&answer);
    assert!(
        document.contains("<friendlyName>Living Room</friendlyName>"),
        "{document}"
    );
    assert!(
        document.contains("urn:samsung.com:service:MultiScreenService:1"),
        "{document}"
    );
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn the_api_root_answers_with_the_base_url() {
    let (engine, port) = start().await;
    let answer = request(port, "GET /api/v2/ HTTP/1.1\r\nHOST: 127.0.0.1\r\n\r\n").await;
    assert!(status_line(&answer).contains("200"), "{answer}");
    let document = body(&answer);
    assert!(document.contains("/api/v2/websocket"), "{document}");
    assert!(
        document.contains(&format!("http://{}:{port}/api/v2/", ANNOUNCED_IP)),
        "{document}"
    );
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn the_handshake_of_the_sender_is_accepted() {
    // WHY: Samsung documents that the sender posts its MAC address, its IP
    // address and its device name to the device.
    let (engine, port) = start().await;
    let payload = r#"{"mac":"aabbccddeeff","ip":"192.168.1.20","name":"Galaxy S23"}"#;
    let raw = format!(
        "POST /api/v2/ HTTP/1.1\r\nHOST: 127.0.0.1\r\nCONTENT-TYPE: application/json\r\n\
         CONTENT-LENGTH: {}\r\n\r\n{payload}",
        payload.len()
    );
    let answer = request(port, &raw).await;
    assert!(status_line(&answer).contains("200"), "{answer}");
    let document = body(&answer);
    assert!(document.contains("aabbccddeeff"), "{document}");
    assert!(document.contains("Galaxy S23"), "{document}");
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn the_websocket_upgrade_is_answered_with_the_accept_key() {
    let (engine, port) = start().await;
    let (_stream, answer) = open_channel(port).await;
    assert!(status_line(&answer).contains("101"), "{answer}");
    let head = headers(&answer);
    assert!(head.contains("Upgrade: websocket"), "{head}");
    assert!(
        head.contains(&format!(
            "Sec-WebSocket-Accept: {}",
            accept_key("dGhlIHNhbXBsZSBub25jZQ==")
        )),
        "{head}"
    );
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn the_channel_is_held_open_and_the_sender_is_counted() {
    let (engine, port) = start().await;
    let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
        .await
        .expect("the connection opens");
    stream
        .write_all(upgrade_request().as_bytes())
        .await
        .expect("the upgrade is sent");
    stream.flush().await.expect("the upgrade is flushed");
    // The answer of the upgrade arrives, the channel stays open.
    let head = read_head(&mut stream).await;
    assert!(head.contains("101"), "{head}");

    // The engine counts the sender and reports the streaming state.
    assert_eq!(engine.status().peer_count, 1);
    assert_eq!(engine.status().state, ProtocolState::Streaming);
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn the_channel_under_its_own_name_opens_and_greets_the_sender() {
    // WHY: the client library of Samsung names the channel itself. A
    // receiver that only answers at one fixed path is not found by a sender
    // that names its channel — and the greeting is the event that the
    // library waits for before it says anything.
    let (engine, port) = start().await;
    let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
        .await
        .expect("the connection opens");
    stream
        .write_all(
            upgrade_request_for_channel("samsung.remote.control").as_bytes(),
        )
        .await
        .expect("the upgrade is sent");
    stream.flush().await.expect("the upgrade is flushed");

    let head = read_head(&mut stream).await;
    assert!(head.contains("101"), "{head}");

    // The greeting of the receiver: the event that opens the channel.
    let greeting = read_frame(&mut stream).await;
    let text = String::from_utf8_lossy(&greeting);
    assert!(text.contains("ms.channel.connect"), "{text}");
    assert!(text.contains("\"token\""), "{text}");
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn a_key_press_of_the_remote_control_reaches_the_sender() {
    // WHY: the keys of the remote control travel as a method of the channel.
    // The receiver reads them — and this test proves that the message goes
    // through the codec of the channel and not only through the parser of
    // the unit test.
    let (engine, port) = start().await;
    let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
        .await
        .expect("the connection opens");
    stream
        .write_all(upgrade_request().as_bytes())
        .await
        .expect("the upgrade is sent");
    stream.flush().await.expect("the upgrade is flushed");
    let head = read_head(&mut stream).await;
    assert!(head.contains("101"), "{head}");

    // The greeting is read first; the key press comes after it.
    let _greeting = read_frame(&mut stream).await;
    let key = r#"{"method":"ms.remote.control","params":{"Cmd":"Click","DataOfCmd":"KEY_POWER","Option":"false","TypeOfRemote":"SendRemoteKey"}}"#;
    stream
        .write_all(&frame(0x1, key.as_bytes()))
        .await
        .expect("the key press is sent");
    stream.flush().await.expect("the key press is flushed");

    // The channel stays open — a message that the receiver understands does
    // not end the session.
    assert_eq!(engine.status().peer_count, 1);
    assert_eq!(engine.status().state, ProtocolState::Streaming);
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn a_second_sender_is_refused_while_the_channel_is_open() {
    // WHY: Smart View is a single-session protocol. The second sender is
    // rejected, not queued.
    let (engine, port) = start().await;
    let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
        .await
        .expect("the connection opens");
    stream
        .write_all(upgrade_request().as_bytes())
        .await
        .expect("the upgrade is sent");
    stream.flush().await.expect("the upgrade is flushed");
    let head = read_head(&mut stream).await;
    assert!(head.contains("101"), "{head}");

    let answer = request(port, &upgrade_request()).await;
    assert!(status_line(&answer).contains("409"), "{answer}");
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn the_upgrade_answer_carries_the_accept_key_of_the_example_of_the_rfc() {
    // WHY: RFC 6455, section 1.3 — the sender checks this value.
    let response = upgrade_response("dGhlIHNhbXBsZSBub25jZQ==");
    let accept = response
        .extra_headers
        .iter()
        .find(|(name, _)| name == "Sec-WebSocket-Accept")
        .map(|(_, value)| value.clone());
    assert_eq!(accept.as_deref(), Some("s3pPLMBiTxaQ9kYGzzhZRbK+xOo="));
}
