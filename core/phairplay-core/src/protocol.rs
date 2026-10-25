//! The protocol abstraction that every receiver engine implements.
//!
//! WHY: PhairPlayRust speaks five receiver protocols — AirPlay 2, Miracast,
//! Google Cast, DLNA/UPnP and Samsung Smart View. They share a shape: advertise a
//! service, accept a peer, negotiate, stream media, tear down. Encoding that shape once
//! as a trait means the session registry, the metrics, the capability
//! advertisement and the native bridge all work the same way regardless of
//! which protocol is running.
//!
//! HOW: an engine is a state machine driven by [`ProtocolEngine::start`] and
//! [`ProtocolEngine::stop`]. It reports its state through
//! [`ProtocolEngine::status`] and reports *what happened* through the event
//! channel in [`EngineContext`]. The core never calls into an engine's
//! protocol internals — it only knows the trait.

use std::sync::Arc;

use crate::capabilities::MediaCapabilities;
use crate::error::EngineError;
use crate::identity::DeviceIdentity;
use crate::media::{AudioSink, VideoSink};
use crate::session::SessionEvent;

/// The receiver protocols PhairPlayRust implements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ProtocolKind {
    /// AirPlay 2 — screen mirroring and audio streaming (Apple platforms).
    AirPlay,
    /// Miracast / Wi-Fi Display — screen mirroring (Windows, Android).
    Miracast,
    /// Google Cast — media playback and mirroring (Chromecast, Google TV).
    Cast,
    /// DLNA / UPnP MediaRenderer — media playback (Windows "Cast to Device", VLC).
    Dlna,
    /// Samsung Smart View — the MultiScreen Framework (Galaxy phones and tablets).
    SmartView,
}

impl ProtocolKind {
    /// Every protocol, in a stable order.
    pub const ALL: [ProtocolKind; 5] = [
        ProtocolKind::AirPlay,
        ProtocolKind::Miracast,
        ProtocolKind::Cast,
        ProtocolKind::Dlna,
        ProtocolKind::SmartView,
    ];

    /// The stable lowercase identifier used in configuration, metrics and the API.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            ProtocolKind::AirPlay => "airplay",
            ProtocolKind::Miracast => "miracast",
            ProtocolKind::Cast => "cast",
            ProtocolKind::Dlna => "dlna",
            ProtocolKind::SmartView => "smartview",
        }
    }

    /// The human-readable product name.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            ProtocolKind::AirPlay => "AirPlay 2",
            ProtocolKind::Miracast => "Miracast",
            ProtocolKind::Cast => "Google Cast",
            ProtocolKind::Dlna => "DLNA / UPnP",
            ProtocolKind::SmartView => "Samsung Smart View",
        }
    }

    /// The DNS-SD service types this protocol advertises over mDNS.
    ///
    /// WHY: AirPlay requires *two* service types — `_airplay._tcp` for
    /// mirroring and `_raop._tcp` for the audio pipeline — and a receiver that
    /// advertises only the first is invisible to some senders.
    ///
    /// DLNA advertises **nothing** over mDNS: a UPnP renderer is found with
    /// SSDP, and a renderer that also published `_googlecast._tcp` would
    /// show up in the Chromecast picker of every sender as a device that
    /// answers no Cast request.
    ///
    /// The spellings live in [`crate::config::service_types`], which is the
    /// single owner of every service type of the workspace.
    #[must_use]
    pub const fn mdns_service_types(self) -> &'static [&'static str] {
        use crate::config::service_types as types;
        match self {
            ProtocolKind::AirPlay => &[types::AIRPLAY_TCP, types::RAOP_TCP],
            ProtocolKind::Miracast => &[types::MIRACAST_TCP],
            ProtocolKind::Cast => &[types::GOOGLECAST_TCP],
            ProtocolKind::Dlna => &[],
            ProtocolKind::SmartView => &[types::SAMSUNG_MSF_TCP],
        }
    }

    /// Whether this protocol carries a video stream.
    #[must_use]
    pub const fn carries_video(self) -> bool {
        matches!(
            self,
            ProtocolKind::AirPlay
                | ProtocolKind::Miracast
                | ProtocolKind::Cast
                | ProtocolKind::SmartView
        )
    }

    /// Whether this protocol carries an audio stream.
    #[must_use]
    pub const fn carries_audio(self) -> bool {
        true
    }
}

impl std::fmt::Display for ProtocolKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for ProtocolKind {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "airplay" | "airplay2" | "airplay-2" => Ok(ProtocolKind::AirPlay),
            "miracast" | "wfd" | "wifi-display" => Ok(ProtocolKind::Miracast),
            "cast" | "googlecast" | "google-cast" => Ok(ProtocolKind::Cast),
            "dlna" | "upnp" | "mediarenderer" => Ok(ProtocolKind::Dlna),
            "smartview" | "smart-view" | "samsung" | "msf" => Ok(ProtocolKind::SmartView),
            other => Err(format!("unknown protocol `{other}`")),
        }
    }
}

/// The lifecycle state of a protocol engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolState {
    /// The engine has been constructed but not started.
    Idle,
    /// The engine is starting up (binding sockets, registering services).
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

impl std::fmt::Display for ProtocolState {
    /// The name of the state, as the dashboard and the command line show it.
    ///
    /// WHY: the state is shown in three places — the panel of the browser, the
    /// line of the command line and the log. A single spelling keeps the
    /// three of them from drifting apart.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            ProtocolState::Idle => "idle",
            ProtocolState::Starting => "starting",
            ProtocolState::Advertising => "advertising",
            ProtocolState::Streaming => "streaming",
            ProtocolState::Stopping => "stopping",
            ProtocolState::Stopped => "stopped",
            ProtocolState::Failed => "failed",
        };
        f.write_str(name)
    }
}

impl ProtocolState {
    /// Whether a peer may connect in this state.
    #[must_use]
    pub const fn accepts_peers(self) -> bool {
        matches!(self, ProtocolState::Advertising | ProtocolState::Streaming)
    }
}

/// A snapshot of one engine's public state.
#[derive(Debug, Clone, PartialEq)]
pub struct ProtocolStatus {
    /// Which protocol this status describes.
    pub protocol: ProtocolKind,
    /// The current lifecycle state.
    pub state: ProtocolState,
    /// Whether the protocol is enabled in the configuration.
    pub enabled: bool,
    /// The TCP port the engine listens on, if it listens on TCP.
    pub tcp_port: Option<u16>,
    /// The UDP port the engine listens on, if it listens on UDP.
    pub udp_port: Option<u16>,
    /// The number of peers currently connected.
    pub peer_count: u32,
    /// A human-readable reason for the last state change.
    pub last_error: Option<String>,
}

/// The environment an engine is started in.
///
/// WHY: an engine must not reach for global state. Everything it is allowed
/// to touch — the device identity, the advertised capabilities, the media
/// sinks, the event channel — is handed to it here, which keeps engines
/// testable without a real network or a real decoder.
#[derive(Clone)]
pub struct EngineContext {
    /// The identity this receiver advertises to senders.
    pub identity: Arc<DeviceIdentity>,
    /// The media capabilities this receiver may advertise.
    pub capabilities: Arc<MediaCapabilities>,
    /// The sink that receives encoded video frames.
    pub video_sink: Arc<dyn VideoSink>,
    /// The sink that receives encoded audio frames.
    pub audio_sink: Arc<dyn AudioSink>,
    /// The channel on which the engine reports session events.
    pub events: tokio::sync::mpsc::Sender<SessionEvent>,
}

impl std::fmt::Debug for EngineContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EngineContext")
            .field("identity", &self.identity)
            .field("capabilities", &self.capabilities)
            .field("video_sink", &"<dyn VideoSink>")
            .field("audio_sink", &"<dyn AudioSink>")
            .field("events", &"<mpsc::Sender>")
            .finish()
    }
}

/// A protocol receiver engine.
///
/// Implementations are expected to be safe to call [`ProtocolEngine::status`]
/// from any thread at any time, including while the engine is starting or
/// stopping.
#[async_trait::async_trait]
pub trait ProtocolEngine: Send + Sync + std::fmt::Debug {
    /// Which protocol this engine implements.
    fn kind(&self) -> ProtocolKind;

    /// The current public state of the engine.
    fn status(&self) -> ProtocolStatus;

    /// Starts the engine: binds sockets and begins advertising.
    ///
    /// # Errors
    /// Returns [`EngineError::AlreadyRunning`] if the engine is already
    /// running, and [`EngineError::Start`] if a socket could not be bound or
    /// the service could not be registered.
    async fn start(&self, ctx: EngineContext) -> Result<(), EngineError>;

    /// Stops the engine: stops advertising, tears down peer connections and
    /// releases every socket.
    ///
    /// # Errors
    /// Returns [`EngineError::NotRunning`] if the engine was not running.
    async fn stop(&self) -> Result<(), EngineError>;
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    #[test]
    fn protocol_kind_round_trips_through_its_string_form() {
        for kind in ProtocolKind::ALL {
            let text = kind.as_str();
            let parsed: ProtocolKind = text.parse().expect("round trip");
            assert_eq!(kind, parsed);
        }
    }

    #[test]
    fn protocol_kind_accepts_common_aliases() {
        assert_eq!(
            "WFD".parse::<ProtocolKind>().unwrap(),
            ProtocolKind::Miracast
        );
        assert_eq!(
            "googlecast".parse::<ProtocolKind>().unwrap(),
            ProtocolKind::Cast
        );
        assert_eq!("upnp".parse::<ProtocolKind>().unwrap(), ProtocolKind::Dlna);
        assert!("nonsense".parse::<ProtocolKind>().is_err());
    }

    #[test]
    fn airplay_advertises_both_service_types() {
        // A receiver that omits _raop._tcp is invisible to some AirPlay senders.
        let types = ProtocolKind::AirPlay.mdns_service_types();
        assert!(types.contains(&"_airplay._tcp"));
        assert!(types.contains(&"_raop._tcp"));
    }

    #[test]
    fn a_dlna_renderer_advertises_nothing_over_mdns() {
        // WHY: a UPnP renderer is found with SSDP. Publishing the Cast
        // service type as well would put the device into the Chromecast
        // picker of every sender, where it answers no Cast request.
        assert!(ProtocolKind::Dlna.mdns_service_types().is_empty());
    }

    #[test]
    fn every_advertised_type_has_a_single_owner() {
        // WHY: the spellings live in `config::service_types`; the protocol
        // table must not spell one of them out a second time.
        for kind in ProtocolKind::ALL {
            for service_type in kind.mdns_service_types() {
                assert!(
                    crate::config::service_types::is_advertised(service_type),
                    "{kind:?} advertises the unknown type {service_type}"
                );
            }
        }
    }

    #[test]
    fn a_smart_view_receiver_advertises_the_multi_screen_service() {
        // WHY: a Galaxy phone looks for exactly this type when the user
        // opens Smart View. A receiver that publishes another type is
        // invisible to every Samsung sender.
        assert_eq!(
            ProtocolKind::SmartView.mdns_service_types(),
            &["_samsungmsf._tcp"]
        );
    }

    #[test]
    fn only_streaming_protocols_carry_video() {
        assert!(ProtocolKind::AirPlay.carries_video());
        assert!(ProtocolKind::Miracast.carries_video());
        assert!(ProtocolKind::Cast.carries_video());
        assert!(ProtocolKind::SmartView.carries_video());
        assert!(!ProtocolKind::Dlna.carries_video());
    }
}
