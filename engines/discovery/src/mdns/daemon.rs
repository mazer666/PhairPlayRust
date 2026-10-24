//! The registrar that talks to the system mDNS daemon.
//!
//! WHY: the advertisement has to reach the network, and the network is
//! reached through the daemon of the operating system. The daemon is the
//! only part of this crate that can fail because of the environment —
//! therefore it is the only part that is allowed to know about `mdns-sd`.
//!
//! HOW: the registrar translates a [`ServiceSpec`] into the service
//! information of the daemon and registers it. A name collision is
//! resolved by the daemon itself, which appends ` (2)`; the registrar
//! reports the name that was *actually* registered.

use super::{MDNS_DOMAIN, RegistrationOutcome, ServiceRegistrar, ServiceSpec};
use phairplay_core::error::EngineError;

/// The registrar that talks to the system mDNS daemon.
pub struct MdnsDaemonRegistrar {
    daemon: mdns_sd::ServiceDaemon,
}

impl std::fmt::Debug for MdnsDaemonRegistrar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MdnsDaemonRegistrar")
            .finish_non_exhaustive()
    }
}

impl MdnsDaemonRegistrar {
    /// Starts the mDNS daemon.
    ///
    /// # Errors
    /// Returns [`EngineError::Start`] when the daemon could not be started,
    /// for example because no usable network interface exists.
    pub fn new() -> Result<Self, EngineError> {
        let daemon = mdns_sd::ServiceDaemon::new().map_err(|error| EngineError::Start {
            protocol: "discovery",
            reason: format!("the mDNS daemon could not be started: {error}"),
        })?;
        Ok(Self { daemon })
    }
}

impl ServiceRegistrar for MdnsDaemonRegistrar {
    fn register(&self, spec: &ServiceSpec) -> Result<RegistrationOutcome, EngineError> {
        let properties: Vec<(&str, &str)> = spec
            .properties
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let ty = format!("{}.{}", spec.service_type, MDNS_DOMAIN);
        let info = mdns_sd::ServiceInfo::new(
            &ty,
            &spec.instance_name,
            &spec.host_name,
            "0.0.0.0",
            spec.port,
            &properties[..],
        )
        .map_err(|error| EngineError::Start {
            protocol: "discovery",
            reason: format!("the service {} is invalid: {error}", spec.service_type),
        })?
        .enable_addr_auto();
        let full_name = info.get_fullname().to_string();
        self.daemon
            .register(info)
            .map_err(|error| EngineError::Start {
                protocol: "discovery",
                reason: format!("the mDNS daemon refused {}: {error}", spec.service_type),
            })?;
        Ok(RegistrationOutcome { full_name })
    }

    fn unregister(&self, full_name: &str) -> Result<(), EngineError> {
        self.daemon
            .unregister(full_name)
            .map_err(|error| EngineError::Protocol {
                protocol: "discovery",
                reason: format!("the service {full_name} could not be withdrawn: {error}"),
            })?;
        Ok(())
    }

    fn shutdown(&self) {
        let _ = self.daemon.shutdown();
    }
}
