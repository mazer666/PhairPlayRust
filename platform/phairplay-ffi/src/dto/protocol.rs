//! The protocol status as it travels over the bridge.
//!
//! WHAT: The kind of a protocol, the state of its engine and the status
//! snapshot of one protocol.
//!
//! WHY: The dashboard and the native shells show the same four protocols in
//! the same order with the same states. If the bridge handed out free-form
//! strings, every host would grow its own spelling of "advertising", and
//! the states of the receiver and of the dashboard would drift apart.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_ffi::dto::FfiProtocolKind;
//! use phairplay_core::protocol::ProtocolKind;
//!
//! assert_eq!(FfiProtocolKind::from(ProtocolKind::AirPlay), FfiProtocolKind::AirPlay);
//! ```

use phairplay_core::protocol::{ProtocolKind, ProtocolState, ProtocolStatus};

/// The receiver protocol a status belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, uniffi::Enum)]
pub enum FfiProtocolKind {
    /// AirPlay 2 — screen mirroring and audio streaming.
    AirPlay,
    /// Miracast / Wi-Fi Display — screen mirroring.
    Miracast,
    /// Google Cast — media playback and mirroring.
    Cast,
    /// DLNA / UPnP MediaRenderer — media playback.
    Dlna,
    /// Samsung Smart View — the MultiScreen Framework.
    SmartView,
}

impl FfiProtocolKind {
    /// The stable lowercase name of the protocol, as used in the
    /// configuration and in the HTTP API.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            FfiProtocolKind::AirPlay => "airplay",
            FfiProtocolKind::Miracast => "miracast",
            FfiProtocolKind::Cast => "cast",
            FfiProtocolKind::Dlna => "dlna",
            FfiProtocolKind::SmartView => "smartview",
        }
    }

    /// The human-readable product name.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            FfiProtocolKind::AirPlay => "AirPlay 2",
            FfiProtocolKind::Miracast => "Miracast",
            FfiProtocolKind::Cast => "Google Cast",
            FfiProtocolKind::Dlna => "DLNA / UPnP",
            FfiProtocolKind::SmartView => "Samsung Smart View",
        }
    }
}

impl From<ProtocolKind> for FfiProtocolKind {
    fn from(kind: ProtocolKind) -> Self {
        match kind {
            ProtocolKind::AirPlay => FfiProtocolKind::AirPlay,
            ProtocolKind::Miracast => FfiProtocolKind::Miracast,
            ProtocolKind::Cast => FfiProtocolKind::Cast,
            ProtocolKind::Dlna => FfiProtocolKind::Dlna,
            ProtocolKind::SmartView => FfiProtocolKind::SmartView,
        }
    }
}

impl From<FfiProtocolKind> for ProtocolKind {
    fn from(kind: FfiProtocolKind) -> Self {
        match kind {
            FfiProtocolKind::AirPlay => ProtocolKind::AirPlay,
            FfiProtocolKind::Miracast => ProtocolKind::Miracast,
            FfiProtocolKind::Cast => ProtocolKind::Cast,
            FfiProtocolKind::Dlna => ProtocolKind::Dlna,
            FfiProtocolKind::SmartView => ProtocolKind::SmartView,
        }
    }
}

/// The lifecycle state of a protocol engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum FfiProtocolState {
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

impl From<ProtocolState> for FfiProtocolState {
    fn from(state: ProtocolState) -> Self {
        match state {
            ProtocolState::Idle => FfiProtocolState::Idle,
            ProtocolState::Starting => FfiProtocolState::Starting,
            ProtocolState::Advertising => FfiProtocolState::Advertising,
            ProtocolState::Streaming => FfiProtocolState::Streaming,
            ProtocolState::Stopping => FfiProtocolState::Stopping,
            ProtocolState::Stopped => FfiProtocolState::Stopped,
            ProtocolState::Failed => FfiProtocolState::Failed,
        }
    }
}

impl From<FfiProtocolState> for ProtocolState {
    fn from(state: FfiProtocolState) -> Self {
        match state {
            FfiProtocolState::Idle => ProtocolState::Idle,
            FfiProtocolState::Starting => ProtocolState::Starting,
            FfiProtocolState::Advertising => ProtocolState::Advertising,
            FfiProtocolState::Streaming => ProtocolState::Streaming,
            FfiProtocolState::Stopping => ProtocolState::Stopping,
            FfiProtocolState::Stopped => ProtocolState::Stopped,
            FfiProtocolState::Failed => ProtocolState::Failed,
        }
    }
}

/// The public state of one protocol engine.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct FfiProtocolStatus {
    /// Which protocol this status describes.
    pub protocol: FfiProtocolKind,
    /// The current lifecycle state.
    pub state: FfiProtocolState,
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

impl From<&ProtocolStatus> for FfiProtocolStatus {
    fn from(status: &ProtocolStatus) -> Self {
        FfiProtocolStatus {
            protocol: status.protocol.into(),
            state: status.state.into(),
            enabled: status.enabled,
            tcp_port: status.tcp_port,
            udp_port: status.udp_port,
            peer_count: status.peer_count,
            last_error: status.last_error.clone(),
        }
    }
}

impl From<ProtocolStatus> for FfiProtocolStatus {
    fn from(status: ProtocolStatus) -> Self {
        FfiProtocolStatus::from(&status)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    #[test]
    fn the_protocol_kind_survives_the_conversion() {
        for kind in ProtocolKind::ALL {
            assert_eq!(ProtocolKind::from(FfiProtocolKind::from(kind)), kind);
        }
    }

    #[test]
    fn the_protocol_state_survives_the_conversion() {
        let states = [
            ProtocolState::Idle,
            ProtocolState::Starting,
            ProtocolState::Advertising,
            ProtocolState::Streaming,
            ProtocolState::Stopping,
            ProtocolState::Stopped,
            ProtocolState::Failed,
        ];
        for state in states {
            assert_eq!(ProtocolState::from(FfiProtocolState::from(state)), state);
        }
    }

    #[test]
    fn the_status_survives_the_conversion() {
        let status = ProtocolStatus {
            protocol: ProtocolKind::Miracast,
            state: ProtocolState::Streaming,
            enabled: true,
            tcp_port: Some(5540),
            udp_port: None,
            peer_count: 2,
            last_error: Some("a peer was rejected".into()),
        };
        let converted = FfiProtocolStatus::from(&status);
        assert_eq!(converted.protocol, FfiProtocolKind::Miracast);
        assert_eq!(converted.state, FfiProtocolState::Streaming);
        assert_eq!(converted.tcp_port, Some(5540));
        assert_eq!(converted.udp_port, None);
        assert_eq!(converted.peer_count, 2);
        assert_eq!(converted.last_error.as_deref(), Some("a peer was rejected"));
    }

    #[test]
    fn the_names_of_the_protocols_are_the_names_of_the_api() {
        // The HTTP API of the dashboard uses the same names.
        assert_eq!(FfiProtocolKind::AirPlay.as_str(), "airplay");
        assert_eq!(FfiProtocolKind::Miracast.as_str(), "miracast");
        assert_eq!(FfiProtocolKind::Cast.as_str(), "cast");
        assert_eq!(FfiProtocolKind::Dlna.as_str(), "dlna");
    }
}
