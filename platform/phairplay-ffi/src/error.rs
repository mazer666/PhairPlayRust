//! The structured error of the bridge.
//!
//! WHAT: Every failure that leaves the bridge is a [`FfiError`]. It carries
//! the stable numeric code of `phairplay_core::error::ErrorCode` and a
//! human-readable explanation. Nothing else crosses the boundary.
//!
//! WHY: The native shells (Kotlin, Swift) and the web dashboard have to
//! distinguish "the user typed a wrong protocol name" from "the socket
//! could not be bound" without parsing an English sentence. A numeric code
//! is the only thing that survives a change of wording, a translation and a
//! change of the error hierarchy in the core.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_ffi::error::{FfiError, FfiErrorCode};
//! use phairplay_core::error::{ConfigError, CoreError};
//!
//! let error = FfiError::from_core(&CoreError::Config(ConfigError::Invalid {
//!     field: "limits.video_queue_frames".into(),
//!     reason: "must be greater than zero".into(),
//! }));
//!
//! assert_eq!(error.code(), FfiErrorCode::InvalidInput.as_i32());
//! assert!(error.details().contains("video_queue_frames"));
//! ```

use phairplay_core::error::{CoreError, EngineError, ErrorCode, ErrorKind};

/// The stable classification of a failure, mirrored from the core.
///
/// WHY: The foreign side wants a *closed* set it can switch on. The numeric
/// code in [`FfiError`] is the wire format; this enum is the readable
/// mirror of it. The mapping in both directions is covered by a test, so the
/// two can never drift apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum FfiErrorCode {
    /// The input was invalid.
    InvalidInput,
    /// A configured limit was reached.
    LimitExceeded,
    /// The peer violated the protocol.
    ProtocolViolation,
    /// A resource was exhausted.
    ResourceExhausted,
    /// The operation timed out.
    Timeout,
    /// The feature is unsupported on this platform.
    Unsupported,
    /// The engine was in the wrong state.
    InvalidState,
    /// An I/O operation failed.
    Io,
    /// An unspecified internal failure.
    Internal,
}

impl FfiErrorCode {
    /// Every code, in the order of the core enumeration.
    pub const ALL: [FfiErrorCode; 9] = [
        FfiErrorCode::InvalidInput,
        FfiErrorCode::LimitExceeded,
        FfiErrorCode::ProtocolViolation,
        FfiErrorCode::ResourceExhausted,
        FfiErrorCode::Timeout,
        FfiErrorCode::Unsupported,
        FfiErrorCode::InvalidState,
        FfiErrorCode::Io,
        FfiErrorCode::Internal,
    ];

    /// The numeric value of the code.
    ///
    /// WHY: This is the value that travels over the boundary. It is identical
    /// to [`ErrorCode::as_i32`] of the core, because the core owns the
    /// numbering.
    #[must_use]
    pub const fn as_i32(self) -> i32 {
        match self {
            FfiErrorCode::InvalidInput => ErrorCode::InvalidInput.as_i32(),
            FfiErrorCode::LimitExceeded => ErrorCode::LimitExceeded.as_i32(),
            FfiErrorCode::ProtocolViolation => ErrorCode::ProtocolViolation.as_i32(),
            FfiErrorCode::ResourceExhausted => ErrorCode::ResourceExhausted.as_i32(),
            FfiErrorCode::Timeout => ErrorCode::Timeout.as_i32(),
            FfiErrorCode::Unsupported => ErrorCode::Unsupported.as_i32(),
            FfiErrorCode::InvalidState => ErrorCode::InvalidState.as_i32(),
            FfiErrorCode::Io => ErrorCode::Io.as_i32(),
            FfiErrorCode::Internal => ErrorCode::Internal.as_i32(),
        }
    }

    /// The code of the core that belongs to this classification.
    #[must_use]
    pub const fn as_core(self) -> ErrorCode {
        match self {
            FfiErrorCode::InvalidInput => ErrorCode::InvalidInput,
            FfiErrorCode::LimitExceeded => ErrorCode::LimitExceeded,
            FfiErrorCode::ProtocolViolation => ErrorCode::ProtocolViolation,
            FfiErrorCode::ResourceExhausted => ErrorCode::ResourceExhausted,
            FfiErrorCode::Timeout => ErrorCode::Timeout,
            FfiErrorCode::Unsupported => ErrorCode::Unsupported,
            FfiErrorCode::InvalidState => ErrorCode::InvalidState,
            FfiErrorCode::Io => ErrorCode::Io,
            FfiErrorCode::Internal => ErrorCode::Internal,
        }
    }

    /// The classification of a core code.
    ///
    /// # Errors
    /// Returns `None` for a number that the core does not define. The
    /// bridge never invents a code, so an unknown number is a bug and is
    /// reported as such.
    #[must_use]
    pub const fn from_i32(code: i32) -> Option<Self> {
        match code {
            1 => Some(FfiErrorCode::InvalidInput),
            2 => Some(FfiErrorCode::LimitExceeded),
            3 => Some(FfiErrorCode::ProtocolViolation),
            4 => Some(FfiErrorCode::ResourceExhausted),
            5 => Some(FfiErrorCode::Timeout),
            6 => Some(FfiErrorCode::Unsupported),
            7 => Some(FfiErrorCode::InvalidState),
            8 => Some(FfiErrorCode::Io),
            9 => Some(FfiErrorCode::Internal),
            _ => None,
        }
    }
}

impl From<ErrorCode> for FfiErrorCode {
    fn from(code: ErrorCode) -> Self {
        match code {
            ErrorCode::InvalidInput => FfiErrorCode::InvalidInput,
            ErrorCode::LimitExceeded => FfiErrorCode::LimitExceeded,
            ErrorCode::ProtocolViolation => FfiErrorCode::ProtocolViolation,
            ErrorCode::ResourceExhausted => FfiErrorCode::ResourceExhausted,
            ErrorCode::Timeout => FfiErrorCode::Timeout,
            ErrorCode::Unsupported => FfiErrorCode::Unsupported,
            ErrorCode::InvalidState => FfiErrorCode::InvalidState,
            ErrorCode::Io => FfiErrorCode::Io,
            ErrorCode::Internal => FfiErrorCode::Internal,
        }
    }
}

/// A failure that crossed the bridge.
///
/// WHY: The bridge must never panic and must never answer with a bare
/// string: the host has to be able to tell a wrong input from a broken
/// socket without reading an English sentence. The variant carries the
/// stable numeric code of the core and the explanation.
///
/// The field is called `details` and not `message`, because `message` is the
/// name of `Throwable.message` in Kotlin and would collide there.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, uniffi::Error)]
pub enum FfiError {
    /// The operation failed.
    #[error("{details}")]
    Failed {
        /// The stable numeric code of the failure.
        code: i32,
        /// The human-readable explanation of the failure.
        details: String,
    },
}

impl FfiError {
    /// Builds a failure with an explicit code and explanation.
    #[must_use]
    pub fn new(code: FfiErrorCode, details: impl Into<String>) -> Self {
        FfiError::Failed {
            code: code.as_i32(),
            details: details.into(),
        }
    }

    /// Builds a failure with the code of the core.
    #[must_use]
    pub fn from_code(code: ErrorCode, details: impl Into<String>) -> Self {
        FfiError::Failed {
            code: code.as_i32(),
            details: details.into(),
        }
    }

    /// Builds a failure for an invalid input.
    #[must_use]
    pub fn invalid_input(details: impl Into<String>) -> Self {
        Self::new(FfiErrorCode::InvalidInput, details)
    }

    /// Builds a failure for a feature that is not available.
    #[must_use]
    pub fn unsupported(details: impl Into<String>) -> Self {
        Self::new(FfiErrorCode::Unsupported, details)
    }

    /// Builds a failure for a wrong state of the receiver.
    #[must_use]
    pub fn invalid_state(details: impl Into<String>) -> Self {
        Self::new(FfiErrorCode::InvalidState, details)
    }

    /// Builds a failure for an unexpected internal problem.
    #[must_use]
    pub fn internal(details: impl Into<String>) -> Self {
        Self::new(FfiErrorCode::Internal, details)
    }

    /// Builds the failure of a core error.
    ///
    /// WHY: The classification comes from the error *kind*, never from the
    /// wording of the message. A change of the text therefore cannot change
    /// the behaviour of the host.
    #[must_use]
    pub fn from_core(error: &CoreError) -> Self {
        let kind = ErrorKind::classify(error);
        let code = ErrorCode::from(kind);
        FfiError::Failed {
            code: code.as_i32(),
            details: phairplay_core::error::describe(error),
        }
    }

    /// Builds the failure of an engine error.
    #[must_use]
    pub fn from_engine(error: EngineError) -> Self {
        Self::from_core(&CoreError::Engine(error))
    }

    /// The numeric code of the failure.
    #[must_use]
    pub fn code(&self) -> i32 {
        match self {
            FfiError::Failed { code, .. } => *code,
        }
    }

    /// The human-readable explanation of the failure.
    #[must_use]
    pub fn details(&self) -> &str {
        match self {
            FfiError::Failed { details, .. } => details,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phairplay_core::error::{ConfigError, MediaError, NetworkError};

    #[test]
    fn the_numeric_codes_match_the_core() {
        for code in FfiErrorCode::ALL {
            assert_eq!(code.as_i32(), code.as_core().as_i32(), "{code:?}");
            assert_eq!(FfiErrorCode::from_i32(code.as_i32()), Some(code));
        }
    }

    #[test]
    fn an_unknown_number_is_not_mapped_onto_a_code() {
        // The bridge never invents a code: a number the core does not
        // define is a bug in the caller, not a silent "internal error".
        assert_eq!(FfiErrorCode::from_i32(0), None);
        assert_eq!(FfiErrorCode::from_i32(10), None);
        assert_eq!(FfiErrorCode::from_i32(-1), None);
    }

    #[test]
    fn a_core_error_keeps_its_code_and_its_reason() {
        let error = FfiError::from_core(&CoreError::Config(ConfigError::Invalid {
            field: "receiver.port".into(),
            reason: "must be below 65536".into(),
        }));
        assert_eq!(error.code(), ErrorCode::InvalidInput.as_i32());
        assert!(error.details().contains("receiver.port"), "{error}");
        assert!(error.details().contains("65536"), "{error}");
    }

    #[test]
    fn a_media_queue_failure_is_a_resource_problem() {
        let error = FfiError::from_core(&CoreError::Media(MediaError::QueueFull { capacity: 90 }));
        assert_eq!(error.code(), ErrorCode::ResourceExhausted.as_i32());
    }

    #[test]
    fn a_bind_failure_is_a_resource_problem() {
        let error = FfiError::from_core(&CoreError::Network(NetworkError::Bind {
            address: "0.0.0.0:7000".into(),
            reason: "address in use".into(),
        }));
        assert_eq!(error.code(), ErrorCode::ResourceExhausted.as_i32());
        assert!(error.details().contains("address in use"), "{error}");
    }

    #[test]
    fn the_details_of_an_error_are_never_empty() {
        let error = FfiError::internal("something went wrong");
        assert_eq!(error.code(), ErrorCode::Internal.as_i32());
        assert!(!error.details().is_empty());
    }
}
