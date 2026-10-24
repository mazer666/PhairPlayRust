//! Session lifecycle: what a connected peer is doing, and how the rest of the
//! system learns about it.
//!
//! WHY: the native shells, the web dashboard and the metrics all need to
//! answer the same question — "is something streaming right now, and what?".
//! A single session model in the core gives them one answer.
//!
//! HOW: an engine reports [`SessionEvent`]s on the channel in
//! [`crate::protocol::EngineContext`]. The core maintains a
//! [`SessionRegistry`] that folds those events into a per-protocol
//! [`SessionSnapshot`].

use std::collections::BTreeMap;

use crate::protocol::ProtocolKind;

/// The identifier of a streaming session.
///
/// Monotonic within one process run; never reused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SessionId(pub u64);

impl SessionId {
    /// The next session identifier.
    #[must_use]
    pub fn next() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "session-{}", self.0)
    }
}

/// How a session ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionEndReason {
    /// The sender closed the connection cleanly.
    SenderDisconnected,
    /// The receiver stopped the session.
    ReceiverStopped,
    /// The peer violated the protocol.
    ProtocolViolation(String),
    /// A configured limit was reached.
    LimitExceeded(String),
    /// The session failed for another reason.
    Failed(String),
}

impl std::fmt::Display for SessionEndReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionEndReason::SenderDisconnected => f.write_str("sender disconnected"),
            SessionEndReason::ReceiverStopped => f.write_str("receiver stopped"),
            SessionEndReason::ProtocolViolation(reason) => {
                write!(f, "protocol violation: {reason}")
            }
            SessionEndReason::LimitExceeded(reason) => write!(f, "limit exceeded: {reason}"),
            SessionEndReason::Failed(reason) => write!(f, "failed: {reason}"),
        }
    }
}

/// A snapshot of one live session.
#[derive(Debug, Clone, PartialEq)]
pub struct SessionSnapshot {
    /// The identifier of the session.
    pub id: SessionId,
    /// Which protocol carries this session.
    pub protocol: ProtocolKind,
    /// The display name of the sending device, if it announced one.
    pub sender_name: Option<String>,
    /// The address of the sending device.
    pub sender_address: Option<String>,
    /// The negotiated video format, if the session carries video.
    pub video: Option<crate::media::VideoFormat>,
    /// The negotiated audio format, if the session carries audio.
    pub audio: Option<crate::media::AudioFormat>,
    /// The wall-clock time the session started.
    pub started_at: std::time::Instant,
}

/// Something that happened during a streaming session.
///
/// WHY: the engine is the only place that knows the truth about the wire, so
/// the engine reports. The core interprets, the shells display.
#[derive(Debug, Clone, PartialEq)]
pub enum SessionEvent {
    /// A peer connected and a session began.
    Started {
        /// The new session.
        session: SessionSnapshot,
    },
    /// The negotiated video format changed mid-session.
    VideoFormatChanged {
        /// The session the format belongs to.
        session: SessionId,
        /// The new format.
        format: crate::media::VideoFormat,
    },
    /// The negotiated audio format changed mid-session.
    AudioFormatChanged {
        /// The session the format belongs to.
        session: SessionId,
        /// The new format.
        format: crate::media::AudioFormat,
    },
    /// The sender's playback position or state changed.
    PlaybackStateChanged {
        /// The session the change belongs to.
        session: SessionId,
        /// A human-readable description of the new state.
        state: String,
    },
    /// The now-playing metadata changed.
    MetadataChanged {
        /// The session the metadata belongs to.
        session: SessionId,
        /// The track title.
        title: Option<String>,
        /// The track artist.
        artist: Option<String>,
        /// The album title.
        album: Option<String>,
    },
    /// The artwork changed. `None` means the artwork was cleared.
    ArtworkChanged {
        /// The session the artwork belongs to.
        session: SessionId,
        /// The artwork bytes.
        artwork: Option<Vec<u8>>,
    },
    /// The session ended.
    Ended {
        /// The session that ended.
        session: SessionId,
        /// Why it ended.
        reason: SessionEndReason,
    },
}

/// A per-protocol view of the current session state.
#[derive(Debug, Clone, Default)]
pub struct SessionRegistry {
    sessions: BTreeMap<ProtocolKind, SessionSnapshot>,
}

impl SessionRegistry {
    /// Creates an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Folds an event into the registry.
    pub fn apply(&mut self, event: &SessionEvent) {
        match event {
            SessionEvent::Started { session } => {
                self.sessions.insert(session.protocol, session.clone());
            }
            SessionEvent::Ended { session, .. } => {
                // A late "ended" event from an older session must not clear a
                // newer session on the same protocol.
                if let Some(protocol) = self
                    .sessions
                    .iter()
                    .find(|(_, existing)| existing.id == *session)
                    .map(|(protocol, _)| *protocol)
                {
                    self.sessions.remove(&protocol);
                }
            }
            SessionEvent::VideoFormatChanged { session, format } => {
                if let Some(existing) = self.sessions.values_mut().find(|s| s.id == *session) {
                    existing.video = Some(format.clone());
                }
            }
            SessionEvent::AudioFormatChanged { session, format } => {
                if let Some(existing) = self.sessions.values_mut().find(|s| s.id == *session) {
                    existing.audio = Some(format.clone());
                }
            }
            SessionEvent::PlaybackStateChanged { .. }
            | SessionEvent::MetadataChanged { .. }
            | SessionEvent::ArtworkChanged { .. } => {}
        }
    }

    /// The live session of a protocol, if there is one.
    #[must_use]
    pub fn get(&self, protocol: ProtocolKind) -> Option<&SessionSnapshot> {
        self.sessions.get(&protocol)
    }

    /// Whether a protocol currently has a live session.
    #[must_use]
    pub fn is_active(&self, protocol: ProtocolKind) -> bool {
        self.sessions.contains_key(&protocol)
    }

    /// Every live session.
    #[must_use]
    pub fn sessions(&self) -> impl Iterator<Item = &SessionSnapshot> {
        self.sessions.values()
    }

    /// The number of live sessions.
    #[must_use]
    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    /// Whether no session is live.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::{AudioFormat, AudioCodec};

    fn snapshot(protocol: ProtocolKind) -> SessionSnapshot {
        SessionSnapshot {
            id: SessionId::next(),
            protocol,
            sender_name: Some("Test Sender".into()),
            sender_address: Some("192.168.1.50".into()),
            video: None,
            audio: None,
            started_at: std::time::Instant::now(),
        }
    }

    #[test]
    fn started_event_creates_a_live_session() {
        let mut registry = SessionRegistry::new();
        let session = snapshot(ProtocolKind::AirPlay);
        registry.apply(&SessionEvent::Started {
            session: session.clone(),
        });
        assert!(registry.is_active(ProtocolKind::AirPlay));
        assert_eq!(registry.get(ProtocolKind::AirPlay).unwrap().id, session.id);
    }

    #[test]
    fn ended_event_clears_the_session() {
        let mut registry = SessionRegistry::new();
        let session = snapshot(ProtocolKind::Cast);
        registry.apply(&SessionEvent::Started {
            session: session.clone(),
        });
        registry.apply(&SessionEvent::Ended {
            session: session.id,
            reason: SessionEndReason::SenderDisconnected,
        });
        assert!(!registry.is_active(ProtocolKind::Cast));
        assert!(registry.is_empty());
    }

    #[test]
    fn an_end_for_an_unknown_session_does_not_clear_a_newer_one() {
        // A late "ended" event from an old session must not kill the new one.
        let mut registry = SessionRegistry::new();
        let first = snapshot(ProtocolKind::AirPlay);
        let second = snapshot(ProtocolKind::AirPlay);
        registry.apply(&SessionEvent::Started {
            session: second.clone(),
        });
        registry.apply(&SessionEvent::Ended {
            session: first.id,
            reason: SessionEndReason::SenderDisconnected,
        });
        assert!(registry.is_active(ProtocolKind::AirPlay));
    }

    #[test]
    fn audio_format_change_updates_the_snapshot() {
        let mut registry = SessionRegistry::new();
        let session = snapshot(ProtocolKind::Dlna);
        registry.apply(&SessionEvent::Started {
            session: session.clone(),
        });
        let format = AudioFormat {
            codec: AudioCodec::AacLc,
            sample_rate: 44100,
            channels: 2,
            samples_per_packet: 1024,
        };
        registry.apply(&SessionEvent::AudioFormatChanged {
            session: session.id,
            format: format.clone(),
        });
        assert_eq!(registry.get(ProtocolKind::Dlna).unwrap().audio, Some(format));
    }
}
