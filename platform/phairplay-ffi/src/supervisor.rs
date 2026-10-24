//! The supervisor of the protocol engines.
//!
//! WHAT: [`ReceiverCore`] owns the state of the receiver — the
//! configuration, the session registry, the counters, the engines and the
//! event channel — and it is the only place that starts, stops and watches
//! an engine.
//!
//! WHY: The native shells and the dashboard must see *the same* receiver.
//! If each of them kept its own state, the dashboard would show a protocol
//! as advertising while the phone showed it as idle. One supervisor, one
//! state, one source of truth.
//!
//! The supervisor runs its own multi-threaded Tokio runtime. The engines are
//! asynchronous, the hosts are not: a Kotlin or a Swift caller cannot wait
//! for a future. The runtime therefore belongs to the bridge, and the
//! exported methods block on it.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_ffi::supervisor::ReceiverCore;
//! use phairplay_core::config::AppConfig;
//!
//! let receiver = ReceiverCore::new(AppConfig::default())
//!     .unwrap_or_else(|error| panic!("the receiver must start: {error}"));
//! let protocol = "airplay"
//!     .parse()
//!     .unwrap_or_else(|_| panic!("airplay is a known protocol"));
//! let status = receiver.protocol_status(protocol);
//! assert_eq!(status.protocol.as_str(), "airplay");
//! ```

use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;

use phairplay_core::capabilities::MediaCapabilities;
use phairplay_core::config::AppConfig;
use phairplay_core::error::CoreError;
use phairplay_core::identity::DeviceIdentity;
use phairplay_core::metrics::{SharedCounters, StreamStats, new_counters};
use phairplay_core::protocol::{
    EngineContext, ProtocolEngine, ProtocolKind, ProtocolState, ProtocolStatus,
};
use phairplay_core::session::{SessionEvent, SessionRegistry, SessionSnapshot};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

use crate::callbacks::PlatformAudioSink;
use crate::callbacks::PlatformVideoSink;
use crate::dto::{FfiProtocolStatus, FfiReceiverEvent, FfiSessionSnapshot, FfiStreamStats};
use crate::error::FfiError;
use crate::listener::EventListener;
use crate::sinks::CountingAudioSink;
use crate::sinks::CountingVideoSink;

/// The number of events that may wait in the queue before the oldest one is
/// dropped.
///
/// WHY: The queue between an engine and the host must be bounded — a
/// receiver that grows a queue without limit is a denial of service that a
/// single misbehaving peer can trigger. The oldest event is the least
/// interesting one, so it is the one that goes.
const EVENT_QUEUE_CAPACITY: usize = 256;

/// The supervisor of the receiver.
#[derive(Clone)]
pub struct ReceiverCore {
    inner: Arc<Inner>,
}

struct Inner {
    /// The runtime of the bridge.
    ///
    /// WHY: it is an `Option`. Tokio refuses to drop a runtime inside an
    /// asynchronous context — the runtime wants to block the worker thread
    /// while it shuts its blocking pool down — and a receiver that is
    /// dropped by a host that runs its own event loop would panic. The
    /// `Drop` of `Inner` therefore hands the runtime to
    /// `shutdown_background`, which does not block.
    runtime: Option<Runtime>,
    config: RwLock<AppConfig>,
    identity: RwLock<DeviceIdentity>,
    capabilities: RwLock<MediaCapabilities>,
    sessions: Arc<RwLock<SessionRegistry>>,
    counters: Arc<RwLock<BTreeMap<ProtocolKind, SharedCounters>>>,
    engines: RwLock<BTreeMap<ProtocolKind, Arc<dyn ProtocolEngine>>>,
    listener: Arc<RwLock<Option<Arc<dyn EventListener>>>>,
    video_sink: Arc<RwLock<Option<Arc<dyn PlatformVideoSink>>>>,
    audio_sink: Arc<RwLock<Option<Arc<dyn PlatformAudioSink>>>>,
    events: mpsc::Sender<SessionEvent>,
    errors: RwLock<BTreeMap<ProtocolKind, String>>,
    started_at: Instant,
}

impl std::fmt::Debug for ReceiverCore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReceiverCore")
            .field("uptime_ms", &self.uptime_ms())
            .finish_non_exhaustive()
    }
}

impl Inner {
    /// The runtime of the bridge, while the receiver lives.
    ///
    /// WHY: the runtime is an `Option` because the `Drop` of `Inner` takes
    /// it out of the struct — a receiver that is dropped inside an
    /// asynchronous context must not block the thread of the host.
    fn runtime(&self) -> &Runtime {
        match self.runtime.as_ref() {
            Some(runtime) => runtime,
            None => panic!("the runtime of the bridge is already shut down"),
        }
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        if let Some(runtime) = self.runtime.take() {
            // WHY: `shutdown_background` does not block the thread that
            // drops the receiver, so the receiver may be dropped inside an
            // asynchronous context — inside a coroutine of a host, or
            // inside a test. A plain drop of the runtime would panic
            // there.
            runtime.shutdown_background();
        }
    }
}

impl ReceiverCore {
    /// Creates a receiver with the given configuration.
    ///
    /// # Errors
    /// Returns [`FfiError`] when the configuration is invalid or when the
    /// runtime of the bridge cannot be created.
    pub fn new(config: AppConfig) -> Result<Self, FfiError> {
        config
            .validate()
            .map_err(|error| FfiError::from_core(&CoreError::Config(error)))?;

        let runtime = Runtime::new().map_err(|error| {
            FfiError::internal(format!("the runtime of the bridge failed: {error}"))
        })?;
        let (events, receiver) = mpsc::channel::<SessionEvent>(EVENT_QUEUE_CAPACITY);

        let identity = DeviceIdentity {
            name: config.device.name.clone(),
            model: config.device.model.clone(),
            manufacturer: config.device.manufacturer.clone(),
            protocol_version: config.device.protocol_version.clone(),
            ..DeviceIdentity::default()
        };

        let inner = Arc::new(Inner {
            runtime: Some(runtime),
            config: RwLock::new(config),
            identity: RwLock::new(identity),
            capabilities: RwLock::new(MediaCapabilities::empty()),
            sessions: Arc::new(RwLock::new(SessionRegistry::new())),
            counters: Arc::new(RwLock::new(BTreeMap::new())),
            engines: RwLock::new(BTreeMap::new()),
            listener: Arc::new(RwLock::new(None)),
            video_sink: Arc::new(RwLock::new(None)),
            audio_sink: Arc::new(RwLock::new(None)),
            events,
            errors: RwLock::new(BTreeMap::new()),
            started_at: Instant::now(),
        });

        spawn_dispatcher(&inner, receiver);
        Ok(ReceiverCore { inner })
    }

    /// The configuration of the receiver.
    #[must_use]
    pub fn config(&self) -> AppConfig {
        read(&self.inner.config).clone()
    }

    /// Replaces the configuration of the receiver.
    ///
    /// # Errors
    /// Returns [`FfiError`] when the configuration is invalid.
    ///
    /// WHY: The configuration is only accepted when it is valid. A receiver
    /// that runs with a broken limit is a receiver that can be stopped by a
    /// peer on the local network.
    pub fn set_config(&self, config: AppConfig) -> Result<(), FfiError> {
        config
            .validate()
            .map_err(|error| FfiError::from_core(&CoreError::Config(error)))?;
        {
            let mut identity = write(&self.inner.identity);
            identity.name = config.device.name.clone();
            identity.model = config.device.model.clone();
            identity.manufacturer = config.device.manufacturer.clone();
            identity.protocol_version = config.device.protocol_version.clone();
        }
        *write(&self.inner.config) = config;
        Ok(())
    }

    /// The identity the receiver advertises to senders.
    #[must_use]
    pub fn identity(&self) -> DeviceIdentity {
        read(&self.inner.identity).clone()
    }

    /// The media capabilities of the receiver.
    #[must_use]
    pub fn capabilities(&self) -> MediaCapabilities {
        read(&self.inner.capabilities).clone()
    }

    /// Sets the media capabilities of the receiver.
    pub fn set_capabilities(&self, capabilities: MediaCapabilities) {
        *write(&self.inner.capabilities) = capabilities;
    }

    /// Registers the engine of a protocol.
    ///
    /// WHY: The engine implementations live in the `engines/*` crates of the
    /// workspace. The bridge does not know them, and it must not guess: a
    /// protocol without an engine reports the structured error
    /// [`FfiErrorCode::Unsupported`](crate::error::FfiErrorCode::Unsupported)
    /// instead of pretending that it is advertising.
    pub fn register_engine(&self, engine: Arc<dyn ProtocolEngine>) {
        let kind = engine.kind();
        write(&self.inner.engines).insert(kind, engine);
    }

    /// The engine of a protocol, if one is registered.
    #[must_use]
    pub fn engine(&self, protocol: ProtocolKind) -> Option<Arc<dyn ProtocolEngine>> {
        read(&self.inner.engines).get(&protocol).cloned()
    }

    /// The status of every protocol, in the order of the core.
    #[must_use]
    pub fn status(&self) -> Vec<ProtocolStatus> {
        ProtocolKind::ALL
            .iter()
            .map(|kind| self.protocol_status(*kind))
            .collect()
    }

    /// The status of one protocol.
    #[must_use]
    pub fn protocol_status(&self, protocol: ProtocolKind) -> ProtocolStatus {
        match self.engine(protocol) {
            Some(engine) => {
                let mut status = engine.status();
                // WHY: "enabled" is a fact of the *configuration*, not of the
                // engine. An engine reports what it was told when it was
                // built, but the configuration is the authority — a protocol
                // that the user switched off reads *disabled*, whatever the
                // engine remembers.
                status.enabled = self.config().is_enabled(protocol);
                if status.last_error.is_none() {
                    status.last_error = self.last_error(protocol);
                }
                status
            }
            None => ProtocolStatus {
                protocol,
                state: ProtocolState::Idle,
                enabled: self.config().is_enabled(protocol),
                tcp_port: None,
                udp_port: None,
                peer_count: 0,
                last_error: self.last_error(protocol),
            },
        }
    }

    /// Starts the engine of a protocol.
    ///
    /// # Errors
    /// Returns [`FfiError`] when the protocol is disabled in the
    /// configuration, when no engine is registered for it, or when the
    /// engine itself refuses to start.
    pub fn start_protocol(&self, protocol: ProtocolKind) -> Result<ProtocolStatus, FfiError> {
        if !self.config().is_enabled(protocol) {
            return Err(FfiError::invalid_state(format!(
                "the protocol `{protocol}` is disabled in the configuration"
            )));
        }
        let Some(engine) = self.engine(protocol) else {
            let failure = FfiError::unsupported(format!(
                "no engine implementation is registered for the protocol `{protocol}`"
            ));
            self.record_error(protocol, failure.details());
            self.emit_status(protocol);
            return Err(failure);
        };

        let context = self.engine_context(protocol);
        let result = self.block_on(async move { engine.start(context).await });
        match result {
            Ok(()) => {
                self.clear_error(protocol);
                let status = self.protocol_status(protocol);
                self.emit_status(protocol);
                Ok(status)
            }
            Err(error) => {
                let failure = FfiError::from_engine(error);
                self.record_error(protocol, failure.details());
                self.emit_status(protocol);
                Err(failure)
            }
        }
    }

    /// Stops the engine of a protocol.
    ///
    /// # Errors
    /// Returns [`FfiError`] when no engine is registered for the protocol
    /// or when the engine refuses to stop.
    pub fn stop_protocol(&self, protocol: ProtocolKind) -> Result<ProtocolStatus, FfiError> {
        let Some(engine) = self.engine(protocol) else {
            return Err(FfiError::unsupported(format!(
                "no engine implementation is registered for the protocol `{protocol}`"
            )));
        };
        let result = self.block_on(async { engine.stop().await });
        match result {
            Ok(()) => {
                self.clear_error(protocol);
                let status = self.protocol_status(protocol);
                self.emit_status(protocol);
                Ok(status)
            }
            Err(error) => {
                let failure = FfiError::from_engine(error);
                self.record_error(protocol, failure.details());
                self.emit_status(protocol);
                Err(failure)
            }
        }
    }

    /// The live sessions of the receiver.
    #[must_use]
    pub fn sessions(&self) -> Vec<SessionSnapshot> {
        read(&self.inner.sessions)
            .sessions()
            .cloned()
            .collect::<Vec<SessionSnapshot>>()
    }

    /// The counters of one protocol.
    #[must_use]
    pub fn stats(&self, protocol: ProtocolKind) -> StreamStats {
        match read(&self.inner.counters).get(&protocol) {
            Some(counters) => counters.snapshot(),
            None => StreamStats::default(),
        }
    }

    /// The counters of every protocol, in the order of the core.
    #[must_use]
    pub fn all_stats(&self) -> Vec<(ProtocolKind, StreamStats)> {
        ProtocolKind::ALL
            .iter()
            .map(|kind| (*kind, self.stats(*kind)))
            .collect()
    }

    /// Registers the listener that receives the live events.
    pub fn set_listener(&self, listener: Arc<dyn EventListener>) {
        *write(&self.inner.listener) = Some(listener);
    }

    /// Removes the listener of the live events.
    pub fn clear_listener(&self) {
        *write(&self.inner.listener) = None;
    }

    /// Registers the decoder of the platform for the video frames.
    pub fn set_video_sink(&self, sink: Arc<dyn PlatformVideoSink>) {
        *write(&self.inner.video_sink) = Some(sink);
    }

    /// Registers the decoder of the platform for the audio packets.
    pub fn set_audio_sink(&self, sink: Arc<dyn PlatformAudioSink>) {
        *write(&self.inner.audio_sink) = Some(sink);
    }

    /// How long the receiver has been running, in milliseconds.
    #[must_use]
    pub fn uptime_ms(&self) -> u64 {
        self.inner.started_at.elapsed().as_millis() as u64
    }

    /// The reason of the last failure of a protocol, if there was one.
    #[must_use]
    pub fn last_error(&self, protocol: ProtocolKind) -> Option<String> {
        read(&self.inner.errors).get(&protocol).cloned()
    }

    /// The status of one protocol, as it travels over the bridge.
    #[must_use]
    pub fn ffi_status(&self, protocol: ProtocolKind) -> FfiProtocolStatus {
        FfiProtocolStatus::from(&self.protocol_status(protocol))
    }

    /// The live sessions, as they travel over the bridge.
    #[must_use]
    pub fn ffi_sessions(&self) -> Vec<FfiSessionSnapshot> {
        self.sessions()
            .iter()
            .map(FfiSessionSnapshot::from)
            .collect()
    }

    /// The counters of one protocol, as they travel over the bridge.
    #[must_use]
    pub fn ffi_stats(&self, protocol: ProtocolKind) -> FfiStreamStats {
        FfiStreamStats::from(&self.stats(protocol))
    }

    fn engine_context(&self, protocol: ProtocolKind) -> EngineContext {
        let counters = {
            let mut counters = write(&self.inner.counters);
            counters
                .entry(protocol)
                .or_insert_with(new_counters)
                .clone()
        };
        EngineContext {
            identity: Arc::new(self.identity()),
            capabilities: Arc::new(self.capabilities()),
            video_sink: Arc::new(CountingVideoSink::new(
                Arc::clone(&self.inner.video_sink),
                counters.clone(),
            )),
            audio_sink: Arc::new(CountingAudioSink::new(
                Arc::clone(&self.inner.audio_sink),
                counters,
            )),
            events: self.inner.events.clone(),
        }
    }

    fn record_error(&self, protocol: ProtocolKind, details: &str) {
        write(&self.inner.errors).insert(protocol, details.to_string());
    }

    fn clear_error(&self, protocol: ProtocolKind) {
        write(&self.inner.errors).remove(&protocol);
    }

    fn emit_status(&self, protocol: ProtocolKind) {
        let event = FfiReceiverEvent::ProtocolStatusChanged {
            status: self.ffi_status(protocol),
        };
        if let Some(listener) = read(&self.inner.listener).clone() {
            listener.on_event(&event);
        }
    }

    fn block_on<F: std::future::Future>(&self, future: F) -> F::Output {
        match tokio::runtime::Handle::try_current() {
            // The caller is already inside a runtime: leave the worker
            // thread before blocking, so that the other workers keep going.
            Ok(handle) => tokio::task::block_in_place(|| handle.block_on(future)),
            // The caller is an ordinary thread: the runtime of the bridge
            // does the work.
            Err(_) => self.inner.runtime().block_on(future),
        }
    }
}

/// Reads a lock, even when the lock is poisoned.
///
/// WHY: A poisoned lock means that a thread died while it held the lock. The
/// data behind it is still consistent — the guard is only a flag — and
/// refusing to answer because of an unrelated panic would take the whole
/// receiver down. The state is therefore read anyway.
fn read<T>(lock: &RwLock<T>) -> std::sync::RwLockReadGuard<'_, T> {
    match lock.read() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// Writes a lock, even when the lock is poisoned.
fn write<T>(lock: &RwLock<T>) -> std::sync::RwLockWriteGuard<'_, T> {
    match lock.write() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// Starts the task that folds the events of the engines into the state.
fn spawn_dispatcher(inner: &Arc<Inner>, mut events: mpsc::Receiver<SessionEvent>) {
    let sessions = Arc::clone(&inner.sessions);
    let listener = Arc::clone(&inner.listener);
    inner.runtime().spawn(async move {
        while let Some(event) = events.recv().await {
            // WHY: A panic in a foreign callback must not take the
            // dispatcher — and with it every future event — down.
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                write(&sessions).apply(&event);
                let converted = FfiReceiverEvent::from(&event);
                if let Some(listener) = read(&listener).clone() {
                    listener.on_event(&converted);
                }
            }));
            if outcome.is_err() {
                tracing::error!("a receiver event could not be delivered");
            }
        }
    });
}
