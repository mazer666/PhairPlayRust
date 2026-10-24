//! The DLNA engine: the state machine that ties the discovery, the control
//! plane and the event plane together.
//!
//! WHY: the UPnP pieces — the SSDP responder, the description server, the
//! SOAP dispatcher and the GENA manager — only work together. The engine is
//! the piece that owns their lifecycle: it starts them, it stops them, and
//! it reports what the rest of the system needs to know.
//!
//! HOW: the engine follows the state machine of the build contract
//! (`Idle → Starting → Advertising → Streaming → Stopping → Stopped`). The
//! state lives behind a mutex, so that [`ProtocolEngine::status`] can be
//! called from any thread while a transition is in flight.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_core::protocol::ProtocolEngine;
//! use phairplay_dlna::engine::{DlnaConfig, DlnaEngine};
//!
//! let engine = DlnaEngine::new(DlnaConfig::default());
//! assert_eq!(engine.status().protocol.as_str(), "dlna");
//! ```

use std::net::Ipv4Addr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use phairplay_core::error::EngineError;
use phairplay_core::identity::sanitize_service_name;
use phairplay_core::protocol::{
    EngineContext, ProtocolEngine, ProtocolKind, ProtocolState, ProtocolStatus,
};

use crate::gena::GenaEventManager;
use crate::http::HttpLimits;
use crate::reporter::SessionReporter;
use crate::server::{HttpServer, Lifecycle, ServerContext};
use crate::soap::{CompositeEventSink, EventSink, SoapDispatcher};
use crate::ssdp::SsdpResponder;
use crate::state::RendererState;

/// The default port of the SSDP responder.
///
/// WHY: the value is fixed by the UPnP specification. Its single owner is
/// [`phairplay_core::config::ports::SSDP`].
pub const DEFAULT_SSDP_PORT: u16 = phairplay_core::config::ports::SSDP;

/// The configuration of the DLNA engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DlnaConfig {
    /// The port of the description server. `0` picks a free port.
    pub http_port: u16,
    /// The port of the SSDP responder.
    pub ssdp_port: u16,
    /// The local address the renderer announces and checks the GENA
    /// callbacks against.
    pub local_ip: Ipv4Addr,
    /// The limits of the HTTP reader.
    pub limits: HttpLimits,
    /// The deadline of a control connection.
    pub idle_timeout: Duration,
}

impl Default for DlnaConfig {
    fn default() -> Self {
        Self {
            http_port: 0,
            ssdp_port: DEFAULT_SSDP_PORT,
            local_ip: Ipv4Addr::LOCALHOST,
            limits: HttpLimits::default(),
            idle_timeout: Duration::from_secs(30),
        }
    }
}

impl DlnaConfig {
    /// The configuration for a receiver with the given local address.
    #[must_use]
    pub fn with_local_ip(mut self, local_ip: Ipv4Addr) -> Self {
        self.local_ip = local_ip;
        self
    }

    /// Sets the ports of the description server and the SSDP responder.
    #[must_use]
    pub fn with_ports(mut self, http_port: u16, ssdp_port: u16) -> Self {
        self.http_port = http_port;
        self.ssdp_port = ssdp_port;
        self
    }
}

/// The runtime of a started engine.
#[derive(Debug)]
struct Runtime {
    server: HttpServer,
    ssdp: SsdpResponder,
    location: String,
    udn: String,
}

/// The DLNA media renderer.
#[derive(Debug)]
pub struct DlnaEngine {
    config: DlnaConfig,
    lifecycle: Arc<Mutex<Lifecycle>>,
    state: Arc<RendererState>,
    events: Arc<GenaEventManager>,
    reporter: Arc<SessionReporter>,
    dispatcher: Arc<SoapDispatcher>,
    runtime: Mutex<Option<Runtime>>,
}

impl DlnaEngine {
    /// An engine with the given configuration.
    #[must_use]
    pub fn new(config: DlnaConfig) -> Self {
        let state = Arc::new(RendererState::new());
        let events = Arc::new(GenaEventManager::new(config.local_ip));
        let reporter = Arc::new(SessionReporter::new(Arc::clone(&state)));
        let sink: Arc<dyn EventSink> = Arc::new(CompositeEventSink::new(vec![
            Arc::clone(&events) as Arc<dyn EventSink>,
            Arc::clone(&reporter) as Arc<dyn EventSink>,
        ]));
        let dispatcher = Arc::new(SoapDispatcher::new(Arc::clone(&state), sink));
        Self {
            config,
            lifecycle: Arc::new(Mutex::new(Lifecycle::new())),
            state,
            events,
            reporter,
            dispatcher,
            runtime: Mutex::new(None),
        }
    }

    /// The state of the renderer.
    #[must_use]
    pub fn renderer_state(&self) -> &Arc<RendererState> {
        &self.state
    }

    /// The event manager of the renderer.
    #[must_use]
    pub fn event_manager(&self) -> &Arc<GenaEventManager> {
        &self.events
    }

    /// The configuration of the engine.
    #[must_use]
    pub fn config(&self) -> &DlnaConfig {
        &self.config
    }

    /// The URL of the device description, once the engine runs.
    #[must_use]
    pub fn description_url(&self) -> Option<String> {
        self.runtime
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_ref()
            .map(|runtime| runtime.location.clone())
    }

    fn lock_lifecycle(&self) -> std::sync::MutexGuard<'_, Lifecycle> {
        self.lifecycle.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// The address of the first non-loopback IPv4 interface.
    ///
    /// WHY: the renderer has to announce an address that the control points
    /// can reach. A loopback address is announced only when there is no
    /// other one — and the GENA guard then refuses every callback, because
    /// a loopback address is not in a private range.
    #[must_use]
    pub fn detect_local_ip() -> Ipv4Addr {
        let Ok(interfaces) = if_addrs::get_if_addrs() else {
            return Ipv4Addr::LOCALHOST;
        };
        for interface in interfaces {
            if interface.is_loopback() {
                continue;
            }
            let address = interface.ip();
            if let std::net::IpAddr::V4(v4) = address {
                if !v4.is_unspecified() {
                    return v4;
                }
            }
        }
        Ipv4Addr::LOCALHOST
    }
}

#[async_trait::async_trait]
impl ProtocolEngine for DlnaEngine {
    fn kind(&self) -> ProtocolKind {
        ProtocolKind::Dlna
    }

    fn status(&self) -> ProtocolStatus {
        let lifecycle = self.lock_lifecycle();
        ProtocolStatus {
            protocol: ProtocolKind::Dlna,
            state: lifecycle.state(),
            enabled: lifecycle.enabled(),
            tcp_port: lifecycle.tcp_port(),
            udp_port: lifecycle.udp_port(),
            peer_count: lifecycle.connections(),
            last_error: lifecycle.last_error(),
        }
    }

    async fn start(&self, ctx: EngineContext) -> Result<(), EngineError> {
        {
            let lifecycle = self.lock_lifecycle();
            if matches!(
                lifecycle.state(),
                ProtocolState::Starting | ProtocolState::Advertising | ProtocolState::Streaming
            ) {
                return Err(EngineError::AlreadyRunning { protocol: "dlna" });
            }
        }
        self.lock_lifecycle().set_state(ProtocolState::Starting);

        let friendly_name = sanitize_service_name(&ctx.identity.name, "PhairPlay");
        let udn = format!("uuid:{}", ctx.identity.device_id);

        let server = match HttpServer::bind(self.config.http_port, |port| {
            self.server_context(&friendly_name, &udn, port)
        })
        .await
        {
            Ok(server) => server,
            Err(error) => {
                return self.fail_start(format!(
                    "the description server could not be bound: {error}"
                ));
            }
        };
        let tcp_port = server.local_port();

        let ssdp = match SsdpResponder::bind(
            self.config.ssdp_port,
            format!("http://{}:{tcp_port}/description.xml", self.config.local_ip),
            udn.clone(),
        ) {
            Ok(ssdp) => ssdp,
            Err(error) => {
                return self
                    .fail_start(format!("the SSDP responder could not be started: {error}"));
            }
        };
        let udp_port = ssdp.local_port();

        let location = format!("http://{}:{tcp_port}/description.xml", self.config.local_ip);
        if let Err(error) = ssdp.send_alive(&location, &udn) {
            tracing::warn!(reason = %error, "the SSDP announcement could not be sent");
        }

        self.reporter.attach(ctx.events.clone());
        {
            let mut lifecycle = self.lock_lifecycle();
            lifecycle.set_tcp_port(Some(tcp_port));
            lifecycle.set_udp_port(Some(udp_port));
            lifecycle.set_state(ProtocolState::Advertising);
            lifecycle.set_last_error(None);
        }
        *self.runtime.lock().unwrap_or_else(|e| e.into_inner()) = Some(Runtime {
            server,
            ssdp,
            location,
            udn,
        });
        Ok(())
    }

    async fn stop(&self) -> Result<(), EngineError> {
        let runtime = self
            .runtime
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take();
        let Some(mut runtime) = runtime else {
            return Err(EngineError::NotRunning { protocol: "dlna" });
        };
        self.lock_lifecycle().set_state(ProtocolState::Stopping);
        if let Err(error) = runtime.ssdp.send_byebye(&runtime.udn) {
            tracing::warn!(reason = %error, "the SSDP farewell could not be sent");
        }
        runtime.server.stop().await;
        runtime.ssdp.stop();
        self.reporter.detach();
        {
            let mut lifecycle = self.lock_lifecycle();
            lifecycle.set_tcp_port(None);
            lifecycle.set_udp_port(None);
            lifecycle.set_state(ProtocolState::Stopped);
        }
        Ok(())
    }
}

impl DlnaEngine {
    /// Builds the context the HTTP server works with.
    fn server_context(&self, friendly_name: &str, udn: &str, port: u16) -> Arc<ServerContext> {
        Arc::new(ServerContext {
            friendly_name: friendly_name.to_string(),
            udn: udn.to_string(),
            presentation_url: format!("http://{}:{port}/", self.config.local_ip),
            state: Arc::clone(&self.state),
            events: Arc::clone(&self.events),
            dispatcher: Arc::clone(&self.dispatcher),
            limits: self.config.limits.clone(),
            lifecycle: Arc::clone(&self.lifecycle),
        })
    }

    /// Records a start failure and returns the matching error.
    fn fail_start<T>(&self, reason: String) -> Result<T, EngineError> {
        let mut lifecycle = self.lock_lifecycle();
        lifecycle.set_state(ProtocolState::Failed);
        lifecycle.set_last_error(Some(reason.clone()));
        Err(EngineError::Start {
            protocol: "dlna",
            reason,
        })
    }
}

#[cfg(test)]
mod tests;
