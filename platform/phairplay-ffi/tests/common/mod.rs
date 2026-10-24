//! The helpers that the tests of the bridge share.
//!
//! WHY: A test of the bridge needs a protocol engine — something that can be
//! started, that can refuse to start, and that reports a session. The real
//! engines live in the `engines/*` crates and bind sockets; a test of the
//! *bridge* must not depend on a socket, a peer or a network. The engine of
//! this module is therefore a real implementation of the trait of the core
//! that does its work in memory.

use std::sync::{Arc, Mutex};
use std::time::Instant;

use phairplay_core::error::{EngineError, MediaError};
use phairplay_core::media::EncodedVideoFrame;
use phairplay_core::protocol::{
    EngineContext, ProtocolEngine, ProtocolKind, ProtocolState, ProtocolStatus,
};
use phairplay_core::session::{SessionEvent, SessionId, SessionSnapshot};

/// Unwraps the result of a call in a test.
///
/// WHY: the workspace denies `unwrap()` and `expect()`. A test that fails
/// has to say *why* it failed, and a panic has to be a deliberate decision
/// instead of a habit.
pub trait Must<T> {
    /// Unwraps the value or fails with the given context.
    fn must(self, context: &str) -> T;
}

impl<T, E: std::fmt::Debug> Must<T> for Result<T, E> {
    fn must(self, context: &str) -> T {
        match self {
            Ok(value) => value,
            Err(error) => panic!("{context}: {error:?}"),
        }
    }
}

/// Unwraps the error of a call that had to fail in a test.
pub trait MustErr<T, E> {
    /// Unwraps the error or fails with the given context.
    fn must_err(self, context: &str) -> E;
}

impl<T: std::fmt::Debug, E: std::fmt::Debug> MustErr<T, E> for Result<T, E> {
    fn must_err(self, context: &str) -> E {
        match self {
            Ok(value) => {
                panic!("{context}: the call was expected to fail, but it returned {value:?}")
            }
            Err(error) => error,
        }
    }
}

impl<T> Must<T> for Option<T> {
    fn must(self, context: &str) -> T {
        match self {
            Some(value) => value,
            None => panic!("{context}: the value was missing"),
        }
    }
}

/// A protocol engine that the tests can drive.
#[derive(Debug)]
pub struct TestEngine {
    kind: ProtocolKind,
    state: Mutex<ProtocolState>,
    fail_start: Mutex<bool>,
    context: Mutex<Option<EngineContext>>,
    sessions: Mutex<Vec<SessionSnapshot>>,
}

impl TestEngine {
    /// Creates an engine that starts successfully.
    #[must_use]
    pub fn new(kind: ProtocolKind) -> Arc<Self> {
        Arc::new(TestEngine {
            kind,
            state: Mutex::new(ProtocolState::Idle),
            fail_start: Mutex::new(false),
            context: Mutex::new(None),
            sessions: Mutex::new(Vec::new()),
        })
    }

    /// Creates an engine that refuses to start.
    #[must_use]
    pub fn failing(kind: ProtocolKind) -> Arc<Self> {
        let engine = TestEngine::new(kind);
        if let Ok(mut fail) = engine.fail_start.lock() {
            *fail = true;
        }
        engine
    }

    /// The current state of the engine.
    #[must_use]
    pub fn state(&self) -> ProtocolState {
        match self.state.lock() {
            Ok(state) => *state,
            Err(poisoned) => *poisoned.into_inner(),
        }
    }

    /// The sessions the engine has reported.
    #[must_use]
    pub fn sessions(&self) -> Vec<SessionSnapshot> {
        match self.sessions.lock() {
            Ok(sessions) => sessions.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }

    /// Hands an encoded video frame to the sink of the platform.
    ///
    /// # Errors
    /// Returns [`MediaError::Closed`] when the engine was never started, so
    /// that there is no sink to hand the frame to.
    pub fn submit_video_frame(&self, payload: Vec<u8>) -> Result<(), MediaError> {
        let context = match self.context.lock() {
            Ok(context) => context.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        };
        match context {
            Some(context) => context
                .video_sink
                .submit(EncodedVideoFrame::new(0, payload)),
            None => Err(MediaError::Closed),
        }
    }
}

/// The engine as the trait object that the bridge expects.
///
/// WHY: The test keeps the concrete type, so that it can drive the engine;
/// the bridge only knows the trait.
#[must_use]
pub fn as_protocol_engine(engine: &Arc<TestEngine>) -> Arc<dyn ProtocolEngine> {
    Arc::clone(engine) as Arc<dyn ProtocolEngine>
}

#[async_trait::async_trait]
impl ProtocolEngine for TestEngine {
    fn kind(&self) -> ProtocolKind {
        self.kind
    }

    fn status(&self) -> ProtocolStatus {
        ProtocolStatus {
            protocol: self.kind,
            state: self.state(),
            enabled: true,
            tcp_port: Some(7000),
            udp_port: None,
            peer_count: 0,
            last_error: None,
        }
    }

    async fn start(&self, context: EngineContext) -> Result<(), EngineError> {
        let fails = match self.fail_start.lock() {
            Ok(fail) => *fail,
            Err(poisoned) => *poisoned.into_inner(),
        };
        if fails {
            return Err(EngineError::Start {
                protocol: self.kind.as_str(),
                reason: "the port is already in use".to_string(),
            });
        }

        if let Ok(mut state) = self.state.lock() {
            *state = ProtocolState::Advertising;
        }
        if let Ok(mut slot) = self.context.lock() {
            *slot = Some(context.clone());
        }

        let session = SessionSnapshot {
            id: SessionId::next(),
            protocol: self.kind,
            sender_name: Some("Test Sender".to_string()),
            sender_address: Some("192.168.1.50".to_string()),
            video: None,
            audio: None,
            started_at: Instant::now(),
        };
        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.push(session.clone());
        }
        let _ = context.events.send(SessionEvent::Started { session }).await;
        Ok(())
    }

    async fn stop(&self) -> Result<(), EngineError> {
        if let Ok(mut state) = self.state.lock() {
            *state = ProtocolState::Stopped;
        }
        if let Ok(mut slot) = self.context.lock() {
            *slot = None;
        }
        Ok(())
    }
}

/// A host that records the events the bridge delivers.
#[derive(Debug, Default)]
pub struct EventRecorder {
    events: Arc<Mutex<Vec<phairplay_ffi::dto::FfiReceiverEvent>>>,
}

impl EventRecorder {
    /// Creates an empty recorder.
    #[must_use]
    pub fn new() -> Arc<Self> {
        Arc::new(EventRecorder::default())
    }

    /// The callback interface the host hands to the bridge.
    #[must_use]
    pub fn sink(&self) -> Box<dyn phairplay_ffi::callbacks::ReceiverEventSink> {
        Box::new(EventRecorder {
            events: Arc::clone(&self.events),
        })
    }

    /// The events that were recorded.
    #[must_use]
    pub fn events(&self) -> Vec<phairplay_ffi::dto::FfiReceiverEvent> {
        match self.events.lock() {
            Ok(events) => events.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }
}

impl phairplay_ffi::callbacks::ReceiverEventSink for EventRecorder {
    fn on_event(&self, event: phairplay_ffi::dto::FfiReceiverEvent) {
        if let Ok(mut events) = self.events.lock() {
            events.push(event);
        }
    }
}

/// The state a decoder of the platform shares with its test.
#[derive(Debug, Default)]
pub struct DecoderState {
    frames: Mutex<Vec<u64>>,
    formats: Mutex<Vec<phairplay_ffi::dto::FfiVideoFormat>>,
    resets: Mutex<usize>,
}

/// A decoder of the platform that counts the frames it receives.
#[derive(Debug, Default)]
pub struct FakeVideoDecoder {
    state: Arc<DecoderState>,
}

impl FakeVideoDecoder {
    /// Creates a decoder that has not seen a frame.
    #[must_use]
    pub fn new() -> Arc<Self> {
        Arc::new(FakeVideoDecoder::default())
    }

    /// The sequences of the frames the decoder received.
    #[must_use]
    pub fn frames(&self) -> Vec<u64> {
        match self.state.frames.lock() {
            Ok(frames) => frames.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }

    /// The formats the decoder was told about.
    #[must_use]
    #[allow(dead_code)]
    pub fn formats(&self) -> Vec<phairplay_ffi::dto::FfiVideoFormat> {
        match self.state.formats.lock() {
            Ok(formats) => formats.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }

    /// How often the decoder was reset.
    #[must_use]
    #[allow(dead_code)]
    pub fn resets(&self) -> usize {
        match self.state.resets.lock() {
            Ok(resets) => *resets,
            Err(poisoned) => *poisoned.into_inner(),
        }
    }
}

/// The decoder as the callback interface that the bridge expects.
///
/// WHY: The test keeps a handle on the decoder to look at what it received;
/// the bridge only knows the callback interface. The two share their state.
#[must_use]
pub fn video_sink(
    decoder: &Arc<FakeVideoDecoder>,
) -> Box<dyn phairplay_ffi::callbacks::PlatformVideoSink> {
    Box::new(FakeVideoDecoder {
        state: Arc::clone(&decoder.state),
    })
}

impl phairplay_ffi::callbacks::PlatformVideoSink for FakeVideoDecoder {
    fn submit_frame(
        &self,
        frame: phairplay_ffi::dto::FfiEncodedVideoFrame,
    ) -> Result<(), phairplay_ffi::error::FfiError> {
        if let Ok(mut frames) = self.state.frames.lock() {
            frames.push(frame.sequence);
        }
        Ok(())
    }

    fn set_video_format(
        &self,
        format: phairplay_ffi::dto::FfiVideoFormat,
    ) -> Result<(), phairplay_ffi::error::FfiError> {
        if let Ok(mut formats) = self.state.formats.lock() {
            formats.push(format);
        }
        Ok(())
    }

    fn reset_video(&self) {
        if let Ok(mut resets) = self.state.resets.lock() {
            *resets += 1;
        }
    }
}

/// Waits until the recorder has seen at least `count` events.
///
/// # Panics
/// Panics when the events do not arrive within the timeout. A test that
/// waits for something that never comes must fail, not pass.
#[allow(dead_code)]
pub fn wait_for_events(
    recorder: &EventRecorder,
    count: usize,
    timeout: std::time::Duration,
) -> Vec<phairplay_ffi::dto::FfiReceiverEvent> {
    let deadline = std::time::Instant::now() + timeout;
    loop {
        let events = recorder.events();
        if events.len() >= count {
            return events;
        }
        if std::time::Instant::now() >= deadline {
            panic!(
                "only {} of {count} events arrived within {timeout:?}",
                events.len()
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

/// Waits until the recorder has seen an event that the predicate accepts.
///
/// # Panics
/// Panics when no matching event arrives within the timeout.
pub fn wait_for_event<F>(
    recorder: &EventRecorder,
    predicate: F,
    timeout: std::time::Duration,
) -> phairplay_ffi::dto::FfiReceiverEvent
where
    F: Fn(&phairplay_ffi::dto::FfiReceiverEvent) -> bool,
{
    let deadline = std::time::Instant::now() + timeout;
    loop {
        for event in recorder.events() {
            if predicate(&event) {
                return event;
            }
        }
        if std::time::Instant::now() >= deadline {
            panic!("no matching event arrived within {timeout:?}");
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}
