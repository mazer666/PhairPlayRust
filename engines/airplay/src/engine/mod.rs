//! The AirPlay engine: the state machine and the sockets.
//!
//! WHY: the engine is the piece that owns the *lifetime* of an AirPlay
//! session: it binds the ports, accepts the sender, drives the RTSP state
//! machine and hands the media to the platform layer. Two rules from the
//! build contract shape it:
//!
//! * **One peer at a time.** AirPlay is a single-session protocol. A second
//!   sender is answered with `503 Service Unavailable` and disconnected — it
//!   is not queued.
//! * **The status is always available.** [`ProtocolEngine::status`] may be
//!   called from any thread while a transition is in flight.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_airplay::{AirPlayConfig, AirPlayEngine};
//! use phairplay_core::protocol::{ProtocolEngine, ProtocolKind, ProtocolState};
//!
//! let engine = AirPlayEngine::new(AirPlayConfig::default());
//! assert_eq!(engine.kind(), ProtocolKind::AirPlay);
//! assert_eq!(engine.status().state, ProtocolState::Idle);
//! ```

use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use phairplay_core::error::EngineError;
use phairplay_core::protocol::{
    EngineContext, ProtocolEngine, ProtocolKind, ProtocolState, ProtocolStatus,
};

use tokio::net::TcpListener;
use tokio::task::JoinHandle;

use crate::error::AirPlayError;
use crate::limits::AirPlayLimits;
use crate::pairing::identity::PairingIdentity;
use crate::pairing::store::PairingStore;
use crate::photo::PhotoPayload;

use crate::timing::TimingResponder;

/// The TCP port the AirPlay control channel listens on.
///
/// WHY: the value is the port every AirPlay sender looks for. Its single
/// owner is [`phairplay_core::config::ports::AIRPLAY_RTSP`].
pub const DEFAULT_RTSP_PORT: u16 = phairplay_core::config::ports::AIRPLAY_RTSP;

/// The number of attempts to bind the control port.
/// The number of attempts to bind the control port.
pub const BIND_ATTEMPTS: u32 = 12;

/// The pause between two bind attempts.
/// The pause between two bind attempts.
pub const BIND_RETRY: Duration = Duration::from_millis(250);

/// The configuration of the AirPlay engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AirPlayConfig {
    /// The resource limits of the engine.
    pub limits: AirPlayLimits,
    /// The TCP port of the control channel; `0` picks a free port.
    pub port: u16,
    /// The UDP port of the timing service; `0` picks a free port.
    pub timing_port: u16,
    /// Whether a PIN is required before a sender may stream.
    pub require_pin: bool,
    /// The width the receiver advertises.
    pub display_width: u32,
    /// The height the receiver advertises.
    pub display_height: u32,
    /// Whether the mirrored audio is accepted.
    pub audio_enabled: bool,
    /// Whether the URL playback mode is accepted.
    pub playback_enabled: bool,
}

impl Default for AirPlayConfig {
    fn default() -> Self {
        Self {
            limits: AirPlayLimits::default(),
            port: DEFAULT_RTSP_PORT,
            timing_port: crate::rtsp::http::TIMING_PORT,
            require_pin: false,
            display_width: 1920,
            display_height: 1080,
            audio_enabled: true,
            playback_enabled: true,
        }
    }
}

impl AirPlayConfig {
    /// Reads the configuration out of the application configuration.
    #[must_use]
    pub fn from_app_config(config: &phairplay_core::config::AppConfig) -> Self {
        let protocol = config.protocol(ProtocolKind::AirPlay);
        let mut limits = AirPlayLimits::from_config(config);
        limits.max_message_bytes = protocol.max_message_bytes.max(1024);
        Self {
            limits,
            port: protocol.port,
            timing_port: crate::rtsp::http::TIMING_PORT,
            require_pin: protocol.require_pin,
            display_width: 1920,
            display_height: 1080,
            audio_enabled: true,
            playback_enabled: true,
        }
    }
}

#[derive(Debug, Default)]
struct EngineState {
    state: Option<ProtocolState>,
    tcp_port: Option<u16>,
    udp_port: Option<u16>,
    peer_count: u32,
    last_error: Option<String>,
}

impl EngineState {
    fn state(&self) -> ProtocolState {
        self.state.unwrap_or(ProtocolState::Idle)
    }
}

/// The shared state of the engine.
#[derive(Debug, Default)]
struct EngineInner {
    state: Mutex<EngineState>,
    photos: Mutex<Option<PhotoPayload>>,
    ports: Mutex<Option<Arc<MirrorPorts>>>,
    accept_task: Mutex<Option<JoinHandle<()>>>,
    timing_task: Mutex<Option<JoinHandle<()>>>,
    dacp: Mutex<crate::dacp::DacpClient>,
    pairing_store: Mutex<Option<Arc<dyn PairingStore>>>,
}

/// The AirPlay 2 receiver engine.
#[derive(Debug)]
pub struct AirPlayEngine {
    config: AirPlayConfig,
    identity: Arc<PairingIdentity>,
    inner: Arc<EngineInner>,
}

impl AirPlayEngine {
    /// Creates an engine with the given configuration.
    #[must_use]
    pub fn new(config: AirPlayConfig) -> Self {
        Self {
            config,
            identity: Arc::new(PairingIdentity::from_seed(&[0u8; 32])),
            inner: Arc::new(EngineInner::default()),
        }
    }

    /// Creates an engine with a persistent identity.
    #[must_use]
    pub fn with_identity(config: AirPlayConfig, identity: Arc<PairingIdentity>) -> Self {
        Self {
            config,
            identity,
            inner: Arc::new(EngineInner::default()),
        }
    }

    /// The configuration of the engine.
    #[must_use]
    pub fn config(&self) -> &AirPlayConfig {
        &self.config
    }

    /// The store of the paired controllers.
    pub fn set_pairing_store(&self, store: Arc<dyn PairingStore>) {
        if let Ok(mut guard) = self.inner.pairing_store.lock() {
            *guard = Some(store);
        }
    }

    /// The data ports of the mirrored streams that are currently allocated.
    #[must_use]
    pub fn allocated_stream_ports(&self) -> Vec<(u8, u16)> {
        self.inner
            .ports
            .lock()
            .ok()
            .and_then(|guard| guard.as_ref().map(|ports| ports.allocated_ports()))
            .unwrap_or_default()
    }

    /// The last photo the sender shared.
    #[must_use]
    pub fn latest_photo(&self) -> Option<PhotoPayload> {
        self.inner
            .photos
            .lock()
            .ok()
            .and_then(|guard| guard.clone())
    }

    /// The DACP client of the reverse control channel.
    #[must_use]
    pub fn dacp_client(&self) -> crate::dacp::DacpClient {
        self.inner
            .dacp
            .lock()
            .map(|guard| {
                let mut client = crate::dacp::DacpClient::new();
                if let Some(identity) = guard.identity() {
                    client.configure(Some(&identity.dacp_id), Some(&identity.active_remote));
                }
                if let Some((host, port)) = guard.endpoint() {
                    client.set_endpoint(host, port);
                }
                client
            })
            .unwrap_or_default()
    }

    /// The state of the engine, readable from any thread.
    #[must_use]
    pub fn status(&self) -> ProtocolStatus {
        let Ok(state) = self.inner.state.lock() else {
            return ProtocolStatus {
                protocol: ProtocolKind::AirPlay,
                state: ProtocolState::Failed,
                enabled: true,
                tcp_port: None,
                udp_port: None,
                peer_count: 0,
                last_error: Some("the engine state is poisoned".to_string()),
            };
        };
        ProtocolStatus {
            protocol: ProtocolKind::AirPlay,
            state: state.state(),
            enabled: true,
            tcp_port: state.tcp_port,
            udp_port: state.udp_port,
            peer_count: state.peer_count,
            last_error: state.last_error.clone(),
        }
    }

    /// Binds the control port, retrying while the port is still in use.
    async fn bind_listener(&self) -> Result<TcpListener, AirPlayError> {
        let mut last_error = None;
        for attempt in 0..BIND_ATTEMPTS {
            match TcpListener::bind(("0.0.0.0", self.config.port)).await {
                Ok(listener) => return Ok(listener),
                Err(error) => {
                    tracing::warn!(
                        attempt,
                        %error,
                        "the AirPlay control port is busy — retrying"
                    );
                    last_error = Some(error);
                    tokio::time::sleep(BIND_RETRY).await;
                }
            }
        }
        Err(AirPlayError::Io(last_error.unwrap_or_else(|| {
            std::io::Error::other("the AirPlay control port could not be bound")
        })))
    }
}

#[async_trait]
impl ProtocolEngine for AirPlayEngine {
    fn kind(&self) -> ProtocolKind {
        ProtocolKind::AirPlay
    }

    fn status(&self) -> ProtocolStatus {
        AirPlayEngine::status(self)
    }

    async fn start(&self, ctx: EngineContext) -> Result<(), EngineError> {
        {
            let Ok(mut state) = self.inner.state.lock() else {
                return Err(EngineError::Start {
                    protocol: "airplay",
                    reason: "the engine state is poisoned".to_string(),
                });
            };
            match state.state() {
                ProtocolState::Idle | ProtocolState::Stopped => {}
                _ => {
                    return Err(EngineError::AlreadyRunning {
                        protocol: "airplay",
                    });
                }
            }
            state.state = Some(ProtocolState::Starting);
            state.last_error = None;
        }

        let listener = match self.bind_listener().await {
            Ok(listener) => listener,
            Err(error) => {
                self.fail(&error.to_string());
                return Err(EngineError::Start {
                    protocol: "airplay",
                    reason: error.to_string(),
                });
            }
        };
        let local_port = listener.local_addr().map(|addr| addr.port()).ok();

        let ports = Arc::new(MirrorPorts::new(
            ctx.video_sink.clone(),
            ctx.audio_sink.clone(),
            self.config.limits.clone(),
        ));
        if let Ok(mut guard) = self.inner.ports.lock() {
            *guard = Some(ports.clone());
        }

        let accept_task = tokio::spawn(accept_loop(
            listener,
            self.inner.clone(),
            self.config.clone(),
            ctx.clone(),
            self.identity.clone(),
            ports,
        ));

        let timing_port = self.start_timing(&ctx);

        if let Ok(mut state) = self.inner.state.lock() {
            state.state = Some(ProtocolState::Advertising);
            state.tcp_port = local_port;
            state.udp_port = timing_port;
            state.peer_count = 0;
        }
        if let Ok(mut guard) = self.inner.accept_task.lock() {
            *guard = Some(accept_task);
        }
        Ok(())
    }

    async fn stop(&self) -> Result<(), EngineError> {
        {
            let Ok(mut state) = self.inner.state.lock() else {
                return Err(EngineError::NotRunning {
                    protocol: "airplay",
                });
            };
            match state.state() {
                ProtocolState::Idle | ProtocolState::Stopped => {
                    return Err(EngineError::NotRunning {
                        protocol: "airplay",
                    });
                }
                _ => state.state = Some(ProtocolState::Stopping),
            }
        }
        if let Ok(mut guard) = self.inner.accept_task.lock() {
            if let Some(task) = guard.take() {
                task.abort();
            }
        }
        if let Ok(mut guard) = self.inner.timing_task.lock() {
            if let Some(task) = guard.take() {
                task.abort();
            }
        }
        if let Ok(mut guard) = self.inner.ports.lock() {
            *guard = None;
        }
        if let Ok(mut state) = self.inner.state.lock() {
            state.state = Some(ProtocolState::Stopped);
            state.peer_count = 0;
            state.tcp_port = None;
            state.udp_port = None;
        }
        Ok(())
    }
}

impl AirPlayEngine {
    /// Records a failure in the state.
    fn fail(&self, reason: &str) {
        if let Ok(mut state) = self.inner.state.lock() {
            state.state = Some(ProtocolState::Failed);
            state.last_error = Some(reason.to_string());
        }
    }

    /// Starts the NTP timing service.
    fn start_timing(&self, _ctx: &EngineContext) -> Option<u16> {
        let port = self.config.timing_port;
        let socket = match std::net::UdpSocket::bind(("0.0.0.0", port)) {
            Ok(socket) => socket,
            Err(error) => {
                tracing::warn!(%error, "the AirPlay timing port could not be bound");
                return None;
            }
        };
        let local_port = socket.local_addr().ok().map(|addr| addr.port());
        if let Err(error) = socket.set_nonblocking(true) {
            tracing::warn!(%error, "the timing socket could not be switched to non-blocking");
            return None;
        }
        let socket = match tokio::net::UdpSocket::from_std(socket) {
            Ok(socket) => socket,
            Err(error) => {
                tracing::warn!(%error, "the timing socket could not be registered");
                return None;
            }
        };
        let task = tokio::spawn(timing_loop(socket));
        if let Ok(mut guard) = self.inner.timing_task.lock() {
            *guard = Some(task);
        }
        local_port
    }
}

/// The NTP timing service.
async fn timing_loop(socket: tokio::net::UdpSocket) {
    let responder = TimingResponder::new();
    let mut buffer = [0u8; 64];
    loop {
        let (read, peer) = match socket.recv_from(&mut buffer).await {
            Ok(result) => result,
            Err(error) => {
                tracing::warn!(%error, "the timing service failed");
                return;
            }
        };
        let receive = TimingResponder::current_ntp_timestamp();
        match responder.handle_probe(&buffer[..read], receive) {
            Ok(Some(response)) => {
                let _ = socket.send_to(&response, peer).await;
            }
            Ok(None) => {}
            Err(error) => {
                tracing::warn!(%error, "a timing probe could not be answered");
            }
        }
    }
}

mod connection;
mod formats;
mod ports;

use connection::accept_loop;
pub use formats::{audio_format_of, video_format_of};
use ports::MirrorPorts;

#[cfg(test)]
mod tests;
