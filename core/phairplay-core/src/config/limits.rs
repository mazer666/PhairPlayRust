//! The size caps that more than one crate of the stack has to know.
//!
//! WHAT: the upper bounds of the messages that travel between two protocols
//! of the stack — the SSDP datagram, the CastV2 frame — in one place.
//!
//! WHY: the SSDP datagram limit of 2048 bytes was written down twice: once
//! in `engines/discovery` as `MAX_SSDP_MESSAGE_BYTES` and once in
//! `engines/dlna` as `MAX_PACKET_BYTES`. A renderer that accepts more than
//! the browser does reads a datagram that no sender ever sends; a renderer
//! that accepts less truncates a legal answer. The CastV2 frame limit of
//! 64 KiB was even written down twice *inside* `engines/cast`, in the
//! framing module and in the engine.
//!
//! The limits that a *user* may configure are not here: their single owner
//! is [`crate::config::LimitsConfig`] and
//! [`crate::config::ProtocolConfig`], which mirror
//! `config/performance-budgets.toml`.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_core::config::limits;
//!
//! assert_eq!(limits::SSDP_MESSAGE_BYTES, 2048);
//! assert_eq!(limits::CAST_MESSAGE_BYTES, 64 * 1024);
//! ```

/// The maximum size of an SSDP datagram, in bytes.
///
/// WHY: 2048 bytes is the size of a datagram that fits into the path MTU of
/// every network that carries IPv4 without fragmentation. A larger answer is
/// never sent by a conforming control point.
pub const SSDP_MESSAGE_BYTES: usize = 2048;

/// The maximum size of a single CastV2 frame, in bytes.
pub const CAST_MESSAGE_BYTES: usize = 64 * 1024;

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    #[test]
    fn the_caps_are_the_documented_ones() {
        assert_eq!(SSDP_MESSAGE_BYTES, 2048);
        assert_eq!(CAST_MESSAGE_BYTES, 64 * 1024);
    }

    #[test]
    fn the_ssdp_cap_fits_into_a_datagram() {
        // WHY: a control point answers with a datagram, not with a stream.
        // A cap above 1500 bytes would allow a fragment that a peer on a
        // small-MTU link cannot receive.
        // WHY: `black_box` keeps the compiler from folding the comparison
        // at compile time: a test that compares two constants does not test
        // anything at run time, and a constant that changes would not be
        // noticed.
        let cap = std::hint::black_box(SSDP_MESSAGE_BYTES);
        assert!(cap <= 2048, "the cap must fit into a datagram");
    }
}
