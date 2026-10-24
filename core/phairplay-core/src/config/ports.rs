//! The well-known ports of the receiver protocols.
//!
//! WHAT: the default TCP and UDP ports every receiver of PhairPlayRust
//! binds, in one table.
//!
//! WHY: a port is a *contract with the sender*, not a private choice of an
//! engine. AirPlay senders look for the receiver on port 7000, Cast senders
//! on 8009, WFD senders on 7236. Before this module existed, the port 7000
//! was written down in three crates (`engines/airplay`, `engines/discovery`
//! and the tests of both) and the port 1900 in three more. A receiver that
//! advertises the right service on the wrong port is invisible, and a
//! documentation page that names the wrong port is a support ticket.
//!
//! HOW: the constants below are the single definition. The engine crates
//! keep their own public names (`DEFAULT_RTSP_PORT`, `SSDP_PORT`, …) so that
//! no public API breaks, but the *value* is read from here.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_core::config::ports;
//! use phairplay_core::protocol::ProtocolKind;
//!
//! assert_eq!(ports::AIRPLAY_RTSP, 7000);
//! assert_eq!(ports::default_port(ProtocolKind::Cast), Some(ports::CAST_TLS));
//! ```

use crate::protocol::ProtocolKind;

/// The TCP port of the AirPlay 2 / RTSP control channel.
pub const AIRPLAY_RTSP: u16 = 7000;

/// The UDP port of the Apple NTP timing service.
pub const AIRPLAY_TIMING: u16 = 6002;

/// The TCP port of the Miracast / Wi-Fi Display RTSP control plane.
pub const MIRACAST_WFD: u16 = 7236;

/// The TCP port of the Google Cast (CastV2 over TLS) control channel.
pub const CAST_TLS: u16 = 8009;

/// The UDP port of the UPnP / SSDP multicast group.
pub const SSDP: u16 = 1900;

/// Every well-known port, in a stable order.
pub const ALL: [u16; 5] = [AIRPLAY_RTSP, AIRPLAY_TIMING, MIRACAST_WFD, CAST_TLS, SSDP];

/// The default port a protocol listens on.
///
/// WHY: DLNA has no single port — the renderer announces itself over SSDP
/// and serves the description documents on an ephemeral HTTP port. `None` is
/// the honest answer for it, and a caller that binds a port of its own must
/// not be told that 1900 is a renderer port.
///
/// EXAMPLE:
///
/// ```
/// use phairplay_core::config::ports;
/// use phairplay_core::protocol::ProtocolKind;
///
/// assert_eq!(ports::default_port(ProtocolKind::AirPlay), Some(7000));
/// assert_eq!(ports::default_port(ProtocolKind::Dlna), None);
/// ```
#[must_use]
pub const fn default_port(protocol: ProtocolKind) -> Option<u16> {
    match protocol {
        ProtocolKind::AirPlay => Some(AIRPLAY_RTSP),
        ProtocolKind::Miracast => Some(MIRACAST_WFD),
        ProtocolKind::Cast => Some(CAST_TLS),
        ProtocolKind::Dlna => None,
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    #[test]
    fn the_ports_are_the_ones_the_senders_expect() {
        // WHY: these numbers are the contract with the senders. A change is
        // a change of the product, not a refactoring.
        assert_eq!(AIRPLAY_RTSP, 7000);
        assert_eq!(AIRPLAY_TIMING, 6002);
        assert_eq!(MIRACAST_WFD, 7236);
        assert_eq!(CAST_TLS, 8009);
        assert_eq!(SSDP, 1900);
    }

    #[test]
    fn every_port_is_listed_exactly_once() {
        let mut sorted = ALL;
        sorted.sort_unstable();
        let mut deduplicated = sorted.to_vec();
        deduplicated.dedup();
        assert_eq!(deduplicated.len(), sorted.len(), "{sorted:?}");
    }

    #[test]
    fn the_default_port_matches_the_table() {
        for protocol in ProtocolKind::ALL {
            if let Some(port) = default_port(protocol) {
                assert!(ALL.contains(&port), "{protocol:?} -> {port}");
            }
        }
        assert_eq!(default_port(ProtocolKind::Dlna), None);
    }
}
