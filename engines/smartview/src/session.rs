//! The single-session registry of the Smart View receiver.
//!
//! WHAT: the state of the one session a Smart View receiver serves — the
//! peer that announced itself and the channel that is currently open.
//!
//! WHY: Smart View is a single-session protocol: one Galaxy device mirrors
//! onto one television. The build contract demands that a second concurrent
//! connection is **rejected**, not queued — a queued second peer would wait
//! for a session that may never end, and the sender would show a device
//! that hangs.
//!
//! HOW: the registry owns a mutex and nothing else. The API server asks it
//! for a ticket before it accepts a WebSocket upgrade, and gives the ticket
//! back when the channel closes.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_smartview::session::SessionRegistry;
//!
//! let registry = SessionRegistry::new();
//! assert!(registry.active().is_none());
//! ```

use std::net::SocketAddr;
use std::sync::Mutex;
use std::time::Instant;

use phairplay_core::protocol::PeerLimit;
use phairplay_core::session::SessionId;

/// What the sender told the receiver about itself.
///
/// WHY: Samsung documents that the sender posts its MAC address, its IP
/// address and its device name to the device after it has read the
/// description document. The three values are kept apart, because only the
/// name is shown to a user and only the address is logged.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PeerInfo {
    /// The MAC address the sender announced.
    pub mac: Option<String>,
    /// The IP address the sender announced.
    pub ip: Option<String>,
    /// The device name the sender announced.
    pub name: Option<String>,
}

impl PeerInfo {
    /// The name of the peer, or a fallback when the sender announced none.
    #[must_use]
    pub fn display_name(&self) -> String {
        self.name
            .clone()
            .filter(|name| !name.trim().is_empty())
            .unwrap_or_else(|| "Unknown sender".to_string())
    }

    /// The peer as a short, log-safe description.
    ///
    /// WHY: the logs of the receiver are read by a human. The MAC address
    /// and the address of the peer are enough to identify a session; the
    /// name of the peer is not needed for that and may be private.
    #[must_use]
    pub fn log_summary(&self) -> String {
        let address = self.ip.as_deref().unwrap_or("unknown address");
        let mac = self.mac.as_deref().unwrap_or("unknown MAC");
        format!("{address} ({mac})")
    }
}

/// An open channel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveSession {
    /// The identifier of the session.
    pub id: SessionId,
    /// What the sender told the receiver about itself.
    pub peer: PeerInfo,
    /// The address of the sender's socket.
    pub address: SocketAddr,
    /// The moment the channel was opened.
    pub started_at: Instant,
}

/// The registry of the one session the receiver serves.
#[derive(Debug, Default)]
pub struct SessionRegistry {
    inner: Mutex<RegistryState>,
}

#[derive(Debug, Default)]
struct RegistryState {
    announced: Option<PeerInfo>,
    active: Option<ActiveSession>,
}

impl SessionRegistry {
    /// An empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records the peer that announced itself.
    ///
    /// WHY: the sender announces itself *before* it opens the channel, so
    /// that the receiver knows the name of the device when the channel
    /// arrives. The announcement is kept even when no channel is open — the
    /// sender may announce itself again before it connects.
    pub fn announce(&self, peer: PeerInfo) {
        let mut state = self.lock();
        state.announced = Some(peer);
    }

    /// The peer that announced itself, if it did.
    #[must_use]
    pub fn announced(&self) -> Option<PeerInfo> {
        self.lock().announced.clone()
    }

    /// Opens the one session of the receiver.
    ///
    /// WHY the limit is an argument and not a constant of this registry: the
    /// rule that says how many peers a protocol serves has **one** owner —
    /// the list `session.multi_peer_protocols` of the configuration — and
    /// the registry asks it. Smart View is the protocol of the list that
    /// serves **one** peer, and the registry has to know that from the
    /// configuration and not from its own memory.
    ///
    /// # Errors
    /// Returns [`crate::error::SmartViewError::Protocol`] when a session is
    /// already open. The second peer is **rejected**, never queued.
    pub fn open(
        &self,
        address: SocketAddr,
        limit: PeerLimit,
    ) -> Result<ActiveSession, crate::error::SmartViewError> {
        let mut state = self.lock();
        if let Some(active) = &state.active {
            if limit.max_peers() <= 1 {
                return Err(crate::error::SmartViewError::Protocol(format!(
                    "a session with {} is already open",
                    active.id
                )));
            }
        }
        let peer = state.announced.clone().unwrap_or_default();
        let session = ActiveSession {
            id: SessionId::next(),
            peer,
            address,
            started_at: Instant::now(),
        };
        state.active = Some(session.clone());
        Ok(session)
    }

    /// Closes the session with the given identifier.
    ///
    /// WHY: the identifier is checked, so that a late close of an old
    /// session cannot release the session of a newer peer.
    ///
    /// Returns the closed session, if it was the open one.
    pub fn close(&self, id: SessionId) -> Option<ActiveSession> {
        let mut state = self.lock();
        if state.active.as_ref().map(|active| active.id) != Some(id) {
            return None;
        }
        state.active.take()
    }

    /// The open session, if there is one.
    #[must_use]
    pub fn active(&self) -> Option<ActiveSession> {
        self.lock().active.clone()
    }

    /// Whether a session is open.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.lock().active.is_some()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, RegistryState> {
        self.inner.lock().unwrap_or_else(|error| error.into_inner())
    }
}

#[cfg(test)]
mod tests;
