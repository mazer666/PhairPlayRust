//! The session snapshot and the live events as they travel over the bridge.
//!
//! WHAT: A snapshot of one running session and the event that describes
//! something that happened during a session or to a protocol engine.
//!
//! WHY: The engines report what happens on the wire; the core interprets it;
//! the shells display it. The event is the only channel between these three
//! worlds, so its shape decides what a shell can show. Everything the shell
//! needs to render a session has to be in it — otherwise every host would
//! have to ask for more.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_ffi::dto::FfiReceiverEvent;
//! use phairplay_core::session::SessionEndReason;
//!
//! let event = FfiReceiverEvent::SessionEnded {
//!     session_id: 7,
//!     reason: SessionEndReason::SenderDisconnected.to_string(),
//! };
//!
//! assert!(matches!(event, FfiReceiverEvent::SessionEnded { .. }));
//! ```

use phairplay_core::session::{SessionEvent, SessionId, SessionSnapshot};

use super::media::{FfiAudioFormat, FfiVideoFormat};
use super::protocol::{FfiProtocolKind, FfiProtocolStatus};

/// A snapshot of one live session.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct FfiSessionSnapshot {
    /// The identifier of the session.
    pub id: u64,
    /// Which protocol carries this session.
    pub protocol: FfiProtocolKind,
    /// The display name of the sending device, if it announced one.
    pub sender_name: Option<String>,
    /// The address of the sending device.
    pub sender_address: Option<String>,
    /// The negotiated video format, if the session carries video.
    pub video: Option<FfiVideoFormat>,
    /// The negotiated audio format, if the session carries audio.
    pub audio: Option<FfiAudioFormat>,
    /// How long the session has been running, in milliseconds.
    ///
    /// WHY: The core keeps the start time as a monotonic instant, which has
    /// no calendar representation. The age of the session is what a shell
    /// actually shows, and it is free of the clock jumps of the system.
    pub uptime_ms: u64,
}

impl From<&SessionSnapshot> for FfiSessionSnapshot {
    fn from(session: &SessionSnapshot) -> Self {
        FfiSessionSnapshot {
            id: session.id.0,
            protocol: session.protocol.into(),
            sender_name: session.sender_name.clone(),
            sender_address: session.sender_address.clone(),
            video: session.video.as_ref().map(FfiVideoFormat::from),
            audio: session.audio.as_ref().map(FfiAudioFormat::from),
            uptime_ms: session.started_at.elapsed().as_millis() as u64,
        }
    }
}

impl From<SessionSnapshot> for FfiSessionSnapshot {
    fn from(session: SessionSnapshot) -> Self {
        FfiSessionSnapshot::from(&session)
    }
}

/// Something that happened in the receiver.
///
/// WHY: The dashboard shows a live list of sessions; the native shells show
/// a notification when a sender appears. Both need the same information, so
/// the event carries the whole picture instead of a name and a payload that
/// every host would have to interpret on its own.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum FfiReceiverEvent {
    /// A peer connected and a session began.
    SessionStarted {
        /// The new session.
        session: FfiSessionSnapshot,
    },
    /// The negotiated video format changed during the session.
    VideoFormatChanged {
        /// The session the format belongs to.
        session_id: u64,
        /// The new format.
        format: FfiVideoFormat,
    },
    /// The negotiated audio format changed during the session.
    AudioFormatChanged {
        /// The session the format belongs to.
        session_id: u64,
        /// The new format.
        format: FfiAudioFormat,
    },
    /// The playback state of the sender changed.
    PlaybackStateChanged {
        /// The session the change belongs to.
        session_id: u64,
        /// The new state, as the sender described it.
        state: String,
    },
    /// The now-playing metadata changed.
    MetadataChanged {
        /// The session the metadata belongs to.
        session_id: u64,
        /// The title of the track.
        title: Option<String>,
        /// The artist of the track.
        artist: Option<String>,
        /// The album of the track.
        album: Option<String>,
    },
    /// The artwork changed. `None` means the artwork was cleared.
    ArtworkChanged {
        /// The session the artwork belongs to.
        session_id: u64,
        /// The artwork bytes, if there are any.
        artwork: Option<Vec<u8>>,
    },
    /// The session ended.
    SessionEnded {
        /// The session that ended.
        session_id: u64,
        /// Why it ended.
        reason: String,
    },
    /// The status of a protocol engine changed.
    ProtocolStatusChanged {
        /// The new status of the protocol.
        status: FfiProtocolStatus,
    },
}

impl FfiReceiverEvent {
    /// The session the event belongs to, if it belongs to one.
    ///
    /// WHY: The web dashboard groups the events by session. Deciding that in
    /// the bridge keeps the rule in one place.
    #[must_use]
    pub const fn session_id(&self) -> Option<u64> {
        match self {
            FfiReceiverEvent::SessionStarted { session } => Some(session.id),
            FfiReceiverEvent::VideoFormatChanged { session_id, .. }
            | FfiReceiverEvent::AudioFormatChanged { session_id, .. }
            | FfiReceiverEvent::PlaybackStateChanged { session_id, .. }
            | FfiReceiverEvent::MetadataChanged { session_id, .. }
            | FfiReceiverEvent::ArtworkChanged { session_id, .. }
            | FfiReceiverEvent::SessionEnded { session_id, .. } => Some(*session_id),
            FfiReceiverEvent::ProtocolStatusChanged { .. } => None,
        }
    }
}

impl From<&SessionEvent> for FfiReceiverEvent {
    fn from(event: &SessionEvent) -> Self {
        match event {
            SessionEvent::Started { session } => FfiReceiverEvent::SessionStarted {
                session: FfiSessionSnapshot::from(session),
            },
            SessionEvent::VideoFormatChanged { session, format } => {
                FfiReceiverEvent::VideoFormatChanged {
                    session_id: session.0,
                    format: FfiVideoFormat::from(format),
                }
            }
            SessionEvent::AudioFormatChanged { session, format } => {
                FfiReceiverEvent::AudioFormatChanged {
                    session_id: session.0,
                    format: FfiAudioFormat::from(format),
                }
            }
            SessionEvent::PlaybackStateChanged { session, state } => {
                FfiReceiverEvent::PlaybackStateChanged {
                    session_id: session.0,
                    state: state.clone(),
                }
            }
            SessionEvent::MetadataChanged {
                session,
                title,
                artist,
                album,
            } => FfiReceiverEvent::MetadataChanged {
                session_id: session.0,
                title: title.clone(),
                artist: artist.clone(),
                album: album.clone(),
            },
            SessionEvent::ArtworkChanged { session, artwork } => FfiReceiverEvent::ArtworkChanged {
                session_id: session.0,
                artwork: artwork.clone(),
            },
            SessionEvent::Ended { session, reason } => FfiReceiverEvent::SessionEnded {
                session_id: session.0,
                reason: reason.to_string(),
            },
        }
    }
}

impl From<SessionEvent> for FfiReceiverEvent {
    fn from(event: SessionEvent) -> Self {
        FfiReceiverEvent::from(&event)
    }
}

/// The identifier of a session as it is used in the bridge.
#[must_use]
pub const fn session_id_value(id: &SessionId) -> u64 {
    id.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::media::FfiAudioCodec;
    use phairplay_core::capabilities::AudioCodec;
    use phairplay_core::media::AudioFormat;
    use phairplay_core::protocol::ProtocolKind;
    use phairplay_core::session::SessionEndReason;

    fn snapshot() -> SessionSnapshot {
        SessionSnapshot {
            id: SessionId(42),
            protocol: ProtocolKind::AirPlay,
            sender_name: Some("Kitchen iPhone".into()),
            sender_address: Some("192.168.1.50".into()),
            video: None,
            audio: Some(AudioFormat {
                codec: AudioCodec::AacLc,
                sample_rate: 44100,
                channels: 2,
                samples_per_packet: 1024,
            }),
            started_at: std::time::Instant::now(),
        }
    }

    #[test]
    fn the_snapshot_survives_the_conversion() {
        let converted = FfiSessionSnapshot::from(&snapshot());
        assert_eq!(converted.id, 42);
        assert_eq!(converted.protocol, FfiProtocolKind::AirPlay);
        assert_eq!(converted.sender_name.as_deref(), Some("Kitchen iPhone"));
        assert_eq!(converted.sender_address.as_deref(), Some("192.168.1.50"));
        assert_eq!(
            converted.audio.map(|audio| audio.codec),
            Some(FfiAudioCodec::AacLc)
        );
        assert!(converted.video.is_none());
    }

    #[test]
    fn the_started_event_carries_the_whole_session() {
        let event = FfiReceiverEvent::from(&SessionEvent::Started {
            session: snapshot(),
        });
        match &event {
            FfiReceiverEvent::SessionStarted { session } => {
                assert_eq!(session.id, 42);
                assert_eq!(session.protocol, FfiProtocolKind::AirPlay);
            }
            other => panic!("unexpected event: {other:?}"),
        }
        assert_eq!(event.session_id(), Some(42));
    }

    #[test]
    fn the_ended_event_carries_the_reason() {
        let event = FfiReceiverEvent::from(&SessionEvent::Ended {
            session: SessionId(42),
            reason: SessionEndReason::SenderDisconnected,
        });
        match &event {
            FfiReceiverEvent::SessionEnded { session_id, reason } => {
                assert_eq!(*session_id, 42);
                assert_eq!(reason, "sender disconnected");
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn a_status_event_belongs_to_no_session() {
        let event = FfiReceiverEvent::ProtocolStatusChanged {
            status: FfiProtocolStatus {
                protocol: FfiProtocolKind::Cast,
                state: crate::dto::FfiProtocolState::Advertising,
                enabled: true,
                tcp_port: Some(8009),
                udp_port: None,
                peer_count: 0,
                last_error: None,
            },
        };
        assert_eq!(event.session_id(), None);
    }
}
