//! The DNS-SD advertisement of the Cast receiver.
//!
//! WHY: a sender finds a Cast receiver the same way it finds a printer: by
//! browsing DNS-SD for `_googlecast._tcp`. The TXT record is what tells the
//! sender *which* receiver it found — the device id, the model and the
//! friendly name — and what state it is in. A receiver that omits the TXT
//! record appears in the sender's list but cannot be connected to.
//!
//! WHAT: [`txt_properties`] builds the TXT record, [`device_id`] derives the
//! 32 hex character identifier the Cast protocol uses, and
//! [`ServiceRegistrar`] is the port through which the engine publishes the
//! service — the mDNS implementation lives in [`MdnsRegistrar`].
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_cast::advertisement::{READY_STATE_IDLE, txt_properties};
//! use phairplay_core::identity::DeviceIdentity;
//!
//! let identity = DeviceIdentity::with_name("Living Room");
//! let properties = txt_properties(&identity, "PhairPlay Rust Receiver", "05", READY_STATE_IDLE);
//! assert!(properties.iter().any(|(key, value)| key == "fn" && value == "Living Room"));
//! ```

use std::net::{IpAddr, Ipv4Addr};
use std::sync::Mutex;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use phairplay_core::identity::DeviceIdentity;
use sha2::{Digest, Sha256};

/// The DNS-SD service type of a Google Cast receiver.
pub const GOOGLECAST_SERVICE_TYPE: &str = "_googlecast._tcp";

/// The version the receiver reports in its TXT record.
pub const DEFAULT_TXT_VERSION: &str = "05";

/// The ready state of an idle receiver.
pub const READY_STATE_IDLE: &str = "IDLE";

/// The ready state of a receiver that is serving a sender.
pub const READY_STATE_CONNECTED: &str = "CONNECTED";

/// The number of hex characters a Cast device id has.
pub const DEVICE_ID_LENGTH: usize = 32;

/// A failure while advertising the service.
#[derive(Debug, thiserror::Error)]
pub enum AdvertiseError {
    /// The service could not be registered with the DNS-SD daemon.
    #[error("could not register the DNS-SD service: {0}")]
    Registration(String),
    /// No usable network interface was found.
    #[error("no usable network interface: {0}")]
    NoInterface(String),
}

/// The DNS-SD service the receiver offers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CastService {
    /// The name the sender shows, e.g. "Living Room".
    pub instance: String,
    /// The service type, always `_googlecast._tcp`.
    pub service_type: String,
    /// The host name of the receiver.
    pub host: String,
    /// The port the TLS listener runs on.
    pub port: u16,
    /// The TXT record of the service.
    pub properties: Vec<(String, String)>,
}

/// Builds the TXT record of the `_googlecast._tcp` service.
///
/// The record carries the keys the Cast protocol defines: `id` (the device
/// id), `cd` (the base64 encoded model description), `fn` (the friendly
/// name), `md` (the model), `rs` (the ready state), `ve` (the version) and
/// `ic` (the icon).
#[must_use]
pub fn txt_properties(
    identity: &DeviceIdentity,
    model: &str,
    version: &str,
    ready_state: &str,
) -> Vec<(String, String)> {
    let mut properties = Vec::with_capacity(7);
    properties.push(("id".to_string(), device_id(identity)));
    properties.push(("cd".to_string(), BASE64.encode(model.as_bytes())));
    properties.push(("fn".to_string(), identity.name.clone()));
    properties.push(("md".to_string(), model.to_string()));
    properties.push(("rs".to_string(), ready_state.to_string()));
    properties.push(("ve".to_string(), version.to_string()));
    properties.push(("ic".to_string(), String::new()));
    properties
}

/// The 32 hex character device id of the receiver.
///
/// WHY: the Cast protocol identifies a device by a 32 hex character id that
/// never changes. The identity of the receiver carries a UUID; when it is
/// already 32 hex characters it is used as is, otherwise the id is derived
/// from it with SHA-256 so that it stays stable across restarts and stays
/// the right length.
#[must_use]
pub fn device_id(identity: &DeviceIdentity) -> String {
    let raw = identity.device_id.trim();
    let hex: String = raw.chars().filter(char::is_ascii_hexdigit).collect();
    if hex.len() == DEVICE_ID_LENGTH {
        return hex.to_ascii_lowercase();
    }
    let digest = Sha256::digest(raw.as_bytes());
    digest[..DEVICE_ID_LENGTH / 2]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// The port a service registrar publishes services on.
pub trait ServiceRegistrar: Send + Sync + std::fmt::Debug {
    /// Publishes the service.
    ///
    /// # Errors
    /// Returns [`AdvertiseError`] when the service cannot be published.
    fn register(&self, service: &CastService) -> Result<(), AdvertiseError>;

    /// Withdraws every service this registrar published.
    fn unregister(&self);
}

/// A registrar that publishes the service over DNS-SD.
#[derive(Default)]
pub struct MdnsRegistrar {
    state: Mutex<MdnsState>,
}

#[derive(Default)]
struct MdnsState {
    daemon: Option<mdns_sd::ServiceDaemon>,
    full_name: Option<String>,
}

impl std::fmt::Debug for MdnsRegistrar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = self.state.lock().ok();
        f.debug_struct("MdnsRegistrar")
            .field(
                "registered",
                &state.and_then(|state| state.full_name.clone()),
            )
            .finish()
    }
}

impl MdnsRegistrar {
    /// Creates a registrar without a running daemon.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The name of the service that is currently published.
    #[must_use]
    pub fn published_service(&self) -> Option<String> {
        self.state
            .lock()
            .ok()
            .and_then(|state| state.full_name.clone())
    }
}

impl ServiceRegistrar for MdnsRegistrar {
    fn register(&self, service: &CastService) -> Result<(), AdvertiseError> {
        let address = local_address()?;
        let properties: Vec<(&str, &str)> = service
            .properties
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
            .collect();
        let domain = format!("{}.local.", service.service_type);
        let info = mdns_sd::ServiceInfo::new(
            &domain,
            &service.instance,
            &service.host,
            address,
            service.port,
            properties.as_slice(),
        )
        .map_err(|error| AdvertiseError::Registration(error.to_string()))?;
        let daemon = mdns_sd::ServiceDaemon::new()
            .map_err(|error| AdvertiseError::Registration(error.to_string()))?;
        daemon
            .register(info)
            .map_err(|error| AdvertiseError::Registration(error.to_string()))?;
        let full_name = format!("{}.{}", service.instance, service.service_type);
        let mut state = self
            .state
            .lock()
            .map_err(|_| AdvertiseError::Registration("the registrar is poisoned".to_string()))?;
        state.daemon = Some(daemon);
        state.full_name = Some(full_name);
        Ok(())
    }

    fn unregister(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.daemon = None;
            state.full_name = None;
        }
    }
}

/// The address the service is advertised on.
///
/// A routable IPv4 address is preferred; the loopback is used when the
/// device has no other interface, so that the receiver still starts and can
/// be reached from the machine it runs on.
fn local_address() -> Result<IpAddr, AdvertiseError> {
    let interfaces =
        if_addrs::get_if_addrs().map_err(|error| AdvertiseError::NoInterface(error.to_string()))?;
    let mut fallback = None;
    for interface in interfaces {
        if !interface.is_loopback() {
            if let IpAddr::V4(address) = interface.ip() {
                if !address.is_unspecified() {
                    return Ok(IpAddr::V4(address));
                }
            }
        } else {
            fallback = Some(interface.ip());
        }
    }
    fallback
        .or(Some(IpAddr::V4(Ipv4Addr::LOCALHOST)))
        .ok_or_else(|| AdvertiseError::NoInterface("no address found".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity() -> DeviceIdentity {
        DeviceIdentity {
            name: "Living Room".to_string(),
            model: "PhairPlay Rust Receiver".to_string(),
            manufacturer: "PhairPlayRust".to_string(),
            protocol_version: "1.0".to_string(),
            device_id: "0123456789abcdef0123456789abcdef".to_string(),
        }
    }

    #[test]
    fn the_txt_record_carries_every_key_the_cast_protocol_defines() {
        let properties = txt_properties(
            &identity(),
            "PhairPlay Rust Receiver",
            DEFAULT_TXT_VERSION,
            READY_STATE_IDLE,
        );
        let keys: Vec<&str> = properties.iter().map(|(key, _)| key.as_str()).collect();
        assert!(keys.contains(&"id"));
        assert!(keys.contains(&"cd"));
        assert!(keys.contains(&"fn"));
        assert!(keys.contains(&"md"));
        assert!(keys.contains(&"rs"));
        assert!(keys.contains(&"ve"));
        assert!(keys.contains(&"ic"));
    }

    #[test]
    fn the_friendly_name_is_the_device_name() {
        let properties = txt_properties(&identity(), "model", "05", READY_STATE_IDLE);
        let friendly = properties
            .iter()
            .find(|(key, _)| key == "fn")
            .map(|(_, value)| value.clone());
        assert_eq!(friendly, Some("Living Room".to_string()));
    }

    #[test]
    fn the_model_description_is_base64_encoded() {
        let properties = txt_properties(&identity(), "Chromecast,Ultra", "05", READY_STATE_IDLE);
        let encoded = properties
            .iter()
            .find(|(key, _)| key == "cd")
            .map(|(_, value)| value.clone());
        assert_eq!(encoded, Some(BASE64.encode("Chromecast,Ultra".as_bytes())));
    }

    #[test]
    fn the_device_id_is_thirty_two_hex_characters() {
        let id = device_id(&identity());
        assert_eq!(id.len(), DEVICE_ID_LENGTH);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn a_uuid_device_id_is_hashed_into_a_stable_id() {
        let mut identity = identity();
        identity.device_id = "6f0a1b2c-3d4e-5f60-7182-93a4b5c6d7e8".to_string();
        let first = device_id(&identity);
        let second = device_id(&identity);
        assert_eq!(first, second);
        assert_eq!(first.len(), DEVICE_ID_LENGTH);
        assert!(first.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn two_different_devices_get_different_ids() {
        let mut other = identity();
        other.device_id = "ffffffffffffffffffffffffffffffff".to_string();
        assert_ne!(device_id(&identity()), device_id(&other));
    }

    #[test]
    fn the_service_type_is_the_one_the_senders_browse() {
        assert_eq!(GOOGLECAST_SERVICE_TYPE, "_googlecast._tcp");
    }
}
