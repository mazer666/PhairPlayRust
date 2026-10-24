//! The live counters of a session as they travel over the bridge.
//!
//! WHAT: The number of frames, packets and bytes that passed through the
//! receiver, and the share of the frames that were dropped.
//!
//! WHY: "Is it working?" is the first question a user asks. Counters that
//! answer it objectively are more useful than a spinner, and they are the
//! evidence an acceptance document needs. The counters are owned by the core
//! (`phairplay_core::metrics`); the bridge only carries them.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_ffi::dto::FfiStreamStats;
//! use phairplay_core::metrics::StreamStats;
//!
//! let stats = FfiStreamStats::from(&StreamStats {
//!     video_frames: 120,
//!     video_frames_dropped: 0,
//!     ..StreamStats::default()
//! });
//!
//! assert_eq!(stats.video_frames, 120);
//! assert_eq!(stats.drop_ratio, 0.0);
//! ```

use phairplay_core::metrics::StreamStats;

/// The counters of one session at one moment.
#[derive(Debug, Clone, Copy, PartialEq, uniffi::Record)]
pub struct FfiStreamStats {
    /// The number of video frames delivered to the decoder of the platform.
    pub video_frames: u64,
    /// The number of video frames dropped because the decoder refused them.
    pub video_frames_dropped: u64,
    /// The number of audio packets delivered to the decoder of the platform.
    pub audio_packets: u64,
    /// The number of audio packets dropped because the decoder refused them.
    pub audio_packets_dropped: u64,
    /// The number of bytes received from the peer.
    pub bytes_received: u64,
    /// The number of bytes sent to the peer.
    pub bytes_sent: u64,
    /// The number of packets the sender reported as lost.
    pub packets_lost: u64,
    /// The number of frames the sender reported as skipped.
    pub frames_skipped: u64,
    /// The share of the video frames that were dropped, between 0.0 and 1.0.
    pub drop_ratio: f64,
}

impl From<&StreamStats> for FfiStreamStats {
    fn from(stats: &StreamStats) -> Self {
        let total = stats.video_frames + stats.video_frames_dropped;
        let drop_ratio = if total == 0 {
            0.0
        } else {
            stats.video_frames_dropped as f64 / total as f64
        };
        FfiStreamStats {
            video_frames: stats.video_frames,
            video_frames_dropped: stats.video_frames_dropped,
            audio_packets: stats.audio_packets,
            audio_packets_dropped: stats.audio_packets_dropped,
            bytes_received: stats.bytes_received,
            bytes_sent: stats.bytes_sent,
            packets_lost: stats.packets_lost,
            frames_skipped: stats.frames_skipped,
            drop_ratio,
        }
    }
}

impl From<StreamStats> for FfiStreamStats {
    fn from(stats: StreamStats) -> Self {
        FfiStreamStats::from(&stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_counters_survive_the_conversion() {
        let stats = StreamStats {
            video_frames: 300,
            video_frames_dropped: 100,
            audio_packets: 50,
            audio_packets_dropped: 5,
            bytes_received: 12_345,
            bytes_sent: 678,
            packets_lost: 9,
            frames_skipped: 4,
        };
        let converted = FfiStreamStats::from(&stats);
        assert_eq!(converted.video_frames, 300);
        assert_eq!(converted.video_frames_dropped, 100);
        assert_eq!(converted.audio_packets, 50);
        assert_eq!(converted.audio_packets_dropped, 5);
        assert_eq!(converted.bytes_received, 12_345);
        assert_eq!(converted.bytes_sent, 678);
        assert_eq!(converted.packets_lost, 9);
        assert_eq!(converted.frames_skipped, 4);
    }

    #[test]
    fn the_drop_ratio_is_zero_without_traffic() {
        // "No traffic" must not be reported as "everything was lost".
        let converted = FfiStreamStats::from(&StreamStats::default());
        assert_eq!(converted.drop_ratio, 0.0);
    }

    #[test]
    fn the_drop_ratio_reflects_the_dropped_frames() {
        let stats = StreamStats {
            video_frames: 300,
            video_frames_dropped: 100,
            ..StreamStats::default()
        };
        let converted = FfiStreamStats::from(&stats);
        assert!(
            (converted.drop_ratio - 0.25).abs() < 1e-9,
            "{}",
            converted.drop_ratio
        );
    }
}
