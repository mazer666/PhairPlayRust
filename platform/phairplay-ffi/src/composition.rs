//! The composition root of the receiver.
//!
//! WHAT: the one place that knows which engines exist, builds them from the
//! configuration and hands them to the supervisor.
//!
//! WHY: a receiver that is started without its protocols is a receiver that
//! shows four times "not started" and waits for a click in a browser. On a
//! Raspberry Pi there is no browser: the program *is* the product. The start
//! therefore belongs to the start of the process — and it belongs to exactly
//! one place, because a second registration of the engines would be a second
//! truth about what runs.
//!
//! WHY it lives in the bridge and not in an application: the native shells
//! and the local dashboard reach the receiver through the bridge. If the
//! composition root lived in one of them, the other one would start a
//! receiver without a single engine — and the supervisor would answer every
//! request with "no engine implementation is registered for the protocol".
//!
//! WHY the failure of one protocol does not stop the others: a container
//! without multicast cannot register a DNS-SD service, and a device without
//! a Wi-Fi Direct radio cannot form a Miracast group. Neither is a reason to
//! leave the AirPlay receiver of the same device switched off. The start is
//! a **total judgement**: the receiver runs, and every protocol that could
//! not start is reported with its reason.
//!
//! EXAMPLE:
//!
//! ```text
//! let core = phairplay_ffi::composition::build(&config)?;
//! let startup = phairplay_ffi::composition::start(&core);
//! // Every protocol of the configuration is answered for — started or not.
//! assert_eq!(startup.outcomes.len(), ProtocolKind::ALL.len());
//! phairplay_ffi::composition::stop(&core);
//! ```

use std::net::Ipv4Addr;
use std::sync::Arc;

use phairplay_airplay::{AirPlayConfig, AirPlayEngine};
use phairplay_cast::{CastConfig, CastEngine};
use phairplay_core::config::{AppConfig, ports};
use phairplay_core::protocol::{ProtocolKind, ProtocolState};
use phairplay_dlna::{DlnaConfig, DlnaEngine, HttpLimits};
use phairplay_miracast::{MiracastConfig, MiracastEngine};
use phairplay_smartview::{ApiLimits, SmartViewConfig, SmartViewEngine};

use crate::error::{FfiError, FfiErrorCode};
use crate::supervisor::ReceiverCore;

/// What happened to one protocol during the start of the receiver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolOutcome {
    /// The protocol this outcome is about.
    pub protocol: ProtocolKind,
    /// The state the protocol is in after the start.
    pub state: ProtocolState,
    /// Why the protocol did not start, if it did not.
    pub reason: Option<String>,
}

impl ProtocolOutcome {
    /// Whether the protocol is running.
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.state.accepts_peers()
    }
}

/// The report of the start of the receiver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Startup {
    /// The outcome of every protocol, in the order of the core.
    pub outcomes: Vec<ProtocolOutcome>,
}

impl Startup {
    /// The number of protocols that are running.
    #[must_use]
    pub fn running(&self) -> usize {
        self.outcomes
            .iter()
            .filter(|outcome| outcome.is_running())
            .count()
    }

    /// The number of protocols that could not start.
    #[must_use]
    pub fn failed(&self) -> usize {
        self.outcomes
            .iter()
            .filter(|outcome| outcome.reason.is_some() && !outcome.is_running())
            .count()
    }

    /// The protocols that are running, with their names.
    #[must_use]
    pub fn running_protocols(&self) -> Vec<&'static str> {
        self.outcomes
            .iter()
            .filter(|outcome| outcome.is_running())
            .map(|outcome| outcome.protocol.as_str())
            .collect()
    }
}

/// Creates the receiver and registers the engines.
///
/// # Errors
/// Returns [`FfiError`] when the configuration is invalid or when the
/// runtime of the bridge cannot be created.
pub fn build(config: &AppConfig) -> Result<ReceiverCore, FfiError> {
    let receiver = ReceiverCore::new(config.clone())?;
    register(&receiver, config);
    Ok(receiver)
}

/// Registers the engine of every protocol with the supervisor.
///
/// WHY: the supervisor is the only place that starts, stops and watches an
/// engine. The engines are handed to it once, here, and are never talked
/// to again — the status of a protocol is read from the supervisor, not
/// from the engine.
fn register(receiver: &ReceiverCore, config: &AppConfig) {
    receiver.register_engine(Arc::new(AirPlayEngine::new(airplay(config))));
    receiver.register_engine(Arc::new(MiracastEngine::new(miracast(config))));
    receiver.register_engine(Arc::new(CastEngine::new(CastConfig::from_app_config(
        config,
    ))));
    receiver.register_engine(Arc::new(DlnaEngine::new(dlna(config))));
    receiver.register_engine(Arc::new(SmartViewEngine::new(smartview(config))));
}

/// Starts every protocol that the configuration enables.
///
/// WHY: the failure of one protocol is the failure of that protocol. The
/// receiver keeps running with the protocols that work, and the reason of
/// every failure is in the status of the protocol — which is what the
/// dashboard shows.
pub fn start(receiver: &ReceiverCore) -> Startup {
    let config = receiver.config();
    let mut outcomes = Vec::with_capacity(ProtocolKind::ALL.len());

    for protocol in ProtocolKind::ALL {
        if !config.is_enabled(protocol) {
            outcomes.push(ProtocolOutcome {
                protocol,
                state: ProtocolState::Idle,
                reason: Some("the protocol is switched off in the configuration".to_string()),
            });
            continue;
        }
        let outcome = match receiver.start_protocol(protocol) {
            Ok(status) => ProtocolOutcome {
                protocol,
                state: status.state,
                reason: None,
            },
            Err(error) => {
                tracing::error!(
                    protocol = protocol.as_str(),
                    reason = %error.details(),
                    "the protocol could not be started"
                );
                let status = receiver.protocol_status(protocol);
                ProtocolOutcome {
                    protocol,
                    state: status.state,
                    reason: Some(error.details().to_string()),
                }
            }
        };
        outcomes.push(outcome);
    }

    Startup { outcomes }
}

/// Stops every protocol that runs.
///
/// WHY: the process may only leave when the sockets are released. A
/// protocol that is not running answers with a failure — that is not an
/// error of the shutdown, and it is not reported as one.
pub fn stop(receiver: &ReceiverCore) {
    for protocol in ProtocolKind::ALL {
        // WHY: a protocol without an engine was never started and cannot be
        // stopped. Asking the supervisor to stop it would only produce a
        // failure that says nothing.
        if receiver.engine(protocol).is_none() {
            continue;
        }
        if let Err(error) = receiver.stop_protocol(protocol) {
            if FfiErrorCode::from_i32(error.code()) != Some(FfiErrorCode::InvalidState) {
                tracing::warn!(
                    protocol = protocol.as_str(),
                    reason = %error.details(),
                    "the protocol could not be stopped"
                );
            }
        }
    }
}

/// The configuration of the AirPlay engine.
fn airplay(config: &AppConfig) -> AirPlayConfig {
    let mut engine = AirPlayConfig::from_app_config(config);
    if engine.port == 0 {
        // WHY: an AirPlay sender looks for the receiver on port 7000. A
        // receiver that listens on an ephemeral port is invisible to every
        // sender — the port of the protocol is a contract, not a choice.
        engine.port = ports::AIRPLAY_RTSP;
    }
    engine
}

/// The configuration of the Miracast engine.
fn miracast(config: &AppConfig) -> MiracastConfig {
    let protocol = config.protocol(ProtocolKind::Miracast);
    MiracastConfig {
        enabled: protocol.enabled,
        port: if protocol.port == 0 {
            ports::MIRACAST_WFD
        } else {
            protocol.port
        },
        // WHY: Miracast serves one sender at a time. A second sender is
        // refused, not queued.
    }
}

/// The configuration of the DLNA engine.
fn dlna(config: &AppConfig) -> DlnaConfig {
    let protocol = config.protocol(ProtocolKind::Dlna);
    DlnaConfig {
        // WHY: the description server announces its own port in the SSDP
        // answer. A port of zero asks the kernel for a free one.
        http_port: protocol.port,
        ssdp_port: ports::SSDP,
        // WHY: the renderer announces the address the sender can reach. A
        // renderer that announces the loopback interface is unreachable
        // for everybody but the device itself.
        local_ip: local_address(),
        limits: HttpLimits {
            max_header_count: config.limits.max_header_count,
            max_header_bytes: config.limits.max_header_bytes,
            ..HttpLimits::default()
        },
        idle_timeout: config.performance.idle_timeout(),
    }
}

/// The configuration of the Samsung Smart View engine.
///
/// WHY: a Galaxy sender looks for the MultiScreen Framework of a receiver
/// on port 8001 — the port a Samsung television uses. A receiver that
/// answers somewhere else is not found.
fn smartview(config: &AppConfig) -> SmartViewConfig {
    let protocol = config.protocol(ProtocolKind::SmartView);
    SmartViewConfig {
        http_port: if protocol.port == 0 {
            ports::SMARTVIEW_API
        } else {
            protocol.port
        },
        ssdp_port: ports::SSDP,
        // WHY: the sender reaches the receiver at the address the receiver
        // announces. A receiver that announces the loopback interface is
        // unreachable for everybody but the device itself.
        local_ip: local_address(),
        limits: ApiLimits {
            max_header_count: config.limits.max_header_count,
            max_header_bytes: config.limits.max_header_bytes,
            ..ApiLimits::default()
        },
    }
}

/// The address of the device in the local network.
///
/// WHY: the renderer needs an address that a sender can reach. The first
/// non-loopback IPv4 address of the device is that address. A device
/// without one — a container without a network, for example — falls back
/// to the loopback interface, which is the honest answer: there is no
/// other address.
fn local_address() -> Ipv4Addr {
    if_addrs::get_if_addrs()
        .ok()
        .and_then(|interfaces| {
            interfaces
                .into_iter()
                .filter(|interface| !interface.is_loopback())
                .find_map(|interface| match interface.ip() {
                    std::net::IpAddr::V4(address) => Some(address),
                    std::net::IpAddr::V6(_) => None,
                })
        })
        .unwrap_or(Ipv4Addr::LOCALHOST)
}

#[cfg(test)]
mod tests;
