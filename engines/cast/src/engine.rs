//! The Google Cast engine.
//!
//! WHY: Google Cast is the one protocol of the four that is *multi-session*:
//! a receiver may serve several senders at the same time, and the build
//! contract names Cast as the documented exception to the one-peer rule. The
//! engine therefore keeps a table of live sessions instead of a single slot —
//! but the table is bounded, because "unlimited senders" is a denial of
//! service on the local network just as much as one is.
//!
//! WHAT: [`CastEngine`] implements [`phairplay_core::protocol::ProtocolEngine`]
//! for the Cast protocol: it binds the TLS listener, advertises the
//! `_googlecast._tcp` service and hands every accepted connection to the
//! peer loop.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_cast::{CastConfig, CastEngine};
//! use phairplay_core::protocol::ProtocolEngine;
//!
//! let engine = CastEngine::new(CastConfig::default());
//! assert_eq!(engine.kind(), phairplay_core::protocol::ProtocolKind::Cast);
//! assert_eq!(engine.status().state, phairplay_core::protocol::ProtocolState::Idle);
//! ```

use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use phairplay_core::config::AppConfig;
use phairplay_core::error::EngineError;
use phairplay_core::privacy::mask_socket_addr;
use phairplay_core::protocol::{
    EngineContext, ProtocolEngine, ProtocolKind, ProtocolState, ProtocolStatus,
};
use phairplay_core::session::SessionId;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use tokio_rustls::rustls::ServerConfig;

use crate::advertisement::{
    AdvertiseError, CastService, GOOGLECAST_SERVICE_TYPE, MdnsRegistrar, READY_STATE_IDLE,
    ServiceRegistrar, txt_properties,
};
use crate::message::DEFAULT_MEDIA_APP_ID;
use crate::tls::TlsError;

/// The port a Google Cast receiver listens on.
///
/// WHY: the value is the port every Cast sender looks for. Its single owner
/// is [`phairplay_core::config::ports::CAST_TLS`].
pub const DEFAULT_PORT: u16 = phairplay_core::config::ports::CAST_TLS;

/// The number of senders that may be connected at the same time.
pub const DEFAULT_MAX_SESSIONS: usize = 8;

/// The largest message the receiver accepts.
///
/// WHY: the same cap applies to the framing layer of the crate. Its single
/// owner is [`phairplay_core::config::limits::CAST_MESSAGE_BYTES`].
pub const DEFAULT_MAX_MESSAGE_BYTES: usize = phairplay_core::config::limits::CAST_MESSAGE_BYTES;

/// The time a peer may take to complete the TLS handshake.
///
/// WHY: the deadline of a protocol step belongs to one place. Its single
/// owner is
/// [`phairplay_core::config::timeouts::CAST_HANDSHAKE_TIMEOUT_MS`].
pub const DEFAULT_HANDSHAKE_TIMEOUT_MS: u64 =
    phairplay_core::config::timeouts::CAST_HANDSHAKE_TIMEOUT_MS;

/// How the receiver terminates TLS.
#[derive(Debug, Clone)]
pub enum CastTls {
    /// A self-signed certificate generated when the engine starts.
    SelfSigned,
    /// A configuration the caller provides, e.g. one with a stored
    /// certificate.
    Provided(Arc<ServerConfig>),
}

/// The configuration of the Cast engine.
#[derive(Debug, Clone)]
pub struct CastConfig {
    /// Whether the protocol is enabled.
    pub enabled: bool,
    /// The port the TLS listener binds.
    pub port: u16,
    /// The Cast application id the receiver offers.
    pub app_id: String,
    /// The name of the application the receiver offers.
    pub app_name: String,
    /// The largest message the receiver accepts.
    pub max_message_bytes: usize,
    /// The time a peer may stay silent before it is dropped.
    pub idle_timeout_ms: u64,
    /// The time a peer may take to complete the TLS handshake.
    pub handshake_timeout_ms: u64,
    /// How the receiver terminates TLS.
    pub tls: CastTls,
}

impl Default for CastConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            port: DEFAULT_PORT,
            app_id: DEFAULT_MEDIA_APP_ID.to_string(),
            app_name: crate::message::DEFAULT_MEDIA_APP_NAME.to_string(),
            max_message_bytes: DEFAULT_MAX_MESSAGE_BYTES,
            idle_timeout_ms: 30_000,
            handshake_timeout_ms: DEFAULT_HANDSHAKE_TIMEOUT_MS,
            tls: CastTls::SelfSigned,
        }
    }
}

impl CastConfig {
    /// Builds the configuration from the application configuration.
    ///
    /// WHY: the limits live in [`AppConfig`], not in the engine. A port of
    /// zero means "any free port", but a Cast sender only ever connects to
    /// 8009 — so the default port is used instead of an ephemeral one.
    #[must_use]
    pub fn from_app_config(config: &AppConfig) -> Self {
        let protocol = config.protocol(ProtocolKind::Cast);
        let port = if protocol.port == 0 {
            DEFAULT_PORT
        } else {
            protocol.port
        };
        Self {
            enabled: protocol.enabled,
            port,
            max_message_bytes: protocol.max_message_bytes.min(DEFAULT_MAX_MESSAGE_BYTES),
            idle_timeout_ms: config.performance.idle_timeout_ms,
            ..Self::default()
        }
    }

    /// The idle timeout as a duration.
    #[must_use]
    pub fn idle_timeout(&self) -> Duration {
        Duration::from_millis(self.idle_timeout_ms)
    }

    /// The handshake timeout as a duration.
    #[must_use]
    pub fn handshake_timeout(&self) -> Duration {
        Duration::from_millis(self.handshake_timeout_ms)
    }
}

/// The state of the engine.
#[derive(Debug, Default)]
struct EngineState {
    state: Option<ProtocolState>,
    tcp_port: Option<u16>,
    last_error: Option<String>,
}

/// The runtime of a running engine.
#[derive(Debug)]
struct Runtime {
    task: tokio::task::JoinHandle<()>,
    certificate: Option<tokio_rustls::rustls::pki_types::CertificateDer<'static>>,
}

/// The shared state of the engine.
pub(crate) struct Inner {
    pub(crate) config: CastConfig,
    state: Mutex<EngineState>,
    registrar: Arc<dyn ServiceRegistrar>,
    runtime: Mutex<Option<Runtime>>,
    sessions: Mutex<HashMap<SessionId, String>>,
    transport_counter: AtomicU64,
}

impl std::fmt::Debug for Inner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Inner")
            .field("config", &self.config)
            .field("state", &self.state)
            .field("sessions", &self.sessions)
            .finish()
    }
}

impl Inner {
    /// The next transport identifier, e.g. `web-1`.
    pub(crate) fn next_transport_index(&self) -> u64 {
        self.transport_counter.fetch_add(1, Ordering::Relaxed) + 1
    }

    /// Records a peer that connected.
    pub(crate) fn register_session(&self, session: SessionId, transport_id: String) {
        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.insert(session, transport_id);
        }
    }

    /// Forgets a peer that disconnected.
    pub(crate) fn unregister_session(&self, session: SessionId) {
        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.remove(&session);
        }
    }

    /// The number of peers that are connected.
    fn peer_count(&self) -> u32 {
        self.sessions
            .lock()
            .map(|sessions| sessions.len())
            .unwrap_or(0) as u32
    }
}

/// The Google Cast receiver engine.
pub struct CastEngine {
    inner: Arc<Inner>,
}

impl std::fmt::Debug for CastEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CastEngine")
            .field("inner", &self.inner)
            .finish()
    }
}

impl CastEngine {
    /// Creates an engine with the default DNS-SD registrar.
    #[must_use]
    pub fn new(config: CastConfig) -> Self {
        Self::with_registrar(config, Arc::new(MdnsRegistrar::new()))
    }

    /// Creates an engine with a registrar the caller provides.
    #[must_use]
    pub fn with_registrar(config: CastConfig, registrar: Arc<dyn ServiceRegistrar>) -> Self {
        Self {
            inner: Arc::new(Inner {
                config,
                state: Mutex::new(EngineState {
                    state: Some(ProtocolState::Idle),
                    tcp_port: None,
                    last_error: None,
                }),
                registrar,
                runtime: Mutex::new(None),
                sessions: Mutex::new(HashMap::new()),
                transport_counter: AtomicU64::new(0),
            }),
        }
    }

    /// The configuration of the engine.
    #[must_use]
    pub fn config(&self) -> &CastConfig {
        &self.inner.config
    }

    /// The certificate the receiver presents, once the engine has started.
    #[must_use]
    pub fn certificate(&self) -> Option<tokio_rustls::rustls::pki_types::CertificateDer<'static>> {
        self.inner.runtime.lock().ok().and_then(|runtime| {
            runtime
                .as_ref()
                .and_then(|runtime| runtime.certificate.clone())
        })
    }

    /// The number of senders that are connected right now.
    #[must_use]
    pub fn peer_count(&self) -> u32 {
        self.inner.peer_count()
    }
}

#[async_trait::async_trait]
impl ProtocolEngine for CastEngine {
    fn kind(&self) -> ProtocolKind {
        ProtocolKind::Cast
    }

    fn status(&self) -> ProtocolStatus {
        let state = self.inner.state.lock().ok();
        ProtocolStatus {
            protocol: ProtocolKind::Cast,
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
                protocol: "cast",
                reason: "the engine state is poisoned".to_string(),
            })?;
            match state.state.unwrap_or(ProtocolState::Idle) {
                ProtocolState::Idle | ProtocolState::Stopped => {
                    state.state = Some(ProtocolState::Starting);
                    state.last_error = None;
                }
                _ => {
                    return Err(EngineError::AlreadyRunning { protocol: "cast" });
                }
            }
        }

        match self.start_inner(ctx).await {
            Ok(()) => Ok(()),
            Err(error) => {
                if let Ok(mut state) = self.inner.state.lock() {
                    state.state = Some(ProtocolState::Failed);
                    state.last_error = Some(error.to_string());
                }
                Err(error)
            }
        }
    }

    async fn stop(&self) -> Result<(), EngineError> {
        {
            let mut state = self
                .inner
                .state
                .lock()
                .map_err(|_| EngineError::Stopped { protocol: "cast" })?;
            match state.state.unwrap_or(ProtocolState::Idle) {
                ProtocolState::Advertising | ProtocolState::Streaming => {
                    state.state = Some(ProtocolState::Stopping);
                }
                _ => {
                    return Err(EngineError::NotRunning { protocol: "cast" });
                }
            }
        }

        if let Ok(mut runtime) = self.inner.runtime.lock() {
            if let Some(runtime) = runtime.take() {
                runtime.task.abort();
            }
        }
        self.inner.registrar.unregister();
        if let Ok(mut sessions) = self.inner.sessions.lock() {
            sessions.clear();
        }
        if let Ok(mut state) = self.inner.state.lock() {
            state.state = Some(ProtocolState::Stopped);
            state.tcp_port = None;
        }
        Ok(())
    }
}

impl CastEngine {
    async fn start_inner(&self, ctx: EngineContext) -> Result<(), EngineError> {
        let (acceptor, certificate) = self.build_tls()?;
        let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, self.inner.config.port));
        let listener = TcpListener::bind(address)
            .await
            .map_err(|error| EngineError::Start {
                protocol: "cast",
                reason: format!("could not bind {address}: {error}"),
            })?;
        let local = listener.local_addr().map_err(|error| EngineError::Start {
            protocol: "cast",
            reason: format!("could not read the local address: {error}"),
        })?;

        let service = self.service(local.port());
        self.inner
            .registrar
            .register(&service)
            .map_err(|error| EngineError::Start {
                protocol: "cast",
                reason: error.to_string(),
            })?;

        let inner = Arc::clone(&self.inner);
        let task = tokio::spawn(accept_loop(inner, listener, acceptor, ctx));

        if let Ok(mut runtime) = self.inner.runtime.lock() {
            *runtime = Some(Runtime { task, certificate });
        }
        if let Ok(mut state) = self.inner.state.lock() {
            state.state = Some(ProtocolState::Advertising);
            state.tcp_port = Some(local.port());
            state.last_error = None;
        }
        Ok(())
    }

    fn build_tls(
        &self,
    ) -> Result<
        (
            TlsAcceptor,
            Option<tokio_rustls::rustls::pki_types::CertificateDer<'static>>,
        ),
        EngineError,
    > {
        match &self.inner.config.tls {
            CastTls::Provided(config) => Ok((TlsAcceptor::from(Arc::clone(config)), None)),
            CastTls::SelfSigned => {
                let names = vec![
                    "localhost".to_string(),
                    "127.0.0.1".to_string(),
                    self.inner.config.app_name.clone(),
                ];
                let (certificate, config) =
                    crate::tls::self_signed(&names).map_err(|error| EngineError::Start {
                        protocol: "cast",
                        reason: match error {
                            TlsError::Certificate(detail) => {
                                format!("could not generate the certificate: {detail}")
                            }
                            TlsError::Configuration(detail) => {
                                format!("could not build the TLS configuration: {detail}")
                            }
                        },
                    })?;
                Ok((TlsAcceptor::from(config), Some(certificate)))
            }
        }
    }

    fn service(&self, port: u16) -> CastService {
        CastService {
            instance: self.inner.config.app_name.clone(),
            service_type: GOOGLECAST_SERVICE_TYPE.to_string(),
            host: format!("{}.local.", self.inner.config.app_name),
            port,
            properties: txt_properties(
                &phairplay_core::identity::DeviceIdentity::default(),
                &self.inner.config.app_name,
                crate::advertisement::DEFAULT_TXT_VERSION,
                READY_STATE_IDLE,
            ),
        }
    }
}

/// Accepts connections until the engine stops.
async fn accept_loop(
    inner: Arc<Inner>,
    listener: TcpListener,
    acceptor: TlsAcceptor,
    ctx: EngineContext,
) {
    loop {
        let (stream, address) = match listener.accept().await {
            Ok(pair) => pair,
            Err(error) => {
                tracing::warn!(%error, "cast could not accept a connection");
                tokio::time::sleep(Duration::from_millis(50)).await;
                continue;
            }
        };
        // WHY the limit comes from the context: the rule that says how many
        // peers a protocol serves has **one** owner — the list
        // `session.multi_peer_protocols` of the configuration — and the
        // engine asks it instead of carrying a number of its own. Cast is
        // the protocol that the list names, and it is the only one.
        if inner.peer_count() as usize >= ctx.peer_limit.max_peers() {
            tracing::warn!(peer = %mask_socket_addr(&address), "cast refused a peer: the session limit is reached");
            drop(stream);
            continue;
        }
        let inner = Arc::clone(&inner);
        let acceptor = acceptor.clone();
        let ctx = ctx.clone();
        tokio::spawn(async move {
            crate::peer::run_peer(inner, stream, acceptor, ctx, address).await;
        });
    }
}

/// Converts an advertisement failure into an engine error.
#[allow(dead_code)]
fn advertise_error(error: AdvertiseError) -> EngineError {
    EngineError::Start {
        protocol: "cast",
        reason: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;
    use crate::advertisement::CastService;

    #[derive(Debug, Default)]
    struct RecordingRegistrar {
        services: Mutex<Vec<CastService>>,
        fail: bool,
    }

    impl ServiceRegistrar for RecordingRegistrar {
        fn register(&self, service: &CastService) -> Result<(), AdvertiseError> {
            if self.fail {
                return Err(AdvertiseError::Registration("the test refused".to_string()));
            }
            if let Ok(mut services) = self.services.lock() {
                services.push(service.clone());
            }
            Ok(())
        }

        fn unregister(&self) {
            if let Ok(mut services) = self.services.lock() {
                services.clear();
            }
        }
    }

    fn engine() -> (CastEngine, Arc<RecordingRegistrar>) {
        let registrar = Arc::new(RecordingRegistrar::default());
        // Port zero: the test must not fight over the port 8009 with a
        // receiver that runs on the same machine.
        let config = CastConfig {
            port: 0,
            ..CastConfig::default()
        };
        let engine = CastEngine::with_registrar(config, registrar.clone());
        (engine, registrar)
    }

    #[test]
    fn the_engine_starts_in_the_idle_state() {
        let (engine, _registrar) = engine();
        let status = engine.status();
        assert_eq!(status.protocol, ProtocolKind::Cast);
        assert_eq!(status.state, ProtocolState::Idle);
        assert_eq!(status.peer_count, 0);
        assert!(status.tcp_port.is_none());
    }

    #[test]
    fn the_configuration_is_derived_from_the_application_configuration() {
        let mut config = AppConfig::default();
        config.performance.idle_timeout_ms = 1234;
        let cast = config.protocols.entry("cast".to_string()).or_default();
        cast.port = 8009;
        cast.max_message_bytes = 4096;
        let engine_config = CastConfig::from_app_config(&config);
        assert_eq!(engine_config.port, 8009);
        assert_eq!(engine_config.max_message_bytes, 4096);
        assert_eq!(engine_config.idle_timeout_ms, 1234);
    }

    #[test]
    fn a_zeroport_falls_back_to_the_port_the_senders_use() {
        let config = AppConfig::default();
        assert_eq!(CastConfig::from_app_config(&config).port, DEFAULT_PORT);
    }

    #[test]
    fn an_oversized_message_limit_is_capped() {
        let mut config = AppConfig::default();
        let cast = config.protocols.entry("cast".to_string()).or_default();
        cast.max_message_bytes = usize::MAX / 2;
        assert_eq!(
            CastConfig::from_app_config(&config).max_message_bytes,
            DEFAULT_MAX_MESSAGE_BYTES
        );
    }

    #[tokio::test]
    async fn starting_binds_the_listener_and_advertises_the_service() {
        let (engine, registrar) = engine();
        let ctx = phairplay_core::protocol::EngineContext {
            identity: Arc::new(phairplay_core::identity::DeviceIdentity::default()),
            capabilities: Arc::new(phairplay_core::capabilities::MediaCapabilities::default()),
            peer_limit: phairplay_core::protocol::PeerLimit::Several { max: 8 },
            video_sink: Arc::new(phairplay_core::media::NullVideoSink),
            audio_sink: Arc::new(phairplay_core::media::NullAudioSink),
            events: tokio::sync::mpsc::channel(16).0,
        };
        engine.start(ctx).await.expect("start");
        let status = engine.status();
        assert_eq!(status.state, ProtocolState::Advertising);
        assert!(status.tcp_port.is_some());
        {
            // WHY: the lock of the registrar is released **before** the
            // engine is stopped. `stop()` withdraws the service and locks
            // the very same mutex — a lock that is held across the call
            // deadlocks the test, and the test suite of the workspace never
            // finishes.
            let services = registrar.services.lock().expect("lock");
            assert_eq!(services.len(), 1);
            assert_eq!(services[0].service_type, "_googlecast._tcp");
        }
        engine.stop().await.expect("stop");
        assert_eq!(engine.status().state, ProtocolState::Stopped);
    }

    #[tokio::test]
    async fn a_failing_advertisement_fails_the_start() {
        let registrar = Arc::new(RecordingRegistrar {
            fail: true,
            ..Default::default()
        });
        let engine = CastEngine::with_registrar(
            CastConfig {
                port: 0,
                ..CastConfig::default()
            },
            registrar,
        );
        let ctx = phairplay_core::protocol::EngineContext {
            identity: Arc::new(phairplay_core::identity::DeviceIdentity::default()),
            capabilities: Arc::new(phairplay_core::capabilities::MediaCapabilities::default()),
            peer_limit: phairplay_core::protocol::PeerLimit::Several { max: 8 },
            video_sink: Arc::new(phairplay_core::media::NullVideoSink),
            audio_sink: Arc::new(phairplay_core::media::NullAudioSink),
            events: tokio::sync::mpsc::channel(16).0,
        };
        let error = engine.start(ctx).await.expect_err("must fail");
        assert!(matches!(error, EngineError::Start { .. }));
        assert_eq!(engine.status().state, ProtocolState::Failed);
    }

    #[tokio::test]
    async fn stopping_an_engine_that_does_not_run_fails() {
        let (engine, _registrar) = engine();
        let error = engine.stop().await.expect_err("must fail");
        assert!(matches!(error, EngineError::NotRunning { .. }));
    }

    #[tokio::test]
    async fn starting_twice_fails() {
        let (engine, _registrar) = engine();
        let ctx = phairplay_core::protocol::EngineContext {
            identity: Arc::new(phairplay_core::identity::DeviceIdentity::default()),
            capabilities: Arc::new(phairplay_core::capabilities::MediaCapabilities::default()),
            peer_limit: phairplay_core::protocol::PeerLimit::Several { max: 8 },
            video_sink: Arc::new(phairplay_core::media::NullVideoSink),
            audio_sink: Arc::new(phairplay_core::media::NullAudioSink),
            events: tokio::sync::mpsc::channel(16).0,
        };
        engine.start(ctx.clone()).await.expect("start");
        let error = engine.start(ctx).await.expect_err("must fail");
        assert!(matches!(error, EngineError::AlreadyRunning { .. }));
        engine.stop().await.expect("stop");
    }
}
