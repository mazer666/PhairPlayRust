//! The protocol-internal timeouts of the receiver stack.
//!
//! WHAT: the deadlines a protocol applies to a step of its own handshake —
//! the TLS handshake of a Cast sender, the DACP probe, the GENA notification,
//! the UPnP advertisement lifetime.
//!
//! WHY: a deadline that is written down twice is a deadline that is one
//! change away from being different in two places. The UPnP advertisement
//! lifetime, for example, appeared as `CACHE_CONTROL_MAX_AGE` in the SSDP
//! responder and as `SUBSCRIPTION_TIMEOUT_SECONDS` in the event manager —
//! two names for the same 1800 seconds of the UPnP specification.
//!
//! The *idle* timeout of a peer is **not** here: it is a user-visible
//! performance budget and its single owner is
//! [`crate::config::PerformanceConfig::idle_timeout_ms`], which the
//! configuration file mirrors.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_core::config::timeouts;
//!
//! assert_eq!(timeouts::CAST_HANDSHAKE_TIMEOUT_MS, 10_000);
//! assert_eq!(timeouts::UPNP_MAX_AGE_SECONDS, 1800);
//! ```

/// The time a Cast sender may take to complete the TLS handshake.
pub const CAST_HANDSHAKE_TIMEOUT_MS: u64 = 10_000;

/// The time the DACP control channel waits for an answer of the sender.
pub const DACP_TIMEOUT_MS: u64 = 2_000;

/// The time the GENA event manager waits for a notification to be accepted.
pub const GENA_NOTIFY_TIMEOUT_MS: u64 = 3_000;

/// The lifetime of a UPnP announcement, in seconds.
///
/// WHY: the UPnP specification gives the `CACHE-CONTROL: max-age` of an
/// advertisement and the duration of a GENA subscription the same value.
/// They are one number of the specification, so they are one constant here.
pub const UPNP_MAX_AGE_SECONDS: u32 = 1800;

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    #[test]
    fn the_timeouts_are_the_documented_ones() {
        assert_eq!(CAST_HANDSHAKE_TIMEOUT_MS, 10_000);
        assert_eq!(DACP_TIMEOUT_MS, 2_000);
        assert_eq!(GENA_NOTIFY_TIMEOUT_MS, 3_000);
        assert_eq!(UPNP_MAX_AGE_SECONDS, 1800);
    }

    #[test]
    fn every_timeout_is_positive() {
        // WHY: a timeout of zero is a denial of service, not a timeout.
        // WHY: `black_box` keeps the compiler from folding the comparisons
        // at compile time — a test that compares two constants does not
        // test anything at run time.
        let handshake = std::hint::black_box(CAST_HANDSHAKE_TIMEOUT_MS);
        let dacp = std::hint::black_box(DACP_TIMEOUT_MS);
        let gena = std::hint::black_box(GENA_NOTIFY_TIMEOUT_MS);
        let max_age = std::hint::black_box(UPNP_MAX_AGE_SECONDS);
        assert!(handshake > 0);
        assert!(dacp > 0);
        assert!(gena > 0);
        assert!(max_age > 0);
    }
}
