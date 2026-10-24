//! The mDNS (DNS-SD) advertisement layer.
//!
//! WHY: a sender finds a receiver through DNS-SD. Without a registration
//! there is nothing to find: the device never appears in the AirPlay, Cast
//! or Miracast picker of any sender. The advertisement is therefore the
//! first thing the receiver does and the last thing it gives up.
//!
//! HOW: the [`ServiceRegistrar`] trait is the seam between *what* is
//! advertised and *how* it reaches the network. The production implementation
//! talks to the `mdns-sd` daemon; the tests use a recording double, which
//! makes the exact service names, ports and TXT records assertable without
//! a network in the loop.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_discovery::mdns::{DiscoveryConfig, DiscoveryService, RecordingRegistrar};
//! use phairplay_core::identity::DeviceIdentity;
//! use std::sync::Arc;
//!
//! let registrar = Arc::new(RecordingRegistrar::default());
//! let service = DiscoveryService::new(registrar.clone());
//! let config = DiscoveryConfig::new(DeviceIdentity::with_name("Living Room"));
//! service.start(&config).expect("registration succeeds");
//!
//! let advertised = registrar.registered();
//! assert!(advertised.iter().any(|s| s.service_type == "_airplay._tcp"));
//! assert!(advertised.iter().any(|s| s.service_type == "_raop._tcp"));
//! ```

use std::sync::{Arc, Mutex};

use phairplay_core::error::EngineError;

use crate::service_types::{
    AIRPLAY_TCP, GOOGLECAST_TCP, MIRACAST_TCP, RAOP_TCP, pseudo_mac, raop_instance_name,
};
use crate::txt::TxtRecord;

/// The default port of the AirPlay RTSP listener.
///
/// WHY: the value is the port every AirPlay sender looks for. Its single
/// owner is [`phairplay_core::config::ports::AIRPLAY_RTSP`].
pub const DEFAULT_AIRPLAY_PORT: u16 = phairplay_core::config::ports::AIRPLAY_RTSP;

/// The default port of the Google Cast TLS listener.
///
/// WHY: the value is the port every Cast sender looks for. Its single
/// owner is [`phairplay_core::config::ports::CAST_TLS`].
pub const DEFAULT_CAST_PORT: u16 = phairplay_core::config::ports::CAST_TLS;

/// The default port of the Miracast / WFD RTSP listener.
///
/// WHY: the value is the port every WFD sender looks for. Its single owner
/// is [`phairplay_core::config::ports::MIRACAST_WFD`].
pub const DEFAULT_MIRACAST_PORT: u16 = phairplay_core::config::ports::MIRACAST_WFD;

/// The mDNS domain every service is registered in.
pub const MDNS_DOMAIN: &str = "local.";

/// One DNS-SD service to advertise.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceSpec {
    /// The service type, e.g. `_airplay._tcp`.
    pub service_type: String,
    /// The instance name, e.g. `Living Room`.
    pub instance_name: String,
    /// The host name the sender resolves, e.g. `phairplay.local.`.
    pub host_name: String,
    /// The port the service listens on.
    pub port: u16,
    /// The TXT records.
    pub properties: Vec<(String, String)>,
}

impl ServiceSpec {
    /// Builds a specification from a TXT record.
    #[must_use]
    pub fn new(
        service_type: &str,
        instance_name: &str,
        host_name: &str,
        port: u16,
        record: &TxtRecord,
    ) -> Self {
        Self {
            service_type: service_type.to_string(),
            instance_name: instance_name.to_string(),
            host_name: host_name.to_string(),
            port,
            properties: record.entries().to_vec(),
        }
    }

    /// The value of a TXT record.
    #[must_use]
    pub fn property(&self, key: &str) -> Option<&str> {
        self.properties
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }
}

/// The result of a successful registration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrationOutcome {
    /// The fully qualified name the daemon actually registered.
    ///
    /// WHY: an mDNS daemon resolves a name collision by appending ` (2)`.
    /// The registered name therefore differs from the requested one, and the
    /// receiver has to report the *actual* name to the user interface.
    pub full_name: String,
}

/// A sink for DNS-SD registrations.
///
/// WHY: the trait exists so that the advertisement logic — which names, which
/// ports, which TXT records — can be tested without a network, and so that
/// the platform layer can replace the daemon with its own implementation.
pub trait ServiceRegistrar: Send + Sync + std::fmt::Debug {
    /// Registers one service.
    ///
    /// # Errors
    /// Returns [`EngineError::Start`] when the daemon refused the
    /// registration, for example because the instance name is invalid.
    fn register(&self, spec: &ServiceSpec) -> Result<RegistrationOutcome, EngineError>;

    /// Removes a previously registered service.
    ///
    /// # Errors
    /// Returns [`EngineError::Protocol`] when the daemon does not know the
    /// service. An unknown service is *not* a fatal error: the sender
    /// caches the registration until it expires, so the receiver simply
    /// stops advertising.
    fn unregister(&self, full_name: &str) -> Result<(), EngineError>;

    /// Releases every resource the registrar holds.
    fn shutdown(&self) {}
}

/// A registrar that records what it was asked to do.
///
/// WHY: the test double makes the advertisement assertable: the exact
/// instance names, the ports and the TXT records are visible without a
/// network, a daemon or a timing dependency.
#[derive(Debug, Default)]
pub struct RecordingRegistrar {
    registered: Mutex<Vec<ServiceSpec>>,
    outcomes: Mutex<Vec<RegistrationOutcome>>,
    unregister_calls: Mutex<Vec<String>>,
    fail_on: Mutex<Option<String>>,
}

impl RecordingRegistrar {
    /// A registrar that accepts everything.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A registrar that refuses the given service type.
    #[must_use]
    pub fn failing_on(service_type: &str) -> Self {
        let registrar = Self::default();
        *registrar.fail_on.lock().unwrap_or_else(|e| e.into_inner()) =
            Some(service_type.to_string());
        registrar
    }

    /// Every service that was registered, in registration order.
    #[must_use]
    pub fn registered(&self) -> Vec<ServiceSpec> {
        self.registered
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// Every name that was unregistered.
    #[must_use]
    pub fn unregistered(&self) -> Vec<String> {
        self.unregister_calls
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
}

impl ServiceRegistrar for RecordingRegistrar {
    fn register(&self, spec: &ServiceSpec) -> Result<RegistrationOutcome, EngineError> {
        let fail = self
            .fail_on
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        if fail.as_deref() == Some(spec.service_type.as_str()) {
            return Err(EngineError::Start {
                protocol: "discovery",
                reason: format!(
                    "the mDNS daemon refused {}: simulated failure",
                    spec.service_type
                ),
            });
        }
        let outcome = RegistrationOutcome {
            full_name: format!(
                "{}.{}.{}",
                spec.instance_name, spec.service_type, MDNS_DOMAIN
            ),
        };
        self.registered
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(spec.clone());
        self.outcomes
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(outcome.clone());
        Ok(outcome)
    }

    fn unregister(&self, full_name: &str) -> Result<(), EngineError> {
        self.unregister_calls
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(full_name.to_string());
        Ok(())
    }
}

mod config;
mod daemon;

pub use config::DiscoveryConfig;
pub use daemon::MdnsDaemonRegistrar;

/// One service that is currently advertised.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvertisedService {
    /// The service type.
    pub service_type: String,
    /// The instance name that was requested.
    pub instance_name: String,
    /// The fully qualified name the daemon registered.
    pub full_name: String,
}

/// The public state of the discovery layer.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DiscoveryStatus {
    /// Whether the layer is currently advertising.
    pub running: bool,
    /// The services that are currently advertised.
    pub advertised: Vec<AdvertisedService>,
    /// The reason of the last failure.
    pub last_error: Option<String>,
}

/// The internal state of the discovery layer.
#[derive(Debug, Default)]
struct DiscoveryState {
    running: bool,
    advertised: Vec<AdvertisedService>,
    last_error: Option<String>,
}

/// The mDNS advertisement layer.
///
/// WHY: the four protocols of PhairPlayRust all rely on the receiver being
/// discoverable. Centralising the advertisement here means one place decides
/// the instance names, the TXT records and the failure handling — and one
/// place has to be correct.
#[derive(Debug)]
pub struct DiscoveryService {
    registrar: Arc<dyn ServiceRegistrar>,
    state: Mutex<DiscoveryState>,
}

impl DiscoveryService {
    /// A discovery layer that registers through the given registrar.
    #[must_use]
    pub fn new(registrar: Arc<dyn ServiceRegistrar>) -> Self {
        Self {
            registrar,
            state: Mutex::new(DiscoveryState::default()),
        }
    }

    /// The public state of the layer.
    #[must_use]
    pub fn status(&self) -> DiscoveryStatus {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        DiscoveryStatus {
            running: state.running,
            advertised: state.advertised.clone(),
            last_error: state.last_error.clone(),
        }
    }

    /// Registers every configured service.
    ///
    /// WHY: the services are registered as a unit. A receiver that advertises
    /// `_airplay._tcp` but not `_raop._tcp` is *worse* than one that
    /// advertises neither, because the sender shows a device that then
    /// cannot stream. If one registration fails, the ones that already
    /// succeeded are withdrawn again.
    ///
    /// # Errors
    /// Returns [`EngineError::AlreadyRunning`] when the layer is already
    /// advertising and [`EngineError::Start`] when a registration failed.
    pub fn start(&self, config: &DiscoveryConfig) -> Result<(), EngineError> {
        {
            let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
            if state.running {
                return Err(EngineError::AlreadyRunning {
                    protocol: "discovery",
                });
            }
        }

        let mut advertised: Vec<AdvertisedService> = Vec::new();
        for spec in config.specs() {
            match self.registrar.register(&spec) {
                Ok(outcome) => advertised.push(AdvertisedService {
                    service_type: spec.service_type.clone(),
                    instance_name: spec.instance_name.clone(),
                    full_name: outcome.full_name,
                }),
                Err(error) => {
                    // Withdraw the services that were registered before the
                    // failure, so that the receiver is either fully
                    // discoverable or not discoverable at all.
                    for done in &advertised {
                        let _ = self.registrar.unregister(&done.full_name);
                    }
                    let reason = error.to_string();
                    let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
                    state.running = false;
                    state.advertised.clear();
                    state.last_error = Some(reason.clone());
                    return Err(EngineError::Start {
                        protocol: "discovery",
                        reason,
                    });
                }
            }
        }

        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.running = true;
        state.advertised = advertised;
        state.last_error = None;
        Ok(())
    }

    /// Withdraws every advertised service.
    ///
    /// # Errors
    /// Returns [`EngineError::NotRunning`] when the layer was not
    /// advertising.
    pub fn stop(&self) -> Result<(), EngineError> {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if !state.running {
            return Err(EngineError::NotRunning {
                protocol: "discovery",
            });
        }
        let mut failure = None;
        for service in &state.advertised {
            if let Err(error) = self.registrar.unregister(&service.full_name) {
                failure = Some(error.to_string());
            }
        }
        if let Some(reason) = failure {
            state.last_error = Some(reason);
        }
        state.running = false;
        state.advertised.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests;
