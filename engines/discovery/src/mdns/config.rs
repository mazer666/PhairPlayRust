//! What the discovery layer advertises.
//!
//! WHY: the advertisement is a decision, not an accident: which service
//! types are published, under which name, on which port and with which
//! records. The configuration is where that decision is written down.
//!
//! HOW: the configuration knows the identity and the capabilities of the
//! receiver and turns them into the service specifications that the
//! registrar publishes.

use phairplay_core::capabilities::MediaCapabilities;
use phairplay_core::identity::{DeviceIdentity, sanitize_service_name};

use super::{
    AIRPLAY_TCP, DEFAULT_AIRPLAY_PORT, DEFAULT_CAST_PORT, DEFAULT_MIRACAST_PORT, GOOGLECAST_TCP,
    MIRACAST_TCP, RAOP_TCP, ServiceSpec, pseudo_mac, raop_instance_name,
};
use crate::txt::{airplay_txt, cast_txt, miracast_txt, raop_txt};

/// What the discovery layer advertises.
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// The identity advertised to senders.
    pub identity: DeviceIdentity,
    /// The capabilities of this receiver.
    pub capabilities: MediaCapabilities,
    /// The host name the senders resolve, e.g. `phairplay.local.`.
    pub host_name: String,
    /// The port of the AirPlay RTSP listener.
    pub airplay_port: u16,
    /// The port of the Google Cast listener.
    pub cast_port: u16,
    /// The port of the Miracast / WFD RTSP listener.
    pub miracast_port: u16,
    /// The service types to advertise.
    pub service_types: Vec<String>,
}

impl DiscoveryConfig {
    /// A configuration that advertises every service type.
    #[must_use]
    pub fn new(identity: DeviceIdentity) -> Self {
        Self {
            identity,
            capabilities: MediaCapabilities::empty(),
            host_name: "phairplay.local.".to_string(),
            airplay_port: DEFAULT_AIRPLAY_PORT,
            cast_port: DEFAULT_CAST_PORT,
            miracast_port: DEFAULT_MIRACAST_PORT,
            service_types: vec![
                AIRPLAY_TCP.to_string(),
                RAOP_TCP.to_string(),
                GOOGLECAST_TCP.to_string(),
                MIRACAST_TCP.to_string(),
            ],
        }
    }

    /// Restricts the advertisement to the given service types.
    #[must_use]
    pub fn with_service_types(mut self, service_types: Vec<String>) -> Self {
        self.service_types = service_types;
        self
    }

    /// Sets the ports of the protocol listeners.
    #[must_use]
    pub fn with_ports(mut self, airplay: u16, cast: u16, miracast: u16) -> Self {
        self.airplay_port = airplay;
        self.cast_port = cast;
        self.miracast_port = miracast;
        self
    }

    /// The DNS-SD instance name of this receiver.
    ///
    /// WHY: the instance name is what the user sees in the sender's device
    /// picker. It has to be sanitised, because an mDNS daemon rejects a
    /// name with control characters and the rejection takes the whole
    /// registration path down with it.
    #[must_use]
    pub fn instance_name(&self) -> String {
        sanitize_service_name(&self.identity.name, "PhairPlay")
    }

    /// The DNS-SD instance name of the RAOP service.
    #[must_use]
    pub fn raop_instance_name(&self) -> String {
        raop_instance_name(&pseudo_mac(&self.identity.device_id), &self.instance_name())
    }

    /// The service specifications that correspond to this configuration.
    #[must_use]
    pub fn specs(&self) -> Vec<ServiceSpec> {
        let instance = self.instance_name();
        let mut specs = Vec::new();
        for service_type in &self.service_types {
            let spec = match service_type.as_str() {
                AIRPLAY_TCP => ServiceSpec::new(
                    AIRPLAY_TCP,
                    &instance,
                    &self.host_name,
                    self.airplay_port,
                    &airplay_txt(&self.identity),
                ),
                RAOP_TCP => ServiceSpec::new(
                    RAOP_TCP,
                    &self.raop_instance_name(),
                    &self.host_name,
                    self.airplay_port,
                    &raop_txt(&self.identity),
                ),
                GOOGLECAST_TCP => ServiceSpec::new(
                    GOOGLECAST_TCP,
                    &instance,
                    &self.host_name,
                    self.cast_port,
                    &cast_txt(&self.identity),
                ),
                MIRACAST_TCP => ServiceSpec::new(
                    MIRACAST_TCP,
                    &instance,
                    &self.host_name,
                    self.miracast_port,
                    &miracast_txt(&self.identity),
                ),
                _ => continue,
            };
            specs.push(spec);
        }
        specs
    }
}
