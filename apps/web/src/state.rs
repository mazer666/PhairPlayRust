//! The state of the dashboard.
//!
//! WHAT: The one object that the handlers of the API share: the receiver
//! that is being watched and the channel that carries the live events to
//! the WebSocket clients.
//!
//! WHY: The dashboard shows the receiver, not a copy of it. Every answer
//! of the API is read from the supervisor of the bridge, which in turn
//! reads it from the session registry and the counters of the core. The
//! API therefore cannot show a session that does not exist or a counter
//! that was never counted.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_core::config::AppConfig;
//! use phairplay_core::protocol::ProtocolKind;
//! use phairplay_web::state::AppState;
//!
//! let receiver = phairplay_ffi::ReceiverCore::new(AppConfig::default())
//!     .unwrap_or_else(|error| panic!("the receiver must start: {error}"));
//! let state = AppState::new(receiver);
//! assert_eq!(state.protocol_statuses().len(), ProtocolKind::ALL.len());
//! ```

use std::sync::Arc;

use phairplay_core::config::AppConfig;
use phairplay_core::protocol::ProtocolKind;
use phairplay_ffi::dto::FfiReceiverEvent;
use phairplay_ffi::listener::EventListener;
use phairplay_ffi::supervisor::ReceiverCore;
use tokio::sync::broadcast;

use crate::dto::{ProtocolStatusDto, SessionDto, WebEvent};

/// The number of events that may wait in the channel before the oldest one
/// is dropped.
///
/// WHY: A dashboard that is not open must not make the receiver grow a
/// queue. The channel is bounded, and a client that is too slow misses the
/// oldest events instead of stopping the receiver.
const EVENT_CHANNEL_CAPACITY: usize = 128;

/// The state that the handlers of the API share.
#[derive(Debug, Clone)]
pub struct AppState {
    receiver: ReceiverCore,
    events: broadcast::Sender<WebEvent>,
}

impl AppState {
    /// Creates the state for a receiver and starts forwarding its events.
    #[must_use]
    pub fn new(receiver: ReceiverCore) -> Self {
        let (events, _) = broadcast::channel(EVENT_CHANNEL_CAPACITY);
        let state = AppState {
            receiver,
            events: events.clone(),
        };
        state
            .receiver
            .set_listener(Arc::new(BroadcastListener { sender: events }));
        state
    }

    /// Creates the state for a receiver with the default configuration.
    ///
    /// # Errors
    /// Returns the failure of the bridge when the receiver cannot start.
    pub fn with_default_configuration() -> Result<Self, phairplay_ffi::error::FfiError> {
        Ok(AppState::new(ReceiverCore::new(AppConfig::default())?))
    }

    /// The receiver that the dashboard watches.
    #[must_use]
    pub fn receiver(&self) -> &ReceiverCore {
        &self.receiver
    }

    /// Subscribes to the live events.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<WebEvent> {
        self.events.subscribe()
    }

    /// The status of every protocol.
    #[must_use]
    pub fn protocol_statuses(&self) -> Vec<ProtocolStatusDto> {
        self.receiver
            .status()
            .iter()
            .map(ProtocolStatusDto::new)
            .collect()
    }

    /// The live sessions.
    #[must_use]
    pub fn sessions(&self) -> Vec<SessionDto> {
        self.receiver
            .sessions()
            .iter()
            .map(SessionDto::new)
            .collect()
    }

    /// The name of a protocol, or the failure of an unknown name.
    ///
    /// # Errors
    /// Returns [`crate::error::ApiError`] when the name is not a protocol.
    pub fn protocol(&self, name: &str) -> Result<ProtocolKind, crate::error::ApiError> {
        name.parse::<ProtocolKind>()
            .map_err(|_| crate::error::ApiError::unknown_protocol(name))
    }
}

/// The listener that hands the events of the receiver to the dashboard.
#[derive(Debug)]
struct BroadcastListener {
    sender: broadcast::Sender<WebEvent>,
}

impl EventListener for BroadcastListener {
    fn on_event(&self, event: &FfiReceiverEvent) {
        // WHY: a client that is too slow must not stop the receiver. The
        // channel is bounded and the error of a full channel is ignored on
        // purpose: the next event carries the current state anyway.
        let _ = self.sender.send(convert_event(event));
    }
}

/// Converts an event of the bridge into an event of the dashboard.
#[must_use]
pub fn convert_event(event: &FfiReceiverEvent) -> WebEvent {
    match event {
        FfiReceiverEvent::SessionStarted { session } => WebEvent::SessionStarted {
            session: SessionDto {
                id: session.id,
                protocol: session.protocol.as_str().to_string(),
                sender_name: session.sender_name.clone(),
                sender_address: session.sender_address.clone(),
                video: session.video.as_ref().map(|format| {
                    format!(
                        "{} {}x{}",
                        format.codec.mime_type(),
                        format.width,
                        format.height
                    )
                }),
                audio: session.audio.as_ref().map(|format| {
                    format!(
                        "{} {} Hz {} ch",
                        format.codec.mime_type(),
                        format.sample_rate,
                        format.channels
                    )
                }),
                uptime_ms: session.uptime_ms,
            },
        },
        FfiReceiverEvent::SessionEnded { session_id, reason } => WebEvent::SessionEnded {
            session_id: *session_id,
            reason: reason.clone(),
        },
        FfiReceiverEvent::VideoFormatChanged { session_id, format } => {
            WebEvent::VideoFormatChanged {
                session_id: *session_id,
                format: format!(
                    "{} {}x{}",
                    format.codec.mime_type(),
                    format.width,
                    format.height
                ),
            }
        }
        FfiReceiverEvent::AudioFormatChanged { session_id, format } => {
            WebEvent::AudioFormatChanged {
                session_id: *session_id,
                format: format!(
                    "{} {} Hz {} ch",
                    format.codec.mime_type(),
                    format.sample_rate,
                    format.channels
                ),
            }
        }
        FfiReceiverEvent::PlaybackStateChanged { session_id, state } => {
            WebEvent::PlaybackStateChanged {
                session_id: *session_id,
                state: state.clone(),
            }
        }
        FfiReceiverEvent::MetadataChanged {
            session_id,
            title,
            artist,
            album,
        } => WebEvent::MetadataChanged {
            session_id: *session_id,
            title: title.clone(),
            artist: artist.clone(),
            album: album.clone(),
        },
        FfiReceiverEvent::ArtworkChanged {
            session_id,
            artwork,
        } => WebEvent::ArtworkChanged {
            session_id: *session_id,
            artwork_bytes: artwork.as_ref().map_or(0, Vec::len),
        },
        FfiReceiverEvent::ProtocolStatusChanged { status } => WebEvent::ProtocolStatusChanged {
            status: ProtocolStatusDto {
                protocol: status.protocol.as_str().to_string(),
                display_name: status.protocol.display_name().to_string(),
                state: status.state.into(),
                enabled: status.enabled,
                tcp_port: status.tcp_port,
                udp_port: status.udp_port,
                peer_count: status.peer_count,
                last_error: status.last_error.clone(),
            },
        },
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;
    use phairplay_ffi::dto::{FfiProtocolKind, FfiProtocolState, FfiProtocolStatus};

    use crate::test_support::MustErr;

    #[test]
    fn the_state_covers_every_protocol() {
        let state = AppState::with_default_configuration()
            .unwrap_or_else(|error| panic!("the state must start: {error}"));
        let statuses = state.protocol_statuses();
        // WHY: the number of protocols is the one of the core. A hard
        // number in the test breaks the day the core learns a protocol.
        assert_eq!(statuses.len(), ProtocolKind::ALL.len());
        assert_eq!(statuses[0].protocol, "airplay");
        assert_eq!(statuses[3].protocol, "dlna");
    }

    #[test]
    fn an_unknown_protocol_is_refused() {
        let state = AppState::with_default_configuration()
            .unwrap_or_else(|error| panic!("the state must start: {error}"));
        let error = state
            .protocol("telepathy")
            .must_err("the protocol must be refused");
        assert_eq!(error.status.as_u16(), 404);
    }

    #[test]
    fn a_status_event_becomes_a_dashboard_event() {
        let event = FfiReceiverEvent::ProtocolStatusChanged {
            status: FfiProtocolStatus {
                protocol: FfiProtocolKind::Cast,
                state: FfiProtocolState::Advertising,
                enabled: true,
                tcp_port: Some(8009),
                udp_port: None,
                peer_count: 0,
                last_error: None,
            },
        };
        match convert_event(&event) {
            WebEvent::ProtocolStatusChanged { status } => {
                assert_eq!(status.protocol, "cast");
                assert_eq!(status.state, crate::dto::ProtocolStateDto::Advertising);
                assert_eq!(status.tcp_port, Some(8009));
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }
}
