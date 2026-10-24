//! The router of the AirPlay control channel.
//!
//! WHY: an AirPlay sender mixes two dialects on one socket — the RTSP verbs
//! and the HTTP-like handshake routes — and it decides which one to use from
//! the *first* request it sends. The router therefore dispatches on the
//! method first and on the path second, and every route answers with a typed
//! response instead of writing to a socket.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_airplay::rtsp::RtspRequest;
//! use phairplay_airplay::rtsp::router::{RouteContext, route};
//! use phairplay_airplay::session::AirPlaySession;
//! use phairplay_airplay::limits::AirPlayLimits;
//! use phairplay_core::identity::DeviceIdentity;
//! use phairplay_core::capabilities::MediaCapabilities;
//!
//! let request = RtspRequest::parse(
//!     b"OPTIONS rtsp://10.0.0.1/live RTSP/1.0\r\nCSeq: 1\r\n\r\n",
//!     4096,
//! )
//! .expect("valid request");
//! let mut session = AirPlaySession::new();
//! let identity = DeviceIdentity::default();
//! let capabilities = MediaCapabilities::empty();
//! let limits = AirPlayLimits::default();
//! let context = RouteContext::new(&identity, &capabilities, &limits);
//! let response = route(&request, &mut session, &context);
//! assert_eq!(response.status, 200);
//! ```

use phairplay_core::capabilities::MediaCapabilities;
use phairplay_core::identity::DeviceIdentity;
use phairplay_core::session::SessionEvent;
use tokio::sync::mpsc::Sender;

use crate::error::AirPlayError;
use crate::limits::AirPlayLimits;
use crate::photo::PhotoPayload;
use crate::rtsp::http;
use crate::rtsp::request::{RtspMethod, RtspRequest};
use crate::rtsp::response::RtspResponse;
use crate::session::AirPlaySession;
use std::sync::{Arc, Mutex};

/// The slot the received photos are published in.
pub type PhotoSlot = Arc<Mutex<Option<PhotoPayload>>>;

/// The allocator of the data ports of the mirrored streams.
pub trait StreamPortAllocator: Send + Sync + std::fmt::Debug {
    /// Allocates a port for a stream of the given type.
    fn allocate(&self, stream_type: u8) -> Option<u16>;
}

/// An allocator that never allocates a port.
///
/// Used when the engine runs without the mirroring data channels.
#[derive(Debug, Default)]
pub struct NoStreamPorts;

impl StreamPortAllocator for NoStreamPorts {
    fn allocate(&self, _stream_type: u8) -> Option<u16> {
        None
    }
}

/// The context a request is routed in.
#[derive(Debug)]
pub struct RouteContext<'a> {
    /// The identity the receiver advertises.
    pub identity: &'a DeviceIdentity,
    /// The capabilities the receiver advertises.
    pub capabilities: &'a MediaCapabilities,
    /// The limits the engine enforces.
    pub limits: &'a AirPlayLimits,
    /// Whether a PIN is required before the sender may stream.
    pub require_pin: bool,
    /// The channel the session events are reported on.
    pub events: Option<&'a Sender<SessionEvent>>,
    /// The slot the received photos are published in.
    pub photos: Option<&'a PhotoSlot>,
    /// The store of the paired controllers.
    pub pairing_store: Option<&'a dyn crate::pairing::store::PairingStore>,
    /// The allocator of the data ports of the mirrored streams.
    pub stream_ports: &'a dyn StreamPortAllocator,
    /// The Ed25519 public key of the receiver, announced in `GET /info`.
    pub pairing_public_key: Option<[u8; 32]>,
}

impl<'a> RouteContext<'a> {
    /// Creates a context without the optional parts.
    #[must_use]
    pub fn new(
        identity: &'a DeviceIdentity,
        capabilities: &'a MediaCapabilities,
        limits: &'a AirPlayLimits,
    ) -> Self {
        Self {
            identity,
            capabilities,
            limits,
            require_pin: false,
            events: None,
            photos: None,
            pairing_store: None,
            stream_ports: &NoStreamPorts,
            pairing_public_key: None,
        }
    }

    /// Reports a session event, ignoring a closed channel.
    pub fn report(&self, event: SessionEvent) {
        if let Some(events) = self.events {
            let _ = events.try_send(event);
        }
    }
}

/// Routes one request.
///
/// # Errors
/// The router never fails: a request it cannot handle is answered with the
/// matching error status.
pub fn route(
    request: &RtspRequest,
    session: &mut AirPlaySession,
    context: &RouteContext<'_>,
) -> RtspResponse {
    let protocol = request.response_protocol();
    match request.method {
        RtspMethod::Options => options(protocol),
        RtspMethod::Announce => announce(request, session),
        RtspMethod::Setup => setup(request, session, context),
        RtspMethod::Record => record(request, session, context),
        RtspMethod::Teardown => teardown(request, session, context),
        RtspMethod::GetParameter => get_parameter(request, session),
        RtspMethod::SetParameter => set_parameter(request, session, context),
        RtspMethod::Flush => ok(protocol),
        RtspMethod::Pause => ok(protocol),
        RtspMethod::Get => http::route_get(request, session, context),
        RtspMethod::Post => http::route_post(request, session, context),
        RtspMethod::Put => http::route_put(request, session, context),
        RtspMethod::Delete => http::route_delete(request, session, context),
        RtspMethod::SetRateAnchorTime | RtspMethod::SetPeers | RtspMethod::FlushBuffered => {
            ok(protocol)
        }
        RtspMethod::Other(_) => RtspResponse::not_implemented().with_protocol(protocol),
    }
}

/// A plain `200 OK` answer.
fn ok(protocol: &str) -> RtspResponse {
    RtspResponse::ok().with_protocol(protocol)
}

/// The `OPTIONS` answer: the verbs the receiver supports.
fn options(protocol: &str) -> RtspResponse {
    RtspResponse::ok().with_protocol(protocol).with_header(
        "Public",
        "ANNOUNCE, SETUP, RECORD, PAUSE, FLUSH, TEARDOWN, OPTIONS, GET_PARAMETER, SET_PARAMETER",
    )
}

/// The `ANNOUNCE` answer: the session description of the sender.
fn announce(request: &RtspRequest, session: &mut AirPlaySession) -> RtspResponse {
    let Some(body) = request.body_text() else {
        return RtspResponse::bad_request().with_protocol(request.response_protocol());
    };
    let limits = AirPlayLimits::default();
    match crate::sdp::parse_sdp(&body, &limits) {
        Ok(description) => {
            session.sender_name = request
                .header("User-Agent")
                .map(|value| value.split('/').next().unwrap_or(value).trim().to_string())
                .filter(|name| !name.is_empty());
            session.announced = Some(description);
            session.setup_count = 0;
            ok(request.response_protocol())
        }
        Err(error) => {
            tracing::warn!(%error, "ANNOUNCE with an unusable session description");
            RtspResponse::bad_request().with_protocol(request.response_protocol())
        }
    }
}

/// The `SETUP` answer: the transport of the negotiated stream.
fn setup(
    request: &RtspRequest,
    session: &mut AirPlaySession,
    context: &RouteContext<'_>,
) -> RtspResponse {
    if request.is_binary_plist() {
        return http::mirror_setup(request, session, context);
    }
    session.setup_count += 1;
    let transport = if session.setup_count == 1 && session.has_video() {
        "RTP/AVP/TCP;unicast;interleaved=0-1".to_string()
    } else {
        "RTP/AVP/UDP;unicast;client_port=6001-6002;server_port=6001-6002;timing-port=6002"
            .to_string()
    };
    ok(request.response_protocol())
        .with_header("Session", "PhairPlaySession")
        .with_header("Transport", transport)
}

/// The `RECORD` answer: the sender starts the media.
fn record(
    request: &RtspRequest,
    session: &mut AirPlaySession,
    context: &RouteContext<'_>,
) -> RtspResponse {
    if session.mirror {
        session.streaming = true;
        context.report(started_event(session));
        return ok(request.response_protocol()).with_header("Audio-Latency", "0");
    }
    if session.announced.is_none() {
        return RtspResponse::method_not_valid().with_protocol(request.response_protocol());
    }
    if context.require_pin && !session.pin_paired && !session.authorized {
        return RtspResponse::authorization_required().with_protocol(request.response_protocol());
    }
    session.streaming = true;
    session.authorized = true;
    context.report(started_event(session));
    ok(request.response_protocol())
}

/// The `TEARDOWN` answer: the sender releases a stream or the session.
fn teardown(
    request: &RtspRequest,
    session: &mut AirPlaySession,
    context: &RouteContext<'_>,
) -> RtspResponse {
    let types = http::teardown_stream_types(request);
    if let Some(types) = types {
        for stream_type in types {
            session.remove_stream_type(stream_type);
        }
        if !session.has_no_active_streams() {
            return ok(request.response_protocol());
        }
    }
    session.active_stream_types.clear();
    session.streaming = false;
    context.report(ended_event(session));
    ok(request.response_protocol())
}

/// The `GET_PARAMETER` answer: the state the sender asks for.
fn get_parameter(request: &RtspRequest, session: &AirPlaySession) -> RtspResponse {
    let query = request.body_text().unwrap_or_default();
    if query.trim_start().starts_with("volume") {
        return RtspResponse::ok()
            .with_protocol(request.response_protocol())
            .with_typed_body(
                "text/parameters",
                format!("volume: {:.6}\r\n", session.volume).into_bytes(),
            );
    }
    ok(request.response_protocol())
}

/// The `SET_PARAMETER` answer: the state the sender pushes.
fn set_parameter(
    request: &RtspRequest,
    session: &mut AirPlaySession,
    context: &RouteContext<'_>,
) -> RtspResponse {
    let body = request.body_text().unwrap_or_default();
    let content_type = request.content_type().unwrap_or_default();
    if body.trim_start().starts_with("volume") {
        if let Some(value) = body
            .split_once(':')
            .and_then(|(_, rest)| rest.trim().parse::<f32>().ok())
        {
            session.volume = value;
        }
        return ok(request.response_protocol());
    }
    if content_type.starts_with("image/") {
        if request.body.len() > context.limits.max_artwork_bytes {
            return RtspResponse::bad_request().with_protocol(request.response_protocol());
        }
        context.report(SessionEvent::ArtworkChanged {
            session: session.id,
            artwork: Some(request.body.clone()),
        });
        return ok(request.response_protocol());
    }
    if content_type.contains("dmap") || request.body.starts_with(b"mlit") {
        match crate::dmap::parse_now_playing(&request.body) {
            Ok(metadata) => {
                context.report(SessionEvent::MetadataChanged {
                    session: session.id,
                    title: metadata.title,
                    artist: metadata.artist,
                    album: metadata.album,
                });
            }
            Err(error) => {
                tracing::warn!(%error, "SET_PARAMETER with an unusable DMAP body");
            }
        }
        return ok(request.response_protocol());
    }
    ok(request.response_protocol())
}

/// The `Started` event of a session.
fn started_event(session: &AirPlaySession) -> SessionEvent {
    SessionEvent::Started {
        session: phairplay_core::session::SessionSnapshot {
            id: session.id,
            protocol: phairplay_core::protocol::ProtocolKind::AirPlay,
            sender_name: Some(session.sender_name().to_string()),
            sender_address: None,
            video: session.video.clone(),
            audio: session.audio.clone(),
            started_at: std::time::Instant::now(),
        },
    }
}

/// The `Ended` event of a session.
fn ended_event(session: &AirPlaySession) -> SessionEvent {
    SessionEvent::Ended {
        session: session.id,
        reason: phairplay_core::session::SessionEndReason::SenderDisconnected,
    }
}

/// The error answer of a failed request.
#[must_use]
pub fn error_response(error: &AirPlayError, protocol: &str) -> RtspResponse {
    match error {
        AirPlayError::LimitExceeded { .. } => {
            RtspResponse::new(413, "Payload Too Large").with_protocol(protocol)
        }
        AirPlayError::Unsupported { .. } => {
            RtspResponse::new(501, "Not Implemented").with_protocol(protocol)
        }
        _ => RtspResponse::bad_request().with_protocol(protocol),
    }
}

#[cfg(test)]
mod tests;
