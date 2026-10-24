#![allow(clippy::expect_used, clippy::unwrap_used)]

use super::*;

use crate::error::AirPlayError;

const VIDEO_SDP: &str = "v=0\r\n\
o=AirTunes AA:BB:CC:DD:EE:FF 1 IN IP4 192.168.1.10\r\n\
s=AirTunes\r\n\
t=0 0\r\n\
m=video 0 RTP/AVP 96\r\n\
a=rtpmap:96 H264/90000\r\n\
a=fmtp:96 packetization-mode=1;profile-level-id=640020;sprop-parameter-sets=Z2QAKKzZQFAFuwFqAgICgAAAMAgAAB5HgYwL,aM4yEg==\r\n\
m=audio 0 RTP/AVP 96\r\n\
a=rtpmap:96 mpeg4-generic/44100/2\r\n\
a=rsaaeskey:MDEyMzQ1Njc4OWFiY2RlZg==\r\n\
a=aesiv:MDEyMzQ1Njc4OWFiY2RlZg==\r\n";

const ALAC_SDP: &str = "v=0\r\n\
m=audio 6001 RTP/AVP 96\r\n\
a=rtpmap:96 AppleLossless\r\n\
a=fmtp:96 352 0 16 40 10 14 2 255 0 0 44100\r\n";

#[test]
fn a_video_and_audio_session_is_parsed() {
    let session = parse_sdp(VIDEO_SDP, &AirPlayLimits::default()).expect("valid sdp");
    assert!(session.has_video);
    assert!(session.has_audio);
    assert_eq!(session.video_payload_type, 96);
    assert_eq!(session.audio_payload_type, 96);
    assert_eq!(session.sample_rate, 44_100);
    assert_eq!(session.channels, 2);
    assert_eq!(session.profile_level_id.as_deref(), Some("640020"));
}

#[test]
fn the_h264_parameter_sets_are_extracted() {
    let session = parse_sdp(VIDEO_SDP, &AirPlayLimits::default()).expect("valid sdp");
    let sps = session.sps.expect("sps");
    let pps = session.pps.expect("pps");
    assert!(!sps.is_empty());
    assert!(!pps.is_empty());
    assert_eq!(sps[0] & 0x1F, 7, "the SPS must be an SPS NAL unit");
    assert_eq!(pps[0] & 0x1F, 8, "the PPS must be a PPS NAL unit");
}

#[test]
fn the_aes_key_and_iv_are_extracted() {
    let session = parse_sdp(VIDEO_SDP, &AirPlayLimits::default()).expect("valid sdp");
    assert_eq!(session.aes_key.as_deref(), Some(&b"0123456789abcdef"[..]));
    assert_eq!(session.aes_iv.as_deref(), Some(&b"0123456789abcdef"[..]));
    assert!(session.is_audio_encrypted());
}

#[test]
fn an_alac_session_carries_the_frame_geometry() {
    let session = parse_sdp(ALAC_SDP, &AirPlayLimits::default()).expect("valid sdp");
    assert_eq!(session.audio_codec, AirPlayAudioCodec::Alac);
    assert_eq!(session.alac_frames_per_packet, 352);
    assert_eq!(session.sample_rate, 44_100);
    assert_eq!(session.channels, 2);
    assert_eq!(session.audio_port, 6001);
    assert!(session.is_audio_only());
}

#[test]
fn an_empty_sdp_is_rejected() {
    let error = parse_sdp("   ", &AirPlayLimits::default()).expect_err("must fail");
    assert!(matches!(error, AirPlayError::Parse { .. }));
}

#[test]
fn an_sdp_without_a_media_section_is_rejected() {
    let error = parse_sdp("v=0\r\ns=AirTunes\r\nt=0 0\r\n", &AirPlayLimits::default())
        .expect_err("must fail");
    assert!(matches!(error, AirPlayError::Parse { .. }));
}

#[test]
fn an_oversized_sdp_is_rejected() {
    let limits = AirPlayLimits {
        max_sdp_bytes: 16,
        ..AirPlayLimits::default()
    };
    let error = parse_sdp(VIDEO_SDP, &limits).expect_err("must fail");
    assert!(matches!(error, AirPlayError::LimitExceeded { .. }));
}

#[test]
fn a_malformed_base64_parameter_set_is_ignored() {
    let sdp = "m=video 0 RTP/AVP 96\r\n\
a=fmtp:96 packetization-mode=1;sprop-parameter-sets=!!!not-base64!!!,aMuMsg==\r\n";
    let error = parse_sdp(sdp, &AirPlayLimits::default()).expect_err("must fail");
    assert!(matches!(error, AirPlayError::Parse { .. }));
}

#[test]
fn a_session_with_only_audio_is_recognised() {
    let sdp = "m=audio 6001 RTP/AVP 96\r\na=rtpmap:96 AppleLossless\r\n";
    let session = parse_sdp(sdp, &AirPlayLimits::default()).expect("valid sdp");
    assert!(session.has_audio);
    assert!(!session.has_video);
}

#[test]
fn an_aac_session_keeps_the_eld_codec() {
    let sdp = "m=audio 6001 RTP/AVP 96\r\na=rtpmap:96 mpeg4-generic/48000/1\r\n";
    let session = parse_sdp(sdp, &AirPlayLimits::default()).expect("valid sdp");
    assert_eq!(session.audio_codec, AirPlayAudioCodec::AacEld);
    assert_eq!(session.sample_rate, 48_000);
    assert_eq!(session.channels, 1);
}
