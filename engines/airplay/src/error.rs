//! The error type of the AirPlay engine.
//!
//! WHY: the core crate owns the *closed* error hierarchy of the workspace
//! ([`phairplay_core::error`]). An engine must not invent a parallel error
//! enum that the platform layer would have to translate again. This module
//! therefore defines one internal error type with the granularity the AirPlay
//! protocol needs, plus a single conversion into
//! [`phairplay_core::error::EngineError`].
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_airplay::error::AirPlayError;
//! use phairplay_core::error::EngineError;
//!
//! let error = AirPlayError::limit_exceeded("header count");
//! let engine_error: EngineError = error.into();
//! assert!(engine_error.to_string().contains("limit exceeded"));
//! ```

use phairplay_core::error::EngineError;

/// A failure raised by the AirPlay engine.
#[derive(Debug, thiserror::Error)]
pub enum AirPlayError {
    /// A peer violated the AirPlay protocol.
    #[error("AirPlay protocol violation: {reason}")]
    Protocol {
        /// What the peer did wrong.
        reason: String,
    },

    /// A configured limit was reached.
    #[error("AirPlay limit exceeded: {reason}")]
    LimitExceeded {
        /// Which limit was reached.
        reason: String,
    },

    /// A blocking read or handshake step ran out of time.
    #[error("AirPlay timeout: {reason}")]
    Timeout {
        /// What timed out.
        reason: String,
    },

    /// A parser rejected malformed input.
    #[error("AirPlay parse error: {reason}")]
    Parse {
        /// What could not be parsed.
        reason: String,
    },

    /// A cryptographic operation failed.
    #[error("AirPlay crypto error: {reason}")]
    Crypto {
        /// What went wrong.
        reason: String,
    },

    /// The feature exists in the protocol but is not implemented by this build.
    ///
    /// WHY: the FairPlay stream-key decryption needs Apple's proprietary
    /// cipher. Rather than pretending, the engine reports the boundary
    /// explicitly so the caller can say *why* the stream stays encrypted.
    #[error("AirPlay unsupported: {reason}")]
    Unsupported {
        /// What is not supported.
        reason: String,
    },

    /// The engine was in a state that does not allow the operation.
    #[error("AirPlay invalid state: {reason}")]
    InvalidState {
        /// Which invariant was violated.
        reason: String,
    },

    /// An I/O operation failed.
    #[error("AirPlay I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl AirPlayError {
    /// Builds a protocol-violation error.
    #[must_use]
    pub fn protocol(reason: impl Into<String>) -> Self {
        Self::Protocol {
            reason: reason.into(),
        }
    }

    /// Builds a limit-exceeded error.
    #[must_use]
    pub fn limit_exceeded(reason: impl Into<String>) -> Self {
        Self::LimitExceeded {
            reason: reason.into(),
        }
    }

    /// Builds a timeout error.
    #[must_use]
    pub fn timeout(reason: impl Into<String>) -> Self {
        Self::Timeout {
            reason: reason.into(),
        }
    }

    /// Builds a parse error.
    #[must_use]
    pub fn parse(reason: impl Into<String>) -> Self {
        Self::Parse {
            reason: reason.into(),
        }
    }

    /// Builds a crypto error.
    #[must_use]
    pub fn crypto(reason: impl Into<String>) -> Self {
        Self::Crypto {
            reason: reason.into(),
        }
    }

    /// Builds an unsupported-feature error.
    #[must_use]
    pub fn unsupported(reason: impl Into<String>) -> Self {
        Self::Unsupported {
            reason: reason.into(),
        }
    }

    /// Builds an invalid-state error.
    #[must_use]
    pub fn invalid_state(reason: impl Into<String>) -> Self {
        Self::InvalidState {
            reason: reason.into(),
        }
    }
}

impl From<AirPlayError> for phairplay_core::error::CoreError {
    fn from(error: AirPlayError) -> Self {
        Self::Engine(EngineError::from(error))
    }
}

impl From<AirPlayError> for EngineError {
    fn from(error: AirPlayError) -> Self {
        match error {
            AirPlayError::Protocol { reason } => EngineError::Protocol {
                protocol: "airplay",
                reason,
            },
            AirPlayError::LimitExceeded { reason } => EngineError::LimitExceeded {
                protocol: "airplay",
                reason,
            },
            AirPlayError::Timeout { reason } => EngineError::Protocol {
                protocol: "airplay",
                reason: format!("timeout: {reason}"),
            },
            AirPlayError::Parse { reason } => EngineError::Protocol {
                protocol: "airplay",
                reason: format!("malformed input: {reason}"),
            },
            AirPlayError::Crypto { reason } => EngineError::Protocol {
                protocol: "airplay",
                reason: format!("cryptographic failure: {reason}"),
            },
            AirPlayError::Unsupported { reason } => EngineError::Protocol {
                protocol: "airplay",
                reason: format!("unsupported: {reason}"),
            },
            AirPlayError::InvalidState { reason } => EngineError::Protocol {
                protocol: "airplay",
                reason: format!("invalid state: {reason}"),
            },
            AirPlayError::Io(error) => EngineError::Protocol {
                protocol: "airplay",
                reason: format!("I/O error: {error}"),
            },
        }
    }
}

#[cfg(test)]
mod tests;
