//! Live statistics about a streaming session.
//!
//! WHY: "is it working?" is the first question a user asks. Counters that
//! answer it objectively — frames delivered, packets lost, bytes received —
//! are more useful than a spinner, and they are the evidence an acceptance
//! document needs.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// A snapshot of the counters of one session.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StreamStats {
    /// The number of video frames delivered to the sink.
    pub video_frames: u64,
    /// The number of video frames dropped because the sink queue was full.
    pub video_frames_dropped: u64,
    /// The number of audio packets delivered to the sink.
    pub audio_packets: u64,
    /// The number of audio packets dropped because the sink queue was full.
    pub audio_packets_dropped: u64,
    /// The number of bytes received from the peer.
    pub bytes_received: u64,
    /// The number of bytes sent to the peer.
    pub bytes_sent: u64,
    /// The number of RTP packets reported as lost by the sender.
    pub packets_lost: u64,
    /// The number of frames the sender reported as skipped.
    pub frames_skipped: u64,
}

/// The live, atomically updated counters of one session.
#[derive(Debug, Default)]
pub struct StreamCounters {
    video_frames: AtomicU64,
    video_frames_dropped: AtomicU64,
    audio_packets: AtomicU64,
    audio_packets_dropped: AtomicU64,
    bytes_received: AtomicU64,
    bytes_sent: AtomicU64,
    packets_lost: AtomicU64,
    frames_skipped: AtomicU64,
}

impl StreamCounters {
    /// Creates a zeroed counter set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a delivered video frame.
    pub fn add_video_frame(&self) {
        self.video_frames.fetch_add(1, Ordering::Relaxed);
    }

    /// Records a dropped video frame.
    pub fn add_dropped_video_frame(&self) {
        self.video_frames_dropped.fetch_add(1, Ordering::Relaxed);
    }

    /// Records a delivered audio packet.
    pub fn add_audio_packet(&self) {
        self.audio_packets.fetch_add(1, Ordering::Relaxed);
    }

    /// Records a dropped audio packet.
    pub fn add_dropped_audio_packet(&self) {
        self.audio_packets_dropped.fetch_add(1, Ordering::Relaxed);
    }

    /// Records received bytes.
    pub fn add_bytes_received(&self, count: u64) {
        self.bytes_received.fetch_add(count, Ordering::Relaxed);
    }

    /// Records sent bytes.
    pub fn add_bytes_sent(&self, count: u64) {
        self.bytes_sent.fetch_add(count, Ordering::Relaxed);
    }

    /// Records packets the sender reported as lost.
    pub fn add_packets_lost(&self, count: u64) {
        self.packets_lost.fetch_add(count, Ordering::Relaxed);
    }

    /// Records frames the sender reported as skipped.
    pub fn add_frames_skipped(&self, count: u64) {
        self.frames_skipped.fetch_add(count, Ordering::Relaxed);
    }

    /// Reads the current values.
    #[must_use]
    pub fn snapshot(&self) -> StreamStats {
        StreamStats {
            video_frames: self.video_frames.load(Ordering::Relaxed),
            video_frames_dropped: self.video_frames_dropped.load(Ordering::Relaxed),
            audio_packets: self.audio_packets.load(Ordering::Relaxed),
            audio_packets_dropped: self.audio_packets_dropped.load(Ordering::Relaxed),
            bytes_received: self.bytes_received.load(Ordering::Relaxed),
            bytes_sent: self.bytes_sent.load(Ordering::Relaxed),
            packets_lost: self.packets_lost.load(Ordering::Relaxed),
            frames_skipped: self.frames_skipped.load(Ordering::Relaxed),
        }
    }

    /// The share of video frames that were dropped, in the range `0.0..=1.0`.
    ///
    /// Returns `0.0` when no frame was delivered, so that "no traffic" is not
    /// reported as "100 % lost".
    #[must_use]
    pub fn drop_ratio(&self) -> f64 {
        let stats = self.snapshot();
        let total = stats.video_frames + stats.video_frames_dropped;
        if total == 0 {
            return 0.0;
        }
        stats.video_frames_dropped as f64 / total as f64
    }
}

/// A shared handle to a counter set.
pub type SharedCounters = Arc<StreamCounters>;

/// Creates a shared counter set.
#[must_use]
pub fn new_counters() -> SharedCounters {
    Arc::new(StreamCounters::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counters_start_at_zero() {
        let counters = StreamCounters::new();
        assert_eq!(counters.snapshot(), StreamStats::default());
    }

    #[test]
    fn the_drop_ratio_reflects_dropped_frames() {
        let counters = StreamCounters::new();
        counters.add_video_frame();
        counters.add_video_frame();
        counters.add_video_frame();
        counters.add_dropped_video_frame();
        let ratio = counters.drop_ratio();
        assert!((ratio - 0.25).abs() < f64::EPSILON, "{ratio}");
    }

    #[test]
    fn the_drop_ratio_is_zero_without_traffic() {
        let counters = StreamCounters::new();
        assert_eq!(counters.drop_ratio(), 0.0);
    }
}
