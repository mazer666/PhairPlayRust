#![allow(clippy::expect_used, clippy::unwrap_used)]

use super::*;
use crate::rtsp::request::RtspRequest;

fn context<'a>(
    identity: &'a DeviceIdentity,
    capabilities: &'a MediaCapabilities,
    limits: &'a AirPlayLimits,
) -> RouteContext<'a> {
    RouteContext::new(identity, capabilities, limits)
}

fn request(text: &str) -> RtspRequest {
    RtspRequest::parse(text.as_bytes(), 64 * 1024).expect("valid request")
}

#[test]
fn an_options_request_is_answered_with_the_public_verbs() {
    let identity = DeviceIdentity::default();
    let capabilities = MediaCapabilities::empty();
    let limits = AirPlayLimits::default();
    let ctx = context(&identity, &capabilities, &limits);
    let mut session = AirPlaySession::new();
    let response = route(
        &request("OPTIONS rtsp://x/y RTSP/1.0\r\nCSeq: 1\r\n\r\n"),
        &mut session,
        &ctx,
    );
    assert_eq!(response.status, 200);
    assert!(
        response
            .headers
            .iter()
            .any(|(name, value)| name == "Public" && value.contains("ANNOUNCE"))
    );
}

#[test]
fn an_announce_with_a_valid_sdp_is_accepted() {
    let identity = DeviceIdentity::default();
    let capabilities = MediaCapabilities::empty();
    let limits = AirPlayLimits::default();
    let ctx = context(&identity, &capabilities, &limits);
    let mut session = AirPlaySession::new();
    let sdp = "m=video 0 RTP/AVP 96\r\na=fmtp:96 packetization-mode=1;sprop-parameter-sets=Z2QAKKzZQFAFuwFqAgICgAAAMAgAAB5HgYwL,aM4yEg==\r\n";
    let response = route(
        &request(&format!(
            "ANNOUNCE rtsp://x/y RTSP/1.0\r\nCSeq: 2\r\nContent-Length: {}\r\n\r\n{sdp}",
            sdp.len()
        )),
        &mut session,
        &ctx,
    );
    assert_eq!(response.status, 200);
    assert!(session.has_video());
}

#[test]
fn an_announce_with_a_broken_sdp_is_rejected() {
    let identity = DeviceIdentity::default();
    let capabilities = MediaCapabilities::empty();
    let limits = AirPlayLimits::default();
    let ctx = context(&identity, &capabilities, &limits);
    let mut session = AirPlaySession::new();
    let response = route(
        &request("ANNOUNCE rtsp://x/y RTSP/1.0\r\nCSeq: 2\r\n\r\n"),
        &mut session,
        &ctx,
    );
    assert_eq!(response.status, 400);
    assert!(session.announced.is_none());
}

#[test]
fn a_record_without_an_announce_is_rejected() {
    let identity = DeviceIdentity::default();
    let capabilities = MediaCapabilities::empty();
    let limits = AirPlayLimits::default();
    let ctx = context(&identity, &capabilities, &limits);
    let mut session = AirPlaySession::new();
    let response = route(
        &request("RECORD rtsp://x/y RTSP/1.0\r\nCSeq: 3\r\n\r\n"),
        &mut session,
        &ctx,
    );
    assert_eq!(response.status, 455);
}

#[test]
fn an_unknown_method_is_not_implemented() {
    let identity = DeviceIdentity::default();
    let capabilities = MediaCapabilities::empty();
    let limits = AirPlayLimits::default();
    let ctx = context(&identity, &capabilities, &limits);
    let mut session = AirPlaySession::new();
    let response = route(
        &request("FROBNICATE rtsp://x/y RTSP/1.0\r\n\r\n"),
        &mut session,
        &ctx,
    );
    assert_eq!(response.status, 501);
}

#[test]
fn the_volume_is_answered_with_the_last_value() {
    let identity = DeviceIdentity::default();
    let capabilities = MediaCapabilities::empty();
    let limits = AirPlayLimits::default();
    let ctx = context(&identity, &capabilities, &limits);
    let mut session = AirPlaySession::new();
    session.volume = -12.5;
    let response = route(
        &request("GET_PARAMETER rtsp://x/y RTSP/1.0\r\nCSeq: 4\r\n\r\nvolume\r\n"),
        &mut session,
        &ctx,
    );
    assert_eq!(response.status, 200);
    assert!(String::from_utf8_lossy(&response.body).contains("-12.5"));
}

#[test]
fn the_volume_is_stored_when_the_sender_sets_it() {
    let identity = DeviceIdentity::default();
    let capabilities = MediaCapabilities::empty();
    let limits = AirPlayLimits::default();
    let ctx = context(&identity, &capabilities, &limits);
    let mut session = AirPlaySession::new();
    let response = route(
        &request("SET_PARAMETER rtsp://x/y RTSP/1.0\r\nCSeq: 5\r\n\r\nvolume: -7.000000\r\n"),
        &mut session,
        &ctx,
    );
    assert_eq!(response.status, 200);
    assert_eq!(session.volume, -7.0);
}
