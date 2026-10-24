//! The HTTP-like routes of the AirPlay handshake.
//!
//! WHY: the AirPlay 2 handshake rides on the RTSP port but speaks HTTP:
//! `GET /info` advertises the receiver, `POST /pair-setup` and
//! `POST /pair-verify` establish the identity, `POST /fp-setup` runs the
//! FairPlay key exchange, `PUT /photo` transfers a picture. The video
//! playback mode adds `POST /play`, `/rate`, `/scrub` and `/stop`.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_airplay::rtsp::RtspRequest;
//! use phairplay_airplay::rtsp::http::parse_play_request;
//! use phairplay_airplay::plist::PlistValue;
//!
//! let body = PlistValue::dictionary(vec![
//!     ("Content-Location".into(), PlistValue::String("https://example.test/a.m3u8".into())),
//! ]);
//! let (url, start) = parse_play_request(&body, None);
//! assert_eq!(url.as_deref(), Some("https://example.test/a.m3u8"));
//! assert_eq!(start, 0.0);
//! ```

use phairplay_core::session::SessionEvent;
use std::sync::Arc;

use crate::plist::{PlistValue, binary, xml};
use crate::rtsp::request::RtspRequest;
use crate::rtsp::response::RtspResponse;
use crate::rtsp::router::RouteContext;
use crate::session::AirPlaySession;

/// The content type of a binary property list.
pub const BINARY_PLIST: &str = "application/x-apple-binary-plist";

/// The content type of an XML property list.
pub const XML_PLIST: &str = "text/x-apple-plist+xml";

/// The content type of an octet stream.
pub const OCTET_STREAM: &str = "application/octet-stream";

/// The number of digits of the pairing PIN.
pub const PIN_DIGITS: u32 = 4;

/// The number of possible PINs.
pub const PIN_SPACE: u32 = 10_000;

/// The feature bits the receiver advertises.
pub const AIRPLAY_FEATURES: i64 = 0x001E_5A7F_FFF7;

/// The status flags of the receiver.
pub const STATUS_FLAGS: i64 = 68;

/// The status flag that announces a required PIN.
pub const STATUS_FLAG_PIN_REQUIRED: i64 = 0x8;

/// The model the receiver announces.
pub const MODEL: &str = "AppleTV5,3";

/// The source version the receiver announces.
pub const SOURCE_VERSION: &str = "220.68";

/// The UDP port the timing service listens on.
pub const TIMING_PORT: u16 = 6002;

/// Routes an HTTP-like read request.
pub fn route_get(
    request: &RtspRequest,
    session: &mut AirPlaySession,
    context: &RouteContext<'_>,
) -> RtspResponse {
    match request.path() {
        "/info" => info(request, context),
        "/playback-info" => playback_info(request, session),
        "/scrub" => scrub(request, session),
        "/server-info" => server_info(request),
        _ => RtspResponse::not_implemented().with_protocol(request.response_protocol()),
    }
}

/// Routes an HTTP-like write request.
/// Routes an HTTP-like write request.
pub fn route_post(
    request: &RtspRequest,
    session: &mut AirPlaySession,
    context: &RouteContext<'_>,
) -> RtspResponse {
    match request.path() {
        "/pair-setup" => pair_setup(request, session),
        "/pair-verify" => pair_verify(request, session),
        "/pair-pin-start" => pair_pin_start(request, session, context),
        "/pair-setup-pin" => pair_setup_pin(request, session, context),
        "/fp-setup" => fp_setup(request, session),
        "/feedback" => RtspResponse::ok().with_protocol(request.response_protocol()),
        "/audioMode" => RtspResponse::ok().with_protocol(request.response_protocol()),
        "/play" => play(request, session),
        "/rate" => rate(request, session),
        "/scrub" => scrub_post(request, session),
        "/stop" => stop(request, session),
        _ => RtspResponse::not_implemented().with_protocol(request.response_protocol()),
    }
}

/// Routes the upload of a photo.
/// Routes the upload of a photo.
pub fn route_put(
    request: &RtspRequest,
    _session: &mut AirPlaySession,
    context: &RouteContext<'_>,
) -> RtspResponse {
    if request.path() != crate::photo::PHOTO_PATH {
        return RtspResponse::not_implemented().with_protocol(request.response_protocol());
    }
    let content_type = request.header("Content-Type");
    let validation = crate::photo::validate_photo(
        &request.body,
        content_type,
        context.limits.max_photo_bytes,
    );
    match validation {
        crate::photo::PhotoValidation::Valid { image_type } => {
            let payload = crate::photo::PhotoPayload {
                image_type,
                data: Arc::new(request.body.clone()),
            };
            if let Some(slot) = context.photos {
                if let Ok(mut guard) = slot.lock() {
                    *guard = Some(payload);
                }
            }
            RtspResponse::ok().with_protocol(request.response_protocol())
        }
        crate::photo::PhotoValidation::Invalid { reason } => {
            tracing::warn!(%reason, "a photo was rejected");
            RtspResponse::bad_request().with_protocol(request.response_protocol())
        }
    }
}

/// Routes the removal of a photo.
/// Routes the removal of a photo.
pub fn route_delete(
    request: &RtspRequest,
    _session: &mut AirPlaySession,
    context: &RouteContext<'_>,
) -> RtspResponse {
    if request.path() != crate::photo::PHOTO_PATH {
        return RtspResponse::not_implemented().with_protocol(request.response_protocol());
    }
    if let Some(slot) = context.photos {
        if let Ok(mut guard) = slot.lock() {
            *guard = None;
        }
    }
    RtspResponse::ok().with_protocol(request.response_protocol())
}

fn info(request: &RtspRequest, context: &RouteContext<'_>) -> RtspResponse {
    let mut status_flags = STATUS_FLAGS;
    if context.require_pin {
        status_flags |= STATUS_FLAG_PIN_REQUIRED;
    }
    let identity = context.identity;
    let mut entries = vec![
        ("deviceID".into(), PlistValue::String(identity.device_id.clone())),
        ("macAddress".into(), PlistValue::String(identity.device_id.clone())),
        ("features".into(), PlistValue::Integer(AIRPLAY_FEATURES)),
        ("statusFlags".into(), PlistValue::Integer(status_flags)),
        ("model".into(), PlistValue::String(MODEL.into())),
        ("name".into(), PlistValue::String(identity.name.clone())),
        ("sourceVersion".into(), PlistValue::String(SOURCE_VERSION.into())),
        ("pi".into(), PlistValue::String(identity.device_id.clone())),
        ("vv".into(), PlistValue::Integer(2)),
        ("protovers".into(), PlistValue::String("1.1".into())),
        ("keepAliveLowPower".into(), PlistValue::Boolean(true)),
        ("keepAliveSendStatsAsBody".into(), PlistValue::Boolean(true)),
        (
            "audioFormats".into(),
            PlistValue::array(vec![
                PlistValue::dictionary(vec![
                    ("type".into(), PlistValue::Integer(100)),
                    ("audioInputFormats".into(), PlistValue::Integer(67_108_860)),
                    ("audioOutputFormats".into(), PlistValue::Integer(67_108_860)),
                ]),
                PlistValue::dictionary(vec![
                    ("type".into(), PlistValue::Integer(101)),
                    ("audioInputFormats".into(), PlistValue::Integer(67_108_860)),
                    ("audioOutputFormats".into(), PlistValue::Integer(67_108_860)),
                ]),
            ]),
        ),
        (
            "audioLatencies".into(),
            PlistValue::array(vec![
                PlistValue::dictionary(vec![
                    ("type".into(), PlistValue::Integer(100)),
                    ("audioType".into(), PlistValue::String("default".into())),
                    ("inputLatencyMicros".into(), PlistValue::Integer(0)),
                    ("outputLatencyMicros".into(), PlistValue::Integer(0)),
                ]),
                PlistValue::dictionary(vec![
                    ("type".into(), PlistValue::Integer(101)),
                    ("audioType".into(), PlistValue::String("default".into())),
                    ("inputLatencyMicros".into(), PlistValue::Integer(0)),
                    ("outputLatencyMicros".into(), PlistValue::Integer(0)),
                ]),
            ]),
        ),
    ];
    if let Some(public_key) = context.pairing_public_key {
        entries.push(("pk".into(), PlistValue::Data(public_key.to_vec())));
    }
    let document = PlistValue::dictionary(entries);
    let body = binary::encode(&document).unwrap_or_default();
    RtspResponse::ok()
        .with_protocol(request.response_protocol())
        .with_typed_body(BINARY_PLIST, body)
}

/// The `GET /playback-info` answer: the state of the URL playback.
fn playback_info(request: &RtspRequest, session: &AirPlaySession) -> RtspResponse {
    let playback = &session.playback;
    let document = if !playback.ready {
        PlistValue::dictionary(vec![("readyToPlay".into(), PlistValue::Boolean(false))])
    } else {
        let ranges = PlistValue::array(vec![PlistValue::dictionary(vec![
            ("start".into(), PlistValue::Real(0.0)),
            ("duration".into(), PlistValue::Real(playback.duration_sec)),
        ])]);
        PlistValue::dictionary(vec![
            ("duration".into(), PlistValue::Real(playback.duration_sec)),
            ("position".into(), PlistValue::Real(playback.position_sec)),
            ("rate".into(), PlistValue::Real(f64::from(playback.rate))),
            ("readyToPlay".into(), PlistValue::Boolean(true)),
            ("playbackBufferEmpty".into(), PlistValue::Boolean(false)),
            ("playbackBufferFull".into(), PlistValue::Boolean(true)),
            ("playbackLikelyToKeepUp".into(), PlistValue::Boolean(true)),
            ("loadedTimeRanges".into(), ranges.clone()),
            ("seekableTimeRanges".into(), ranges),
        ])
    };
    RtspResponse::ok()
        .with_protocol(request.response_protocol())
        .with_typed_body(XML_PLIST, xml::encode(&document))
}

/// The `GET /scrub` answer: the position and the duration.
fn scrub(request: &RtspRequest, session: &AirPlaySession) -> RtspResponse {
    let body = format!(
        "duration: {:.6}\r\nposition: {:.6}\r\n",
        session.playback.duration_sec, session.playback.position_sec
    );
    RtspResponse::ok()
        .with_protocol(request.response_protocol())
        .with_typed_body("text/parameters", body.into_bytes())
}

/// The `GET /server-info` answer: the identity for the legacy video clients.
fn server_info(request: &RtspRequest) -> RtspResponse {
    let document = PlistValue::dictionary(vec![
        ("deviceid".into(), PlistValue::String(String::new())),
        ("features".into(), PlistValue::Integer(AIRPLAY_FEATURES)),
        ("model".into(), PlistValue::String(MODEL.into())),
        ("protovers".into(), PlistValue::String("1.1".into())),
        ("srcvers".into(), PlistValue::String(SOURCE_VERSION.into())),
    ]);
    RtspResponse::ok()
        .with_protocol(request.response_protocol())
        .with_typed_body(XML_PLIST, xml::encode(&document))
}

/// Parses the media URL and the start fraction of a `POST /play` body.
#[must_use]
pub fn parse_play_request(
    document: &PlistValue,
    text: Option<&str>,
) -> (Option<String>, f64) {
    let url = document
        .get("Content-Location")
        .and_then(PlistValue::as_str)
        .map(str::to_string);
    let start = document
        .get("Start-Position")
        .and_then(PlistValue::as_real)
        .unwrap_or(0.0);
    if url.is_some() {
        return (url, start);
    }
    match text {
        Some(text) => parse_text_play_request(Some(text)).unwrap_or((None, 0.0)),
        None => (None, 0.0),
    }
}

/// Parses the legacy text body of a `POST /play`.
/// Parses the legacy text body of a `POST /play`.
#[must_use]
pub fn parse_text_play_request(text: Option<&str>) -> Option<(Option<String>, f64)> {
    let text = text?;
    let mut url = None;
    let mut start = 0.0;
    for line in text.lines() {
        let lower = line.to_ascii_lowercase();
        if lower.starts_with("content-location:") {
            url = Some(line[line.find(':')? + 1..].trim().to_string());
        } else if lower.starts_with("start-position:") {
            start = line[line.find(':')? + 1..].trim().parse().unwrap_or(0.0);
        }
    }
    Some((url, start))
}

/// Generates a uniformly random four-digit PIN.
/// Generates a uniformly random four-digit PIN.
#[must_use]
pub fn generate_pin() -> String {
    use rand::Rng;
    let value: u32 = rand::rngs::OsRng.gen_range(0..PIN_SPACE);
    format!("{value:04}")
}

/// The `Ended` event of a session, used by the engine.
/// The `Ended` event of a session, used by the engine.
#[must_use]
pub fn ended_event(session: &AirPlaySession) -> SessionEvent {
    SessionEvent::Ended {
        session: session.id,
        reason: phairplay_core::session::SessionEndReason::SenderDisconnected,
    }
}

mod handshake;
mod playback;

use handshake::{
    fp_setup, pair_pin_start, pair_setup, pair_setup_pin, pair_verify,
};
pub use handshake::{mirror_setup, teardown_stream_types};
use playback::{play, rate, scrub_post, stop};

#[cfg(test)]
mod tests;
