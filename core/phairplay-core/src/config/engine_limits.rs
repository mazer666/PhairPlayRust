//! The limits an engine enforces, in one flat structure.
//!
//! WHAT: [`EngineLimits`] is the resolved set of numbers a protocol engine
//! has to obey: how large a message may be, how many headers it may read,
//! how long a peer may be silent.
//!
//! WHY: every engine needs the same limits, and every engine must read them
//! from the configuration of the user — the build contract says so, and the
//! reason is the threat model: *any device on the local network*. A limit
//! that an engine hard-codes is a limit the user cannot see, cannot change
//! and cannot audit. Before this type existed, `engines/airplay` had its own
//! `AirPlayLimits`, `engines/miracast` had three constants of its own and
//! `engines/cast` had two more — five places where the same question ("how
//! long may a peer be silent?") had five different answers.
//!
//! HOW: [`AppConfig::limits_for`] resolves the configuration into an
//! [`EngineLimits`]. An engine that receives the configuration — or a caller
//! that builds one — uses this and never a literal.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_core::config::AppConfig;
//! use phairplay_core::protocol::ProtocolKind;
//!
//! let config = AppConfig::default();
//! let limits = config.limits_for(ProtocolKind::AirPlay);
//!
//! assert_eq!(limits.max_header_count, config.limits.max_header_count);
//! assert_eq!(
//!     limits.idle_timeout.as_millis() as u64,
//!     config.performance.idle_timeout_ms
//! );
//! ```

use std::time::Duration;

use crate::config::AppConfig;
use crate::protocol::ProtocolKind;

/// The resolved limits of one protocol engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineLimits {
    /// The maximum size of a single protocol message, in bytes.
    pub max_message_bytes: usize,
    /// The maximum number of headers accepted in one request.
    pub max_header_count: usize,
    /// The maximum length of a single header line, in bytes.
    pub max_header_bytes: usize,
    /// The maximum number of audio packets buffered in the sink queue.
    pub audio_queue_packets: usize,
    /// The maximum total number of bytes buffered in the audio sink queue.
    pub audio_queue_bytes: usize,
    /// The maximum size of an album artwork payload, in bytes.
    pub max_artwork_bytes: usize,
    /// The maximum size of a photo payload, in bytes.
    pub max_photo_bytes: usize,
    /// The maximum size of a media payload, in bytes.
    pub max_media_bytes: usize,
    /// The maximum number of failed pairing attempts before the lock-out.
    pub max_pairing_attempts: u32,
    /// The time a peer may stay connected without sending anything.
    pub idle_timeout: Duration,
}

impl AppConfig {
    /// Resolves the limits of one protocol out of the configuration.
    ///
    /// WHY: an engine asks the configuration for its limits and never for a
    /// number of its own. The protocol-specific part is the message size —
    /// AirPlay and Cast have a different one — everything else is global.
    ///
    /// EXAMPLE:
    ///
    /// ```
    /// use phairplay_core::config::AppConfig;
    /// use phairplay_core::protocol::ProtocolKind;
    ///
    /// let config = AppConfig::default();
    /// let limits = config.limits_for(ProtocolKind::Cast);
    /// assert_eq!(limits.max_message_bytes, config.protocol(ProtocolKind::Cast).max_message_bytes);
    /// ```
    #[must_use]
    pub fn limits_for(&self, protocol: ProtocolKind) -> EngineLimits {
        let protocol_config = self.protocol(protocol);
        EngineLimits {
            max_message_bytes: protocol_config.max_message_bytes,
            max_header_count: self.limits.max_header_count,
            max_header_bytes: self.limits.max_header_bytes,
            audio_queue_packets: self.limits.audio_queue_packets,
            audio_queue_bytes: self.limits.audio_queue_bytes,
            max_artwork_bytes: self.limits.max_artwork_bytes,
            max_photo_bytes: self.limits.max_photo_bytes,
            max_media_bytes: self.limits.max_media_bytes,
            max_pairing_attempts: self.limits.max_pairing_attempts,
            idle_timeout: Duration::from_millis(self.performance.idle_timeout_ms),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;
    use crate::config::{LimitsConfig, PerformanceConfig};

    #[test]
    fn the_limits_of_every_protocol_come_from_the_configuration() {
        let config = AppConfig::default();
        for protocol in ProtocolKind::ALL {
            let limits = config.limits_for(protocol);
            assert_eq!(
                limits.max_message_bytes,
                config.protocol(protocol).max_message_bytes,
                "{protocol:?}"
            );
            assert_eq!(limits.max_header_count, config.limits.max_header_count);
            assert_eq!(limits.max_header_bytes, config.limits.max_header_bytes);
            assert_eq!(
                limits.audio_queue_packets,
                config.limits.audio_queue_packets
            );
            assert_eq!(limits.audio_queue_bytes, config.limits.audio_queue_bytes);
            assert_eq!(limits.max_artwork_bytes, config.limits.max_artwork_bytes);
            assert_eq!(limits.max_photo_bytes, config.limits.max_photo_bytes);
            assert_eq!(limits.max_media_bytes, config.limits.max_media_bytes);
            assert_eq!(
                limits.max_pairing_attempts,
                config.limits.max_pairing_attempts
            );
        }
    }

    #[test]
    fn the_idle_timeout_is_the_performance_budget() {
        let config = AppConfig::default();
        let limits = config.limits_for(ProtocolKind::AirPlay);
        assert_eq!(
            limits.idle_timeout,
            Duration::from_millis(config.performance.idle_timeout_ms)
        );
    }

    #[test]
    fn a_changed_budget_reaches_the_engine() {
        // WHY: the whole point of the type is that the user can lower a
        // limit and the engine obeys it.
        let config = AppConfig {
            limits: LimitsConfig {
                max_header_count: 4,
                ..LimitsConfig::default()
            },
            performance: PerformanceConfig {
                idle_timeout_ms: 1_500,
                ..PerformanceConfig::default()
            },
            ..AppConfig::default()
        };

        let limits = config.limits_for(ProtocolKind::Miracast);
        assert_eq!(limits.max_header_count, 4);
        assert_eq!(limits.idle_timeout, Duration::from_millis(1_500));
    }

    #[test]
    fn the_default_limits_are_the_documented_ones() {
        let limits = AppConfig::default().limits_for(ProtocolKind::AirPlay);
        assert_eq!(limits.max_message_bytes, 1024 * 1024);
        assert_eq!(limits.max_header_count, 64);
        assert_eq!(limits.max_header_bytes, 8 * 1024);
        assert_eq!(limits.max_pairing_attempts, 5);
        assert_eq!(limits.idle_timeout, Duration::from_secs(30));
    }
}
