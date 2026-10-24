//! The errors of the Smart View engine.
//!
//! WHY: the layers of this crate fail in different ways — the multicast
//! socket does not open, a request is malformed, a handshake is not a
//! handshake. The engine has to be able to say *what* went wrong without
//! string matching on the messages, so every failure carries a reason and a
//! kind.
//!
//! HOW: [`SmartViewError`] is the error of the *internal* layers of the
//! crate. The public engine API speaks the error types of the core
//! ([`phairplay_core::error::EngineError`]), because the build contract
//! forbids a parallel error hierarchy in an engine crate. The mapping
//! happens in [`crate::engine`].
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_smartview::error::SmartViewError;
//!
//! let error = SmartViewError::LimitExceeded("the header block was too large".into());
//! assert!(error.to_string().contains("header block"));
//! ```

/// The failures of the Smart View crate.
#[derive(Debug, thiserror::Error)]
pub enum SmartViewError {
    /// The multicast socket could not be opened or configured.
    #[error("the SSDP socket could not be opened: {0}")]
    Ssdp(String),

    /// The description document could not be built.
    #[error("the description document is invalid: {0}")]
    Description(String),

    /// A request of the API could not be read.
    #[error("the request could not be read: {0}")]
    Request(String),

    /// The WebSocket handshake was refused.
    #[error("the WebSocket handshake was refused: {0}")]
    Handshake(String),

    /// A peer violated the protocol.
    #[error("the peer violated the protocol: {0}")]
    Protocol(String),

    /// A limit of the API was exceeded.
    #[error("a limit was exceeded: {0}")]
    LimitExceeded(String),

    /// The request line or the path was longer than the limit.
    #[error("the request line exceeded the limit of {0} bytes")]
    UriTooLong(usize),

    /// The header block was longer than the limit.
    #[error("the header block exceeded the limit of {0} bytes")]
    HeadersTooLarge(usize),
}

impl SmartViewError {
    /// Whether the failure is the fault of the peer and not of the receiver.
    ///
    /// WHY: the engine reports a *misbehaving peer* differently from a
    /// *broken receiver*. A peer that sends a malformed request is refused
    /// and the receiver keeps running; a socket that cannot be bound stops
    /// the engine.
    #[must_use]
    pub fn is_peer_fault(&self) -> bool {
        matches!(
            self,
            Self::Request(_)
                | Self::Handshake(_)
                | Self::Protocol(_)
                | Self::LimitExceeded(_)
                | Self::UriTooLong(_)
                | Self::HeadersTooLarge(_)
        )
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    #[test]
    fn the_reason_is_part_of_the_message() {
        let error = SmartViewError::Ssdp("the port 1900 is already in use".into());
        let text = error.to_string();
        assert!(text.contains("SSDP socket"), "{text}");
        assert!(text.contains("1900"), "{text}");
    }

    #[test]
    fn a_malformed_request_is_the_fault_of_the_peer() {
        assert!(SmartViewError::Request("no request line".into()).is_peer_fault());
        assert!(SmartViewError::Handshake("no key".into()).is_peer_fault());
        assert!(SmartViewError::LimitExceeded("the body is too large".into()).is_peer_fault());
        assert!(SmartViewError::UriTooLong(8192).is_peer_fault());
        assert!(SmartViewError::HeadersTooLarge(64).is_peer_fault());
    }

    #[test]
    fn a_broken_socket_is_not_the_fault_of_the_peer() {
        // WHY: a socket that cannot be bound is a fault of the receiver and
        // has to stop the engine — the peer cannot fix it.
        assert!(!SmartViewError::Ssdp("address in use".into()).is_peer_fault());
        assert!(!SmartViewError::Description("no name".into()).is_peer_fault());
    }
}
