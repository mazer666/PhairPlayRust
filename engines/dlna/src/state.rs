//! The state of the media renderer: the transport, the rendering control and
//! the single session slot.
//!
//! WHY: a UPnP media renderer has exactly one transport and exactly one
//! volume. A control point may therefore believe that it is the only one
//! — and the renderer has to make that true: while a stream is running, a
//! *second* control point must be **rejected**, not queued. A queued second
//! control point would silently steal the stream ten seconds later, and the
//! user would see the picture change without touching anything.
//!
//! HOW: the state lives behind one mutex, so that every mutation is atomic
//! and the state that a control point reads through `GetTransportInfo` is
//! the state that the renderer actually has.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_dlna::state::{RendererState, TransportState};
//! use std::net::SocketAddr;
//!
//! let state = RendererState::new();
//! let peer: SocketAddr = "192.168.1.20:50000".parse().expect("an address");
//! state.claim_session(peer).expect("the first peer gets the slot");
//! ```

use std::net::SocketAddr;
use std::sync::Mutex;
use std::time::Duration;

/// The transport state of the renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportState {
    /// Nothing is playing.
    Stopped,
    /// A stream is playing.
    Playing,
    /// A stream is paused.
    PausedPlayback,
    /// The renderer is changing to another stream.
    Transitioning,
    /// No media has been set.
    NoMediaPresent,
}

impl TransportState {
    /// The name the UPnP specification uses.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Stopped => "STOPPED",
            Self::Playing => "PLAYING",
            Self::PausedPlayback => "PAUSED_PLAYBACK",
            Self::Transitioning => "TRANSITIONING",
            Self::NoMediaPresent => "NO_MEDIA_PRESENT",
        }
    }

    /// Whether a stream is currently occupying the renderer.
    #[must_use]
    pub const fn is_active(self) -> bool {
        matches!(
            self,
            Self::Playing | Self::PausedPlayback | Self::Transitioning
        )
    }
}

/// The volume the renderer starts with.
pub const DEFAULT_VOLUME: u8 = 50;

/// The maximum volume a control point may ask for.
pub const MAX_VOLUME: u8 = 100;

/// The reasons a session could not be claimed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionError {
    /// Another control point is already streaming.
    AlreadyActive,
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyActive => {
                write!(f, "another control point is already using this renderer")
            }
        }
    }
}

impl std::error::Error for SessionError {}

/// The control point that owns the session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Session {
    /// The identifier of the session.
    pub id: u64,
    /// The address of the control point.
    pub peer: SocketAddr,
}

/// The mutable state of the renderer.
#[derive(Debug)]
struct Inner {
    transport: TransportState,
    current_uri: String,
    current_uri_metadata: String,
    next_uri: String,
    position: Duration,
    duration: Duration,
    volume: u8,
    muted: bool,
    session: Option<Session>,
    session_counter: u64,
}

/// The state of the media renderer.
#[derive(Debug)]
pub struct RendererState {
    inner: Mutex<Inner>,
}

impl Default for RendererState {
    fn default() -> Self {
        Self::new()
    }
}

impl RendererState {
    /// A renderer that is stopped, at the default volume and not muted.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(Inner {
                transport: TransportState::NoMediaPresent,
                current_uri: String::new(),
                current_uri_metadata: String::new(),
                next_uri: String::new(),
                position: Duration::ZERO,
                duration: Duration::ZERO,
                volume: DEFAULT_VOLUME,
                muted: false,
                session: None,
                session_counter: 0,
            }),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// The current transport state.
    #[must_use]
    pub fn transport_state(&self) -> TransportState {
        self.lock().transport
    }

    /// The URI of the current media.
    #[must_use]
    pub fn current_uri(&self) -> String {
        self.lock().current_uri.clone()
    }

    /// The metadata of the current media.
    #[must_use]
    pub fn current_uri_metadata(&self) -> String {
        self.lock().current_uri_metadata.clone()
    }

    /// The URI of the next media.
    #[must_use]
    pub fn next_uri(&self) -> String {
        self.lock().next_uri.clone()
    }

    /// The playback position.
    #[must_use]
    pub fn position(&self) -> Duration {
        self.lock().position
    }

    /// The duration of the current media.
    #[must_use]
    pub fn duration(&self) -> Duration {
        self.lock().duration
    }

    /// The output volume, 0 to 100.
    #[must_use]
    pub fn volume(&self) -> u8 {
        self.lock().volume
    }

    /// Whether the output is muted.
    #[must_use]
    pub fn muted(&self) -> bool {
        self.lock().muted
    }

    /// The session that currently owns the renderer.
    #[must_use]
    pub fn session(&self) -> Option<Session> {
        self.lock().session
    }

    /// Claims the single session slot for a control point.
    ///
    /// # Errors
    /// Returns [`SessionError::AlreadyActive`] when another control point is
    /// streaming. The second peer is *rejected*; it is never queued.
    pub fn claim_session(&self, peer: SocketAddr) -> Result<u64, SessionError> {
        let mut inner = self.lock();
        if let Some(session) = inner.session {
            if session.peer == peer {
                return Ok(session.id);
            }
            if inner.transport.is_active() {
                return Err(SessionError::AlreadyActive);
            }
        }
        inner.session_counter += 1;
        let id = inner.session_counter;
        inner.session = Some(Session { id, peer });
        Ok(id)
    }

    /// Releases the session slot when the given peer owns it.
    pub fn release_session(&self, peer: SocketAddr) -> bool {
        let mut inner = self.lock();
        match inner.session {
            Some(session) if session.peer == peer => {
                inner.session = None;
                true
            }
            _ => false,
        }
    }

    /// Sets the media the renderer shall play.
    pub fn set_media(&self, uri: &str, metadata: &str) {
        let mut inner = self.lock();
        inner.current_uri = uri.to_string();
        inner.current_uri_metadata = metadata.to_string();
        inner.position = Duration::ZERO;
        inner.duration = Duration::ZERO;
        inner.transport = TransportState::Stopped;
    }

    /// Sets the media the renderer shall play next.
    pub fn set_next_media(&self, uri: &str, metadata: &str) {
        let mut inner = self.lock();
        inner.next_uri = uri.to_string();
        if metadata.is_empty() {
            inner.current_uri_metadata = inner.current_uri_metadata.clone();
        }
    }

    /// Moves the transport into the given state.
    pub fn set_transport_state(&self, transport: TransportState) {
        self.lock().transport = transport;
    }

    /// Moves the playback position.
    ///
    /// # Errors
    /// Returns [`SessionError::AlreadyActive`] — the only error the state
    /// machine has — never. The position is clamped to the duration when a
    /// duration is known.
    pub fn set_position(&self, position: Duration) {
        let mut inner = self.lock();
        let duration = inner.duration;
        inner.position = if duration.is_zero() {
            position
        } else {
            position.min(duration)
        };
    }

    /// Sets the output volume.
    ///
    /// A value above [`MAX_VOLUME`] is clamped, a value that is not a number
    /// is ignored by the caller before it gets here.
    pub fn set_volume(&self, volume: u8) {
        self.lock().volume = volume.min(MAX_VOLUME);
    }

    /// Sets the mute state.
    pub fn set_muted(&self, muted: bool) {
        self.lock().muted = muted;
    }

    /// Stops the transport and releases the session slot.
    pub fn stop(&self) {
        let mut inner = self.lock();
        inner.transport = if inner.current_uri.is_empty() {
            TransportState::NoMediaPresent
        } else {
            TransportState::Stopped
        };
        inner.position = Duration::ZERO;
        inner.session = None;
    }
}

#[cfg(test)]
mod tests;
