//! The state of one Cast peer and the dispatch of the messages it sends.
//!
//! WHY: the CastV2 receiver is a small state machine per peer: a transport
//! channel, a set of virtual channels, a media session and a volume. The
//! dispatch rules — which message produces which answer — are the protocol
//! itself, and they must be testable without a socket. So they live here, in
//! a pure function of (state, message) → (replies, actions).
//!
//! WHAT: [`PeerSession`] is the state, [`PeerSession::handle`] is the
//! dispatch, [`SessionAction`] is what the engine has to do afterwards.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_cast::{CastMessage, Namespace, PeerSession};
//!
//! let mut session = PeerSession::new("web-1", "session-1", "CC1AD845");
//! let reply = session
//!     .handle(&CastMessage::text(
//!         Namespace::Heartbeat,
//!         "sender-0",
//!         "heartbeat-0",
//!         r#"{"type":"PING"}"#,
//!     ))
//!     .expect("dispatch");
//! assert_eq!(reply.replies.len(), 1);
//! ```

use std::collections::BTreeSet;

use crate::media::{MediaCommand, MediaSession, MediaStatusResponse};
use crate::message::{
    CastMessage, DEFAULT_MEDIA_APP_ID, DEFAULT_MEDIA_APP_NAME, MEDIA_CHANNEL, Namespace,
    RECEIVER_CHANNEL,
};
use crate::payloads::{
    ConnectRequest, ConnectResponse, ReceiverApplication, ReceiverNamespace, ReceiverStatus,
    ReceiverStatusResponse, SetVolumeRequest, VolumeStatus, message_type,
};

/// The channel a sender opens on the receiver.
pub const SENDER_CHANNEL: &str = "sender-0";

/// A violation of the CastV2 protocol by a peer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolViolation {
    /// The message names a namespace this receiver does not implement.
    UnknownNamespace(String),
    /// The message names a command this receiver does not implement.
    UnknownMessageType(String),
    /// The message is not valid JSON, or not of the expected shape.
    MalformedPayload(String),
    /// The peer asked for an application this receiver does not offer.
    UnknownAppId(String),
}

impl std::fmt::Display for ProtocolViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownNamespace(name) => write!(f, "unknown namespace `{name}`"),
            Self::UnknownMessageType(name) => write!(f, "unknown message type `{name}`"),
            Self::MalformedPayload(detail) => write!(f, "malformed payload: {detail}"),
            Self::UnknownAppId(app_id) => write!(f, "unknown application id `{app_id}`"),
        }
    }
}

impl std::error::Error for ProtocolViolation {}

/// What the engine has to do after a message was dispatched.
#[derive(Debug, Clone, PartialEq)]
pub enum SessionAction {
    /// The peer asked to change the volume of the receiver.
    SetVolume {
        /// The new level between `0.0` and `1.0`.
        level: f32,
        /// Whether the output is muted.
        muted: bool,
    },
    /// The peer sent new metadata.
    MetadataChanged {
        /// The title of the track.
        title: Option<String>,
        /// The artist of the track.
        artist: Option<String>,
        /// The album of the track.
        album: Option<String>,
    },
    /// The playback state changed.
    PlaybackStateChanged {
        /// The new state, e.g. `PLAYING`.
        state: String,
    },
    /// The peer asked to close the connection.
    Close,
}

/// The result of dispatching one message.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Dispatch {
    /// The messages that are sent back to the peer.
    pub replies: Vec<CastMessage>,
    /// The things the engine has to do.
    pub actions: Vec<SessionAction>,
}

/// The state of one Cast peer.
#[derive(Debug, Clone)]
pub struct PeerSession {
    transport_id: String,
    session_id: String,
    app_id: String,
    app_name: String,
    open_channels: BTreeSet<String>,
    media: MediaSession,
    volume: VolumeStatus,
    sender_name: Option<String>,
}

impl PeerSession {
    /// Creates the state of a peer that just connected.
    #[must_use]
    pub fn new(
        transport_id: impl Into<String>,
        session_id: impl Into<String>,
        app_id: impl Into<String>,
    ) -> Self {
        Self {
            transport_id: transport_id.into(),
            session_id: session_id.into(),
            app_id: app_id.into(),
            app_name: DEFAULT_MEDIA_APP_NAME.to_string(),
            open_channels: BTreeSet::new(),
            media: MediaSession::new(),
            volume: VolumeStatus::default(),
            sender_name: None,
        }
    }

    /// The transport the peer is connected on.
    #[must_use]
    pub fn transport_id(&self) -> &str {
        &self.transport_id
    }

    /// The session identifier of the peer.
    #[must_use]
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// The name the peer announced, if it announced one.
    #[must_use]
    pub fn sender_name(&self) -> Option<&str> {
        self.sender_name.as_deref()
    }

    /// The media session of the peer.
    #[must_use]
    pub fn media(&self) -> &MediaSession {
        &self.media
    }

    /// The volume of the receiver.
    #[must_use]
    pub fn volume(&self) -> &VolumeStatus {
        &self.volume
    }

    /// The channels the peer has opened.
    #[must_use]
    pub fn open_channels(&self) -> &BTreeSet<String> {
        &self.open_channels
    }

    /// The messages the receiver sends when the connection is established.
    ///
    /// WHY: a Cast sender waits for the receiver to open the receiver
    /// channel before it asks for the status. The receiver therefore greets
    /// the sender with a `CONNECT` on the receiver channel.
    #[must_use]
    pub fn greeting(&self) -> Vec<CastMessage> {
        vec![CastMessage::text(
            Namespace::Connection,
            RECEIVER_CHANNEL,
            SENDER_CHANNEL,
            r#"{"type":"CONNECT","origin":{}}"#,
        )]
    }

    /// Dispatches one message.
    ///
    /// # Errors
    /// Returns [`ProtocolViolation`] when the message names an unknown
    /// namespace, an unknown command, a malformed payload or an application
    /// this receiver does not offer.
    pub fn handle(&mut self, message: &CastMessage) -> Result<Dispatch, ProtocolViolation> {
        let namespace = message.namespace_kind().ok_or_else(|| {
            ProtocolViolation::UnknownNamespace(message.namespace.clone().unwrap_or_default())
        })?;
        let payload = message.payload().unwrap_or_default();
        match namespace {
            Namespace::Connection => self.handle_connection(payload, message),
            Namespace::Heartbeat => self.handle_heartbeat(payload, message),
            Namespace::Receiver => self.handle_receiver(payload, message),
            Namespace::Media => self.handle_media(payload, message),
        }
    }

    fn handle_connection(
        &mut self,
        payload: &str,
        message: &CastMessage,
    ) -> Result<Dispatch, ProtocolViolation> {
        let kind = message_type(payload)
            .ok_or_else(|| ProtocolViolation::MalformedPayload(payload.to_string()))?;
        match kind.as_str() {
            "CONNECT" => {
                if let Ok(request) = serde_json::from_str::<ConnectRequest>(payload) {
                    if let Some(info) = request.sender_info {
                        self.sender_name = info.display_name.or(info.sender_name);
                    }
                }
                let channel = message
                    .destination_id
                    .clone()
                    .unwrap_or_else(|| RECEIVER_CHANNEL.to_string());
                self.open_channels.insert(channel);
                let reply = CastMessage::text(
                    Namespace::Connection,
                    message.destination_id.clone().unwrap_or_default(),
                    message.source_id.clone().unwrap_or_default(),
                    serde_json::to_string(&ConnectResponse::new(
                        self.session_id.clone(),
                        self.transport_id.clone(),
                    ))
                    .unwrap_or_else(|_| String::from("{\"type\":\"CONNECTED\"}")),
                );
                Ok(Dispatch {
                    replies: vec![reply],
                    actions: Vec::new(),
                })
            }
            "CLOSE" => Ok(Dispatch {
                replies: Vec::new(),
                actions: vec![SessionAction::Close],
            }),
            other => Err(ProtocolViolation::UnknownMessageType(other.to_string())),
        }
    }

    fn handle_heartbeat(
        &mut self,
        payload: &str,
        message: &CastMessage,
    ) -> Result<Dispatch, ProtocolViolation> {
        let kind = message_type(payload)
            .ok_or_else(|| ProtocolViolation::MalformedPayload(payload.to_string()))?;
        match kind.as_str() {
            "PING" => {
                let reply = CastMessage::text(
                    Namespace::Heartbeat,
                    message.destination_id.clone().unwrap_or_default(),
                    message.source_id.clone().unwrap_or_default(),
                    r#"{"type":"PONG"}"#,
                );
                Ok(Dispatch {
                    replies: vec![reply],
                    actions: Vec::new(),
                })
            }
            "PONG" => Ok(Dispatch::default()),
            other => Err(ProtocolViolation::UnknownMessageType(other.to_string())),
        }
    }

    fn handle_receiver(
        &mut self,
        payload: &str,
        message: &CastMessage,
    ) -> Result<Dispatch, ProtocolViolation> {
        let kind = message_type(payload)
            .ok_or_else(|| ProtocolViolation::MalformedPayload(payload.to_string()))?;
        match kind.as_str() {
            "GET_STATUS" => {
                let status = self.receiver_status();
                let reply = CastMessage::text(
                    Namespace::Receiver,
                    message.destination_id.clone().unwrap_or_default(),
                    message.source_id.clone().unwrap_or_default(),
                    serde_json::to_string(&ReceiverStatusResponse {
                        kind: "RECEIVER_STATUS",
                        status,
                    })
                    .unwrap_or_default(),
                );
                Ok(Dispatch {
                    replies: vec![reply],
                    actions: Vec::new(),
                })
            }
            "SET_VOLUME" => {
                let request: SetVolumeRequest = serde_json::from_str(payload)
                    .map_err(|_| ProtocolViolation::MalformedPayload(payload.to_string()))?;
                let volume = request.volume.unwrap_or_default();
                let level = volume.level.unwrap_or(self.volume.level).clamp(0.0, 1.0);
                let muted = volume.muted.unwrap_or(self.volume.muted);
                self.volume.level = level;
                self.volume.muted = muted;
                Ok(Dispatch {
                    replies: Vec::new(),
                    actions: vec![SessionAction::SetVolume { level, muted }],
                })
            }
            "LAUNCH" => {
                let app_id = serde_json::from_str::<serde_json::Value>(payload)
                    .ok()
                    .and_then(|value| {
                        value
                            .get("appId")
                            .and_then(serde_json::Value::as_str)
                            .map(str::to_string)
                    })
                    .ok_or_else(|| ProtocolViolation::MalformedPayload(payload.to_string()))?;
                if app_id != self.app_id && app_id != DEFAULT_MEDIA_APP_ID {
                    return Err(ProtocolViolation::UnknownAppId(app_id));
                }
                Ok(Dispatch::default())
            }
            "STOP" => {
                self.media
                    .apply(&MediaCommand::Stop)
                    .map_err(|error| ProtocolViolation::MalformedPayload(error.to_string()))?;
                Ok(Dispatch {
                    replies: Vec::new(),
                    actions: vec![SessionAction::PlaybackStateChanged {
                        state: self.media.player_state().to_string(),
                    }],
                })
            }
            other => Err(ProtocolViolation::UnknownMessageType(other.to_string())),
        }
    }

    fn handle_media(
        &mut self,
        payload: &str,
        message: &CastMessage,
    ) -> Result<Dispatch, ProtocolViolation> {
        let command = MediaCommand::parse(payload)
            .map_err(|error| ProtocolViolation::MalformedPayload(error.to_string()))?;
        let outcome = self
            .media
            .apply(&command)
            .map_err(|error| ProtocolViolation::MalformedPayload(error.to_string()))?;
        let mut actions = Vec::new();
        if let Some(metadata) = outcome.metadata {
            actions.push(SessionAction::MetadataChanged {
                title: metadata.title,
                artist: metadata.artist,
                album: metadata.album,
            });
        }
        actions.push(SessionAction::PlaybackStateChanged {
            state: outcome.player_state.clone(),
        });
        let status = MediaStatusResponse::new(vec![self.media.status()]);
        let reply = CastMessage::text(
            Namespace::Media,
            message.destination_id.clone().unwrap_or_default(),
            message.source_id.clone().unwrap_or_default(),
            serde_json::to_string(&status).unwrap_or_default(),
        );
        Ok(Dispatch {
            replies: vec![reply],
            actions,
        })
    }

    /// The status the receiver reports on the receiver channel.
    #[must_use]
    pub fn receiver_status(&self) -> ReceiverStatus {
        ReceiverStatus {
            applications: vec![ReceiverApplication {
                app_id: self.app_id.clone(),
                display_name: self.app_name.clone(),
                icon_url: String::new(),
                is_idle_screen: false,
                launched_from_cloud: false,
                namespaces: vec![ReceiverNamespace {
                    name: crate::message::Namespace::Media.as_str().to_string(),
                }],
                session_id: self.session_id.clone(),
                status_text: "Ready To Cast".to_string(),
                transport_id: self.transport_id.clone(),
                universal_app_id: self.app_id.clone(),
            }],
            volume: self.volume.clone(),
        }
    }

    /// The media channel the peer may open.
    #[must_use]
    pub fn media_channel(&self) -> &'static str {
        MEDIA_CHANNEL
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    fn session() -> PeerSession {
        PeerSession::new("web-1", "session-1", "CC1AD845")
    }

    #[test]
    fn a_ping_is_answered_with_a_pong() {
        let mut session = session();
        let dispatch = session
            .handle(&CastMessage::text(
                Namespace::Heartbeat,
                "sender-0",
                "heartbeat-0",
                r#"{"type":"PING"}"#,
            ))
            .expect("dispatch");
        assert_eq!(dispatch.replies.len(), 1);
        let reply = &dispatch.replies[0];
        assert_eq!(reply.payload(), Some(r#"{"type":"PONG"}"#));
        assert_eq!(reply.source_id.as_deref(), Some("heartbeat-0"));
        assert_eq!(reply.destination_id.as_deref(), Some("sender-0"));
    }

    #[test]
    fn a_pong_is_answered_with_nothing() {
        let mut session = session();
        let dispatch = session
            .handle(&CastMessage::text(
                Namespace::Heartbeat,
                "sender-0",
                "heartbeat-0",
                r#"{"type":"PONG"}"#,
            ))
            .expect("dispatch");
        assert!(dispatch.replies.is_empty());
    }

    #[test]
    fn a_connect_is_answered_with_a_connected() {
        let mut session = session();
        let dispatch = session
            .handle(&CastMessage::text(
                Namespace::Connection,
                "sender-0",
                "media-0",
                r#"{"type":"CONNECT"}"#,
            ))
            .expect("dispatch");
        assert_eq!(dispatch.replies.len(), 1);
        let payload = dispatch.replies[0].payload().expect("payload");
        assert!(payload.contains("CONNECTED"), "{payload}");
        assert!(payload.contains("session-1"), "{payload}");
        assert!(payload.contains("web-1"), "{payload}");
        assert!(session.open_channels().contains("media-0"));
    }

    #[test]
    fn a_connect_remembers_the_name_of_the_sender() {
        let mut session = session();
        session
            .handle(&CastMessage::text(
                Namespace::Connection,
                "sender-0",
                "receiver-0",
                r#"{"type":"CONNECT","senderInfo":{"displayName":"Phone"}}"#,
            ))
            .expect("dispatch");
        assert_eq!(session.sender_name(), Some("Phone"));
    }

    #[test]
    fn a_close_asks_the_engine_to_close_the_connection() {
        let mut session = session();
        let dispatch = session
            .handle(&CastMessage::text(
                Namespace::Connection,
                "sender-0",
                "web-1",
                r#"{"type":"CLOSE"}"#,
            ))
            .expect("dispatch");
        assert_eq!(dispatch.actions, vec![SessionAction::Close]);
    }

    #[test]
    fn the_receiver_status_names_the_application() {
        let mut session = session();
        let dispatch = session
            .handle(&CastMessage::text(
                Namespace::Receiver,
                "sender-0",
                "receiver-0",
                r#"{"type":"GET_STATUS"}"#,
            ))
            .expect("dispatch");
        assert_eq!(dispatch.replies.len(), 1);
        let payload = dispatch.replies[0].payload().expect("payload");
        assert!(payload.contains("RECEIVER_STATUS"), "{payload}");
        assert!(payload.contains("CC1AD845"), "{payload}");
        assert!(
            payload.contains("urn:x-cast:com.google.cast.media"),
            "{payload}"
        );
    }

    #[test]
    fn a_set_volume_changes_the_volume_and_reports_it() {
        let mut session = session();
        let dispatch = session
            .handle(&CastMessage::text(
                Namespace::Receiver,
                "sender-0",
                "receiver-0",
                r#"{"type":"SET_VOLUME","volume":{"level":0.25}}"#,
            ))
            .expect("dispatch");
        assert_eq!(
            dispatch.actions,
            vec![SessionAction::SetVolume {
                level: 0.25,
                muted: false
            }]
        );
        assert_eq!(session.volume().level, 0.25);
    }

    #[test]
    fn a_volume_above_one_is_clamped() {
        let mut session = session();
        session
            .handle(&CastMessage::text(
                Namespace::Receiver,
                "sender-0",
                "receiver-0",
                r#"{"type":"SET_VOLUME","volume":{"level":7.0}}"#,
            ))
            .expect("dispatch");
        assert_eq!(session.volume().level, 1.0);
    }

    #[test]
    fn a_launch_of_an_unknown_application_is_refused() {
        let mut session = session();
        let error = session
            .handle(&CastMessage::text(
                Namespace::Receiver,
                "sender-0",
                "receiver-0",
                r#"{"type":"LAUNCH","appId":"DEADBEEF"}"#,
            ))
            .expect_err("must fail");
        assert_eq!(error, ProtocolViolation::UnknownAppId("DEADBEEF".into()));
    }

    #[test]
    fn a_launch_of_the_own_application_is_accepted() {
        let mut session = session();
        session
            .handle(&CastMessage::text(
                Namespace::Receiver,
                "sender-0",
                "receiver-0",
                r#"{"type":"LAUNCH","appId":"CC1AD845"}"#,
            ))
            .expect("dispatch");
    }

    #[test]
    fn a_load_is_answered_with_a_media_status() {
        let mut session = session();
        let dispatch = session
            .handle(&CastMessage::text(
                Namespace::Media,
                "sender-0",
                "media-0",
                r#"{"type":"LOAD","media":{"contentId":"http://x/y.mp4","contentType":"video/mp4"},"autoplay":true}"#,
            ))
            .expect("dispatch");
        assert_eq!(dispatch.replies.len(), 1);
        let payload = dispatch.replies[0].payload().expect("payload");
        assert!(payload.contains("MEDIA_STATUS"), "{payload}");
        assert!(payload.contains("PLAYING"), "{payload}");
        assert!(dispatch
            .actions
            .iter()
            .any(|action| matches!(action, SessionAction::PlaybackStateChanged { state } if state == "PLAYING")));
    }

    #[test]
    fn a_load_reports_the_metadata() {
        let mut session = session();
        let dispatch = session
            .handle(&CastMessage::text(
                Namespace::Media,
                "sender-0",
                "media-0",
                r#"{"type":"LOAD","media":{"contentId":"http://x/y.mp4","metadata":{"title":"Song"}}}"#,
            ))
            .expect("dispatch");
        assert!(dispatch.actions.iter().any(|action| matches!(
            action,
            SessionAction::MetadataChanged { title, .. } if title.as_deref() == Some("Song")
        )));
    }

    #[test]
    fn an_unknown_namespace_is_refused() {
        let mut session = session();
        let mut message = CastMessage::text(Namespace::Media, "s", "d", "{}");
        message.namespace = Some("urn:x-cast:com.google.cast.telepathy".to_string());
        assert!(matches!(
            session.handle(&message),
            Err(ProtocolViolation::UnknownNamespace(_))
        ));
    }

    #[test]
    fn an_unknown_command_is_refused() {
        let mut session = session();
        let error = session
            .handle(&CastMessage::text(
                Namespace::Heartbeat,
                "sender-0",
                "heartbeat-0",
                r#"{"type":"TELEPORT"}"#,
            ))
            .expect_err("must fail");
        assert_eq!(
            error,
            ProtocolViolation::UnknownMessageType("TELEPORT".into())
        );
    }

    #[test]
    fn a_malformed_payload_is_refused() {
        let mut session = session();
        let error = session
            .handle(&CastMessage::text(
                Namespace::Heartbeat,
                "sender-0",
                "heartbeat-0",
                "not json",
            ))
            .expect_err("must fail");
        assert!(matches!(error, ProtocolViolation::MalformedPayload(_)));
    }

    #[test]
    fn the_greeting_opens_the_receiver_channel() {
        let session = session();
        let greeting = session.greeting();
        assert_eq!(greeting.len(), 1);
        assert_eq!(
            greeting[0].namespace.as_deref(),
            Some("urn:x-cast:com.google.cast.tp.connection")
        );
        assert_eq!(greeting[0].source_id.as_deref(), Some("receiver-0"));
    }
}
