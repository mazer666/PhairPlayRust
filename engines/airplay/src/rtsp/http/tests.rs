//! The tests of the HTTP-like routes.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use super::*;
use crate::limits::AirPlayLimits;
use crate::pairing::identity::PairingIdentity;
use crate::plist::PlistValue;
use crate::rtsp::request::RtspRequest;
use crate::rtsp::router::RouteContext;
use crate::session::AirPlaySession;
use phairplay_core::capabilities::MediaCapabilities;
use phairplay_core::identity::DeviceIdentity;
use std::sync::Arc;

fn request(text: &str) -> RtspRequest {
    RtspRequest::parse(text.as_bytes(), 1024 * 1024).expect("valid request")
}

/// Builds a request with a binary body.
fn binary_request(head: &str, body: &[u8]) -> RtspRequest {
    let mut raw = head.as_bytes().to_vec();
    raw.extend_from_slice(body);
    RtspRequest::parse(&raw, 1024 * 1024).expect("valid request")
}

fn context<'a>(
    identity: &'a DeviceIdentity,
    capabilities: &'a MediaCapabilities,
    limits: &'a AirPlayLimits,
) -> RouteContext<'a> {
    let mut context = RouteContext::new(identity, capabilities, limits);
    context.pairing_public_key = Some(PairingIdentity::from_seed(&[1u8; 32]).public_key());
    context
}

#[test]
fn the_info_answer_is_a_binary_plist() {
    let identity = DeviceIdentity::default();
    let capabilities = MediaCapabilities::empty();
    let limits = AirPlayLimits::default();
    let ctx = context(&identity, &capabilities, &limits);
    let mut session = AirPlaySession::new();
    let response = route_get(&request("GET /info HTTP/1.1\r\n\r\n"), &mut session, &ctx);
    assert_eq!(response.status, 200);
    assert!(response.body.starts_with(b"bplist00"));
    let document = crate::plist::decode(&response.body, 1024 * 1024).expect("decodable");
    assert_eq!(
        document.get("model").and_then(PlistValue::as_str),
        Some(MODEL)
    );
    assert!(document.get("pk").is_some());
}

#[test]
fn an_unknown_get_route_is_not_implemented() {
    let identity = DeviceIdentity::default();
    let capabilities = MediaCapabilities::empty();
    let limits = AirPlayLimits::default();
    let ctx = context(&identity, &capabilities, &limits);
    let mut session = AirPlaySession::new();
    let response = route_get(
        &request("GET /nonsense HTTP/1.1\r\n\r\n"),
        &mut session,
        &ctx,
    );
    assert_eq!(response.status, 501);
}

#[test]
fn the_pair_setup_answer_carries_the_public_key() {
    let identity = DeviceIdentity::default();
    let capabilities = MediaCapabilities::empty();
    let limits = AirPlayLimits::default();
    let ctx = context(&identity, &capabilities, &limits);
    let mut session =
        AirPlaySession::with_identity(Arc::new(PairingIdentity::from_seed(&[4u8; 32])));
    let body = vec![0u8; 32];
    let text = format!(
        "POST /pair-setup HTTP/1.1\r\nContent-Length: {}\r\n\r\n",
        body.len()
    );
    let response = route_post(&binary_request(&text, &body), &mut session, &ctx);
    assert_eq!(response.status, 200);
    assert_eq!(response.body.len(), 32);
}

#[test]
fn a_pair_setup_with_a_wrong_body_is_rejected() {
    let identity = DeviceIdentity::default();
    let capabilities = MediaCapabilities::empty();
    let limits = AirPlayLimits::default();
    let ctx = context(&identity, &capabilities, &limits);
    let mut session = AirPlaySession::new();
    let response = route_post(
        &request("POST /pair-setup HTTP/1.1\r\nContent-Length: 4\r\n\r\nabcd"),
        &mut session,
        &ctx,
    );
    assert_eq!(response.status, 400);
}

#[test]
fn the_fair_play_phase_one_is_answered() {
    let identity = DeviceIdentity::default();
    let capabilities = MediaCapabilities::empty();
    let limits = AirPlayLimits::default();
    let ctx = context(&identity, &capabilities, &limits);
    let mut session = AirPlaySession::new();
    let mut body = vec![0u8; 16];
    body[4] = 0x03;
    let text = format!(
        "POST /fp-setup HTTP/1.1\r\nContent-Length: {}\r\n\r\n",
        body.len()
    );
    let response = route_post(&binary_request(&text, &body), &mut session, &ctx);
    assert_eq!(response.status, 200);
    assert_eq!(response.body.len(), 142);
}

#[test]
fn a_fair_play_request_of_the_wrong_size_is_rejected() {
    let identity = DeviceIdentity::default();
    let capabilities = MediaCapabilities::empty();
    let limits = AirPlayLimits::default();
    let ctx = context(&identity, &capabilities, &limits);
    let mut session = AirPlaySession::new();
    let response = route_post(
        &request("POST /fp-setup HTTP/1.1\r\nContent-Length: 3\r\n\r\nabc"),
        &mut session,
        &ctx,
    );
    assert_eq!(response.status, 400);
}

#[test]
fn a_photo_is_accepted_and_published() {
    let identity = DeviceIdentity::default();
    let capabilities = MediaCapabilities::empty();
    let limits = AirPlayLimits::default();
    let ctx = context(&identity, &capabilities, &limits);
    let mut session = AirPlaySession::new();
    let photos: super::super::router::PhotoSlot = Arc::new(std::sync::Mutex::new(None));
    let mut ctx = ctx;
    ctx.photos = Some(&photos);
    let jpeg = [0xFFu8, 0xD8, 0xFF, 0xE0];
    let text = format!(
        "PUT /photo HTTP/1.1\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n",
        jpeg.len()
    );
    let response = route_put(&binary_request(&text, &jpeg), &mut session, &ctx);
    assert_eq!(response.status, 200);
    let published = photos.lock().expect("lock").clone();
    assert!(published.is_some());
}

#[test]
fn a_photo_with_random_bytes_is_rejected() {
    let identity = DeviceIdentity::default();
    let capabilities = MediaCapabilities::empty();
    let limits = AirPlayLimits::default();
    let ctx = context(&identity, &capabilities, &limits);
    let mut session = AirPlaySession::new();
    let response = route_put(
        &request("PUT /photo HTTP/1.1\r\nContent-Length: 4\r\n\r\nabcd"),
        &mut session,
        &ctx,
    );
    assert_eq!(response.status, 400);
}

#[test]
fn a_photo_delete_clears_the_published_photo() {
    let identity = DeviceIdentity::default();
    let capabilities = MediaCapabilities::empty();
    let limits = AirPlayLimits::default();
    let ctx = context(&identity, &capabilities, &limits);
    let mut session = AirPlaySession::new();
    let photos: super::super::router::PhotoSlot = Arc::new(std::sync::Mutex::new(None));
    let mut ctx = ctx;
    ctx.photos = Some(&photos);
    {
        let mut guard = photos.lock().expect("lock");
        *guard = Some(crate::photo::PhotoPayload {
            image_type: crate::photo::PhotoImageType::Jpeg,
            data: Arc::new(vec![1, 2, 3]),
        });
    }
    let response = route_delete(
        &request("DELETE /photo HTTP/1.1\r\n\r\n"),
        &mut session,
        &ctx,
    );
    assert_eq!(response.status, 200);
    assert!(photos.lock().expect("lock").is_none());
}

#[test]
fn a_play_request_stores_the_media_url() {
    let identity = DeviceIdentity::default();
    let capabilities = MediaCapabilities::empty();
    let limits = AirPlayLimits::default();
    let ctx = context(&identity, &capabilities, &limits);
    let mut session = AirPlaySession::new();
    let response = route_post(
        &request("POST /play HTTP/1.1\r\n\r\nContent-Location: https://example.test/a.m3u8\r\n"),
        &mut session,
        &ctx,
    );
    assert_eq!(response.status, 200);
    assert_eq!(
        session.playback.url.as_deref(),
        Some("https://example.test/a.m3u8")
    );
}

#[test]
fn a_play_request_without_a_url_is_rejected() {
    let identity = DeviceIdentity::default();
    let capabilities = MediaCapabilities::empty();
    let limits = AirPlayLimits::default();
    let ctx = context(&identity, &capabilities, &limits);
    let mut session = AirPlaySession::new();
    let response = route_post(&request("POST /play HTTP/1.1\r\n\r\n"), &mut session, &ctx);
    assert_eq!(response.status, 400);
}

#[test]
fn the_rate_and_scrub_routes_update_the_playback_state() {
    let identity = DeviceIdentity::default();
    let capabilities = MediaCapabilities::empty();
    let limits = AirPlayLimits::default();
    let ctx = context(&identity, &capabilities, &limits);
    let mut session = AirPlaySession::new();
    let response = route_post(
        &request("POST /rate?value=0 HTTP/1.1\r\n\r\n"),
        &mut session,
        &ctx,
    );
    assert_eq!(response.status, 200);
    assert_eq!(session.playback.rate, 0.0);
    let response = route_post(
        &request("POST /scrub?position=12.5 HTTP/1.1\r\n\r\n"),
        &mut session,
        &ctx,
    );
    assert_eq!(response.status, 200);
    assert_eq!(session.playback.position_sec, 12.5);
}

#[test]
fn the_stop_route_clears_the_playback_state() {
    let identity = DeviceIdentity::default();
    let capabilities = MediaCapabilities::empty();
    let limits = AirPlayLimits::default();
    let ctx = context(&identity, &capabilities, &limits);
    let mut session = AirPlaySession::new();
    session.playback.url = Some("https://example.test/a.m3u8".into());
    let response = route_post(&request("POST /stop HTTP/1.1\r\n\r\n"), &mut session, &ctx);
    assert_eq!(response.status, 200);
    assert!(session.playback.url.is_none());
}

#[test]
fn the_playback_info_answer_is_an_xml_plist() {
    let identity = DeviceIdentity::default();
    let capabilities = MediaCapabilities::empty();
    let limits = AirPlayLimits::default();
    let ctx = context(&identity, &capabilities, &limits);
    let mut session = AirPlaySession::new();
    let response = route_get(
        &request("GET /playback-info HTTP/1.1\r\n\r\n"),
        &mut session,
        &ctx,
    );
    assert_eq!(response.status, 200);
    let text = String::from_utf8_lossy(&response.body);
    assert!(text.contains("<key>readyToPlay</key>"), "{text}");
}

#[test]
fn the_scrub_answer_reports_the_position() {
    let identity = DeviceIdentity::default();
    let capabilities = MediaCapabilities::empty();
    let limits = AirPlayLimits::default();
    let ctx = context(&identity, &capabilities, &limits);
    let mut session = AirPlaySession::new();
    session.playback.position_sec = 3.5;
    let response = route_get(&request("GET /scrub HTTP/1.1\r\n\r\n"), &mut session, &ctx);
    assert_eq!(response.status, 200);
    let text = String::from_utf8_lossy(&response.body);
    assert!(text.contains("position: 3.500000"), "{text}");
}

#[test]
fn the_generated_pin_has_four_digits() {
    let pin = generate_pin();
    assert_eq!(pin.len(), 4);
    assert!(
        pin.chars().all(|character| character.is_ascii_digit()),
        "{pin}"
    );
}

#[test]
fn two_generated_pins_differ() {
    // The PIN space is large; two draws must not collide in practice.
    let first = generate_pin();
    let second = generate_pin();
    assert_ne!(first, second);
}

#[test]
fn the_pin_start_route_is_refused_without_a_pin_policy() {
    let identity = DeviceIdentity::default();
    let capabilities = MediaCapabilities::empty();
    let limits = AirPlayLimits::default();
    let ctx = context(&identity, &capabilities, &limits);
    let mut session = AirPlaySession::new();
    let response = route_post(
        &request("POST /pair-pin-start HTTP/1.1\r\n\r\n"),
        &mut session,
        &ctx,
    );
    assert_eq!(response.status, 501);
}

#[test]
fn the_pin_start_route_shows_a_pin_when_it_is_required() {
    let identity = DeviceIdentity::default();
    let capabilities = MediaCapabilities::empty();
    let limits = AirPlayLimits::default();
    let mut ctx = context(&identity, &capabilities, &limits);
    ctx.require_pin = true;
    let mut session = AirPlaySession::new();
    let response = route_post(
        &request("POST /pair-pin-start HTTP/1.1\r\n\r\n"),
        &mut session,
        &ctx,
    );
    assert_eq!(response.status, 200);
    assert!(session.shown_pin.is_some());
    assert!(session.legacy_pin.is_some());
}

#[test]
fn a_teardown_without_a_body_names_no_stream() {
    let value = teardown_stream_types(&request("TEARDOWN rtsp://x/y RTSP/1.0\r\n\r\n"));
    assert!(value.is_none());
}
