//! The DNS-SD service types of the receiver protocols.
//!
//! WHAT: the service types PhairPlayRust advertises over mDNS, and the one
//! type it looks up on a sender.
//!
//! WHY: the service *type* is what a sender looks for. A receiver that
//! advertises the wrong type is invisible; a receiver that advertises only
//! half of a type pair is invisible to *some* senders. AirPlay is the prime
//! example: an AirPlay receiver must publish **both** `_airplay._tcp` (the
//! mirroring pipeline) and `_raop._tcp` (the audio pipeline) — macOS and iOS
//! refuse to show a device that publishes only `_airplay._tcp` when the user
//! picks the audio entry point, and several senders filter the device list
//! on `_raop._tcp` alone.
//!
//! The spelling of a type used to be written down in three crates at once
//! (`core`, `engines/discovery` and `engines/cast`). A typo in one of them
//! makes the receiver disappear from one sender and stay visible in another.
//!
//! HOW: the constants below are the single definition.
//! [`crate::protocol::ProtocolKind::mdns_service_types`] reads them, the
//! discovery crate re-exports them and the Cast advertisement refers to
//! them.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_core::config::service_types;
//!
//! assert_eq!(service_types::AIRPLAY_TCP, "_airplay._tcp");
//! assert_eq!(service_types::RAOP_TCP, "_raop._tcp");
//! ```

/// The AirPlay mirroring service type.
pub const AIRPLAY_TCP: &str = "_airplay._tcp";

/// The RAOP (Remote Audio Output Protocol) service type.
///
/// WHY: it is required *alongside* [`AIRPLAY_TCP`], not instead of it.
pub const RAOP_TCP: &str = "_raop._tcp";

/// The Google Cast service type.
pub const GOOGLECAST_TCP: &str = "_googlecast._tcp";

/// The Miracast / Wi-Fi Display service type.
pub const MIRACAST_TCP: &str = "_miracast._tcp";

/// The DACP (Digital Audio Control Protocol) service type.
///
/// WHY: this one is *not* advertised by the receiver. It is the service a
/// sender publishes so that the receiver can push a "pause" or a "next
/// track" command back to it.
pub const DACP_TCP: &str = "_dacp._tcp";

/// The Samsung MultiScreen Framework service type.
///
/// WHY: a Galaxy phone looks for exactly this type when the user opens
/// Smart View. A receiver that does not publish it is invisible to every
/// Samsung sender.
pub const SAMSUNG_MSF_TCP: &str = "_samsungmsf._tcp";

/// Every service type the receiver advertises, in a stable order.
pub const ALL: [&str; 5] = [
    AIRPLAY_TCP,
    RAOP_TCP,
    GOOGLECAST_TCP,
    MIRACAST_TCP,
    SAMSUNG_MSF_TCP,
];

/// Whether a service type is one the receiver advertises.
#[must_use]
pub fn is_advertised(service_type: &str) -> bool {
    ALL.contains(&service_type)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    #[test]
    fn the_types_are_spelled_the_way_the_senders_expect() {
        assert_eq!(AIRPLAY_TCP, "_airplay._tcp");
        assert_eq!(RAOP_TCP, "_raop._tcp");
        assert_eq!(GOOGLECAST_TCP, "_googlecast._tcp");
        assert_eq!(MIRACAST_TCP, "_miracast._tcp");
        assert_eq!(SAMSUNG_MSF_TCP, "_samsungmsf._tcp");
        assert_eq!(DACP_TCP, "_dacp._tcp");
    }

    #[test]
    fn every_advertised_type_is_unique() {
        let mut sorted = ALL;
        sorted.sort_unstable();
        let mut deduplicated = sorted.to_vec();
        deduplicated.dedup();
        assert_eq!(deduplicated.len(), sorted.len(), "{sorted:?}");
    }

    #[test]
    fn the_dacp_type_is_not_an_advertised_type() {
        // WHY: the receiver looks DACP up on a sender; it never publishes it.
        assert!(!is_advertised(DACP_TCP));
        assert!(is_advertised(AIRPLAY_TCP));
        assert!(!is_advertised("_telepathy._tcp"));
    }
}
