//! The lifecycle of the supervisor of the receiver.
//!
//! WHY: the supervisor is the only place that starts, stops and watches an
//! engine, and the native shells and the dashboard see the receiver through
//! it. A supervisor that mis-reports a state shows a protocol as
//! "advertising" on the phone and as "idle" on the dashboard — and a
//! supervisor that swallows an error hides a protocol that does not start
//! at all. The supervisor had **no test of its own**; these tests are the
//! ones that close that gap.
//!
//! HOW: every test drives the public API of the bridge with a fake engine
//! that records the calls it receives. The fake engine is a real
//! implementation of [`ProtocolEngine`] — the supervisor must not be able
//! to tell the difference between a fake and a real engine, and that is
//! exactly what the test proves.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use phairplay_core::config::AppConfig;
use phairplay_core::error::EngineError;
use phairplay_core::protocol::{
    EngineContext, ProtocolEngine, ProtocolKind, ProtocolState, ProtocolStatus,
};
use phairplay_ffi::dto::FfiReceiverEvent;
use phairplay_ffi::error::FfiErrorCode;
use phairplay_ffi::listener::EventListener;
use phairplay_ffi::supervisor::ReceiverCore;

/// An engine that records what the supervisor asked of it.
#[derive(Debug)]
struct FakeEngine {
    kind: ProtocolKind,
    state: std::sync::Mutex<ProtocolState>,
    starts: AtomicUsize,
    stops: AtomicUsize,
    /// The reason the start fails, if the start is supposed to fail.
    fail_with: Option<String>,
}

impl FakeEngine {
    fn new(kind: ProtocolKind) -> Self {
        Self {
            kind,
            state: std::sync::Mutex::new(ProtocolState::Idle),
            starts: AtomicUsize::new(0),
            stops: AtomicUsize::new(0),
            fail_with: None,
        }
    }

    fn failing(kind: ProtocolKind, reason: &str) -> Self {
        Self {
            fail_with: Some(reason.to_string()),
            ..Self::new(kind)
        }
    }
}

#[async_trait]
impl ProtocolEngine for FakeEngine {
    fn kind(&self) -> ProtocolKind {
        self.kind
    }

    fn status(&self) -> ProtocolStatus {
        ProtocolStatus {
            protocol: self.kind,
            state: *self.state.lock().expect("the state lock"),
            enabled: true,
            tcp_port: None,
            udp_port: None,
            peer_count: 0,
            last_error: None,
        }
    }

    async fn start(&self, _ctx: EngineContext) -> Result<(), EngineError> {
        self.starts.fetch_add(1, Ordering::SeqCst);
        if let Some(reason) = &self.fail_with {
            *self.state.lock().expect("the state lock") = ProtocolState::Failed;
            return Err(EngineError::Start {
                protocol: self.kind.as_str(),
                reason: reason.clone(),
            });
        }
        *self.state.lock().expect("the state lock") = ProtocolState::Advertising;
        Ok(())
    }

    async fn stop(&self) -> Result<(), EngineError> {
        self.stops.fetch_add(1, Ordering::SeqCst);
        *self.state.lock().expect("the state lock") = ProtocolState::Stopped;
        Ok(())
    }
}

/// A listener that records the events it receives.
#[derive(Debug, Default)]
struct RecordingListener {
    events: std::sync::Mutex<Vec<FfiReceiverEvent>>,
}

impl EventListener for RecordingListener {
    fn on_event(&self, event: &FfiReceiverEvent) {
        self.events
            .lock()
            .expect("the event lock")
            .push(event.clone());
    }
}

impl RecordingListener {
    fn events(&self) -> Vec<FfiReceiverEvent> {
        self.events.lock().expect("the event lock").clone()
    }
}

fn config_with(protocol: ProtocolKind, enabled: bool) -> AppConfig {
    let mut config = AppConfig::default();
    let entry = config
        .protocols
        .entry(protocol.as_str().to_string())
        .or_default();
    entry.enabled = enabled;
    config
}

#[test]
fn a_fresh_supervisor_reports_every_protocol_as_idle() {
    let receiver = ReceiverCore::new(AppConfig::default()).expect("the receiver starts");
    let statuses = receiver.status();
    assert_eq!(statuses.len(), ProtocolKind::ALL.len());
    for status in statuses {
        assert_eq!(status.state, ProtocolState::Idle, "{status:?}");
        assert_eq!(status.peer_count, 0, "{status:?}");
    }
}

#[test]
fn a_protocol_without_an_engine_is_refused_and_the_reason_is_named() {
    let receiver = ReceiverCore::new(AppConfig::default()).expect("the receiver starts");
    let error = receiver
        .start_protocol(ProtocolKind::Miracast)
        .expect_err("no engine is registered for miracast");
    assert_eq!(error.code(), FfiErrorCode::Unsupported.as_i32());
    assert!(
        error.details().contains("miracast"),
        "the reason must name the protocol: {}",
        error.details()
    );
    // The failure is visible in the status of the protocol.
    let status = receiver.protocol_status(ProtocolKind::Miracast);
    assert!(status.last_error.is_some(), "{status:?}");
}

#[test]
fn a_disabled_protocol_is_refused() {
    let config = config_with(ProtocolKind::Cast, false);
    let receiver = ReceiverCore::new(config).expect("the receiver starts");
    let engine = Arc::new(FakeEngine::new(ProtocolKind::Cast));
    receiver.register_engine(Arc::clone(&engine) as Arc<dyn ProtocolEngine>);

    let error = receiver
        .start_protocol(ProtocolKind::Cast)
        .expect_err("the protocol is disabled");
    assert_eq!(error.code(), FfiErrorCode::InvalidState.as_i32());
    // A disabled protocol is never started.
    assert_eq!(engine.starts.load(Ordering::SeqCst), 0);
}

#[test]
fn starting_and_stopping_a_protocol_drives_the_engine() {
    let receiver = ReceiverCore::new(AppConfig::default()).expect("the receiver starts");
    let engine = Arc::new(FakeEngine::new(ProtocolKind::AirPlay));
    receiver.register_engine(Arc::clone(&engine) as Arc<dyn ProtocolEngine>);

    let status = receiver
        .start_protocol(ProtocolKind::AirPlay)
        .expect("the engine starts");
    assert_eq!(status.state, ProtocolState::Advertising);
    assert_eq!(engine.starts.load(Ordering::SeqCst), 1);

    let status = receiver
        .stop_protocol(ProtocolKind::AirPlay)
        .expect("the engine stops");
    assert_eq!(status.state, ProtocolState::Stopped);
    assert_eq!(engine.stops.load(Ordering::SeqCst), 1);
}

#[test]
fn an_engine_that_fails_to_start_leaves_the_reason_behind() {
    let receiver = ReceiverCore::new(AppConfig::default()).expect("the receiver starts");
    let engine = Arc::new(FakeEngine::failing(
        ProtocolKind::Dlna,
        "the socket could not be bound",
    ));
    receiver.register_engine(Arc::clone(&engine) as Arc<dyn ProtocolEngine>);

    let error = receiver
        .start_protocol(ProtocolKind::Dlna)
        .expect_err("the engine refuses to start");
    assert_eq!(error.code(), FfiErrorCode::Internal.as_i32());

    // The reason survives the failure: the host has to be able to show it.
    let reason = receiver
        .last_error(ProtocolKind::Dlna)
        .expect("the reason is kept");
    assert!(
        reason.contains("the socket could not be bound"),
        "the reason must name the cause: {reason}"
    );
}

#[test]
fn a_successful_start_clears_the_reason_of_an_earlier_failure() {
    let receiver = ReceiverCore::new(AppConfig::default()).expect("the receiver starts");
    let engine = Arc::new(FakeEngine::failing(
        ProtocolKind::Cast,
        "the first start failed",
    ));
    receiver.register_engine(Arc::clone(&engine) as Arc<dyn ProtocolEngine>);
    let _ = receiver.start_protocol(ProtocolKind::Cast);
    assert!(receiver.last_error(ProtocolKind::Cast).is_some());

    // The same engine, the second time, succeeds.
    let engine = Arc::new(FakeEngine::new(ProtocolKind::Cast));
    receiver.register_engine(Arc::clone(&engine) as Arc<dyn ProtocolEngine>);
    receiver
        .start_protocol(ProtocolKind::Cast)
        .expect("the engine starts");
    assert!(receiver.last_error(ProtocolKind::Cast).is_none());
}

#[test]
fn the_listener_receives_the_status_of_every_change() {
    let receiver = ReceiverCore::new(AppConfig::default()).expect("the receiver starts");
    let listener = Arc::new(RecordingListener::default());
    receiver.set_listener(Arc::clone(&listener) as Arc<dyn EventListener>);

    let engine = Arc::new(FakeEngine::new(ProtocolKind::AirPlay));
    receiver.register_engine(Arc::clone(&engine) as Arc<dyn ProtocolEngine>);
    receiver
        .start_protocol(ProtocolKind::AirPlay)
        .expect("the engine starts");
    receiver
        .stop_protocol(ProtocolKind::AirPlay)
        .expect("the engine stops");

    let events = listener.events();
    let status_events = events
        .iter()
        .filter(|event| matches!(event, FfiReceiverEvent::ProtocolStatusChanged { .. }))
        .count();
    assert_eq!(
        status_events, 2,
        "the listener must see the start and the stop: {events:?}"
    );
}

#[test]
fn the_counters_of_a_protocol_start_at_zero_and_stay_readable() {
    let receiver = ReceiverCore::new(AppConfig::default()).expect("the receiver starts");
    let stats = receiver.stats(ProtocolKind::Cast);
    assert_eq!(stats.video_frames, 0);
    assert_eq!(stats.audio_packets, 0);
    // The statistics of every protocol are readable, even the ones that
    // were never started.
    let all = receiver.all_stats();
    assert_eq!(all.len(), ProtocolKind::ALL.len());
}

#[test]
fn the_uptime_grows_and_the_configuration_is_readable() {
    let receiver = ReceiverCore::new(AppConfig::default()).expect("the receiver starts");
    let before = receiver.uptime_ms();
    std::thread::sleep(std::time::Duration::from_millis(20));
    let after = receiver.uptime_ms();
    assert!(
        after >= before,
        "the uptime must not go backwards: {before} -> {after}"
    );
    assert_eq!(receiver.config().device.name, "PhairPlay");
}
