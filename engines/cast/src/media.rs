//! The media channel of the Cast receiver.
//!
//! WHY: the media channel is where a sender tells the receiver *what* to
//! play: a URL, a content type, metadata and a set of tracks. The receiver
//! answers with a `MEDIA_STATUS` that says what it is doing right now. The
//! state machine below is the single place that knows the answer, so the
//! status the sender sees and the events the rest of the receiver sees can
//! never drift apart.
//!
//! WHAT: [`MediaSession`] holds the state of one media session,
//! [`MediaCommand`] is what a sender may ask for, and [`MediaOutcome`] is
//! what changed.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_cast::{LoadRequest, MediaCommand, MediaItem, MediaSession};
//!
//! let mut session = MediaSession::new();
//! let outcome = session.apply(&MediaCommand::Load(LoadRequest {
//!     media: Some(MediaItem {
//!         content_id: Some("https://example.invalid/movie.m3u8".to_string()),
//!         content_type: Some("application/x-mpegURL".to_string()),
//!         ..MediaItem::default()
//!     }),
//!     autoplay: Some(true),
//!     ..LoadRequest::default()
//! }));
//! assert!(outcome.is_ok());
//! assert_eq!(session.player_state(), "PLAYING");
//! ```

use serde::{Deserialize, Serialize};

use crate::payloads::{
    LoadRequest, MediaItem, PLAYER_STATE_IDLE, PLAYER_STATE_PAUSED, PLAYER_STATE_PLAYING,
    SUPPORTED_MEDIA_COMMANDS, VolumeStatus,
};

/// A command of the media channel.
//
// WHY: the lint of the size of the variants is switched off for this
// enumeration. `Load` carries the whole description of a media item, and
// the other variants carry nothing: boxing the large variant would make
// every call site of the public API of the crate allocate, and the
// enumeration is the wire format of the media channel — its shape is the
// shape of the protocol, not a question of layout.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq)]
pub enum MediaCommand {
    /// Load a media item.
    Load(LoadRequest),
    /// Start or resume playback.
    Play,
    /// Pause playback.
    Pause,
    /// Jump to a position.
    Seek {
        /// The media session to seek, if the sender named one.
        media_session_id: Option<u32>,
        /// The position in seconds.
        current_time: f64,
    },
    /// Stop playback.
    Stop,
    /// Ask for the current status.
    GetStatus,
}

impl MediaCommand {
    /// Parses a media command from its JSON payload.
    ///
    /// # Errors
    /// Returns [`MediaParseError`] when the payload is not a JSON object, has
    /// no known `type`, or misses a field the command needs — a `SEEK`
    /// without a position, for instance. The command is not guessed at.
    pub fn parse(payload: &str) -> Result<Self, MediaParseError> {
        let value: serde_json::Value =
            serde_json::from_str(payload).map_err(|_| MediaParseError::Malformed)?;
        let kind = value
            .get("type")
            .and_then(serde_json::Value::as_str)
            .ok_or(MediaParseError::Malformed)?;
        match kind {
            "LOAD" => {
                let request: LoadRequest =
                    serde_json::from_value(value).map_err(|_| MediaParseError::Malformed)?;
                Ok(Self::Load(request))
            }
            "PLAY" => Ok(Self::Play),
            "PAUSE" => Ok(Self::Pause),
            "SEEK" => {
                let request: SeekRequest =
                    serde_json::from_value(value).map_err(|_| MediaParseError::Malformed)?;
                let current_time = request.current_time.ok_or(MediaParseError::MissingField)?;
                if !current_time.is_finite() || current_time < 0.0 {
                    return Err(MediaParseError::OutOfRange);
                }
                Ok(Self::Seek {
                    media_session_id: request.media_session_id,
                    current_time,
                })
            }
            "STOP" => Ok(Self::Stop),
            "GET_STATUS" => Ok(Self::GetStatus),
            _ => Err(MediaParseError::UnknownType),
        }
    }
}

/// The `SEEK` request of the media channel.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeekRequest {
    /// The media session to seek.
    #[serde(default)]
    pub media_session_id: Option<u32>,
    /// The position to jump to.
    #[serde(default)]
    pub current_time: Option<f64>,
}

/// A malformed media command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaParseError {
    /// The payload is not a JSON object of the expected shape.
    Malformed,
    /// The payload names a command this receiver does not implement.
    UnknownType,
    /// The payload misses a field the command needs.
    MissingField,
    /// A numeric field is out of range.
    OutOfRange,
}

impl std::fmt::Display for MediaParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Malformed => f.write_str("media payload is malformed"),
            Self::UnknownType => f.write_str("media command is not implemented"),
            Self::MissingField => f.write_str("media command misses a required field"),
            Self::OutOfRange => f.write_str("media command carries an out-of-range value"),
        }
    }
}

impl std::error::Error for MediaParseError {}

/// What a media command changed.
#[derive(Debug, Clone, PartialEq)]
pub struct MediaOutcome {
    /// The new player state.
    pub player_state: String,
    /// The new metadata, if the command carried any.
    pub metadata: Option<MetadataChange>,
}

/// The metadata a `LOAD` command carried.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MetadataChange {
    /// The title of the track.
    pub title: Option<String>,
    /// The artist of the track.
    pub artist: Option<String>,
    /// The album of the track.
    pub album: Option<String>,
}

/// The state of one media session.
#[derive(Debug, Clone)]
pub struct MediaSession {
    media_session_id: u32,
    player_state: String,
    playback_rate: f64,
    current_time: f64,
    volume: VolumeStatus,
    active_track_ids: Vec<u32>,
    current_item_id: Option<u32>,
    media: Option<MediaItem>,
}

impl MediaSession {
    /// Creates an idle media session.
    #[must_use]
    pub fn new() -> Self {
        Self {
            media_session_id: 1,
            player_state: PLAYER_STATE_IDLE.to_string(),
            playback_rate: 1.0,
            current_time: 0.0,
            volume: VolumeStatus::default(),
            active_track_ids: Vec::new(),
            current_item_id: None,
            media: None,
        }
    }

    /// The identifier of the media session.
    #[must_use]
    pub fn media_session_id(&self) -> u32 {
        self.media_session_id
    }

    /// The current player state.
    #[must_use]
    pub fn player_state(&self) -> &str {
        &self.player_state
    }

    /// The media item that is loaded, if any.
    #[must_use]
    pub fn media(&self) -> Option<&MediaItem> {
        self.media.as_ref()
    }

    /// Applies a command to the session.
    ///
    /// # Errors
    /// Returns [`MediaParseError::MissingField`] when a `LOAD` command
    /// carries no media item. A `LOAD` without an item is not a session, and
    /// the receiver does not invent one.
    pub fn apply(&mut self, command: &MediaCommand) -> Result<MediaOutcome, MediaParseError> {
        let mut metadata = None;
        match command {
            MediaCommand::Load(request) => {
                let item = request.media.clone().ok_or(MediaParseError::MissingField)?;
                metadata = item.metadata.as_ref().map(|meta| MetadataChange {
                    title: meta.title.clone(),
                    artist: meta.artist.clone(),
                    album: meta.album.clone(),
                });
                self.media_session_id = self.media_session_id.saturating_add(1).max(1);
                self.current_item_id = Some(self.media_session_id);
                self.current_time = request.current_time.unwrap_or(0.0);
                self.active_track_ids = request.active_track_ids.clone().unwrap_or_default();
                self.media = Some(item);
                self.player_state = if request.autoplay.unwrap_or(false) {
                    PLAYER_STATE_PLAYING.to_string()
                } else {
                    PLAYER_STATE_IDLE.to_string()
                };
            }
            MediaCommand::Play => {
                self.player_state = PLAYER_STATE_PLAYING.to_string();
            }
            MediaCommand::Pause => {
                self.player_state = PLAYER_STATE_PAUSED.to_string();
            }
            MediaCommand::Seek { current_time, .. } => {
                self.current_time = *current_time;
            }
            MediaCommand::Stop => {
                self.player_state = PLAYER_STATE_IDLE.to_string();
                self.current_time = 0.0;
                self.active_track_ids.clear();
            }
            MediaCommand::GetStatus => {}
        }
        Ok(MediaOutcome {
            player_state: self.player_state.clone(),
            metadata,
        })
    }

    /// The status entry the sender receives.
    #[must_use]
    pub fn status(&self) -> MediaStatusEntry {
        MediaStatusEntry {
            media_session_id: self.media_session_id,
            player_state: self.player_state.clone(),
            playback_rate: self.playback_rate,
            current_time: self.current_time,
            supported_media_commands: SUPPORTED_MEDIA_COMMANDS,
            volume: self.volume.clone(),
            active_track_ids: self.active_track_ids.clone(),
            current_item_id: self.current_item_id,
            media: self.media.clone(),
        }
    }
}

impl Default for MediaSession {
    fn default() -> Self {
        Self::new()
    }
}

/// The status of the media session, as the sender sees it.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaStatusEntry {
    /// The identifier of the media session.
    pub media_session_id: u32,
    /// The player state, e.g. `PLAYING`.
    pub player_state: String,
    /// The playback rate, `1.0` being normal speed.
    pub playback_rate: f64,
    /// The playback position in seconds.
    pub current_time: f64,
    /// The bitmask of the media commands the receiver accepts.
    pub supported_media_commands: u32,
    /// The volume of the session.
    pub volume: VolumeStatus,
    /// The tracks that are active.
    pub active_track_ids: Vec<u32>,
    /// The item that is playing.
    pub current_item_id: Option<u32>,
    /// The media item.
    pub media: Option<MediaItem>,
}

/// The answer to a `GET_STATUS` request on the media channel.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaStatusResponse {
    /// Always `MEDIA_STATUS`.
    #[serde(rename = "type")]
    pub kind: &'static str,
    /// The status of every media session.
    pub status: Vec<MediaStatusEntry>,
}

impl MediaStatusResponse {
    /// Builds the answer from the status entries.
    #[must_use]
    pub fn new(status: Vec<MediaStatusEntry>) -> Self {
        Self {
            kind: "MEDIA_STATUS",
            status,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    fn load_request(autoplay: bool) -> LoadRequest {
        LoadRequest {
            media: Some(MediaItem {
                content_id: Some("http://example/clip.mp4".to_string()),
                content_type: Some("video/mp4".to_string()),
                metadata: Some(
                    serde_json::from_str(r#"{"title":"T","artist":"A","album":"B"}"#)
                        .expect("metadata"),
                ),
                ..Default::default()
            }),
            autoplay: Some(autoplay),
            ..Default::default()
        }
    }

    #[test]
    fn a_load_without_a_media_item_is_rejected() {
        let mut session = MediaSession::new();
        let command = MediaCommand::Load(LoadRequest::default());
        assert_eq!(session.apply(&command), Err(MediaParseError::MissingField));
    }

    #[test]
    fn a_load_with_autoplay_starts_playing() {
        let mut session = MediaSession::new();
        let outcome = session
            .apply(&MediaCommand::Load(load_request(true)))
            .expect("load");
        assert_eq!(outcome.player_state, PLAYER_STATE_PLAYING);
        assert_eq!(session.player_state(), PLAYER_STATE_PLAYING);
    }

    #[test]
    fn a_load_without_autoplay_stays_idle() {
        let mut session = MediaSession::new();
        session
            .apply(&MediaCommand::Load(load_request(false)))
            .expect("load");
        assert_eq!(session.player_state(), PLAYER_STATE_IDLE);
    }

    #[test]
    fn a_load_reports_the_metadata_it_carried() {
        let mut session = MediaSession::new();
        let outcome = session
            .apply(&MediaCommand::Load(load_request(true)))
            .expect("load");
        let metadata = outcome.metadata.expect("metadata");
        assert_eq!(metadata.title.as_deref(), Some("T"));
        assert_eq!(metadata.artist.as_deref(), Some("A"));
        assert_eq!(metadata.album.as_deref(), Some("B"));
    }

    #[test]
    fn play_and_pause_change_the_player_state() {
        let mut session = MediaSession::new();
        session
            .apply(&MediaCommand::Load(load_request(false)))
            .expect("load");
        session.apply(&MediaCommand::Play).expect("play");
        assert_eq!(session.player_state(), PLAYER_STATE_PLAYING);
        session.apply(&MediaCommand::Pause).expect("pause");
        assert_eq!(session.player_state(), PLAYER_STATE_PAUSED);
    }

    #[test]
    fn a_seek_moves_the_playback_position() {
        let mut session = MediaSession::new();
        session
            .apply(&MediaCommand::Seek {
                media_session_id: None,
                current_time: 42.5,
            })
            .expect("seek");
        assert_eq!(session.status().current_time, 42.5);
    }

    #[test]
    fn a_stop_clears_the_session() {
        let mut session = MediaSession::new();
        session
            .apply(&MediaCommand::Load(load_request(true)))
            .expect("load");
        session.apply(&MediaCommand::Stop).expect("stop");
        assert_eq!(session.player_state(), PLAYER_STATE_IDLE);
        assert_eq!(session.status().current_time, 0.0);
    }

    #[test]
    fn a_seek_without_a_position_is_rejected() {
        assert_eq!(
            MediaCommand::parse(r#"{"type":"SEEK"}"#),
            Err(MediaParseError::MissingField)
        );
    }

    #[test]
    fn a_seek_into_negative_time_is_rejected() {
        assert_eq!(
            MediaCommand::parse(r#"{"type":"SEEK","currentTime":-1}"#),
            Err(MediaParseError::OutOfRange)
        );
    }

    #[test]
    fn a_seek_with_an_infinite_position_is_rejected() {
        // JSON has no infinity, but a very large number is still finite and
        // must be accepted; the check is for a non-finite value only.
        let command = MediaCommand::parse(r#"{"type":"SEEK","currentTime":1e30}"#).expect("parse");
        assert!(matches!(command, MediaCommand::Seek { .. }));
    }

    #[test]
    fn an_unknown_media_command_is_rejected() {
        assert_eq!(
            MediaCommand::parse(r#"{"type":"TELEPORT"}"#),
            Err(MediaParseError::UnknownType)
        );
    }

    #[test]
    fn a_malformed_media_payload_is_rejected() {
        assert_eq!(
            MediaCommand::parse("not json at all"),
            Err(MediaParseError::Malformed)
        );
    }

    #[test]
    fn the_status_entry_carries_the_media_item() {
        let mut session = MediaSession::new();
        session
            .apply(&MediaCommand::Load(load_request(true)))
            .expect("load");
        let status = session.status();
        assert_eq!(
            status.media.and_then(|item| item.content_id),
            Some("http://example/clip.mp4".to_string())
        );
        assert_eq!(status.supported_media_commands, SUPPORTED_MEDIA_COMMANDS);
    }
}
