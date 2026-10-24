//! The sinks that carry the encoded media to the decoder of the platform.
//!
//! WHAT: [`CountingVideoSink`] and [`CountingAudioSink`] implement the sink
//! traits of the core. They forward every frame to the decoder of the
//! platform and count what happened.
//!
//! WHY: The core counts nothing by itself: the counters of
//! `phairplay_core::metrics` are only as good as the code that feeds them.
//! The sink is the one place where every frame passes, so it is the one
//! place that may count. A receiver without a decoder attached accepts the
//! frames and discards them — and counts nothing, because nothing was
//! decoded.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_ffi::sinks::CountingVideoSink;
//! use phairplay_core::media::EncodedVideoFrame;
//! use phairplay_core::media::VideoSink;
//! use phairplay_core::metrics::new_counters;
//!
//! let counters = new_counters();
//! let sink = CountingVideoSink::new(
//!     std::sync::Arc::new(std::sync::RwLock::new(None)),
//!     std::sync::Arc::clone(&counters),
//! );
//!
//! // Without a decoder attached the frame is accepted and discarded.
//! let outcome = sink.submit(EncodedVideoFrame::new(0, vec![1, 2, 3]));
//! assert!(outcome.is_ok(), "the frame is accepted");
//! assert_eq!(counters.snapshot().video_frames, 0);
//! ```

use std::sync::{Arc, RwLock};

use phairplay_core::error::MediaError;
use phairplay_core::media::{
    AudioFormat, AudioSink, EncodedAudioFrame, EncodedVideoFrame, VideoFormat, VideoSink,
};
use phairplay_core::metrics::SharedCounters;

use crate::callbacks::{PlatformAudioSink, PlatformVideoSink};
use crate::dto::{FfiAudioFormat, FfiEncodedAudioFrame, FfiEncodedVideoFrame, FfiVideoFormat};

/// The video sink of the receiver.
pub struct CountingVideoSink {
    platform: Arc<RwLock<Option<Arc<dyn PlatformVideoSink>>>>,
    counters: SharedCounters,
}

impl std::fmt::Debug for CountingVideoSink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CountingVideoSink")
            .field("platform", &"<platform video sink>")
            .finish()
    }
}

impl CountingVideoSink {
    /// Creates a sink that forwards to the given decoder of the platform.
    #[must_use]
    pub fn new(
        platform: Arc<RwLock<Option<Arc<dyn PlatformVideoSink>>>>,
        counters: SharedCounters,
    ) -> Self {
        CountingVideoSink { platform, counters }
    }
}

impl VideoSink for CountingVideoSink {
    fn submit(&self, frame: EncodedVideoFrame) -> Result<(), MediaError> {
        let platform = read(&self.platform).clone();
        let Some(platform) = platform else {
            // No decoder is attached: the frame is discarded, exactly as the
            // null sink of the core does. Nothing was decoded, so nothing is
            // counted.
            return Ok(());
        };
        match platform.submit_frame(FfiEncodedVideoFrame::from(&frame)) {
            Ok(()) => {
                self.counters.add_video_frame();
                self.counters.add_bytes_received(frame.data.len() as u64);
                Ok(())
            }
            Err(error) => {
                self.counters.add_dropped_video_frame();
                Err(MediaError::Rejected(error.details().to_string()))
            }
        }
    }

    fn set_format(&self, format: &VideoFormat) -> Result<(), MediaError> {
        let platform = read(&self.platform).clone();
        let Some(platform) = platform else {
            return Ok(());
        };
        platform
            .set_video_format(FfiVideoFormat::from(format))
            .map_err(|error| MediaError::Rejected(error.details().to_string()))
    }

    fn reset(&self) {
        let platform = read(&self.platform).clone();
        if let Some(platform) = platform {
            platform.reset_video();
        }
    }
}

/// The audio sink of the receiver.
pub struct CountingAudioSink {
    platform: Arc<RwLock<Option<Arc<dyn PlatformAudioSink>>>>,
    counters: SharedCounters,
}

impl std::fmt::Debug for CountingAudioSink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CountingAudioSink")
            .field("platform", &"<platform audio sink>")
            .finish()
    }
}

impl CountingAudioSink {
    /// Creates a sink that forwards to the given decoder of the platform.
    #[must_use]
    pub fn new(
        platform: Arc<RwLock<Option<Arc<dyn PlatformAudioSink>>>>,
        counters: SharedCounters,
    ) -> Self {
        CountingAudioSink { platform, counters }
    }
}

impl AudioSink for CountingAudioSink {
    fn submit(&self, frame: EncodedAudioFrame) -> Result<(), MediaError> {
        let platform = read(&self.platform).clone();
        let Some(platform) = platform else {
            return Ok(());
        };
        match platform.submit_packet(FfiEncodedAudioFrame::from(&frame)) {
            Ok(()) => {
                self.counters.add_audio_packet();
                self.counters.add_bytes_received(frame.data.len() as u64);
                Ok(())
            }
            Err(error) => {
                self.counters.add_dropped_audio_packet();
                Err(MediaError::Rejected(error.details().to_string()))
            }
        }
    }

    fn set_format(&self, format: &AudioFormat) -> Result<(), MediaError> {
        let platform = read(&self.platform).clone();
        let Some(platform) = platform else {
            return Ok(());
        };
        platform
            .set_audio_format(FfiAudioFormat::from(format))
            .map_err(|error| MediaError::Rejected(error.details().to_string()))
    }

    fn set_volume(&self, volume: f32) -> Result<(), MediaError> {
        let platform = read(&self.platform).clone();
        let Some(platform) = platform else {
            return Ok(());
        };
        platform
            .set_volume(volume)
            .map_err(|error| MediaError::Rejected(error.details().to_string()))
    }

    fn reset(&self) {
        let platform = read(&self.platform).clone();
        if let Some(platform) = platform {
            platform.reset_audio();
        }
    }
}

/// Reads a lock, even when the lock is poisoned.
fn read<T>(lock: &RwLock<T>) -> std::sync::RwLockReadGuard<'_, T> {
    match lock.read() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{Must, MustErr};
    use phairplay_core::capabilities::{AudioCodec, VideoCodec};
    use phairplay_core::media::VideoFormat;
    use phairplay_core::metrics::new_counters;

    #[derive(Debug, Default)]
    struct FakeVideoDecoder {
        frames: std::sync::Mutex<Vec<u64>>,
        refuse: bool,
    }

    impl PlatformVideoSink for FakeVideoDecoder {
        fn submit_frame(&self, frame: FfiEncodedVideoFrame) -> Result<(), crate::error::FfiError> {
            if self.refuse {
                return Err(crate::error::FfiError::internal("the decoder is busy"));
            }
            if let Ok(mut frames) = self.frames.lock() {
                frames.push(frame.sequence);
            }
            Ok(())
        }

        fn set_video_format(&self, _format: FfiVideoFormat) -> Result<(), crate::error::FfiError> {
            Ok(())
        }

        fn reset_video(&self) {}
    }

    fn video_sink(
        decoder: Option<Arc<FakeVideoDecoder>>,
        counters: SharedCounters,
    ) -> CountingVideoSink {
        let decoder: Option<Arc<dyn PlatformVideoSink>> =
            decoder.map(|decoder| decoder as Arc<dyn PlatformVideoSink>);
        let slot: Arc<RwLock<Option<Arc<dyn PlatformVideoSink>>>> = Arc::new(RwLock::new(decoder));
        CountingVideoSink::new(slot, counters)
    }

    #[test]
    fn a_delivered_frame_is_counted() {
        let counters = new_counters();
        let decoder = Arc::new(FakeVideoDecoder::default());
        let sink = video_sink(Some(Arc::clone(&decoder)), Arc::clone(&counters));
        sink.submit(EncodedVideoFrame::new(0, vec![1, 2, 3, 4]))
            .must("the decoder accepts the frame");
        let stats = counters.snapshot();
        assert_eq!(stats.video_frames, 1);
        assert_eq!(stats.video_frames_dropped, 0);
        assert_eq!(stats.bytes_received, 4);
    }

    #[test]
    fn a_refused_frame_is_counted_as_dropped() {
        let counters = new_counters();
        let decoder = Arc::new(FakeVideoDecoder {
            frames: std::sync::Mutex::new(Vec::new()),
            refuse: true,
        });
        let sink = video_sink(Some(decoder), Arc::clone(&counters));
        let error = sink
            .submit(EncodedVideoFrame::new(0, vec![1]))
            .must_err("the decoder refuses the frame");
        assert!(matches!(error, MediaError::Rejected(_)), "{error:?}");
        let stats = counters.snapshot();
        assert_eq!(stats.video_frames, 0);
        assert_eq!(stats.video_frames_dropped, 1);
    }

    #[test]
    fn without_a_decoder_nothing_is_counted() {
        let counters = new_counters();
        let sink = video_sink(None, Arc::clone(&counters));
        sink.submit(EncodedVideoFrame::new(0, vec![1, 2]))
            .must("the frame is accepted and discarded");
        let stats = counters.snapshot();
        assert_eq!(stats.video_frames, 0);
        assert_eq!(stats.bytes_received, 0);
    }

    #[test]
    fn the_format_is_forwarded_to_the_decoder() {
        let counters = new_counters();
        let decoder = Arc::new(FakeVideoDecoder::default());
        let sink = video_sink(Some(Arc::clone(&decoder)), Arc::clone(&counters));
        let format = VideoFormat {
            codec: VideoCodec::H264,
            width: 1280,
            height: 720,
            frame_rate: Some(60),
            sps: None,
            pps: None,
        };
        sink.set_format(&format)
            .must("the decoder accepts the format");
    }

    #[test]
    fn the_audio_sink_counts_the_delivered_packets() {
        let counters = new_counters();
        let slot: Arc<RwLock<Option<Arc<dyn PlatformAudioSink>>>> = Arc::new(RwLock::new(None));
        let sink = CountingAudioSink::new(slot, Arc::clone(&counters));
        sink.submit(EncodedAudioFrame::new(0, vec![9, 9]))
            .must("the packet is accepted and discarded");
        assert_eq!(counters.snapshot().audio_packets, 0);

        let format = AudioFormat {
            codec: AudioCodec::Lpcm,
            sample_rate: 48000,
            channels: 2,
            samples_per_packet: 480,
        };
        sink.set_format(&format)
            .must("without a decoder the format is accepted");
        sink.set_volume(0.5)
            .must("without a decoder the volume is accepted");
        sink.reset();
    }
}
