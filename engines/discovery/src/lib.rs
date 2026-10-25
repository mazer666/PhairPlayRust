//! Service discovery for the PhairPlayRust receiver stack: mDNS advertisement
//! and SSDP browsing.
//!
//! WHAT: this crate publishes the receiver on the local network. It
//! registers the four DNS-SD services the protocols of PhairPlayRust need
//! (`_airplay._tcp`, `_raop._tcp`, `_googlecast._tcp`, `_miracast._tcp`) and
//! it can browse the network over SSDP to find the devices that are out
//! there.
//!
//! WHY: a sender that cannot find the receiver never talks to it. The
//! advertisement is therefore part of every protocol, not an optional
//! extra — and it is centralised here so that one place decides the
//! instance names, the TXT records and the failure handling.
//!
//! HOW: the pure parts — the service types, the naming rules and the TXT
//! records — are plain functions that are unit tested without a network.
//! The network parts sit behind the [`mdns::ServiceRegistrar`] and
//! [`ssdp::SsdpBrowser`] seams, so that a test can assert on the exact
//! bytes that would go on the wire.
//!
//! ## A note on [`phairplay_core::protocol::ProtocolEngine`]
//!
//! This crate does **not** implement `ProtocolEngine`, and the reason is
//! structural: the trait requires a
//! [`ProtocolKind`](phairplay_core::protocol::ProtocolKind), and the core
//! crate knows four of them — AirPlay, Miracast, Cast and DLNA. The
//! discovery layer is not a fifth protocol; it is the facility that the
//! protocols share. Adding a `ProtocolKind::Discovery` variant would
//! change the core vocabulary, which the build contract reserves for the
//! owner (see `docs/design/BUILD-CONTRACT.md`, section 2).
//!
//! ## Example
//!
//! ```
//! use phairplay_core::identity::DeviceIdentity;
//! use phairplay_discovery::mdns::{DiscoveryConfig, DiscoveryService, RecordingRegistrar};
//! use std::sync::Arc;
//!
//! let registrar = Arc::new(RecordingRegistrar::new());
//! let discovery = DiscoveryService::new(registrar.clone());
//! discovery
//!     .start(&DiscoveryConfig::new(DeviceIdentity::with_name("Living Room")))
//!     .expect("the services are registered");
//!
//! // AirPlay needs *two* service types; a receiver that publishes only
//! // one of them is invisible to some senders.
//! let types: Vec<String> = registrar
//!     .registered()
//!     .into_iter()
//!     .map(|service| service.service_type)
//!     .collect();
//! assert!(types.contains(&"_airplay._tcp".to_string()));
//! assert!(types.contains(&"_raop._tcp".to_string()));
//! ```

#![warn(missing_docs)]

pub mod mdns;
pub mod service_types;
pub mod ssdp;
pub mod txt;

pub use mdns::{
    AdvertisedService, DiscoveryConfig, DiscoveryService, DiscoveryStatus, MdnsDaemonRegistrar,
    RecordingRegistrar, RegistrationOutcome, ServiceRegistrar, ServiceSpec,
};
pub use service_types::{
    AIRPLAY_TCP, ALL_SERVICE_TYPES, GOOGLECAST_TCP, MAX_SERVICE_NAME_LENGTH, MIRACAST_TCP,
    RAOP_TCP, format_mac, is_known_service_type, normalise_mac, pseudo_mac, raop_instance_name,
};
pub use ssdp::{
    SsdpBrowser, SsdpError, SsdpEvent, SsdpNotification, SsdpResponse, SsdpSearchQuery,
};
pub use txt::TxtRecord;
