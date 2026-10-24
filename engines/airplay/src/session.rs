//! The state of one AirPlay session.
//!
//! WHY: an AirPlay session has more state than a socket: the announced SDP,
//! the negotiated formats, the pairing state, the FairPlay state, the volume,
//! the reverse-control identity and the set of active streams. All of it
//! belongs to *one* peer — the receiver serves a single sender at a time — so
//! it lives in one struct that the connection task owns.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_airplay::session::AirPlaySession;
//!
//! let session = AirPlaySession::new();
//! assert!(!session.has_video());
//! assert!(!session.is_streaming());
//! ```

use std::collections::BTreeSet;
use std::sync::Arc;

use phairplay_core::media::{AudioFormat, VideoFormat};
use phairplay_core::session::SessionId;

use crate::dacp::DacpClient;
use crate::fairplay::FairPlaySession;
use crate::pairing::identity::PairingIdentity;
use crate::pairing::legacy::LegacyPinPairing;
use crate::pairing::verify::PairingSession;
use crate::sdp::SdpSession;

/// The stream type of the realtime audio stream.
pub const STREAM_TYPE_AUDIO: u8 = 96;

/// The stream type of the buffered audio stream.
pub const STREAM_TYPE_BUFFERED_AUDIO: u8 = 103;

/// The stream type of the video stream.
pub const STREAM_TYPE_VIDEO: u8 = 110;

/// The state of the URL playback mode.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PlaybackState {
    /// The media URL the sender asked to play.
    pub url: Option<String>,
    /// The position the playback starts at, as a fraction of the duration.
    pub start_fraction: f64,
    /// The playback rate; `0.0` means paused.
    pub rate: f32,
    /// The current position in seconds.
    pub position_sec: f64,
    /// The duration of the media in seconds.
    pub duration_sec: f64,
    /// Whether the receiver is ready to play.
    pub ready: bool,
}

impl PlaybackState {
    /// Whether the receiver has something to play.
    #[must_use]
    pub fn has_media(&self) -> bool {
        self.url.is_some()
    }
}

/// The state of one AirPlay session.
#[derive(Debug)]
pub struct AirPlaySession {
    /// The identifier of the session.
    pub id: SessionId,
    /// The name the sender announced.
    pub sender_name: Option<String>,
    /// The SDP the sender announced.
    pub announced: Option<SdpSession>,
    /// The number of `SETUP` requests the sender sent.
    pub setup_count: u32,
    /// Whether the session is a mirroring session (AirPlay 2, no SDP).
    pub mirror: bool,
    /// The stream types that are currently active.
    pub active_stream_types: BTreeSet<u8>,
    /// The volume the sender set, in decibel.
    pub volume: f32,
    /// The reverse control channel.
    pub dacp: DacpClient,
    /// The pairing state of the connection.
    pub pairing: PairingSession,
    /// The FairPlay state of the connection.
    pub fairplay: FairPlaySession,
    /// The legacy PIN pairing, when the sender started one.
    pub legacy_pin: Option<LegacyPinPairing>,
    /// Whether the sender proved that it knows the PIN.
    pub pin_paired: bool,
    /// Whether the sender may stream.
    pub authorized: bool,
    /// Whether the session has started streaming.
    pub streaming: bool,
    /// The negotiated video format.
    pub video: Option<VideoFormat>,
    /// The negotiated audio format.
    pub audio: Option<AudioFormat>,
    /// The shared secret of the pairing exchange.
    pub shared_secret: Option<[u8; 32]>,
    /// The state of the URL playback mode.
    pub playback: PlaybackState,
    /// The PIN the receiver shows, if the PIN flow is running.
    pub shown_pin: Option<String>,
}

impl AirPlaySession {
    /// Creates the state of a new session.
    #[must_use]
    pub fn new() -> Self {
        Self::with_identity(Arc::new(PairingIdentity::from_seed(&[0u8; 32])))
    }

    /// Creates the state of a new session for the given receiver identity.
    #[must_use]
    pub fn with_identity(identity: Arc<PairingIdentity>) -> Self {
        Self {
            id: SessionId::next(),
            sender_name: None,
            announced: None,
            setup_count: 0,
            mirror: false,
            active_stream_types: BTreeSet::new(),
            volume: 0.0,
            dacp: DacpClient::new(),
            pairing: PairingSession::new(identity),
            fairplay: FairPlaySession::new(),
            legacy_pin: None,
            pin_paired: false,
            authorized: false,
            streaming: false,
            video: None,
            audio: None,
            shared_secret: None,
            playback: PlaybackState::default(),
            shown_pin: None,
        }
    }

    /// Whether the session carries video.
    #[must_use]
    pub fn has_video(&self) -> bool {
        self.announced.as_ref().is_some_and(|sdp| sdp.has_video)
    }

    /// Whether the session carries audio.
    #[must_use]
    pub fn has_audio(&self) -> bool {
        self.announced.as_ref().is_some_and(|sdp| sdp.has_audio)
    }

    /// Whether the session is streaming.
    #[must_use]
    pub fn is_streaming(&self) -> bool {
        self.streaming
    }

    /// The name of the sender, or a generic one.
    #[must_use]
    pub fn sender_name(&self) -> &str {
        self.sender_name.as_deref().unwrap_or("AirPlay Sender")
    }

    /// Registers a stream type as active.
    pub fn add_stream_type(&mut self, stream_type: u8) {
        self.active_stream_types.insert(stream_type);
    }

    /// Removes a stream type from the active ones.
    pub fn remove_stream_type(&mut self, stream_type: u8) {
        self.active_stream_types.remove(&stream_type);
    }

    /// Whether no stream is active any more.
    #[must_use]
    pub fn has_no_active_streams(&self) -> bool {
        self.active_stream_types.is_empty()
    }
}

impl Default for AirPlaySession {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
