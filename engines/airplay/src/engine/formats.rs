//! The negotiated formats of a session.

use phairplay_core::media::{AudioFormat, VideoFormat};

use crate::session::AirPlaySession;

/// The video format of a negotiated session.
#[must_use]
pub fn video_format_of(session: &AirPlaySession) -> Option<VideoFormat> {
    let description = session.announced.as_ref()?;
    if !description.has_video {
        return None;
    }
    let resolution = description.sps_info;
    Some(VideoFormat {
        codec: phairplay_core::capabilities::VideoCodec::H264,
        width: resolution.map_or(1920, |info| info.width),
        height: resolution.map_or(1080, |info| info.height),
        frame_rate: Some(60),
        sps: description.sps.clone(),
        pps: description.pps.clone(),
    })
}

/// The audio format of a negotiated session.
/// The audio format of a negotiated session.
#[must_use]
pub fn audio_format_of(session: &AirPlaySession) -> Option<AudioFormat> {
    let description = session.announced.as_ref()?;
    if !description.has_audio {
        return None;
    }
    Some(AudioFormat {
        codec: description.audio_codec.as_core_codec(),
        sample_rate: description.sample_rate,
        channels: description.channels,
        samples_per_packet: description.alac_frames_per_packet,
    })
}
