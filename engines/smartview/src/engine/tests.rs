//! The tests of the engine.
//!
//! WHY: the engine owns the lifecycle of the sockets. A receiver that
//! reports a port it does not listen on, or that cannot be stopped, is a
//! receiver that cannot be restarted.

#![allow(clippy::expect_used, clippy::unwrap_used)]
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::time::Duration;

use phairplay_core::capabilities::MediaCapabilities;
use phairplay_core::error::EngineError;
use phairplay_core::identity::DeviceIdentity;
use phairplay_core::media::{NullAudioSink, NullVideoSink};
use phairplay_core::protocol::{EngineContext, ProtocolEngine, ProtocolState};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::*;

/// A context for the engine.
fn context() -> EngineContext {
    let (sender, _receiver) = tokio::sync::mpsc::channel(32);
    EngineContext {
        identity: Arc::new(DeviceIdentity::with_name("Living Room")),
        capabilities: Arc::new(MediaCapabilities::empty()),
        video_sink: Arc::new(NullVideoSink),
        audio_sink: Arc::new(NullAudioSink),
        events: sender,
    }
}

/// The configuration of a test engine: free ports, loopback address.
fn test_config() -> SmartViewConfig {
    SmartViewConfig::default()
        .with_ports(0, 0)
        .with_local_ip(Ipv4Addr::LOCALHOST)
}

#[tokio::test]
async fn the_engine_starts_in_the_idle_state() {
    let engine = SmartViewEngine::new(test_config());
    let status = engine.status();
    assert_eq!(status.protocol, ProtocolKind::SmartView);
    assert_eq!(status.state, ProtocolState::Idle);
    assert_eq!(status.tcp_port, None);
    assert_eq!(status.udp_port, None);
    assert_eq!(status.peer_count, 0);
}

#[tokio::test]
async fn the_engine_advertises_after_it_started() {
    let engine = SmartViewEngine::new(test_config());
    engine.start(context()).await.expect("the engine starts");
    let status = engine.status();
    assert_eq!(status.state, ProtocolState::Advertising);
    assert!(status.tcp_port.is_some(), "{status:?}");
    assert!(status.udp_port.is_some(), "{status:?}");
    assert!(status.last_error.is_none(), "{status:?}");
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn the_engine_answers_with_its_configuration() {
    let engine = SmartViewEngine::new(test_config());
    let config = engine.config();
    assert_eq!(config.http_port, 0);
    assert_eq!(config.ssdp_port, 0);
    assert_eq!(config.local_ip, Ipv4Addr::LOCALHOST);
}

#[tokio::test]
async fn the_engine_publishes_the_urls_of_the_discovery() {
    let engine = SmartViewEngine::new(test_config());
    engine.start(context()).await.expect("the engine starts");
    let base_url = engine.base_url().expect("the base URL is published");
    let description_url = engine
        .description_url()
        .expect("the description URL is published");
    assert!(base_url.ends_with("/api/v2/"), "{base_url}");
    assert!(
        description_url.ends_with("/description.xml"),
        "{description_url}"
    );
    assert!(
        description_url.starts_with("http://127.0.0.1:"),
        "{description_url}"
    );
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn the_engine_refuses_to_start_twice() {
    let engine = SmartViewEngine::new(test_config());
    engine.start(context()).await.expect("the engine starts");
    let error = engine
        .start(context())
        .await
        .expect_err("the second start is refused");
    assert!(matches!(
        error,
        EngineError::AlreadyRunning {
            protocol: "smartview"
        }
    ));
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn the_engine_stops_and_frees_the_ports() {
    let engine = SmartViewEngine::new(test_config());
    engine.start(context()).await.expect("the engine starts");
    let port = engine.status().tcp_port;
    engine.stop().await.expect("the engine stops");
    let status = engine.status();
    assert_eq!(status.state, ProtocolState::Stopped);
    assert_eq!(status.tcp_port, None);
    assert_eq!(status.udp_port, None);
    // The socket is really gone: the port can be bound again.
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port.unwrap_or(0)))
        .await
        .expect("the port is free");
    drop(listener);
}

#[tokio::test]
async fn the_engine_reports_that_it_is_not_running() {
    let engine = SmartViewEngine::new(test_config());
    let error = engine.stop().await.expect_err("the engine is not running");
    assert!(matches!(
        error,
        EngineError::NotRunning {
            protocol: "smartview"
        }
    ));
}

#[tokio::test]
async fn the_engine_can_be_started_again_after_it_was_stopped() {
    let engine = SmartViewEngine::new(test_config());
    engine.start(context()).await.expect("the engine starts");
    engine.stop().await.expect("the engine stops");
    engine
        .start(context())
        .await
        .expect("the engine starts again");
    assert_eq!(engine.status().state, ProtocolState::Advertising);
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn the_engine_serves_the_description_document() {
    let engine = SmartViewEngine::new(test_config());
    engine.start(context()).await.expect("the engine starts");
    let port = engine.status().tcp_port.expect("the API server has a port");
    let answer = request(
        port,
        "GET /description.xml HTTP/1.1\r\nHOST: 127.0.0.1\r\n\r\n",
    )
    .await;
    assert!(answer.starts_with("HTTP/1.1 200 OK"), "{answer}");
    assert!(
        answer.contains("<friendlyName>Living Room</friendlyName>"),
        "{answer}"
    );
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn the_status_is_readable_while_the_engine_starts() {
    // WHY: the status has to be callable from any thread while a
    // transition is in flight.
    let engine = Arc::new(SmartViewEngine::new(test_config()));
    let reader = Arc::clone(&engine);
    let handle = std::thread::spawn(move || {
        for _ in 0..200 {
            let status = reader.status();
            assert!(matches!(
                status.state,
                ProtocolState::Idle
                    | ProtocolState::Starting
                    | ProtocolState::Advertising
                    | ProtocolState::Stopped
            ));
            std::thread::sleep(Duration::from_millis(1));
        }
    });
    engine.start(context()).await.expect("the engine starts");
    handle.join().expect("the reader finishes");
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn a_peer_that_opened_a_channel_is_counted() {
    let engine = SmartViewEngine::new(test_config());
    engine.start(context()).await.expect("the engine starts");
    let session = engine
        .sessions()
        .open("127.0.0.1:40000".parse().expect("the address is valid"))
        .expect("the session opens");
    assert_eq!(engine.status().peer_count, 1);
    assert_eq!(engine.status().state, ProtocolState::Streaming);
    engine.sessions().close(session.id);
    assert_eq!(engine.status().peer_count, 0);
    assert_eq!(engine.status().state, ProtocolState::Advertising);
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn the_detected_address_is_a_usable_one() {
    // WHY: the receiver has to announce an address a Galaxy device can
    // reach. The loopback address is the fallback, not the first choice.
    let address = SmartViewEngine::detect_local_ip();
    assert!(!address.is_unspecified(), "{address}");
}

/// Sends a raw request to the API and returns the whole answer.
async fn request(port: u16, raw: &str) -> String {
    let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
        .await
        .expect("the connection opens");
    stream
        .write_all(raw.as_bytes())
        .await
        .expect("the request is sent");
    stream.flush().await.expect("the request is flushed");
    let mut answer = Vec::new();
    tokio::time::timeout(Duration::from_secs(5), stream.read_to_end(&mut answer))
        .await
        .expect("the answer arrives")
        .expect("the answer is read");
    String::from_utf8_lossy(&answer).to_string()
}
