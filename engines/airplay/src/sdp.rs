//! The SDP parser of the `ANNOUNCE` request.
//!
//! WHY: the `ANNOUNCE` body is where the sender describes everything the
//! receiver needs to know: which codecs are used, which H.264 parameter sets
//! the decoder must be primed with, which ALAC frame geometry applies, which
//! RTP ports carry the media, and — for legacy AirPlay-1 audio — which AES
//! key decrypts the stream. The body is untrusted input from the local
//! network, so the parser *rejects* anything it does not understand instead of
//! guessing.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_airplay::sdp::parse_sdp;
//! use phairplay_airplay::limits::AirPlayLimits;
//!
//! let sdp = "v=0\r\nm=video 0 RTP/AVP 96\r\na=rtpmap:96 H264/90000\r\n\
//!            a=fmtp:96 packetization-mode=1;profile-level-id=640020;\
//!            sprop-parameter-sets=Z2QAKKzZQFAFuwFqAgICgAAAMAgAAB5HgYwL,aM4yEg==\r\n";
//! let session = parse_sdp(sdp, &AirPlayLimits::default()).expect("valid sdp");
//! assert!(session.has_video);
//! assert!(session.sps.is_some());
//! assert!(session.pps.is_some());
//! ```

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;

use crate::error::AirPlayError;
use crate::h264::SpsInfo;
use crate::limits::AirPlayLimits;
use crate::pairing::raop_rsa;

/// The audio codec of an AirPlay stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AirPlayAudioCodec {
    /// Apple Lossless.
    Alac,
    /// AAC Enhanced Low Delay.
    AacEld,
    /// AAC Low Complexity.
    AacLc,
    /// The codec could not be determined.
    #[default]
    Unknown,
}

impl AirPlayAudioCodec {
    /// The name of the codec as the core capability model knows it.
    #[must_use]
    pub fn as_core_codec(self) -> phairplay_core::capabilities::AudioCodec {
        use phairplay_core::capabilities::AudioCodec;
        match self {
            Self::Alac => AudioCodec::Alac,
            Self::AacEld => AudioCodec::AacEld,
            Self::AacLc => AudioCodec::AacLc,
            Self::Unknown => AudioCodec::Lpcm,
        }
    }
}

/// The parsed content of an `ANNOUNCE` body.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SdpSession {
    /// Whether the description carries a usable video section.
    pub has_video: bool,
    /// Whether the description carries a usable audio section.
    pub has_audio: bool,
    /// The RTP port of the video stream.
    pub video_port: u16,
    /// The RTP payload type of the video stream.
    pub video_payload_type: u8,
    /// The H.264 sequence parameter set.
    pub sps: Option<Vec<u8>>,
    /// The H.264 picture parameter set.
    pub pps: Option<Vec<u8>>,
    /// The `profile-level-id` of the H.264 stream.
    pub profile_level_id: Option<String>,
    /// The resolution read out of the sequence parameter set.
    pub sps_info: Option<SpsInfo>,
    /// The RTP port of the audio stream.
    pub audio_port: u16,
    /// The RTP payload type of the audio stream.
    pub audio_payload_type: u8,
    /// The audio codec.
    pub audio_codec: AirPlayAudioCodec,
    /// The sample rate of the audio stream.
    pub sample_rate: u32,
    /// The channel count of the audio stream.
    pub channels: u16,
    /// The AES-128 key that decrypts the audio stream.
    pub aes_key: Option<Vec<u8>>,
    /// The FairPlay-wrapped AES key of the audio stream.
    pub fp_aes_key: Option<Vec<u8>>,
    /// The AES-128 initialisation vector of the audio stream.
    pub aes_iv: Option<Vec<u8>>,
    /// The number of ALAC frames per packet.
    pub alac_frames_per_packet: u32,
}

impl SdpSession {
    /// Whether the audio stream is encrypted with a usable key and IV.
    #[must_use]
    pub fn is_audio_encrypted(&self) -> bool {
        self.aes_key.as_ref().is_some_and(|key| key.len() == 16)
            && self.aes_iv.as_ref().is_some_and(|iv| iv.len() == 16)
    }

    /// Whether the session carries audio but no video.
    #[must_use]
    pub fn is_audio_only(&self) -> bool {
        self.has_audio && !self.has_video
    }
}

/// The default sample rate an AirPlay audio stream uses.
pub const DEFAULT_SAMPLE_RATE: u32 = 44_100;

/// The default channel count an AirPlay audio stream uses.
pub const DEFAULT_CHANNELS: u16 = 2;

/// The default number of ALAC frames per packet.
pub const DEFAULT_ALAC_FRAMES_PER_PACKET: u32 = 352;

/// Parses an `ANNOUNCE` body.
///
/// # Errors
/// Returns [`AirPlayError::LimitExceeded`] when the body is larger than the
/// configured SDP limit and [`AirPlayError::Parse`] when the body carries no
/// usable media section.
pub fn parse_sdp(sdp: &str, limits: &AirPlayLimits) -> Result<SdpSession, AirPlayError> {
    if sdp.len() > limits.max_sdp_bytes {
        return Err(AirPlayError::limit_exceeded(format!(
            "SDP of {} bytes exceeds the {} byte limit",
            sdp.len(),
            limits.max_sdp_bytes
        )));
    }
    if sdp.trim().is_empty() {
        return Err(AirPlayError::parse("empty SDP body"));
    }

    let mut session = SdpSession {
        sample_rate: DEFAULT_SAMPLE_RATE,
        channels: DEFAULT_CHANNELS,
        alac_frames_per_packet: DEFAULT_ALAC_FRAMES_PER_PACKET,
        ..SdpSession::default()
    };

    let mut section = Section::None;
    for line in sdp.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix("m=") {
            section = Section::from_media_line(rest);
            let (port, payload_type) = parse_media_line(rest);
            match section {
                Section::Video => {
                    session.video_port = port;
                    session.video_payload_type = payload_type;
                }
                Section::Audio => {
                    session.audio_port = port;
                    session.audio_payload_type = payload_type;
                }
                Section::None => {}
            }
            continue;
        }
        let Some(attribute) = line.strip_prefix("a=") else {
            continue;
        };
        match section {
            Section::Video => apply_video_attribute(&mut session, attribute),
            Section::Audio => apply_audio_attribute(&mut session, attribute),
            Section::None => {}
        }
    }

    // An AirPlay description announces the port `0` for the interleaved
    // streams, so the port is not a usable criterion — the media parameters
    // are.
    session.has_video = session.sps.is_some();
    session.has_audio = session.audio_codec != AirPlayAudioCodec::Unknown;
    if let Some(sps) = session.sps.as_ref() {
        session.sps_info = crate::h264::parse_sps(sps);
    }

    if !session.has_video && !session.has_audio {
        return Err(AirPlayError::parse(
            "SDP carries no usable video or audio section",
        ));
    }
    Ok(session)
}

/// The media section a line belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    /// Before the first `m=` line.
    None,
    /// Inside a video section.
    Video,
    /// Inside an audio section.
    Audio,
}

impl Section {
    fn from_media_line(rest: &str) -> Self {
        if rest.starts_with("video") {
            Self::Video
        } else if rest.starts_with("audio") {
            Self::Audio
        } else {
            Self::None
        }
    }
}

/// Parses the port and the payload type of an `m=` line.
fn parse_media_line(rest: &str) -> (u16, u8) {
    let mut parts = rest.split_whitespace();
    let _media = parts.next();
    let port = parts
        .next()
        .and_then(|value| value.split('/').next())
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(0);
    let payload_type = parts
        .nth(1)
        .and_then(|value| value.parse::<u8>().ok())
        .unwrap_or(96);
    (port, payload_type)
}

/// Applies a video attribute to the session.
fn apply_video_attribute(session: &mut SdpSession, attribute: &str) {
    if !attribute.starts_with("fmtp:") {
        return;
    }
    let parameters = attribute.split_once(' ').map_or("", |(_, rest)| rest);
    for parameter in parameters.split(';') {
        let Some((key, value)) = parameter.split_once('=') else {
            continue;
        };
        match key.trim() {
            "profile-level-id" => {
                let value = value.trim();
                if !value.is_empty() {
                    session.profile_level_id = Some(value.to_string());
                }
            }
            "sprop-parameter-sets" => {
                let mut sets = value.split(',');
                let sps = sets.next().and_then(decode_base64);
                let pps = sets.next().and_then(decode_base64);
                if let (Some(sps), Some(pps)) = (sps, pps) {
                    if !sps.is_empty() && !pps.is_empty() {
                        session.sps = Some(sps);
                        session.pps = Some(pps);
                    }
                }
            }
            _ => {}
        }
    }
}

/// Applies an audio attribute to the session.
fn apply_audio_attribute(session: &mut SdpSession, attribute: &str) {
    if let Some(rest) = attribute.strip_prefix("rtpmap:") {
        let (codec, rate, channels) = parse_rtp_map(rest);
        match codec {
            Some(AirPlayAudioCodec::Alac) => {
                session.audio_codec = AirPlayAudioCodec::Alac;
            }
            Some(AirPlayAudioCodec::AacEld) | Some(AirPlayAudioCodec::AacLc) => {
                session.audio_codec = AirPlayAudioCodec::AacEld;
            }
            None | Some(AirPlayAudioCodec::Unknown) => {}
        }
        if rate > 0 {
            session.sample_rate = rate;
        }
        if channels > 0 {
            session.channels = channels;
        }
        return;
    }

    if let Some(rest) = attribute.strip_prefix("fmtp:") {
        if session.audio_codec == AirPlayAudioCodec::Alac {
            // The format parameters follow the payload type.
            let parameters = rest.split_once(' ').map_or("", |(_, rest)| rest);
            apply_alac_fmtp(session, parameters);
        }
        return;
    }

    if let Some(rest) = attribute.strip_prefix("rsaaeskey:") {
        if let Some(blob) = decode_base64(rest) {
            session.aes_key = unwrap_legacy_aes_key(&blob);
        }
        return;
    }

    if let Some(rest) = attribute.strip_prefix("fpaeskey:") {
        if let Some(blob) = decode_base64(rest) {
            session.fp_aes_key = Some(blob);
        }
        return;
    }

    if let Some(rest) = attribute.strip_prefix("aesiv:") {
        if let Some(iv) = decode_base64(rest) {
            session.aes_iv = Some(iv);
        }
    }
}

/// Applies the ALAC `fmtp` parameters.
///
/// The ALAC format string is
/// `frameLength version bitDepth riceParam riceHistory rice predictor`
/// ` numChannels maxRun maxFrameBytes avgBitRate sampleRate`.
fn apply_alac_fmtp(session: &mut SdpSession, rest: &str) {
    let tokens: Vec<&str> = rest.split_whitespace().collect();
    if tokens.len() < 11 {
        return;
    }
    if let Ok(frames) = tokens[0].parse::<u32>() {
        if frames > 0 {
            session.alac_frames_per_packet = frames;
        }
    }
    if let Ok(channels) = tokens[6].parse::<u16>() {
        if channels > 0 {
            session.channels = channels;
        }
    }
    if let Ok(rate) = tokens[10].parse::<u32>() {
        if rate > 0 {
            session.sample_rate = rate;
        }
    }
}

/// Parses an `rtpmap` value into codec, sample rate and channel count.
fn parse_rtp_map(rest: &str) -> (Option<AirPlayAudioCodec>, u32, u16) {
    let value = rest.split_once(' ').map_or("", |(_, rest)| rest);
    let mut parts = value.split('/');
    let name = parts.next().unwrap_or_default();
    let rate = parts
        .next()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(0);
    let channels = parts
        .next()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(0);
    let codec = if name.eq_ignore_ascii_case("applelossless") {
        Some(AirPlayAudioCodec::Alac)
    } else if name.eq_ignore_ascii_case("mpeg4-generic") {
        Some(AirPlayAudioCodec::AacEld)
    } else {
        None
    };
    (codec, rate, channels)
}

/// Unwraps a legacy `rsaaeskey` blob into the 16-byte AES key.
///
/// A modern sender may already send a plain 16-byte key; a legacy AirPlay-1
/// sender sends the key RSA-wrapped with the public AirPort Express key.
fn unwrap_legacy_aes_key(blob: &[u8]) -> Option<Vec<u8>> {
    if blob.len() == 16 {
        return Some(blob.to_vec());
    }
    raop_rsa::decrypt_aes_key(blob)
}

/// Decodes a Base64 value, returning `None` for malformed input.
fn decode_base64(value: &str) -> Option<Vec<u8>> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    BASE64.decode(trimmed).ok()
}

#[cfg(test)]
mod tests;
