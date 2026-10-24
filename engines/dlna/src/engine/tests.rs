#![allow(clippy::expect_used, clippy::unwrap_used)]

use super::*;
use phairplay_core::capabilities::MediaCapabilities;
use phairplay_core::identity::DeviceIdentity;
use phairplay_core::media::{NullAudioSink, NullVideoSink};

fn context() -> EngineContext {
    let (sender, _receiver) = tokio::sync::mpsc::channel(16);
    EngineContext {
        identity: Arc::new(DeviceIdentity::with_name("Living Room")),
        capabilities: Arc::new(MediaCapabilities::empty()),
        video_sink: Arc::new(NullVideoSink),
        audio_sink: Arc::new(NullAudioSink),
        events: sender,
    }
}

fn config() -> DlnaConfig {
    DlnaConfig::default()
        .with_ports(0, 0)
        .with_local_ip(Ipv4Addr::new(127, 0, 0, 1))
}

#[tokio::test]
async fn the_engine_reports_its_kind_and_state() {
    let engine = DlnaEngine::new(config());
    assert_eq!(engine.kind(), ProtocolKind::Dlna);
    let status = engine.status();
    assert_eq!(status.protocol, ProtocolKind::Dlna);
    assert_eq!(status.state, ProtocolState::Idle);
    assert!(status.tcp_port.is_none());
}

#[tokio::test]
async fn the_engine_starts_and_stops() {
    let engine = DlnaEngine::new(config());
    engine.start(context()).await.expect("the engine starts");
    let status = engine.status();
    assert_eq!(status.state, ProtocolState::Advertising);
    assert!(status.tcp_port.is_some());
    assert!(status.udp_port.is_some());
    assert!(engine.description_url().is_some());

    engine.stop().await.expect("the engine stops");
    let status = engine.status();
    assert_eq!(status.state, ProtocolState::Stopped);
    assert!(status.tcp_port.is_none());
}

#[tokio::test]
async fn starting_twice_is_refused() {
    let engine = DlnaEngine::new(config());
    engine.start(context()).await.expect("the engine starts");
    let error = engine
        .start(context())
        .await
        .expect_err("the second start fails");
    assert!(matches!(error, EngineError::AlreadyRunning { .. }));
    engine.stop().await.expect("the engine stops");
}

#[tokio::test]
async fn stopping_an_engine_that_never_started_is_refused() {
    let engine = DlnaEngine::new(config());
    let error = engine.stop().await.expect_err("the stop fails");
    assert!(matches!(error, EngineError::NotRunning { .. }));
}

#[tokio::test]
async fn the_engine_can_be_started_again_after_a_stop() {
    let engine = DlnaEngine::new(config());
    engine.start(context()).await.expect("the engine starts");
    engine.stop().await.expect("the engine stops");
    engine
        .start(context())
        .await
        .expect("the engine starts again");
    assert_eq!(engine.status().state, ProtocolState::Advertising);
    engine.stop().await.expect("the engine stops");
}
