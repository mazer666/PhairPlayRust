//! The JSON payloads of the CastV2 control channels.
//!
//! WHY: the control channels of the Cast protocol speak JSON, not protobuf.
//! The envelope is protobuf, the payload is an object with a `type` field
//! that says what the object is. Modelling the payloads as typed structures
//! keeps the wire format honest: a sender that sends a `LOAD` without a
//! `media` object is rejected, not guessed at.
//!
//! WHAT: [`message_type`] reads the discriminator of a payload, the
//! `*Request` types are what a sender may send, the `*Response` types are
//! what the receiver answers.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_cast::message_type;
//!
//! assert_eq!(message_type(r#"{"type":"PING"}"#), Some("PING".to_string()));
//! assert_eq!(message_type("not json"), None);
//! ```

use serde::{Deserialize, Serialize};

/// The player state of a media session.
pub const PLAYER_STATE_IDLE: &str = "IDLE";
/// The player state of a playing media session.
pub const PLAYER_STATE_PLAYING: &str = "PLAYING";
/// The player state of a paused media session.
pub const PLAYER_STATE_PAUSED: &str = "PAUSED";

/// The bitmask of the media commands the receiver supports.
///
/// WHY: the sender asks the receiver which media commands it may issue. The
/// value is the one the Google default media receiver reports; a sender that
/// sees a smaller mask simply offers fewer controls.
pub const SUPPORTED_MEDIA_COMMANDS: u32 = 274_447;

/// The `type` field of a control payload.
///
/// Returns `None` for a payload that is not a JSON object or that has no
/// `type` field. The caller then treats the message as a protocol
/// violation — it must not guess what was meant.
#[must_use]
pub fn message_type(payload: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(payload).ok()?;
    value.get("type")?.as_str().map(str::to_string)
}

/// The `CONNECT` request a sender sends on the transport channel.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConnectRequest {
    /// The information about the sending device, if it sent any.
    #[serde(default)]
    pub sender_info: Option<SenderInfo>,
    /// The user agent of the sender, if it sent any.
    #[serde(default)]
    pub user_agent: Option<String>,
}

/// What a sender tells the receiver about itself.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SenderInfo {
    /// The type of the sender, e.g. `user`.
    #[serde(default)]
    pub sender_type: Option<String>,
    /// The display name of the sending device.
    #[serde(default)]
    pub display_name: Option<String>,
    /// The name of the sending application.
    #[serde(default)]
    pub sender_name: Option<String>,
}

/// The answer to a `CONNECT` request.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectResponse {
    /// Always `CONNECTED`.
    #[serde(rename = "type")]
    pub kind: &'static str,
    /// The identifier of the session that was opened.
    pub session_id: String,
    /// The identifier of the transport the session runs on.
    pub transport_id: String,
}

impl ConnectResponse {
    /// Builds the answer to a `CONNECT` request.
    #[must_use]
    pub fn new(session_id: impl Into<String>, transport_id: impl Into<String>) -> Self {
        Self {
            kind: "CONNECTED",
            session_id: session_id.into(),
            transport_id: transport_id.into(),
        }
    }
}

/// The answer to a `GET_STATUS` request on the receiver channel.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiverStatusResponse {
    /// Always `RECEIVER_STATUS`.
    #[serde(rename = "type")]
    pub kind: &'static str,
    /// The status of the receiver.
    pub status: ReceiverStatus,
}

/// The status of the receiver.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiverStatus {
    /// The applications the receiver offers.
    pub applications: Vec<ReceiverApplication>,
    /// The volume of the receiver.
    pub volume: VolumeStatus,
}

/// One application the receiver offers.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiverApplication {
    /// The Cast application id.
    pub app_id: String,
    /// The name shown to the user.
    pub display_name: String,
    /// The URL of the application icon.
    pub icon_url: String,
    /// Whether this application is the idle screen.
    pub is_idle_screen: bool,
    /// Whether the application was launched from the cloud.
    pub launched_from_cloud: bool,
    /// The namespaces the application answers on.
    pub namespaces: Vec<ReceiverNamespace>,
    /// The identifier of the session the application runs in.
    pub session_id: String,
    /// The status line shown to the user.
    pub status_text: String,
    /// The transport the application runs on.
    pub transport_id: String,
    /// The universal application id.
    pub universal_app_id: String,
}

/// One namespace an application answers on.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiverNamespace {
    /// The name of the namespace.
    pub name: String,
}

/// The volume of the receiver.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeStatus {
    /// How the volume is controlled, e.g. `attenuation`.
    pub control_type: String,
    /// The volume level between `0.0` and `1.0`.
    pub level: f32,
    /// Whether the output is muted.
    pub muted: bool,
    /// The step size of the volume control.
    pub step_interval: f32,
}

impl Default for VolumeStatus {
    fn default() -> Self {
        Self {
            control_type: "attenuation".to_string(),
            level: 1.0,
            muted: false,
            step_interval: 0.05,
        }
    }
}

/// The `LOAD` request of the media channel.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LoadRequest {
    /// The media item to play.
    #[serde(default)]
    pub media: Option<MediaItem>,
    /// Whether playback starts right away.
    #[serde(default)]
    pub autoplay: Option<bool>,
    /// The playback position to start at.
    #[serde(default)]
    pub current_time: Option<f64>,
    /// The tracks that are active.
    #[serde(default)]
    pub active_track_ids: Option<Vec<u32>>,
}

/// One media item.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MediaItem {
    /// The URL of the media.
    #[serde(default)]
    pub content_id: Option<String>,
    /// The MIME type of the media.
    #[serde(default)]
    pub content_type: Option<String>,
    /// How the media is streamed, e.g. `BUFFERED` or `LIVE`.
    #[serde(default)]
    pub stream_type: Option<String>,
    /// The length of the media in seconds.
    #[serde(default)]
    pub duration: Option<f64>,
    /// The metadata of the media.
    #[serde(default)]
    pub metadata: Option<MediaMetadata>,
    /// The tracks of the media.
    #[serde(default)]
    pub tracks: Vec<MediaTrack>,
}

/// The metadata of a media item.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MediaMetadata {
    /// The kind of metadata, `0` being the generic kind.
    #[serde(default)]
    pub metadata_type: Option<u32>,
    /// The title of the track.
    #[serde(default)]
    pub title: Option<String>,
    /// The artist of the track.
    #[serde(default)]
    pub artist: Option<String>,
    /// The album of the track.
    #[serde(default)]
    pub album: Option<String>,
    /// The album artist of the track.
    #[serde(default)]
    pub album_artist: Option<String>,
    /// The subtitle of the track.
    #[serde(default)]
    pub subtitle: Option<String>,
    /// The artwork of the track.
    #[serde(default)]
    pub artwork: Option<Artwork>,
}

/// The artwork of a media item.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Artwork {
    /// The URL of the artwork.
    #[serde(default)]
    pub url: Option<String>,
}

/// One track of a media item.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MediaTrack {
    /// The identifier of the track.
    #[serde(default)]
    pub track_id: Option<u32>,
    /// The kind of the track, e.g. `TEXT` or `AUDIO`.
    #[serde(rename = "type")]
    pub kind: Option<String>,
    /// The URL of the track.
    #[serde(default)]
    pub track_content_id: Option<String>,
    /// The sub kind of the track, e.g. `SUBTITLES`.
    #[serde(default)]
    pub subtype: Option<String>,
    /// The name of the track.
    #[serde(default)]
    pub name: Option<String>,
    /// The language of the track.
    #[serde(default)]
    pub language: Option<String>,
}

/// The `SET_VOLUME` request of the receiver channel.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SetVolumeRequest {
    /// The volume to set, between `0.0` and `1.0`.
    #[serde(default)]
    pub volume: Option<VolumeRequest>,
}

/// The volume of a `SET_VOLUME` request.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VolumeRequest {
    /// The level between `0.0` and `1.0`.
    #[serde(default)]
    pub level: Option<f32>,
    /// Whether the output is muted.
    #[serde(default)]
    pub muted: Option<bool>,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    #[test]
    fn the_message_type_is_read_from_the_payload() {
        assert_eq!(message_type(r#"{"type":"PING"}"#), Some("PING".to_string()));
        assert_eq!(
            message_type(r#"{"type":"GET_STATUS"}"#),
            Some("GET_STATUS".to_string())
        );
    }

    #[test]
    fn a_payload_that_is_not_json_has_no_type() {
        assert_eq!(message_type("this is not json"), None);
        assert_eq!(message_type("[]"), None);
        assert_eq!(message_type(r#"{"no":"type"}"#), None);
    }

    #[test]
    fn a_connect_request_is_parsed_with_its_sender_info() {
        let request: ConnectRequest = serde_json::from_str(
            r#"{"type":"CONNECT","senderInfo":{"displayName":"Kitchen phone"},"userAgent":"x"}"#,
        )
        .expect("parse");
        assert_eq!(
            request.sender_info.and_then(|info| info.display_name),
            Some("Kitchen phone".to_string())
        );
        assert_eq!(request.user_agent.as_deref(), Some("x"));
    }

    #[test]
    fn a_load_request_without_media_is_parsed_but_has_no_item() {
        let request: LoadRequest = serde_json::from_str(r#"{"type":"LOAD"}"#).expect("parse");
        assert!(request.media.is_none());
        assert!(request.autoplay.is_none());
    }

    #[test]
    fn a_load_request_carries_its_metadata() {
        let request: LoadRequest = serde_json::from_str(
            r#"{"type":"LOAD","media":{"contentId":"http://example/x.mp4",
                "contentType":"video/mp4","metadata":{"title":"Song","artist":"Band"},
                "tracks":[{"trackId":1,"type":"TEXT"}]},"autoplay":true}"#,
        )
        .expect("parse");
        let media = request.media.expect("media");
        assert_eq!(media.content_id.as_deref(), Some("http://example/x.mp4"));
        assert_eq!(
            media.metadata.and_then(|meta| meta.title),
            Some("Song".to_string())
        );
        assert_eq!(media.tracks.len(), 1);
        assert_eq!(request.autoplay, Some(true));
    }

    #[test]
    fn the_connect_response_carries_the_session_and_transport() {
        let response = ConnectResponse::new("session-1", "web-1");
        let json = serde_json::to_value(&response).expect("serialise");
        assert_eq!(json["type"], "CONNECTED");
        assert_eq!(json["sessionId"], "session-1");
        assert_eq!(json["transportId"], "web-1");
    }

    #[test]
    fn the_volume_status_has_a_sane_default() {
        let status = VolumeStatus::default();
        assert_eq!(status.level, 1.0);
        assert!(!status.muted);
        assert_eq!(status.control_type, "attenuation");
    }
}
