//! The media descriptions as they travel over the bridge.
//!
//! WHAT: The negotiated formats of a session, the encoded frames that reach
//! the decoder of the platform, and the capabilities of the receiver.
//!
//! WHY: The engines never decode. They hand *encoded* frames to a sink, and
//! the platform decodes them. For that to work, the format has to travel
//! with the frame, and the capability set has to travel into the engine —
//! a receiver that advertises a codec it cannot decode produces a picture
//! that never appears on the television.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_ffi::dto::{FfiAudioCodec, FfiVideoCodec};
//!
//! assert_eq!(FfiVideoCodec::H264.mime_type(), "video/avc");
//! assert_eq!(FfiAudioCodec::AacLc.mime_type(), "audio/mp4a-latm");
//! ```

use phairplay_core::capabilities::{AudioCodec, MediaCapabilities, PassthroughCodec, VideoCodec};
use phairplay_core::media::{
    AudioFormat, EncodedAudioFrame, EncodedVideoFrame, PixelFormat, VideoFormat,
};

/// A video codec the receiver may decode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, uniffi::Enum)]
pub enum FfiVideoCodec {
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

impl FfiVideoCodec {
    /// The MIME type of the codec.
    #[must_use]
    pub const fn mime_type(self) -> &'static str {
        match self {
            FfiVideoCodec::H264 => "video/avc",
            FfiVideoCodec::Hevc => "video/hevc",
            FfiVideoCodec::Vp8 => "video/x-vnd.on2.vp8",
            FfiVideoCodec::Vp9 => "video/x-vnd.on2.vp9",
            FfiVideoCodec::Av1 => "video/av01",
        }
    }
}

impl From<VideoCodec> for FfiVideoCodec {
    fn from(codec: VideoCodec) -> Self {
        match codec {
            VideoCodec::H264 => FfiVideoCodec::H264,
            VideoCodec::Hevc => FfiVideoCodec::Hevc,
            VideoCodec::Vp8 => FfiVideoCodec::Vp8,
            VideoCodec::Vp9 => FfiVideoCodec::Vp9,
            VideoCodec::Av1 => FfiVideoCodec::Av1,
        }
    }
}

impl From<FfiVideoCodec> for VideoCodec {
    fn from(codec: FfiVideoCodec) -> Self {
        match codec {
            FfiVideoCodec::H264 => VideoCodec::H264,
            FfiVideoCodec::Hevc => VideoCodec::Hevc,
            FfiVideoCodec::Vp8 => VideoCodec::Vp8,
            FfiVideoCodec::Vp9 => VideoCodec::Vp9,
            FfiVideoCodec::Av1 => VideoCodec::Av1,
        }
    }
}

/// An audio codec the receiver may decode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, uniffi::Enum)]
pub enum FfiAudioCodec {
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

impl FfiAudioCodec {
    /// The MIME type of the codec.
    #[must_use]
    pub const fn mime_type(self) -> &'static str {
        match self {
            FfiAudioCodec::AacLc => "audio/mp4a-latm",
            FfiAudioCodec::AacEld => "audio/mp4a-latm",
            FfiAudioCodec::Alac => "audio/alac",
            FfiAudioCodec::Lpcm => "audio/raw",
            FfiAudioCodec::Opus => "audio/opus",
            FfiAudioCodec::Mp3 => "audio/mpeg",
            FfiAudioCodec::Flac => "audio/flac",
        }
    }
}

impl From<AudioCodec> for FfiAudioCodec {
    fn from(codec: AudioCodec) -> Self {
        match codec {
            AudioCodec::AacLc => FfiAudioCodec::AacLc,
            AudioCodec::AacEld => FfiAudioCodec::AacEld,
            AudioCodec::Alac => FfiAudioCodec::Alac,
            AudioCodec::Lpcm => FfiAudioCodec::Lpcm,
            AudioCodec::Opus => FfiAudioCodec::Opus,
            AudioCodec::Mp3 => FfiAudioCodec::Mp3,
            AudioCodec::Flac => FfiAudioCodec::Flac,
        }
    }
}

impl From<FfiAudioCodec> for AudioCodec {
    fn from(codec: FfiAudioCodec) -> Self {
        match codec {
            FfiAudioCodec::AacLc => AudioCodec::AacLc,
            FfiAudioCodec::AacEld => AudioCodec::AacEld,
            FfiAudioCodec::Alac => AudioCodec::Alac,
            FfiAudioCodec::Lpcm => AudioCodec::Lpcm,
            FfiAudioCodec::Opus => AudioCodec::Opus,
            FfiAudioCodec::Mp3 => AudioCodec::Mp3,
            FfiAudioCodec::Flac => AudioCodec::Flac,
        }
    }
}

/// An audio format that can be passed through to a capable output device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, uniffi::Enum)]
pub enum FfiPassthroughCodec {
    /// Dolby Digital (AC-3).
    Ac3,
    /// Dolby Digital Plus (E-AC-3).
    EAc3,
    /// E-AC-3 with JOC (Atmos).
    EAc3Joc,
}

impl From<PassthroughCodec> for FfiPassthroughCodec {
    fn from(codec: PassthroughCodec) -> Self {
        match codec {
            PassthroughCodec::Ac3 => FfiPassthroughCodec::Ac3,
            PassthroughCodec::EAc3 => FfiPassthroughCodec::EAc3,
            PassthroughCodec::EAc3Joc => FfiPassthroughCodec::EAc3Joc,
        }
    }
}

impl From<FfiPassthroughCodec> for PassthroughCodec {
    fn from(codec: FfiPassthroughCodec) -> Self {
        match codec {
            FfiPassthroughCodec::Ac3 => PassthroughCodec::Ac3,
            FfiPassthroughCodec::EAc3 => PassthroughCodec::EAc3,
            FfiPassthroughCodec::EAc3Joc => PassthroughCodec::EAc3Joc,
        }
    }
}

/// The negotiated video format of a session.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct FfiVideoFormat {
    /// The codec of the stream.
    pub codec: FfiVideoCodec,
    /// The frame width in pixels.
    pub width: u32,
    /// The frame height in pixels.
    pub height: u32,
    /// The frame rate in frames per second, if it is known.
    pub frame_rate: Option<u32>,
    /// The H.264 sequence parameter set, if the codec is H.264.
    pub sps: Option<Vec<u8>>,
    /// The H.264 picture parameter set, if the codec is H.264.
    pub pps: Option<Vec<u8>>,
}

impl From<&VideoFormat> for FfiVideoFormat {
    fn from(format: &VideoFormat) -> Self {
        FfiVideoFormat {
            codec: format.codec.into(),
            width: format.width,
            height: format.height,
            frame_rate: format.frame_rate,
            sps: format.sps.clone(),
            pps: format.pps.clone(),
        }
    }
}

impl From<VideoFormat> for FfiVideoFormat {
    fn from(format: VideoFormat) -> Self {
        FfiVideoFormat::from(&format)
    }
}

/// The negotiated audio format of a session.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct FfiAudioFormat {
    /// The codec of the stream.
    pub codec: FfiAudioCodec,
    /// The sample rate in Hz.
    pub sample_rate: u32,
    /// The number of channels.
    pub channels: u16,
    /// The number of samples per channel in one packet.
    pub samples_per_packet: u32,
}

impl From<&AudioFormat> for FfiAudioFormat {
    fn from(format: &AudioFormat) -> Self {
        FfiAudioFormat {
            codec: format.codec.into(),
            sample_rate: format.sample_rate,
            channels: format.channels,
            samples_per_packet: format.samples_per_packet,
        }
    }
}

impl From<AudioFormat> for FfiAudioFormat {
    fn from(format: AudioFormat) -> Self {
        FfiAudioFormat::from(&format)
    }
}

/// One encoded video frame on its way to the decoder of the platform.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct FfiEncodedVideoFrame {
    /// The sequence number of the frame within the session.
    pub sequence: u64,
    /// The presentation timestamp in microseconds.
    pub pts_us: u64,
    /// The encoded payload.
    pub data: Vec<u8>,
    /// The pixel format of the payload, for raw video.
    pub pixel_format: Option<String>,
}

impl From<&EncodedVideoFrame> for FfiEncodedVideoFrame {
    fn from(frame: &EncodedVideoFrame) -> Self {
        FfiEncodedVideoFrame {
            sequence: frame.sequence,
            pts_us: frame.pts_us,
            data: frame.data.as_ref().clone(),
            pixel_format: frame.pixel_format.map(|format| format.as_str().to_string()),
        }
    }
}

impl From<EncodedVideoFrame> for FfiEncodedVideoFrame {
    fn from(frame: EncodedVideoFrame) -> Self {
        FfiEncodedVideoFrame::from(&frame)
    }
}

/// One encoded audio packet on its way to the decoder of the platform.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct FfiEncodedAudioFrame {
    /// The sequence number of the packet within the session.
    pub sequence: u64,
    /// The presentation timestamp in microseconds.
    pub pts_us: u64,
    /// The encoded payload.
    pub data: Vec<u8>,
}

impl From<&EncodedAudioFrame> for FfiEncodedAudioFrame {
    fn from(frame: &EncodedAudioFrame) -> Self {
        FfiEncodedAudioFrame {
            sequence: frame.sequence,
            pts_us: frame.pts_us,
            data: frame.data.as_ref().clone(),
        }
    }
}

impl From<EncodedAudioFrame> for FfiEncodedAudioFrame {
    fn from(frame: EncodedAudioFrame) -> Self {
        FfiEncodedAudioFrame::from(&frame)
    }
}

/// The media capabilities of the receiver.
#[derive(Debug, Clone, Default, PartialEq, Eq, uniffi::Record)]
pub struct FfiMediaCapabilities {
    /// The video codecs that can be decoded.
    pub video: Vec<FfiVideoCodec>,
    /// The audio codecs that can be decoded.
    pub audio: Vec<FfiAudioCodec>,
    /// The audio formats that can be passed through.
    pub passthrough: Vec<FfiPassthroughCodec>,
}

impl From<&MediaCapabilities> for FfiMediaCapabilities {
    fn from(capabilities: &MediaCapabilities) -> Self {
        FfiMediaCapabilities {
            video: capabilities
                .video
                .iter()
                .copied()
                .map(FfiVideoCodec::from)
                .collect(),
            audio: capabilities
                .audio
                .iter()
                .copied()
                .map(FfiAudioCodec::from)
                .collect(),
            passthrough: capabilities
                .passthrough
                .iter()
                .copied()
                .map(FfiPassthroughCodec::from)
                .collect(),
        }
    }
}

impl From<MediaCapabilities> for FfiMediaCapabilities {
    fn from(capabilities: MediaCapabilities) -> Self {
        FfiMediaCapabilities::from(&capabilities)
    }
}

impl From<&FfiMediaCapabilities> for MediaCapabilities {
    fn from(capabilities: &FfiMediaCapabilities) -> Self {
        let mut result = MediaCapabilities::empty();
        for codec in &capabilities.video {
            result.add_video((*codec).into());
        }
        for codec in &capabilities.audio {
            result.add_audio((*codec).into());
        }
        for codec in &capabilities.passthrough {
            result.add_passthrough((*codec).into());
        }
        result
    }
}

/// The pixel format of a raw video frame.
#[must_use]
pub fn pixel_format_name(format: PixelFormat) -> &'static str {
    format.as_str()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_video_format_survives_the_conversion() {
        let format = VideoFormat {
            codec: VideoCodec::H264,
            width: 1920,
            height: 1080,
            frame_rate: Some(30),
            sps: Some(vec![0x67, 0x42]),
            pps: Some(vec![0x68, 0xce]),
        };
        let converted = FfiVideoFormat::from(&format);
        assert_eq!(converted.codec, FfiVideoCodec::H264);
        assert_eq!(converted.width, 1920);
        assert_eq!(converted.height, 1080);
        assert_eq!(converted.frame_rate, Some(30));
        assert_eq!(converted.sps, Some(vec![0x67, 0x42]));
        assert_eq!(converted.pps, Some(vec![0x68, 0xce]));
    }

    #[test]
    fn the_audio_format_survives_the_conversion() {
        let format = AudioFormat {
            codec: AudioCodec::AacLc,
            sample_rate: 44100,
            channels: 2,
            samples_per_packet: 1024,
        };
        let converted = FfiAudioFormat::from(&format);
        assert_eq!(converted.codec, FfiAudioCodec::AacLc);
        assert_eq!(converted.sample_rate, 44100);
        assert_eq!(converted.channels, 2);
        assert_eq!(converted.samples_per_packet, 1024);
    }

    #[test]
    fn the_capabilities_survive_the_conversion() {
        let mut capabilities = MediaCapabilities::empty();
        capabilities.add_video(VideoCodec::H264);
        capabilities.add_audio(AudioCodec::AacLc);
        capabilities.add_passthrough(PassthroughCodec::Ac3);

        let converted = FfiMediaCapabilities::from(&capabilities);
        assert_eq!(converted.video, vec![FfiVideoCodec::H264]);
        assert_eq!(converted.audio, vec![FfiAudioCodec::AacLc]);
        assert_eq!(converted.passthrough, vec![FfiPassthroughCodec::Ac3]);

        let back = MediaCapabilities::from(&converted);
        assert_eq!(back, capabilities);
    }

    #[test]
    fn the_frame_survives_the_conversion() {
        let frame = EncodedVideoFrame::new(1234, vec![1, 2, 3, 4]);
        let converted = FfiEncodedVideoFrame::from(&frame);
        assert_eq!(converted.sequence, frame.sequence);
        assert_eq!(converted.pts_us, 1234);
        assert_eq!(converted.data, vec![1, 2, 3, 4]);
    }
}
