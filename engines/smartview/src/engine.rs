//! The Smart View engine: the state machine that ties the discovery, the
//! description and the API together.
//!
//! WHY: the pieces of Smart View — the SSDP responder, the description
//! document and the HTTP API — only work together. The engine is the piece
//! that owns their lifecycle: it starts them, it stops them, and it reports
//! what the rest of the system needs to know.
//!
//! HOW: the engine follows the state machine of the build contract
//! (`Idle → Starting → Advertising → Streaming → Stopping → Stopped`). The
//! state lives behind a mutex, so that [`ProtocolEngine::status`] can be
//! called from any thread while a transition is in flight. The *streaming*
//! state is derived from the session registry: a sender that opened the
//! channel is a sender that is connected — **no picture flows yet**, because
//! the message format of the channel is proprietary and not implemented.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_core::protocol::ProtocolEngine;
//! use phairplay_smartview::engine::{SmartViewConfig, SmartViewEngine};
//!
//! let engine = SmartViewEngine::new(SmartViewConfig::default());
//! assert_eq!(engine.status().protocol.as_str(), "smartview");
//! ```

use std::net::Ipv4Addr;
use std::sync::{Arc, Mutex};

use phairplay_core::error::EngineError;
use phairplay_core::identity::sanitize_service_name;
use phairplay_core::protocol::{
    EngineContext, ProtocolEngine, ProtocolKind, ProtocolState, ProtocolStatus,
};

use crate::api::{ApiContext, ApiLimits, ApiServer};
use crate::description::{DESCRIPTION_PATH, DeviceDescription};
use crate::lifecycle::Lifecycle;
use crate::session::SessionRegistry;
use crate::ssdp::SsdpResponder;

/// The default port of the API server.
///
/// WHY: the port is the one Samsung names in the documentation of the
/// Smart View SDK server. Its single owner is
/// [`phairplay_core::config::ports::SMARTVIEW_API`].
pub const DEFAULT_API_PORT: u16 = phairplay_core::config::ports::SMARTVIEW_API;

/// The default port of the SSDP responder.
pub const DEFAULT_SSDP_PORT: u16 = phairplay_core::config::ports::SSDP;

/// The configuration of the Smart View engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmartViewConfig {
    /// The port of the API server. `0` picks a free port.
    pub http_port: u16,
    /// The port of the SSDP responder.
    pub ssdp_port: u16,
    /// The local address the receiver announces.
    pub local_ip: Ipv4Addr,
    /// The limits of the HTTP reader and of the channel.
    pub limits: ApiLimits,
}

impl Default for SmartViewConfig {
    fn default() -> Self {
        Self {
            http_port: DEFAULT_API_PORT,
            ssdp_port: DEFAULT_SSDP_PORT,
            local_ip: Ipv4Addr::LOCALHOST,
            limits: ApiLimits::default(),
        }
    }
}

impl SmartViewConfig {
    /// The configuration for a receiver with the given local address.
    #[must_use]
    pub fn with_local_ip(mut self, local_ip: Ipv4Addr) -> Self {
        self.local_ip = local_ip;
        self
    }

    /// Sets the ports of the API server and of the SSDP responder.
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
    server: ApiServer,
    ssdp: SsdpResponder,
    base_url: String,
    description_url: String,
    udn: String,
}

/// The Samsung Smart View receiver.
#[derive(Debug)]
pub struct SmartViewEngine {
    config: SmartViewConfig,
    lifecycle: Arc<Mutex<Lifecycle>>,
    sessions: Arc<SessionRegistry>,
    runtime: Mutex<Option<Runtime>>,
}

impl SmartViewEngine {
    /// An engine with the given configuration.
    #[must_use]
    pub fn new(config: SmartViewConfig) -> Self {
        Self {
            config,
            lifecycle: Arc::new(Mutex::new(Lifecycle::new())),
            sessions: Arc::new(SessionRegistry::new()),
            runtime: Mutex::new(None),
        }
    }

    /// The configuration of the engine.
    #[must_use]
    pub fn config(&self) -> &SmartViewConfig {
        &self.config
    }

    /// The registry of the one session the receiver serves.
    #[must_use]
    pub fn sessions(&self) -> &Arc<SessionRegistry> {
        &self.sessions
    }

    /// The URL of the description document, once the engine runs.
    #[must_use]
    pub fn description_url(&self) -> Option<String> {
        self.runtime
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .as_ref()
            .map(|runtime| runtime.description_url.clone())
    }

    /// The base URL of the API, once the engine runs.
    #[must_use]
    pub fn base_url(&self) -> Option<String> {
        self.runtime
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .as_ref()
            .map(|runtime| runtime.base_url.clone())
    }

    /// The address of the first non-loopback IPv4 interface.
    ///
    /// WHY: the receiver has to announce an address that a Galaxy device
    /// can reach. A loopback address is announced only when there is no
    /// other one.
    #[must_use]
    pub fn detect_local_ip() -> Ipv4Addr {
        let Ok(interfaces) = if_addrs::get_if_addrs() else {
            return Ipv4Addr::LOCALHOST;
        };
        for interface in interfaces {
            if interface.is_loopback() {
                continue;
            }
            if let std::net::IpAddr::V4(address) = interface.ip() {
                if !address.is_unspecified() {
                    return address;
                }
            }
        }
        Ipv4Addr::LOCALHOST
    }

    fn lock_lifecycle(&self) -> std::sync::MutexGuard<'_, Lifecycle> {
        self.lifecycle
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }
}

#[async_trait::async_trait]
impl ProtocolEngine for SmartViewEngine {
    fn kind(&self) -> ProtocolKind {
        ProtocolKind::SmartView
    }

    fn status(&self) -> ProtocolStatus {
        let lifecycle = self.lock_lifecycle();
        let state = match lifecycle.state() {
            ProtocolState::Advertising if self.sessions.is_active() => ProtocolState::Streaming,
            other => other,
        };
        ProtocolStatus {
            protocol: ProtocolKind::SmartView,
            state,
            enabled: lifecycle.enabled(),
            tcp_port: lifecycle.tcp_port(),
            udp_port: lifecycle.udp_port(),
            peer_count: u32::from(self.sessions.is_active()),
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
                return Err(EngineError::AlreadyRunning {
                    protocol: "smartview",
                });
            }
        }
        self.lock_lifecycle().set_state(ProtocolState::Starting);

        let friendly_name = sanitize_service_name(&ctx.identity.name, "PhairPlay");
        let udn = format!("uuid:{}", ctx.identity.device_id);
        let description = DeviceDescription::new(
            friendly_name,
            ctx.identity.model.clone(),
            udn.clone(),
            String::new(),
        )
        .with_capabilities(&ctx.capabilities);

        let sessions = Arc::clone(&self.sessions);
        let events = ctx.events.clone();
        let limits = self.config.limits.clone();
        let local_ip = self.config.local_ip;
        let mut server = match ApiServer::bind(self.config.http_port, move |port| {
            let base_url = format!("http://{local_ip}:{port}/api/v2/");
            let description_url = format!("http://{local_ip}:{port}{DESCRIPTION_PATH}");
            let mut description = description.clone();
            description.base_url = base_url.clone();
            Arc::new(ApiContext {
                description,
                base_url,
                description_url,
                sessions,
                events,
                limits,
            })
        })
        .await
        {
            Ok(server) => server,
            Err(error) => {
                return self.fail_start(format!("the API server could not be bound: {error}"));
            }
        };
        let tcp_port = server.local_port();
        let base_url = format!("http://{}:{tcp_port}/api/v2/", self.config.local_ip);
        let description_url = format!(
            "http://{}:{tcp_port}{DESCRIPTION_PATH}",
            self.config.local_ip
        );

        let ssdp = match SsdpResponder::bind(
            self.config.ssdp_port,
            description_url.clone(),
            udn.clone(),
        ) {
            Ok(ssdp) => ssdp,
            Err(error) => {
                server.stop().await;
                return self
                    .fail_start(format!("the SSDP responder could not be started: {error}"));
            }
        };
        let udp_port = ssdp.local_port();

        if let Err(error) = ssdp.send_alive(&description_url, &udn) {
            tracing::warn!(reason = %error, "the SSDP announcement could not be sent");
        }

        {
            let mut lifecycle = self.lock_lifecycle();
            lifecycle.set_tcp_port(Some(tcp_port));
            lifecycle.set_udp_port(Some(udp_port));
            lifecycle.set_state(ProtocolState::Advertising);
            lifecycle.set_last_error(None);
        }
        *self
            .runtime
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = Some(Runtime {
            server,
            ssdp,
            base_url,
            description_url,
            udn,
        });
        Ok(())
    }

    async fn stop(&self) -> Result<(), EngineError> {
        let runtime = self
            .runtime
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .take();
        let Some(mut runtime) = runtime else {
            return Err(EngineError::NotRunning {
                protocol: "smartview",
            });
        };
        self.lock_lifecycle().set_state(ProtocolState::Stopping);
        if let Err(error) = runtime.ssdp.send_byebye(&runtime.udn) {
            tracing::warn!(reason = %error, "the SSDP farewell could not be sent");
        }
        runtime.server.stop().await;
        runtime.ssdp.stop();
        {
            let mut lifecycle = self.lock_lifecycle();
            lifecycle.set_tcp_port(None);
            lifecycle.set_udp_port(None);
            lifecycle.set_state(ProtocolState::Stopped);
        }
        Ok(())
    }
}

impl SmartViewEngine {
    /// Records a start failure and returns the matching error.
    fn fail_start<T>(&self, reason: String) -> Result<T, EngineError> {
        let mut lifecycle = self.lock_lifecycle();
        lifecycle.set_state(ProtocolState::Failed);
        lifecycle.set_last_error(Some(reason.clone()));
        Err(EngineError::Start {
            protocol: "smartview",
            reason,
        })
    }
}

#[cfg(test)]
mod tests;
