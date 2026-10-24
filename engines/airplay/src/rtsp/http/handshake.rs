//! The handshake routes of the AirPlay control channel.

use crate::error::AirPlayError;
use crate::plist::{PlistValue, binary};
use crate::rtsp::request::RtspRequest;
use crate::rtsp::response::RtspResponse;
use crate::rtsp::router::RouteContext;
use crate::session::{
    AirPlaySession, STREAM_TYPE_AUDIO, STREAM_TYPE_BUFFERED_AUDIO, STREAM_TYPE_VIDEO,
};

use super::{BINARY_PLIST, OCTET_STREAM, TIMING_PORT, generate_pin};

/// The `POST /pair-setup` answer: the public key of the receiver.
pub(super) fn pair_setup(request: &RtspRequest, session: &mut AirPlaySession) -> RtspResponse {
    match session.pairing.pair_setup(&request.body) {
        Ok(key) => RtspResponse::ok()
            .with_protocol(request.response_protocol())
            .with_typed_body(OCTET_STREAM, key),
        Err(error) => {
            tracing::warn!(%error, "pair-setup failed");
            RtspResponse::bad_request().with_protocol(request.response_protocol())
        }
    }
}

/// The `POST /pair-verify` answer: the second message of the exchange.
pub(super) fn pair_verify(request: &RtspRequest, session: &mut AirPlaySession) -> RtspResponse {
    match session.pairing.pair_verify(&request.body) {
        Ok(body) => {
            session.shared_secret = session.pairing.shared_secret();
            session.authorized = session.pairing.is_verified();
            RtspResponse::ok()
                .with_protocol(request.response_protocol())
                .with_typed_body(OCTET_STREAM, body)
        }
        Err(error) => {
            tracing::warn!(%error, "pair-verify failed");
            RtspResponse::authorization_required().with_protocol(request.response_protocol())
        }
    }
}

/// The `POST /pair-pin-start` answer: the receiver shows a PIN.
pub(super) fn pair_pin_start(
    request: &RtspRequest,
    session: &mut AirPlaySession,
    context: &RouteContext<'_>,
) -> RtspResponse {
    if !context.require_pin {
        return RtspResponse::not_implemented().with_protocol(request.response_protocol());
    }
    if let Some(store) = context.pairing_store {
        if store.failed_attempts().unwrap_or(0) >= context.limits.max_pairing_attempts {
            return RtspResponse::authorization_required()
                .with_protocol(request.response_protocol());
        }
    }
    let pin = generate_pin();
    session.shown_pin = Some(pin.clone());
    session.legacy_pin = Some(crate::pairing::legacy::LegacyPinPairing::new(
        &pin,
        session.pairing.public_key(),
    ));
    RtspResponse::ok().with_protocol(request.response_protocol())
}

/// The `POST /pair-setup-pin` answer: one step of the SRP exchange.
pub(super) fn pair_setup_pin(
    request: &RtspRequest,
    session: &mut AirPlaySession,
    context: &RouteContext<'_>,
) -> RtspResponse {
    if !context.require_pin {
        return RtspResponse::not_implemented().with_protocol(request.response_protocol());
    }
    if let Some(store) = context.pairing_store {
        if store.failed_attempts().unwrap_or(0) >= context.limits.max_pairing_attempts {
            session.shown_pin = None;
            session.legacy_pin = None;
            return RtspResponse::authorization_required()
                .with_protocol(request.response_protocol());
        }
    }
    let document = match crate::plist::decode(&request.body, context.limits.max_message_bytes) {
        Ok(document) => document,
        Err(error) => {
            tracing::warn!(%error, "pair-setup-pin with an unusable body");
            return RtspResponse::bad_request().with_protocol(request.response_protocol());
        }
    };
    if session.legacy_pin.is_none() {
        // The sender may start the exchange without a prior PIN request.
        let pin = generate_pin();
        session.shown_pin = Some(pin.clone());
        session.legacy_pin = Some(crate::pairing::legacy::LegacyPinPairing::new(
            &pin,
            session.pairing.public_key(),
        ));
    }
    let Some(pin) = session.legacy_pin.as_mut() else {
        return RtspResponse::bad_request().with_protocol(request.response_protocol());
    };
    match pin.handle(&document) {
        Ok(reply) => {
            if reply.complete {
                session.pin_paired = true;
                session.shown_pin = None;
                session.legacy_pin = None;
                if let Some(store) = context.pairing_store {
                    let _ = store.reset_failed_attempts();
                }
            }
            let document = PlistValue::Dictionary(reply.values);
            let body = binary::encode(&document).unwrap_or_default();
            RtspResponse::ok()
                .with_protocol(request.response_protocol())
                .with_typed_body(BINARY_PLIST, body)
        }
        Err(error) => {
            tracing::warn!(%error, "pair-setup-pin failed");
            if let Some(store) = context.pairing_store {
                let _ = store.record_failed_attempt();
            }
            session.shown_pin = None;
            session.legacy_pin = None;
            RtspResponse::authorization_required().with_protocol(request.response_protocol())
        }
    }
}

/// The `POST /fp-setup` answer: one phase of the FairPlay handshake.
pub(super) fn fp_setup(request: &RtspRequest, session: &mut AirPlaySession) -> RtspResponse {
    let result = match request.body.len() {
        crate::fairplay::SETUP_REQUEST_BYTES => session.fairplay.setup(&request.body),
        crate::fairplay::HANDSHAKE_REQUEST_BYTES => session.fairplay.handshake(&request.body),
        _ => Err(AirPlayError::parse(format!(
            "unexpected fp-setup size {}",
            request.body.len()
        ))),
    };
    match result {
        Ok(body) => RtspResponse::ok()
            .with_protocol(request.response_protocol())
            .with_typed_body(OCTET_STREAM, body),
        Err(error) => {
            tracing::warn!(%error, "fp-setup failed");
            RtspResponse::bad_request().with_protocol(request.response_protocol())
        }
    }
}

/// The mirroring `SETUP` answer: the ports of the mirrored streams.
pub fn mirror_setup(
    request: &RtspRequest,
    session: &mut AirPlaySession,
    context: &RouteContext<'_>,
) -> RtspResponse {
    let document = match crate::plist::decode(&request.body, context.limits.max_message_bytes) {
        Ok(document) => document,
        Err(error) => {
            tracing::warn!(%error, "mirror SETUP with an unusable body");
            return RtspResponse::bad_request().with_protocol(request.response_protocol());
        }
    };
    session.mirror = true;
    let mut response: Vec<(String, PlistValue)> = Vec::new();

    if let Some(wrapped) = document.get("ekey").and_then(PlistValue::as_data) {
        match session.fairplay.decrypt(wrapped) {
            Ok(_key) => {
                response.push(("ekey".into(), PlistValue::Data(Vec::new())));
            }
            Err(AirPlayError::Unsupported { reason }) => {
                // The honest answer: the key stays wrapped, the stream stays
                // encrypted, and the reason is reported.
                tracing::warn!(%reason, "the mirrored stream stays encrypted");
                response.push(("ekey".into(), PlistValue::Data(Vec::new())));
            }
            Err(error) => {
                tracing::warn!(%error, "the FairPlay key could not be unwrapped");
                return RtspResponse::bad_request().with_protocol(request.response_protocol());
            }
        }
    }
    response.push(("eventPort".into(), PlistValue::Integer(0)));
    response.push((
        "timingPort".into(),
        PlistValue::Integer(i64::from(TIMING_PORT)),
    ));

    if let Some(streams) = document.get("streams").and_then(PlistValue::as_array) {
        let mut answers = Vec::new();
        for stream in streams {
            let Some(stream_type) = stream.get("type").and_then(PlistValue::as_integer) else {
                continue;
            };
            let stream_type = u8::try_from(stream_type).unwrap_or(0);
            if !matches!(
                stream_type,
                STREAM_TYPE_AUDIO | STREAM_TYPE_BUFFERED_AUDIO | STREAM_TYPE_VIDEO
            ) {
                continue;
            }
            let Some(port) = context.stream_ports.allocate(stream_type) else {
                continue;
            };
            session.add_stream_type(stream_type);
            answers.push(PlistValue::dictionary(vec![
                ("type".into(), PlistValue::Integer(i64::from(stream_type))),
                ("dataPort".into(), PlistValue::Integer(i64::from(port))),
            ]));
        }
        response.push(("streams".into(), PlistValue::Array(answers)));
    }

    let body = binary::encode(&PlistValue::Dictionary(response)).unwrap_or_default();
    RtspResponse::ok()
        .with_protocol(request.response_protocol())
        .with_typed_body(BINARY_PLIST, body)
}

/// The stream types a `TEARDOWN` names.
pub fn teardown_stream_types(request: &RtspRequest) -> Option<Vec<u8>> {
    if request.body.is_empty() {
        return None;
    }
    let document = crate::plist::decode(&request.body, 1024 * 1024).ok()?;
    let streams = document.get("streams").and_then(PlistValue::as_array)?;
    let types: Vec<u8> = streams
        .iter()
        .filter_map(|stream| stream.get("type").and_then(PlistValue::as_integer))
        .filter_map(|value| u8::try_from(value).ok())
        .collect();
    if types.is_empty() { None } else { Some(types) }
}
