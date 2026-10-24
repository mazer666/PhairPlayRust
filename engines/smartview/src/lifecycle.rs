//! The public state of the Smart View engine.
//!
//! WHY: the status of the engine has to be readable from any thread while
//! a transition is in flight. The state therefore lives behind one mutex
//! and is copied out for the status snapshot — a caller never holds a lock
//! of the engine.
//!
//! HOW: the lifecycle knows the state, the ports and the reason of the last
//! failure. It knows nothing about sockets: the engine owns those.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_core::protocol::ProtocolState;
//! use phairplay_smartview::lifecycle::Lifecycle;
//!
//! let lifecycle = Lifecycle::new();
//! assert_eq!(lifecycle.state(), ProtocolState::Idle);
//! ```

use phairplay_core::protocol::ProtocolState;

/// The public state of the engine.
///
/// WHY: the status has to be readable from any thread while a transition is
/// in flight. The state therefore lives behind one mutex and is copied out
/// for the status snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lifecycle {
    state: ProtocolState,
    tcp_port: Option<u16>,
    udp_port: Option<u16>,
    last_error: Option<String>,
    enabled: bool,
}

impl Default for Lifecycle {
    fn default() -> Self {
        Self::new()
    }
}

impl Lifecycle {
    /// The state of a receiver that has not been started.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: ProtocolState::Idle,
            tcp_port: None,
            udp_port: None,
            last_error: None,
            enabled: true,
        }
    }

    /// The current protocol state.
    #[must_use]
    pub fn state(&self) -> ProtocolState {
        self.state
    }

    /// Sets the protocol state.
    pub fn set_state(&mut self, state: ProtocolState) {
        self.state = state;
    }

    /// The TCP port of the API server.
    #[must_use]
    pub fn tcp_port(&self) -> Option<u16> {
        self.tcp_port
    }

    /// Sets the TCP port of the API server.
    pub fn set_tcp_port(&mut self, port: Option<u16>) {
        self.tcp_port = port;
    }

    /// The UDP port of the SSDP responder.
    #[must_use]
    pub fn udp_port(&self) -> Option<u16> {
        self.udp_port
    }

    /// Sets the UDP port of the SSDP responder.
    pub fn set_udp_port(&mut self, port: Option<u16>) {
        self.udp_port = port;
    }

    /// The reason of the last failure.
    #[must_use]
    pub fn last_error(&self) -> Option<String> {
        self.last_error.clone()
    }

    /// Records the reason of the last failure.
    pub fn set_last_error(&mut self, reason: Option<String>) {
        self.last_error = reason;
    }

    /// Whether the protocol is enabled.
    #[must_use]
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Sets whether the protocol is enabled.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

#[cfg(test)]
mod tests;
