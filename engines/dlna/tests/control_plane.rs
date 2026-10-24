//! End-to-end tests of the functional behaviour of the DLNA control plane.
//!
//! WHY: the unit tests prove that the pieces behave; these tests prove
//! that the *server* behaves — that it serves the documents, that it
//! answers the actions and that it reports the streaming state.

#![allow(clippy::expect_used, clippy::unwrap_used)]

mod common;

use common::*;
use phairplay_core::protocol::{ProtocolEngine, ProtocolState};
use phairplay_dlna::engine::{DlnaConfig, DlnaEngine};
#[tokio::test]
async fn the_device_description_is_served() {
    let (engine, port) = start().await;
    let answer = request(
        port,
        "GET /description.xml HTTP/1.1\r\nHOST: 127.0.0.1\r\n\r\n",
    )
    .await;
    assert!(status_line(&answer).contains("200"), "{answer}");
    let document = body(&answer);
    assert!(document.contains("urn:schemas-upnp-org:device:MediaRenderer:1"));
    assert!(document.contains("<friendlyName>Living Room</friendlyName>"));
    assert!(document.contains("<UDN>uuid:"));
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn the_service_descriptions_are_served() {
    let (engine, port) = start().await;
    for path in [
        "/ConnectionManager.xml",
        "/AVTransport.xml",
        "/RenderingControl.xml",
    ] {
        let raw = format!("GET {path} HTTP/1.1\r\nHOST: 127.0.0.1\r\n\r\n");
        let answer = request(port, &raw).await;
        assert!(status_line(&answer).contains("200"), "{path}: {answer}");
        assert!(body(&answer).contains("<scpd"), "{path}: {answer}");
    }
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn an_unknown_path_is_a_not_found() {
    let (engine, port) = start().await;
    let answer = request(port, "GET /telepathy HTTP/1.1\r\nHOST: 127.0.0.1\r\n\r\n").await;
    assert!(status_line(&answer).contains("404"), "{answer}");
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn a_soap_action_is_answered() {
    let (engine, port) = start().await;
    let raw = soap(
        "SetAVTransportURI",
        "<CurrentURI>http://10.0.0.9/movie.mp4</CurrentURI>",
    );
    let answer = request(port, &raw).await;
    assert!(status_line(&answer).contains("200"), "{answer}");
    assert!(body(&answer).contains("SetAVTransportURIResponse"));
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn a_gena_callback_to_the_local_subnet_is_accepted() {
    let (engine, port) = start().await;
    let raw = "SUBSCRIBE /upnp/event/AVTransport HTTP/1.1\r\nHOST: 127.0.0.1\r\n\
               CALLBACK: <http://192.168.1.20:8080/events>\r\nNT: upnp:event\r\n\r\n";
    let answer = request(port, raw).await;
    assert!(status_line(&answer).contains("200"), "{answer}");
    assert!(answer.contains("SID: uuid:"), "{answer}");
    assert!(answer.contains("TIMEOUT: Second-"), "{answer}");
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn a_second_control_point_is_rejected_while_the_first_streams() {
    // The first control point takes the renderer and starts a stream.
    let (engine, port) = start().await;
    let raw = soap(
        "SetAVTransportURI",
        "<CurrentURI>http://10.0.0.9/movie.mp4</CurrentURI>",
    );
    let answer = request_from("127.0.0.1", port, &raw).await;
    assert!(status_line(&answer).contains("200"), "{answer}");
    let answer = request_from("127.0.0.1", port, &soap("Play", "")).await;
    assert!(status_line(&answer).contains("200"), "{answer}");
    assert_eq!(engine.status().state, ProtocolState::Streaming);

    // A second control point, from another address, is *rejected* — it is
    // not queued and it does not take the stream away.
    let raw = soap(
        "SetAVTransportURI",
        "<CurrentURI>http://10.0.0.8/other.mp4</CurrentURI>",
    );
    let answer = request_from("127.0.0.2", port, &raw).await;
    assert!(status_line(&answer).contains("503"), "{answer}");
    assert!(
        body(&answer).contains("<errorCode>701</errorCode>"),
        "{answer}"
    );

    // The first control point still owns the renderer.
    let answer = request_from("127.0.0.1", port, &soap("GetTransportInfo", "")).await;
    assert!(
        body(&answer).contains("<CurrentTransportState>PLAYING</CurrentTransportState>"),
        "{answer}"
    );
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn the_engine_reports_the_streaming_state() {
    let (engine, port) = start().await;
    let raw = soap(
        "SetAVTransportURI",
        "<CurrentURI>http://10.0.0.9/movie.mp4</CurrentURI>",
    );
    let answer = request(port, &raw).await;
    assert!(status_line(&answer).contains("200"), "{answer}");
    let answer = request(port, &soap("Play", "")).await;
    assert!(status_line(&answer).contains("200"), "{answer}");
    assert_eq!(engine.status().state, ProtocolState::Streaming);
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn an_unknown_method_is_refused() {
    let (engine, port) = start().await;
    let raw = "DELETE /description.xml HTTP/1.1\r\nHOST: 127.0.0.1\r\n\r\n";
    let answer = request(port, raw).await;
    assert!(status_line(&answer).contains("405"), "{answer}");
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn a_head_request_is_answered_without_a_body() {
    let (engine, port) = start().await;
    let raw = "HEAD /description.xml HTTP/1.1\r\nHOST: 127.0.0.1\r\n\r\n";
    let answer = request(port, raw).await;
    assert!(status_line(&answer).contains("200"), "{answer}");
    assert!(body(&answer).is_empty(), "{answer}");
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn the_engine_stops_cleanly_and_frees_the_port() {
    let (engine, port) = start().await;
    engine.stop().await.expect("the engine stops");
    assert_eq!(engine.status().state, ProtocolState::Stopped);
    // The port is free again: a second engine can take it.
    let second = DlnaEngine::new(
        DlnaConfig::default()
            .with_ports(port, 0)
            .with_local_ip(ANNOUNCED_IP),
    );
    second
        .start(context())
        .await
        .expect("the second engine starts");
    assert_eq!(second.status().tcp_port, Some(port));
    second.stop().await.expect("the second engine stops");
}
