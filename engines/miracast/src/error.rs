//! The failures of the Miracast engine.
//!
//! WHY: the engine speaks to devices on the local network, and every one of
//! them is untrusted. A closed error hierarchy lets the caller tell a broken
//! peer from a broken receiver — the first is expected traffic, the second is
//! a defect.

use phairplay_core::protocol::ProtocolKind;

/// A failure of the Miracast engine.
#[derive(Debug, thiserror::Error)]
pub enum MiracastError {
    /// A peer sent a message that is larger than the configured limit.
    #[error("the message of {protocol} is larger than the limit of {limit} bytes")]
    MessageTooLarge {
        /// The protocol that received the message.
        protocol: &'static str,
        /// The limit that was exceeded.
        limit: usize,
    },

    /// A peer sent a message that ended in the middle of a line.
    #[error("the message of {protocol} ended in the middle of a line")]
    TruncatedMessage {
        /// The protocol that received the message.
        protocol: &'static str,
    },

    /// A peer sent a header that is not allowed.
    #[error("the message of {protocol} carries a malformed header: {reason}")]
    MalformedHeader {
        /// The protocol that received the message.
        protocol: &'static str,
        /// Why the header was rejected.
        reason: String,
    },

    /// A peer sent a request the engine does not implement.
    #[error("the method {method} is not implemented by the {protocol} engine")]
    UnknownMethod {
        /// The protocol that received the request.
        protocol: &'static str,
        /// The method the peer used.
        method: String,
    },

    /// A peer used a protocol version the engine does not speak.
    #[error("the peer speaks the unsupported {protocol} version {version}")]
    UnsupportedVersion {
        /// The protocol that received the request.
        protocol: &'static str,
        /// The version the peer announced.
        version: String,
    },

    /// The information element of the peer is broken.
    #[error("the information element of the peer is broken: {reason}")]
    TruncatedBlock {
        /// The offset at which the block starts.
        offset: usize,
        /// Why the block was rejected.
        reason: String,
    },

    /// The information element of the peer is larger than the limit.
    #[error("the information element of the peer is {length} bytes, the limit is {limit}")]
    ElementTooLarge {
        /// The length of the element.
        length: usize,
        /// The limit that was exceeded.
        limit: usize,
    },

    /// The information element of the peer contains too many blocks.
    #[error("the information element of the peer contains more than {limit} blocks")]
    TooManyBlocks {
        /// The limit that was exceeded.
        limit: usize,
    },

    /// A block of the information element does not have the expected shape.
    #[error("the {kind} block of the peer is malformed: {reason}")]
    MalformedBlock {
        /// The kind of the block.
        kind: &'static str,
        /// Why the block was rejected.
        reason: String,
    },

    /// A socket could not be bound or accepted.
    #[error("the socket of {protocol} failed: {reason}")]
    Socket {
        /// The protocol whose socket failed.
        protocol: &'static str,
        /// The reason of the failure.
        reason: String,
    },

    /// The engine is not in a state that allows the operation.
    #[error("the {protocol} engine cannot {operation} in the state {state}")]
    InvalidState {
        /// The protocol whose engine refused.
        protocol: &'static str,
        /// The operation that was refused.
        operation: &'static str,
        /// The state the engine is in.
        state: &'static str,
    },
}

impl MiracastError {
    /// The protocol this failure belongs to.
    #[must_use]
    pub const fn protocol(&self) -> ProtocolKind {
        ProtocolKind::Miracast
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    #[test]
    fn every_failure_belongs_to_miracast() {
        let error = MiracastError::TruncatedMessage {
            protocol: "miracast",
        };
        assert_eq!(error.protocol(), ProtocolKind::Miracast);
    }

    #[test]
    fn the_message_carries_the_reason() {
        let error = MiracastError::Socket {
            protocol: "miracast",
            reason: "address in use".to_string(),
        };
        assert!(error.to_string().contains("address in use"), "{error}");
    }
}
