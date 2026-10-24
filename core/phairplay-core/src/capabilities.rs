//! The media capability model.
//!
//! WHY: a receiver must only advertise what it can actually decode.
//! Advertising HEVC on a device without an HEVC decoder produces a picture
//! that never appears on the TV and a support ticket that cannot be
//! reproduced. The capability snapshot is the single source of truth for
//! what is advertised over mDNS, WFD and Cast.
//!
//! HOW: the platform layer queries the real decoders and fills in a
//! [`MediaCapabilities`] value. The engines read it; they never query the
//! platform themselves.

use std::collections::BTreeSet;

/// A video codec the receiver may decode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VideoCodec {
    /// H.264 / AVC.
    H264,
    /// HEVC / H.265.
    Hevc,
    /// VP8.
    Vp8,
    /// VP9.
    Vp9,
    /// AV1.
    Av1,
}

impl VideoCodec {
    /// The MIME type of the codec.
    #[must_use]
    pub const fn mime_type(self) -> &'static str {
        match self {
            VideoCodec::H264 => "video/avc",
            VideoCodec::Hevc => "video/hevc",
            VideoCodec::Vp8 => "video/x-vnd.on2.vp8",
            VideoCodec::Vp9 => "video/x-vnd.on2.vp9",
            VideoCodec::Av1 => "video/av01",
        }
    }

    /// Every known video codec.
    pub const ALL: [VideoCodec; 5] = [
        VideoCodec::H264,
        VideoCodec::Hevc,
        VideoCodec::Vp8,
        VideoCodec::Vp9,
        VideoCodec::Av1,
    ];
}

/// An audio codec the receiver may decode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AudioCodec {
    /// AAC-LC.
    AacLc,
    /// AAC-ELD.
    AacEld,
    /// Apple Lossless.
    Alac,
    /// Linear PCM.
    Lpcm,
    /// Opus.
    Opus,
    /// MP3.
    Mp3,
    /// FLAC.
    Flac,
}

impl AudioCodec {
    /// The MIME type of the codec.
    #[must_use]
    pub const fn mime_type(self) -> &'static str {
        match self {
            AudioCodec::AacLc => "audio/mp4a-latm",
            AudioCodec::AacEld => "audio/mp4a-latm",
            AudioCodec::Alac => "audio/alac",
            AudioCodec::Lpcm => "audio/raw",
            AudioCodec::Opus => "audio/opus",
            AudioCodec::Mp3 => "audio/mpeg",
            AudioCodec::Flac => "audio/flac",
        }
    }

    /// Every known audio codec.
    pub const ALL: [AudioCodec; 7] = [
        AudioCodec::AacLc,
        AudioCodec::AacEld,
        AudioCodec::Alac,
        AudioCodec::Lpcm,
        AudioCodec::Opus,
        AudioCodec::Mp3,
        AudioCodec::Flac,
    ];

    /// Whether this codec carries compressed frames that need a decoder.
    ///
    /// Linear PCM is handed to the output device unchanged; every other codec
    /// in this list has to be decoded first.
    #[must_use]
    pub const fn is_compressed(self) -> bool {
        !matches!(self, AudioCodec::Lpcm)
    }
}

/// An audio format the receiver can pass through to a capable output device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PassthroughCodec {
    /// Dolby Digital (AC-3).
    Ac3,
    /// Dolby Digital Plus (E-AC-3).
    EAc3,
    /// E-AC-3 with JOC (Atmos).
    EAc3Joc,
}

/// The media capabilities of this receiver.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MediaCapabilities {
    /// The video codecs that can be decoded.
    pub video: BTreeSet<VideoCodec>,
    /// The audio codecs that can be decoded.
    pub audio: BTreeSet<AudioCodec>,
    /// The audio formats that can be passed through.
    pub passthrough: BTreeSet<PassthroughCodec>,
}

impl MediaCapabilities {
    /// An empty capability set. Nothing is advertised.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Whether a video codec can be decoded.
    #[must_use]
    pub fn supports_video(&self, codec: VideoCodec) -> bool {
        self.video.contains(&codec)
    }

    /// Whether an audio codec can be decoded.
    #[must_use]
    pub fn supports_audio(&self, codec: AudioCodec) -> bool {
        self.audio.contains(&codec)
    }

    /// Whether an audio format can be passed through.
    #[must_use]
    pub fn supports_passthrough(&self, codec: PassthroughCodec) -> bool {
        self.passthrough.contains(&codec)
    }

    /// Whether the baseline required by the Google Cast protocol is present.
    ///
    /// WHY: a Cast receiver that cannot decode the baseline codecs is
    /// rejected by the sender before it ever opens a channel.
    #[must_use]
    pub fn supports_cast_baseline(&self) -> bool {
        self.supports_video(VideoCodec::H264)
            && self.supports_video(VideoCodec::Vp8)
            && self.supports_audio(AudioCodec::AacLc)
            && self.supports_audio(AudioCodec::Mp3)
            && self.supports_audio(AudioCodec::Lpcm)
    }

    /// Whether the baseline required by the Miracast protocol is present.
    #[must_use]
    pub fn supports_miracast_baseline(&self) -> bool {
        self.supports_video(VideoCodec::H264) && self.supports_audio(AudioCodec::Lpcm)
    }

    /// Whether the baseline required by the AirPlay 2 mirroring path is present.
    #[must_use]
    pub fn supports_airplay_baseline(&self) -> bool {
        self.supports_video(VideoCodec::H264)
    }

    /// The capabilities as a sorted list of MIME types, for advertisement.
    #[must_use]
    pub fn advertised_video_mime_types(&self) -> Vec<&'static str> {
        self.video.iter().map(|codec| codec.mime_type()).collect()
    }

    /// The capabilities as a sorted list of MIME types, for advertisement.
    #[must_use]
    pub fn advertised_audio_mime_types(&self) -> Vec<&'static str> {
        self.audio.iter().map(|codec| codec.mime_type()).collect()
    }

    /// Adds a video codec.
    pub fn add_video(&mut self, codec: VideoCodec) {
        self.video.insert(codec);
    }

    /// Adds an audio codec.
    pub fn add_audio(&mut self, codec: AudioCodec) {
        self.audio.insert(codec);
    }

    /// Adds a passthrough format.
    pub fn add_passthrough(&mut self, codec: PassthroughCodec) {
        self.passthrough.insert(codec);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> MediaCapabilities {
        let mut capabilities = MediaCapabilities::empty();
        capabilities.add_video(VideoCodec::H264);
        capabilities.add_video(VideoCodec::Vp8);
        capabilities.add_audio(AudioCodec::AacLc);
        capabilities.add_audio(AudioCodec::Mp3);
        capabilities.add_audio(AudioCodec::Lpcm);
        capabilities
    }

    #[test]
    fn cast_baseline_requires_h264_vp8_and_three_audio_codecs() {
        assert!(baseline().supports_cast_baseline());
    }

    #[test]
    fn cast_baseline_is_incomplete_without_vp8() {
        let mut capabilities = baseline();
        capabilities.video.remove(&VideoCodec::Vp8);
        assert!(!capabilities.supports_cast_baseline());
    }

    #[test]
    fn miracast_baseline_needs_only_h264_and_lpcm() {
        let mut capabilities = MediaCapabilities::empty();
        capabilities.add_video(VideoCodec::H264);
        capabilities.add_audio(AudioCodec::Lpcm);
        assert!(capabilities.supports_miracast_baseline());
        assert!(!capabilities.supports_cast_baseline());
    }

    #[test]
    fn an_empty_capability_set_advertises_nothing() {
        let capabilities = MediaCapabilities::empty();
        assert!(capabilities.advertised_video_mime_types().is_empty());
        assert!(capabilities.advertised_audio_mime_types().is_empty());
    }
}
