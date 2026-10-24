//! The Miracast engine: the state machine and the socket.
//!
//! WHY: Miracast is the protocol of a Windows laptop that wants to show its
//! picture on the television. The sender opens a RTSP connection to the sink,
//! negotiates, and then streams RTP. The engine owns the *lifetime* of that
//! session: it binds the port, accepts the sender, answers the negotiation
//! and reports the session to the rest of the system.
//!
//! HOW: the engine is a state machine — `Idle → Starting → Advertising →
//! Streaming → Stopping → Stopped` — and it is the only piece that owns the
//! socket. The protocol itself lives in [`crate::rtsp`], the information
//! element in [`crate::wfd`]. That split keeps the state machine small enough
//! to be obviously correct.
//!
//! ## Scope
//!
//! The engine implements the *control plane* of Miracast: the RTSP
//! negotiation and the session lifecycle. The RTP media path and the
//! Wi-Fi Direct group formation belong to the platform layer — the group
//! formation needs a system API that only the native shell has.

use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use phairplay_core::error::EngineError;
use phairplay_core::privacy::mask_socket_addr;
use phairplay_core::protocol::{
    EngineContext, ProtocolEngine, ProtocolKind, ProtocolState, ProtocolStatus,
};
use phairplay_core::session::SessionId;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

use crate::error::MiracastError;

/// The port the WFD RTSP control plane listens on.
///
/// WHY: the value is the port every WFD sender looks for. Its single owner
/// is [`phairplay_core::config::ports::MIRACAST_WFD`].
pub const DEFAULT_WFD_PORT: u16 = phairplay_core::config::ports::MIRACAST_WFD;

/// The configuration of the Miracast engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MiracastConfig {
    /// Whether the engine is enabled.
    pub enabled: bool,
    /// The port the control plane listens on. Zero asks the kernel for a free
    /// port, which is what the tests use.
    pub port: u16,
    /// The number of senders that may stream at the same time.
    ///
    /// WHY: Miracast is a single-session protocol. A second sender is
    /// refused with `503 Service Unavailable`, not queued.
    pub max_sessions: usize,
}

impl Default for MiracastConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            port: DEFAULT_WFD_PORT,
            max_sessions: 1,
        }
    }
}

/// The state the engine keeps between calls.
#[derive(Debug, Clone, Default)]
pub(crate) struct EngineState {
    pub(crate) state: Option<ProtocolState>,
    pub(crate) tcp_port: Option<u16>,
    pub(crate) last_error: Option<String>,
    pub(crate) peer_count: u32,
}

/// The shared state of the engine.
pub(crate) struct Inner {
    pub(crate) config: MiracastConfig,
    pub(crate) state: Mutex<EngineState>,
    pub(crate) running: AtomicBool,
    pub(crate) sessions: Mutex<HashMap<SessionId, String>>,
    pub(crate) session_counter: AtomicU64,
    /// The signal that tells every peer to close its connection.
    ///
    /// WHY: stopping the engine has to release every socket, not only the
    /// listener. A peer that waits for more data would otherwise keep the
    /// connection — and the runtime — alive.
    pub(crate) shutdown: tokio::sync::watch::Sender<bool>,
}

impl Inner {
    /// The next session identifier of this engine.
    pub(crate) fn next_session_index(&self) -> u64 {
        self.session_counter.fetch_add(1, Ordering::Relaxed) + 1
    }

    /// Whether the engine is still supposed to run.
    pub(crate) fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// A receiver that learns when the engine stops.
    pub(crate) fn shutdown_signal(&self) -> tokio::sync::watch::Receiver<bool> {
        self.shutdown.subscribe()
    }

    /// Tells every peer to close its connection.
    pub(crate) fn signal_shutdown(&self) {
        let _ = self.shutdown.send(true);
    }

    /// The number of senders that are connected.
    pub(crate) fn peer_count(&self) -> u32 {
        self.sessions
            .lock()
            .map(|sessions| sessions.len())
            .unwrap_or(0) as u32
    }
}

impl std::fmt::Debug for Inner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Inner")
            .field("config", &self.config)
            .field("state", &self.state)
            .field("running", &self.running)
            .finish()
    }
}

/// The Miracast (Wi-Fi Display) sink engine.
pub struct MiracastEngine {
    inner: Arc<Inner>,
    runtime: Mutex<Option<JoinHandle<()>>>,
}

impl std::fmt::Debug for MiracastEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MiracastEngine")
            .field("inner", &self.inner)
            .finish()
    }
}

impl MiracastEngine {
    /// Creates an engine with the given configuration.
    #[must_use]
    pub fn new(config: MiracastConfig) -> Self {
        Self {
            inner: Arc::new(Inner {
                config,
                state: Mutex::new(EngineState::default()),
                running: AtomicBool::new(false),
                sessions: Mutex::new(HashMap::new()),
                session_counter: AtomicU64::new(0),
                shutdown: tokio::sync::watch::Sender::new(false),
            }),
            runtime: Mutex::new(None),
        }
    }
}

#[async_trait::async_trait]
impl ProtocolEngine for MiracastEngine {
    fn kind(&self) -> ProtocolKind {
        ProtocolKind::Miracast
    }

    fn status(&self) -> ProtocolStatus {
        let state = self.inner.state.lock().ok();
        ProtocolStatus {
            protocol: ProtocolKind::Miracast,
            state: state
                .as_ref()
                .and_then(|state| state.state)
                .unwrap_or(ProtocolState::Idle),
            enabled: self.inner.config.enabled,
            tcp_port: state.as_ref().and_then(|state| state.tcp_port),
            udp_port: None,
            peer_count: self.inner.peer_count(),
            last_error: state.as_ref().and_then(|state| state.last_error.clone()),
        }
    }

    async fn start(&self, ctx: EngineContext) -> Result<(), EngineError> {
        {
            let mut state = self.inner.state.lock().map_err(|_| EngineError::Start {
                protocol: "miracast",
                reason: "the engine state is poisoned".to_string(),
            })?;
            match state.state.unwrap_or(ProtocolState::Idle) {
                ProtocolState::Idle | ProtocolState::Stopped => {
                    state.state = Some(ProtocolState::Starting);
                    state.last_error = None;
                }
                _ => {
                    return Err(EngineError::AlreadyRunning {
                        protocol: "miracast",
                    });
                }
            }
        }

        let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, self.inner.config.port));
        let listener = TcpListener::bind(address)
            .await
            .map_err(|error| EngineError::Start {
                protocol: "miracast",
                reason: format!("could not bind {address}: {error}"),
            })?;
        let local = listener.local_addr().map_err(|error| EngineError::Start {
            protocol: "miracast",
            reason: format!("could not read the local address: {error}"),
        })?;

        self.inner.running.store(true, Ordering::SeqCst);
        let _ = self.inner.shutdown.send(false);

        let inner = Arc::clone(&self.inner);
        let task = tokio::spawn(accept_loop(inner, listener, ctx));

        if let Ok(mut runtime) = self.runtime.lock() {
            *runtime = Some(task);
        }
        if let Ok(mut state) = self.inner.state.lock() {
            state.state = Some(ProtocolState::Advertising);
            state.tcp_port = Some(local.port());
            state.last_error = None;
        }
        Ok(())
    }

    async fn stop(&self) -> Result<(), EngineError> {
        {
            let mut state = self.inner.state.lock().map_err(|_| EngineError::Stopped {
                protocol: "miracast",
            })?;
            match state.state.unwrap_or(ProtocolState::Idle) {
                ProtocolState::Advertising | ProtocolState::Streaming => {
                    state.state = Some(ProtocolState::Stopping);
                }
                _ => {
                    return Err(EngineError::NotRunning {
                        protocol: "miracast",
                    });
                }
            }
        }

        self.inner.running.store(false, Ordering::SeqCst);
        self.inner.signal_shutdown();
        if let Ok(mut runtime) = self.runtime.lock() {
            if let Some(task) = runtime.take() {
                task.abort();
            }
        }
        if let Ok(mut sessions) = self.inner.sessions.lock() {
            sessions.clear();
        }
        if let Ok(mut state) = self.inner.state.lock() {
            state.state = Some(ProtocolState::Stopped);
            state.tcp_port = None;
            state.peer_count = 0;
        }
        Ok(())
    }
}

/// Accepts senders until the engine stops.
async fn accept_loop(inner: Arc<Inner>, listener: TcpListener, ctx: EngineContext) {
    loop {
        if !inner.running.load(Ordering::SeqCst) {
            return;
        }
        let (stream, address) = match listener.accept().await {
            Ok(pair) => pair,
            Err(error) => {
                tracing::warn!(%error, "miracast could not accept a connection");
                tokio::time::sleep(Duration::from_millis(50)).await;
                continue;
            }
        };

        let sessions = inner
            .sessions
            .lock()
            .map(|sessions| sessions.len())
            .unwrap_or(0);
        if sessions >= inner.config.max_sessions {
            tracing::warn!(peer = %mask_socket_addr(&address), "miracast refused a sender: a session is already active");
            crate::peer::refuse(stream).await;
            continue;
        }

        let inner = Arc::clone(&inner);
        let ctx = ctx.clone();
        tokio::spawn(async move {
            crate::peer::run_peer(inner, stream, ctx, address).await;
        });
    }
}

/// The failure of the engine, in the vocabulary of the core.
impl From<MiracastError> for EngineError {
    fn from(error: MiracastError) -> Self {
        EngineError::Protocol {
            protocol: "miracast",
            reason: error.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;
    use phairplay_core::media::{NullAudioSink, NullVideoSink};
    use phairplay_core::session::SessionEvent;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;

    fn context() -> EngineContext {
        let (events, _receiver) = tokio::sync::mpsc::channel(32);
        EngineContext {
            identity: Arc::new(phairplay_core::identity::DeviceIdentity::default()),
            capabilities: Arc::new(phairplay_core::capabilities::MediaCapabilities::empty()),
            video_sink: Arc::new(NullVideoSink),
            audio_sink: Arc::new(NullAudioSink),
            events,
        }
    }

    #[test]
    fn the_engine_reports_miracast() {
        let engine = MiracastEngine::new(MiracastConfig::default());
        assert_eq!(engine.kind(), ProtocolKind::Miracast);
    }

    #[tokio::test]
    async fn a_fresh_engine_is_idle() {
        let engine = MiracastEngine::new(MiracastConfig::default());
        let status = engine.status();
        assert_eq!(status.state, ProtocolState::Idle);
        assert!(status.enabled);
        assert_eq!(status.tcp_port, None);
    }

    #[tokio::test]
    async fn starting_binds_the_port_and_advertises() {
        let config = MiracastConfig {
            port: 0,
            ..MiracastConfig::default()
        };
        let engine = MiracastEngine::new(config);
        engine.start(context()).await.expect("start");
        let status = engine.status();
        assert_eq!(status.state, ProtocolState::Advertising);
        assert!(status.tcp_port.is_some(), "the port must be reported");
        engine.stop().await.expect("stop");
        assert_eq!(engine.status().state, ProtocolState::Stopped);
    }

    #[tokio::test]
    async fn starting_twice_fails() {
        let config = MiracastConfig {
            port: 0,
            ..MiracastConfig::default()
        };
        let engine = MiracastEngine::new(config);
        engine.start(context()).await.expect("start");
        let error = engine.start(context()).await.expect_err("must fail");
        assert!(
            matches!(error, EngineError::AlreadyRunning { .. }),
            "{error}"
        );
        engine.stop().await.expect("stop");
    }

    #[tokio::test]
    async fn stopping_a_fresh_engine_fails() {
        let engine = MiracastEngine::new(MiracastConfig::default());
        let error = engine.stop().await.expect_err("must fail");
        assert!(matches!(error, EngineError::NotRunning { .. }), "{error}");
    }

    #[tokio::test]
    async fn a_sender_is_served_and_the_session_is_reported() {
        let config = MiracastConfig {
            port: 0,
            ..MiracastConfig::default()
        };
        let engine = MiracastEngine::new(config);
        let ctx = context();
        let (events, mut receiver) = tokio::sync::mpsc::channel(32);
        let mut ctx = ctx;
        ctx.events = events;
        engine.start(ctx).await.expect("start");
        let port = engine.status().tcp_port.expect("port");

        let mut stream = TcpStream::connect(("127.0.0.1", port))
            .await
            .expect("connect");
        stream
            .write_all(b"OPTIONS rtsp://x/ RTSP/1.0\r\nCSeq: 1\r\n\r\n")
            .await
            .expect("write");
        let mut answer = [0_u8; 512];
        let count = stream.read(&mut answer).await.expect("read");
        let text = String::from_utf8_lossy(&answer[..count]);
        assert!(text.starts_with("RTSP/1.0 200 OK"), "{text}");

        stream
            .write_all(b"PLAY rtsp://x/ RTSP/1.0\r\nCSeq: 2\r\nUser-Agent: TestSender\r\n\r\n")
            .await
            .expect("write");
        let event = receiver.recv().await.expect("an event");
        match event {
            SessionEvent::Started { session } => {
                assert_eq!(session.protocol, ProtocolKind::Miracast);
                assert_eq!(session.sender_name.as_deref(), Some("TestSender"));
            }
            other => panic!("expected a started event, got {other:?}"),
        }

        stream
            .write_all(b"TEARDOWN rtsp://x/ RTSP/1.0\r\nCSeq: 3\r\n\r\n")
            .await
            .expect("write");
        let event = receiver.recv().await.expect("an event");
        assert!(
            matches!(event, SessionEvent::Ended { .. }),
            "expected an ended event, got {event:?}"
        );

        engine.stop().await.expect("stop");
    }

    #[tokio::test]
    async fn a_second_sender_is_refused() {
        let config = MiracastConfig {
            port: 0,
            ..MiracastConfig::default()
        };
        let engine = MiracastEngine::new(config);
        engine.start(context()).await.expect("start");
        let port = engine.status().tcp_port.expect("port");

        let mut first = TcpStream::connect(("127.0.0.1", port))
            .await
            .expect("connect");
        first
            .write_all(b"PLAY rtsp://x/ RTSP/1.0\r\nCSeq: 1\r\n\r\n")
            .await
            .expect("write");
        // Give the engine a moment to register the first sender.
        tokio::time::sleep(Duration::from_millis(80)).await;

        let mut second = TcpStream::connect(("127.0.0.1", port))
            .await
            .expect("connect");
        let mut answer = [0_u8; 256];
        let count = second.read(&mut answer).await.expect("read");
        let text = String::from_utf8_lossy(&answer[..count]);
        assert!(text.contains("503 Service Unavailable"), "{text}");

        engine.stop().await.expect("stop");
    }
}
