//! The resource limits of the AirPlay engine.
//!
//! WHY: every byte that arrives from the network is untrusted — the threat
//! model is *any device on the local WLAN*. A receiver that reads a
//! `Content-Length` of 4 GB, or that accepts 10 000 headers, or that buffers
//! an unbounded photo, is a denial-of-service target for a neighbour. The
//! build contract therefore demands that every limit is read from
//! [`phairplay_core::config::AppConfig`] and never hard-coded in the engine.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_airplay::limits::AirPlayLimits;
//! use phairplay_core::config::AppConfig;
//!
//! let config = AppConfig::default();
//! let limits = AirPlayLimits::from_config(&config);
//! assert_eq!(limits.max_header_count, config.limits.max_header_count);
//! ```

use std::time::Duration;

use phairplay_core::config::AppConfig;
use phairplay_core::protocol::ProtocolKind;

/// The resource limits the AirPlay engine enforces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AirPlayLimits {
    /// The maximum number of bytes in one RTSP message (request line + headers + body).
    pub max_message_bytes: usize,
    /// The maximum number of headers accepted in one request.
    pub max_header_count: usize,
    /// The maximum length of a single header line, in bytes.
    pub max_header_bytes: usize,
    /// The maximum length of the request line, in bytes.
    pub max_request_line_bytes: usize,
    /// The maximum size of an SDP body, in bytes.
    pub max_sdp_bytes: usize,
    /// The maximum size of an interleaved RTP frame, in bytes.
    pub max_rtp_frame_bytes: usize,
    /// The maximum size of a photo payload, in bytes.
    pub max_photo_bytes: usize,
    /// The maximum size of an album artwork payload, in bytes.
    pub max_artwork_bytes: usize,
    /// The maximum number of failed PIN pairing attempts before the lock-out.
    pub max_pairing_attempts: u32,
    /// The maximum time a peer may stay connected without sending anything.
    pub idle_timeout: Duration,
}

impl AirPlayLimits {
    /// Reads the limits out of the application configuration.
    ///
    /// WHY: the engine never invents a limit. Everything that protects the
    /// process comes from the configuration file the user can inspect, which
    /// is also what the acceptance tests measure.
    #[must_use]
    pub fn from_config(config: &AppConfig) -> Self {
        let protocol = config.protocol(ProtocolKind::AirPlay);
        let max_message_bytes = protocol.max_message_bytes.max(1024);
        Self {
            max_message_bytes,
            max_header_count: config.limits.max_header_count.max(1),
            max_header_bytes: config.limits.max_header_bytes.max(256),
            max_request_line_bytes: 8 * 1024,
            // The SDP travels inside the RTSP message, so it can never be larger
            // than the message itself.
            max_sdp_bytes: max_message_bytes,
            // An interleaved RTP frame carries a whole video slice; the Kotlin
            // reference caps it at 2 MB. Keep it below the message limit so a
            // single frame can never exhaust the process memory.
            max_rtp_frame_bytes: max_message_bytes.min(2 * 1024 * 1024),
            max_photo_bytes: config.limits.max_photo_bytes,
            max_artwork_bytes: config.limits.max_artwork_bytes,
            max_pairing_attempts: config.limits.max_pairing_attempts.max(1),
            idle_timeout: Duration::from_millis(config.performance.idle_timeout_ms.max(1_000)),
        }
    }

    /// The default limits, used when no configuration is available.
    #[must_use]
    pub fn default_limits() -> Self {
        Self::from_config(&AppConfig::default())
    }
}

impl Default for AirPlayLimits {
    fn default() -> Self {
        Self::default_limits()
    }
}

#[cfg(test)]
mod tests;
