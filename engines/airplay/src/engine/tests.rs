#![allow(clippy::expect_used, clippy::unwrap_used)]
use super::*;
use phairplay_core::identity::DeviceIdentity;
use phairplay_core::media::{NullAudioSink, NullVideoSink};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc;

fn context() -> EngineContext {
    let (events, _receiver) = mpsc::channel(16);
    EngineContext {
        identity: Arc::new(DeviceIdentity::default()),
        capabilities: Arc::new(phairplay_core::capabilities::MediaCapabilities::empty()),
        video_sink: Arc::new(NullVideoSink),
        audio_sink: Arc::new(NullAudioSink),
        events,
    }
}

#[tokio::test]
async fn a_new_engine_is_idle() {
    let engine = AirPlayEngine::new(AirPlayConfig::default());
    let status = engine.status();
    assert_eq!(status.state, ProtocolState::Idle);
    assert_eq!(status.protocol, ProtocolKind::AirPlay);
    assert_eq!(status.peer_count, 0);
    assert!(status.tcp_port.is_none());
}

#[tokio::test]
async fn a_started_engine_advertises_and_stops_again() {
    let config = AirPlayConfig {
        port: 0,
        ..AirPlayConfig::default()
    };
    let engine = AirPlayEngine::new(config);
    engine.start(context()).await.expect("start");
    let status = engine.status();
    assert_eq!(status.state, ProtocolState::Advertising);
    assert!(status.tcp_port.is_some());
    engine.stop().await.expect("stop");
    assert_eq!(engine.status().state, ProtocolState::Stopped);
}

#[tokio::test]
async fn starting_twice_is_refused() {
    let config = AirPlayConfig {
        port: 0,
        ..AirPlayConfig::default()
    };
    let engine = AirPlayEngine::new(config);
    engine.start(context()).await.expect("start");
    let error = engine.start(context()).await.expect_err("must fail");
    assert!(matches!(error, EngineError::AlreadyRunning { .. }));
    engine.stop().await.expect("stop");
}

#[tokio::test]
async fn stopping_an_idle_engine_is_refused() {
    let engine = AirPlayEngine::new(AirPlayConfig::default());
    let error = engine.stop().await.expect_err("must fail");
    assert!(matches!(error, EngineError::NotRunning { .. }));
}

#[tokio::test]
async fn a_second_sender_is_rejected_with_service_unavailable() {
    let config = AirPlayConfig {
        port: 0,
        ..AirPlayConfig::default()
    };
    let engine = AirPlayEngine::new(config);
    engine.start(context()).await.expect("start");
    let port = engine.status().tcp_port.expect("port");

    let mut first = TcpStream::connect(("127.0.0.1", port))
        .await
        .expect("connect");
    // The first sender occupies the session.
    let mut second = TcpStream::connect(("127.0.0.1", port))
        .await
        .expect("connect");
    let mut buffer = [0u8; 128];
    let read = tokio::time::timeout(Duration::from_secs(2), second.read(&mut buffer))
        .await
        .expect("answer")
        .expect("read");
    let answer = String::from_utf8_lossy(&buffer[..read]).to_string();
    assert!(
        answer.starts_with("RTSP/1.0 503 Service Unavailable"),
        "{answer}"
    );
    let _ = first
        .write_all(b"OPTIONS rtsp://x/y RTSP/1.0\r\n\r\n")
        .await;
    engine.stop().await.expect("stop");
}

#[tokio::test]
async fn the_status_is_readable_from_another_thread() {
    let engine = AirPlayEngine::new(AirPlayConfig::default());
    let handle = std::thread::spawn(move || engine.status());
    let status = handle.join().expect("thread");
    assert_eq!(status.state, ProtocolState::Idle);
}
