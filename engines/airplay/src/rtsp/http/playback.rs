//! The URL playback routes of the AirPlay control channel.

use crate::rtsp::request::RtspRequest;
use crate::rtsp::response::RtspResponse;
use crate::session::AirPlaySession;

use super::{parse_play_request, parse_text_play_request};

pub(super) fn play(request: &RtspRequest, session: &mut AirPlaySession) -> RtspResponse {
    let document = if request.is_binary_plist() {
        crate::plist::decode(&request.body, 1024 * 1024).ok()
    } else {
        None
    };
    let (url, start) = match document {
        Some(document) => parse_play_request(&document, request.body_text().as_deref()),
        None => parse_text_play_request(request.body_text().as_deref()).unwrap_or((None, 0.0)),
    };
    let Some(url) = url else {
        return RtspResponse::bad_request().with_protocol(request.response_protocol());
    };
    session.playback.url = Some(url);
    session.playback.start_fraction = start;
    session.playback.ready = true;
    RtspResponse::ok().with_protocol(request.response_protocol())
}

/// The `POST /rate` answer: the playback rate.
pub(super) fn rate(request: &RtspRequest, session: &mut AirPlaySession) -> RtspResponse {
    let value = request
        .query_param("value")
        .and_then(|value| value.parse::<f32>().ok())
        .unwrap_or(1.0);
    session.playback.rate = value;
    RtspResponse::ok().with_protocol(request.response_protocol())
}

/// The `POST /scrub` answer: the position the sender seeks to.
pub(super) fn scrub_post(request: &RtspRequest, session: &mut AirPlaySession) -> RtspResponse {
    if let Some(position) = request
        .query_param("position")
        .and_then(|value| value.parse::<f64>().ok())
    {
        session.playback.position_sec = position;
    }
    RtspResponse::ok().with_protocol(request.response_protocol())
}

/// The `POST /stop` answer: the playback stops.
pub(super) fn stop(request: &RtspRequest, session: &mut AirPlaySession) -> RtspResponse {
    session.playback = crate::session::PlaybackState::default();
    RtspResponse::ok().with_protocol(request.response_protocol())
}
