//! Media abstractions: what a protocol engine produces and what the platform
//! layer consumes.
//!
//! WHY: the engines never decode. Decoding is a platform concern (MediaCodec
//! on Android, VideoToolbox/AudioToolbox on Apple platforms, software
//! decoders on the web). An engine's only job is to deliver *encoded* frames
//! in a platform-neutral description, which keeps the protocol logic portable
//! and testable on a laptop.
//!
//! HOW: an engine pushes [`EncodedVideoFrame`]s into a [`VideoSink`] and
//! [`EncodedAudioFrame`]s into an [`AudioSink`]. Sinks are bounded: a sink
//! that cannot keep up reports [`crate::error::MediaError::QueueFull`] rather
//! than growing without limit, because a stalled decoder must never turn into
//! an out-of-memory kill of the whole receiver.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

pub use crate::capabilities::{AudioCodec, VideoCodec};
use crate::error::MediaError;

/// A monotonically increasing frame counter.
static FRAME_SEQUENCE: AtomicU64 = AtomicU64::new(1);

/// Returns the next global frame sequence number.
#[must_use]
pub fn next_frame_sequence() -> u64 {
    FRAME_SEQUENCE.fetch_add(1, Ordering::Relaxed)
}

/// The pixel format of an encoded video frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    /// YUV 4:2:0 planar, 8 bits per component.
    Nv12,
    /// YUV 4:2:0 planar, 8 bits per component.
    I420,
}

impl PixelFormat {
    /// The MIME-style name used in the native bridge.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            PixelFormat::Nv12 => "nv12",
            PixelFormat::I420 => "i420",
        }
    }
}

/// The description of the video stream a peer negotiated.
#[derive(Debug, Clone, PartialEq)]
pub struct VideoFormat {
    /// The codec of the stream.
    pub codec: VideoCodec,
    /// The frame width in pixels.
    pub width: u32,
    /// The frame height in pixels.
    pub height: u32,
    /// The frame rate in frames per second, if known.
    pub frame_rate: Option<u32>,
    /// The H.264 sequence parameter set, if the codec is H.264.
    pub sps: Option<Vec<u8>>,
    /// The H.264 picture parameter set, if the codec is H.264.
    pub pps: Option<Vec<u8>>,
}

/// The description of the audio stream a peer negotiated.
#[derive(Debug, Clone, PartialEq)]
pub struct AudioFormat {
    /// The codec of the stream.
    pub codec: AudioCodec,
    /// The sample rate in Hz.
    pub sample_rate: u32,
    /// The number of channels.
    pub channels: u16,
    /// The number of samples per channel in one packet.
    pub samples_per_packet: u32,
}

/// One encoded video frame.
#[derive(Debug, Clone, PartialEq)]
pub struct EncodedVideoFrame {
    /// The sequence number of this frame within the session.
    pub sequence: u64,
    /// The presentation timestamp in microseconds.
    pub pts_us: u64,
    /// The encoded payload, in the codec's native framing.
    pub data: Arc<Vec<u8>>,
    /// The pixel format of `data`, for raw video.
    pub pixel_format: Option<PixelFormat>,
}

impl EncodedVideoFrame {
    /// Creates a new encoded video frame, assigning the next sequence number.
    #[must_use]
    pub fn new(pts_us: u64, data: Vec<u8>) -> Self {
        Self {
            sequence: next_frame_sequence(),
            pts_us,
            data: Arc::new(data),
            pixel_format: None,
        }
    }
}

/// One encoded audio packet.
#[derive(Debug, Clone, PartialEq)]
pub struct EncodedAudioFrame {
    /// The sequence number of this packet within the session.
    pub sequence: u64,
    /// The presentation timestamp in microseconds.
    pub pts_us: u64,
    /// The encoded payload.
    pub data: Arc<Vec<u8>>,
}

impl EncodedAudioFrame {
    /// Creates a new encoded audio packet, assigning the next sequence number.
    #[must_use]
    pub fn new(pts_us: u64, data: Vec<u8>) -> Self {
        Self {
            sequence: next_frame_sequence(),
            pts_us,
            data: Arc::new(data),
        }
    }
}

/// Receives encoded video frames from a protocol engine.
///
/// WHY: the trait is object-safe and takes `&self` so a sink can be shared
/// across engines and moved into a dedicated decoder thread without a
/// channel per frame.
pub trait VideoSink: Send + Sync + std::fmt::Debug {
    /// Delivers one frame.
    ///
    /// # Errors
    /// Returns [`MediaError::QueueFull`] when the sink's bounded queue is full
    /// and [`MediaError::Closed`] when the sink has been shut down.
    fn submit(&self, frame: EncodedVideoFrame) -> Result<(), MediaError>;

    /// Announces the format the following frames use.
    ///
    /// # Errors
    /// Returns [`MediaError::Unsupported`] when the sink cannot decode the
    /// format and [`MediaError::Closed`] when the sink has been shut down.
    fn set_format(&self, format: &VideoFormat) -> Result<(), MediaError>;

    /// Clears any buffered state. Called when a session ends.
    fn reset(&self);
}

/// Receives encoded audio packets from a protocol engine.
pub trait AudioSink: Send + Sync + std::fmt::Debug {
    /// Delivers one packet.
    ///
    /// # Errors
    /// Returns [`MediaError::QueueFull`] when the sink's bounded queue is full
    /// and [`MediaError::Closed`] when the sink has been shut down.
    fn submit(&self, frame: EncodedAudioFrame) -> Result<(), MediaError>;

    /// Announces the format the following packets use.
    ///
    /// # Errors
    /// Returns [`MediaError::Unsupported`] when the sink cannot decode the
    /// format and [`MediaError::Closed`] when the sink has been shut down.
    fn set_format(&self, format: &AudioFormat) -> Result<(), MediaError>;

    /// Sets the output volume, where `1.0` is the device's nominal volume.
    fn set_volume(&self, volume: f32) -> Result<(), MediaError>;

    /// Clears any buffered state. Called when a session ends.
    fn reset(&self);
}

/// A sink that drops everything. Used in tests and when a protocol is
/// enabled but no decoder is attached.
#[derive(Debug, Default)]
pub struct NullVideoSink;

impl VideoSink for NullVideoSink {
    fn submit(&self, _frame: EncodedVideoFrame) -> Result<(), MediaError> {
        Ok(())
    }

    fn set_format(&self, _format: &VideoFormat) -> Result<(), MediaError> {
        Ok(())
    }

    fn reset(&self) {}
}

/// A sink that drops everything. Used in tests and when a protocol is
/// enabled but no decoder is attached.
#[derive(Debug, Default)]
pub struct NullAudioSink;

impl AudioSink for NullAudioSink {
    fn submit(&self, _frame: EncodedAudioFrame) -> Result<(), MediaError> {
        Ok(())
    }

    fn set_format(&self, _format: &AudioFormat) -> Result<(), MediaError> {
        Ok(())
    }

    fn set_volume(&self, _volume: f32) -> Result<(), MediaError> {
        Ok(())
    }

    fn reset(&self) {}
}

/// The maximum age a frame may have before it is considered stale.
pub const MAX_FRAME_AGE: Duration = Duration::from_secs(2);

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    #[test]
    fn frame_sequence_increases_monotonically() {
        let first = EncodedVideoFrame::new(0, vec![1, 2, 3]);
        let second = EncodedVideoFrame::new(1, vec![4]);
        assert!(second.sequence > first.sequence);
    }

    #[test]
    fn lpcm_is_not_compressed() {
        assert!(!AudioCodec::Lpcm.is_compressed());
        assert!(AudioCodec::AacEld.is_compressed());
    }

    #[test]
    fn null_sinks_accept_everything() {
        let sink = NullVideoSink;
        assert!(sink.submit(EncodedVideoFrame::new(0, vec![])).is_ok());
        assert!(
            sink.set_format(&VideoFormat {
                codec: VideoCodec::H264,
                width: 1920,
                height: 1080,
                frame_rate: Some(60),
                sps: None,
                pps: None,
            })
            .is_ok()
        );
    }
}
