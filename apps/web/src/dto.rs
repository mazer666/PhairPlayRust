//! The data transfer objects of the HTTP API.
//!
//! WHAT: The JSON shapes that the dashboard reads: the status of the
//! protocols, the live sessions, the counters, the events of the WebSocket
//! channel and the answers of the actions.
//!
//! WHY: The dashboard is a separate program that may be replaced. It needs
//! a contract it can rely on, and the contract has to be written down in
//! one place. Every field of these structures is a fact about the receiver
//! that comes from the core — the API invents nothing.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_web::dto::{ProtocolStatusDto, ProtocolStateDto};
//!
//! let status = ProtocolStatusDto {
//!     protocol: "airplay".into(),
//!     display_name: "AirPlay 2".into(),
//!     state: ProtocolStateDto::Idle,
//!     enabled: true,
//!     tcp_port: None,
//!     udp_port: None,
//!     peer_count: 0,
//!     last_error: None,
//! };
//!
//! assert_eq!(status.protocol, "airplay");
//! ```

use phairplay_core::config::AppConfig;
use phairplay_core::metrics::StreamStats;
use phairplay_core::protocol::{ProtocolKind, ProtocolState, ProtocolStatus};
use phairplay_core::session::SessionSnapshot;
use phairplay_ffi::dto::protocol::FfiProtocolState;
use serde::{Deserialize, Serialize};

/// The state of a protocol engine, as the dashboard shows it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolStateDto {
    /// The engine has been constructed but not started.
    Idle,
    /// The engine is starting up.
    Starting,
    /// The engine is running and advertising.
    Advertising,
    /// The engine has a peer connected and is streaming.
    Streaming,
    /// The engine is shutting down.
    Stopping,
    /// The engine is stopped.
    Stopped,
    /// The engine failed to start or died unexpectedly.
    Failed,
}

impl From<ProtocolState> for ProtocolStateDto {
    fn from(state: ProtocolState) -> Self {
        match state {
            ProtocolState::Idle => ProtocolStateDto::Idle,
            ProtocolState::Starting => ProtocolStateDto::Starting,
            ProtocolState::Advertising => ProtocolStateDto::Advertising,
            ProtocolState::Streaming => ProtocolStateDto::Streaming,
            ProtocolState::Stopping => ProtocolStateDto::Stopping,
            ProtocolState::Stopped => ProtocolStateDto::Stopped,
            ProtocolState::Failed => ProtocolStateDto::Failed,
        }
    }
}

impl From<FfiProtocolState> for ProtocolStateDto {
    /// Translates the state that arrived over the native bridge.
    ///
    /// WHY: the dashboard must not reach into the bridge types when it renders
    /// a card. It translates once, here, and works with its own vocabulary
    /// afterwards.
    fn from(state: FfiProtocolState) -> Self {
        match state {
            FfiProtocolState::Idle => ProtocolStateDto::Idle,
            FfiProtocolState::Starting => ProtocolStateDto::Starting,
            FfiProtocolState::Advertising => ProtocolStateDto::Advertising,
            FfiProtocolState::Streaming => ProtocolStateDto::Streaming,
            FfiProtocolState::Stopping => ProtocolStateDto::Stopping,
            FfiProtocolState::Stopped => ProtocolStateDto::Stopped,
            FfiProtocolState::Failed => ProtocolStateDto::Failed,
        }
    }
}

/// The status of one protocol.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProtocolStatusDto {
    /// The stable name of the protocol, for example "airplay".
    pub protocol: String,
    /// The human-readable product name.
    pub display_name: String,
    /// The current state of the engine.
    pub state: ProtocolStateDto,
    /// Whether the protocol is enabled in the configuration.
    pub enabled: bool,
    /// The TCP port the engine listens on, if it listens on TCP.
    pub tcp_port: Option<u16>,
    /// The UDP port the engine listens on, if it listens on UDP.
    pub udp_port: Option<u16>,
    /// The number of peers currently connected.
    pub peer_count: u32,
    /// The reason of the last state change, if there was one.
    pub last_error: Option<String>,
}

impl ProtocolStatusDto {
    /// Builds the status of one protocol.
    #[must_use]
    pub fn new(status: &ProtocolStatus) -> Self {
        ProtocolStatusDto {
            protocol: status.protocol.as_str().to_string(),
            display_name: status.protocol.display_name().to_string(),
            state: status.state.into(),
            enabled: status.enabled,
            tcp_port: status.tcp_port,
            udp_port: status.udp_port,
            peer_count: status.peer_count,
            last_error: status.last_error.clone(),
        }
    }
}

/// The counters of one protocol.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StreamStatsDto {
    /// The number of video frames delivered to the decoder.
    pub video_frames: u64,
    /// The number of video frames that were dropped.
    pub video_frames_dropped: u64,
    /// The number of audio packets delivered to the decoder.
    pub audio_packets: u64,
    /// The number of audio packets that were dropped.
    pub audio_packets_dropped: u64,
    /// The number of bytes received from the peer.
    pub bytes_received: u64,
    /// The number of bytes sent to the peer.
    pub bytes_sent: u64,
    /// The number of packets the sender reported as lost.
    pub packets_lost: u64,
    /// The number of frames the sender reported as skipped.
    pub frames_skipped: u64,
    /// The share of the video frames that were dropped, between 0.0 and 1.0.
    pub drop_ratio: f64,
}

impl From<&StreamStats> for StreamStatsDto {
    fn from(stats: &StreamStats) -> Self {
        let total = stats.video_frames + stats.video_frames_dropped;
        let drop_ratio = if total == 0 {
            0.0
        } else {
            stats.video_frames_dropped as f64 / total as f64
        };
        StreamStatsDto {
            video_frames: stats.video_frames,
            video_frames_dropped: stats.video_frames_dropped,
            audio_packets: stats.audio_packets,
            audio_packets_dropped: stats.audio_packets_dropped,
            bytes_received: stats.bytes_received,
            bytes_sent: stats.bytes_sent,
            packets_lost: stats.packets_lost,
            frames_skipped: stats.frames_skipped,
            drop_ratio,
        }
    }
}

/// The counters of one protocol, with the name of the protocol.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProtocolMetricsDto {
    /// The stable name of the protocol.
    ///
    /// WHY: an owned string, not a `&'static str` — the dashboard reads the
    /// answer back in, and a borrowed constant cannot be deserialised.
    pub protocol: String,
    /// The counters of the protocol.
    pub stats: StreamStatsDto,
}

/// A live session of the receiver.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionDto {
    /// The identifier of the session.
    pub id: u64,
    /// The protocol that carries the session.
    pub protocol: String,
    /// The display name of the sending device, if it announced one.
    pub sender_name: Option<String>,
    /// The address of the sending device.
    pub sender_address: Option<String>,
    /// The negotiated video format, if the session carries video.
    pub video: Option<String>,
    /// The negotiated audio format, if the session carries audio.
    pub audio: Option<String>,
    /// How long the session has been running, in milliseconds.
    pub uptime_ms: u64,
}

impl SessionDto {
    /// Builds the description of one session.
    #[must_use]
    pub fn new(session: &SessionSnapshot) -> Self {
        SessionDto {
            id: session.id.0,
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
            uptime_ms: session.started_at.elapsed().as_millis() as u64,
        }
    }
}

/// The answer of `GET /api/status`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatusResponse {
    /// The status of every protocol.
    pub protocols: Vec<ProtocolStatusDto>,
    /// The number of live sessions.
    pub active_sessions: usize,
    /// How long the receiver has been running, in milliseconds.
    pub uptime_ms: u64,
}

/// The answer of `GET /api/sessions`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionsResponse {
    /// The live sessions.
    pub sessions: Vec<SessionDto>,
}

/// The answer of `GET /api/metrics`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricsResponse {
    /// The counters of every protocol.
    pub protocols: Vec<ProtocolMetricsDto>,
}

/// The answer of `GET /api/config` and of `PUT /api/config`.
///
/// WHY: The configuration is the configuration of the core, in JSON. The
/// dashboard therefore shows exactly the values that the receiver uses, and
/// a value that the dashboard does not know is rejected instead of being
/// dropped.
pub type ConfigResponse = AppConfig;

/// The answer of `GET /api/build-info`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuildInfoResponse {
    /// The version of the workspace.
    pub version: String,
    /// The name of the workspace.
    pub name: String,
    /// The licence identifier of the product.
    pub license: String,
    /// The repository the source lives in.
    pub repository: String,
    /// The Rust compiler version the binary was compiled with.
    pub rustc_version: String,
    /// The target triple the binary was compiled for.
    pub target: String,
}

impl From<&phairplay_core::build_info::BuildInfo> for BuildInfoResponse {
    fn from(info: &phairplay_core::build_info::BuildInfo) -> Self {
        BuildInfoResponse {
            version: info.version.to_string(),
            name: info.name.to_string(),
            license: info.license.to_string(),
            repository: info.repository.to_string(),
            rustc_version: info.rustc_version.to_string(),
            target: info.target.to_string(),
        }
    }
}

/// The answer of the two protocol actions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProtocolActionResponse {
    /// The status of the protocol after the action.
    pub status: ProtocolStatusDto,
}

/// The body of every failure of the API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorBody {
    /// The stable numeric code of the failure.
    pub code: i32,
    /// The human-readable explanation of the failure.
    pub message: String,
}

/// The answer of every failure of the API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// The failure.
    pub error: ErrorBody,
}

/// An event of the WebSocket channel.
///
/// WHY: The dashboard shows the live sessions. It must be able to tell a
/// new session from a changed one and from an ended one without parsing a
/// free-form text, so every event carries its kind in the field `type`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebEvent {
    /// A peer connected and a session began.
    SessionStarted {
        /// The new session.
        session: SessionDto,
    },
    /// The session ended.
    SessionEnded {
        /// The session that ended.
        session_id: u64,
        /// Why it ended.
        reason: String,
    },
    /// The negotiated video format changed.
    VideoFormatChanged {
        /// The session the format belongs to.
        session_id: u64,
        /// The new format.
        format: String,
    },
    /// The negotiated audio format changed.
    AudioFormatChanged {
        /// The session the format belongs to.
        session_id: u64,
        /// The new format.
        format: String,
    },
    /// The playback state of the sender changed.
    PlaybackStateChanged {
        /// The session the change belongs to.
        session_id: u64,
        /// The new state.
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
    /// The artwork changed.
    ArtworkChanged {
        /// The session the artwork belongs to.
        session_id: u64,
        /// The number of bytes of the artwork.
        ///
        /// WHY: The artwork is at most eight megabytes. The dashboard only
        /// shows that there is one, so the bytes stay in the receiver.
        artwork_bytes: usize,
    },
    /// The status of a protocol changed.
    ProtocolStatusChanged {
        /// The new status of the protocol.
        status: ProtocolStatusDto,
    },
}

/// The name of a protocol, as the API accepts it.
#[must_use]
pub fn protocol_name(kind: ProtocolKind) -> &'static str {
    kind.as_str()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;
    use phairplay_core::media::{AudioFormat, VideoFormat};
    use phairplay_core::session::SessionId;
    use std::time::Instant;

    #[test]
    fn the_status_is_serialised_with_a_readable_state() {
        let status = ProtocolStatus {
            protocol: ProtocolKind::AirPlay,
            state: ProtocolState::Advertising,
            enabled: true,
            tcp_port: Some(7000),
            udp_port: None,
            peer_count: 1,
            last_error: None,
        };
        let dto = ProtocolStatusDto::new(&status);
        let text = serde_json::to_string(&dto).expect("the status must be serialisable");
        assert!(text.contains("\"state\":\"advertising\""), "{text}");
        assert!(text.contains("\"protocol\":\"airplay\""), "{text}");
        assert!(text.contains("\"tcp_port\":7000"), "{text}");
    }

    #[test]
    fn the_session_is_serialised_with_its_formats() {
        let session = SessionSnapshot {
            id: SessionId(7),
            protocol: ProtocolKind::Cast,
            sender_name: Some("Living Room".into()),
            sender_address: Some("192.168.1.50".into()),
            video: Some(VideoFormat {
                codec: phairplay_core::capabilities::VideoCodec::H264,
                width: 1920,
                height: 1080,
                frame_rate: Some(30),
                sps: None,
                pps: None,
            }),
            audio: Some(AudioFormat {
                codec: phairplay_core::capabilities::AudioCodec::AacLc,
                sample_rate: 44100,
                channels: 2,
                samples_per_packet: 1024,
            }),
            started_at: Instant::now(),
        };
        let dto = SessionDto::new(&session);
        let text = serde_json::to_string(&dto).expect("the session must be serialisable");
        assert!(text.contains("\"id\":7"), "{text}");
        assert!(text.contains("video/avc 1920x1080"), "{text}");
        assert!(text.contains("audio/mp4a-latm 44100 Hz 2 ch"), "{text}");
    }

    #[test]
    fn the_event_carries_its_kind() {
        let event = WebEvent::SessionEnded {
            session_id: 3,
            reason: "sender disconnected".into(),
        };
        let text = serde_json::to_string(&event).expect("the event must be serialisable");
        assert!(text.contains("\"type\":\"session_ended\""), "{text}");
        assert!(text.contains("\"session_id\":3"), "{text}");
    }

    #[test]
    fn the_counters_of_a_quiet_receiver_report_no_loss() {
        let dto = StreamStatsDto::from(&StreamStats::default());
        assert_eq!(dto.drop_ratio, 0.0);
    }
}
