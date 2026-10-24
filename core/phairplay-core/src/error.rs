//! Error types shared by every crate of the PhairPlayRust workspace.
//!
//! WHY: the receiver stack crosses many process boundaries (a LAN socket, a
//! UniFFI bridge, a platform decoder). Every layer needs to be able to say
//! *what* went wrong without knowing the layer above it. A single closed error
//! hierarchy in the core crate gives the whole workspace one vocabulary, and
//! lets the FFI layer map a failure onto a stable, documented exit code without
//! string matching on messages.
//!
//! HOW: [`CoreError`] is the root. Protocol engines return [`EngineError`],
//! which adds the protocol-specific variants. The FFI bridge never sees an
//! engine error directly — it sees [`CoreError::Engine`].

/// The root error type of the PhairPlayRust core.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    /// A configuration file could not be read, parsed or validated.
    #[error("configuration error: {0}")]
    Config(#[from] ConfigError),

    /// A protocol engine failed.
    #[error("protocol engine error: {0}")]
    Engine(#[from] EngineError),

    /// A media sink rejected a frame or an audio buffer.
    #[error("media sink error: {0}")]
    Media(#[from] MediaError),

    /// A session-level invariant was violated.
    #[error("session error: {0}")]
    Session(#[from] SessionError),

    /// A network operation failed.
    #[error("network error: {0}")]
    Network(#[from] NetworkError),

    /// A platform capability could not be determined.
    #[error("capability error: {0}")]
    Capability(#[from] CapabilityError),

    /// The receiver was asked to stop while an operation was in flight.
    #[error("operation aborted: {0}")]
    Aborted(&'static str),
}

/// Configuration failures.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// The configuration file does not exist.
    #[error("configuration file not found: {0}")]
    NotFound(String),

    /// The configuration file is not valid TOML.
    #[error("configuration file is not valid TOML: {0}")]
    Parse(#[from] toml::de::Error),

    /// A value was syntactically valid but semantically rejected.
    #[error("invalid configuration value for `{field}`: {reason}")]
    Invalid {
        /// The dotted path of the offending key.
        field: String,
        /// Why the value was rejected.
        reason: String,
    },

    /// The configuration could not be written back.
    #[error("configuration could not be written: {0}")]
    Io(#[from] std::io::Error),
}

/// Failures raised by a protocol engine.
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    /// The engine could not be started.
    #[error("{protocol} engine could not be started: {reason}")]
    Start {
        /// The protocol that failed to start.
        protocol: &'static str,
        /// Why the start failed.
        reason: String,
    },

    /// The engine was stopped while it was not running.
    #[error("{protocol} engine is not running")]
    NotRunning {
        /// The protocol that was not running.
        protocol: &'static str,
    },

    /// The engine is already running.
    #[error("{protocol} engine is already running")]
    AlreadyRunning {
        /// The protocol that is already running.
        protocol: &'static str,
    },

    /// A peer violated the protocol.
    #[error("{protocol} protocol violation: {reason}")]
    Protocol {
        /// The protocol the violation belongs to.
        protocol: &'static str,
        /// A description of the violation.
        reason: String,
    },

    /// A peer exceeded a configured limit.
    #[error("{protocol} limit exceeded: {reason}")]
    LimitExceeded {
        /// The protocol the limit belongs to.
        protocol: &'static str,
        /// Which limit was exceeded.
        reason: String,
    },

    /// The engine was asked to stop.
    #[error("{protocol} engine stopped")]
    Stopped {
        /// The protocol that stopped.
        protocol: &'static str,
    },
}

/// Failures raised by a media sink.
#[derive(Debug, thiserror::Error)]
pub enum MediaError {
    /// The sink's bounded queue is full and the frame was dropped.
    #[error("media sink queue is full ({capacity} frames)")]
    QueueFull {
        /// The configured queue capacity.
        capacity: usize,
    },

    /// The sink was closed before the frame could be delivered.
    #[error("media sink is closed")]
    Closed,

    /// The decoder rejected the payload.
    #[error("media payload rejected: {0}")]
    Rejected(String),

    /// The sink is not able to accept this kind of media.
    #[error("media sink does not support this format: {0}")]
    Unsupported(String),
}

/// Session lifecycle failures.
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    /// No session is currently active.
    #[error("no active session")]
    NoActiveSession,

    /// A session with this identifier is already known.
    #[error("session {0} already exists")]
    Duplicate(u64),

    /// A second concurrent session was refused.
    #[error("a session is already active on this protocol")]
    AlreadyActive,

    /// The session was not found.
    #[error("session {0} not found")]
    NotFound(u64),

    /// The session ended before the operation completed.
    #[error("session ended while the operation was in flight")]
    Ended,
}

/// Network failures.
#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    /// A socket could not be bound.
    #[error("could not bind {address}: {reason}")]
    Bind {
        /// The address that could not be bound.
        address: String,
        /// Why the bind failed.
        reason: String,
    },

    /// A socket operation failed.
    #[error("socket error: {0}")]
    Io(#[from] std::io::Error),

    /// No usable network interface was found.
    #[error("no usable network interface: {0}")]
    NoInterface(String),

    /// A peer address was outside the permitted local subnet.
    #[error("peer address {0} is outside the permitted local subnet")]
    PeerOutsideSubnet(String),
}

/// Capability detection failures.
#[derive(Debug, thiserror::Error)]
pub enum CapabilityError {
    /// The platform did not answer the capability query.
    #[error("capability query failed: {0}")]
    Query(String),

    /// The platform reported a capability set that is not internally consistent.
    #[error("inconsistent capability report: {0}")]
    Inconsistent(String),
}

/// A short, stable, machine-readable classification of an error.
///
/// WHY: the FFI bridge maps errors onto numeric exit codes that the native
/// shells branch on. Deriving the code from the error *kind* rather than from
/// the message keeps that mapping stable even when the wording changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// The operation was rejected because the input was invalid.
    InvalidInput,
    /// A configured limit was reached.
    LimitExceeded,
    /// The peer violated the protocol.
    ProtocolViolation,
    /// A resource was exhausted (queue, memory, socket).
    ResourceExhausted,
    /// The operation timed out.
    Timeout,
    /// The feature is not implemented on this platform.
    Unsupported,
    /// The engine is not in a state where the operation is allowed.
    InvalidState,
    /// An underlying I/O operation failed.
    Io,
    /// Something else went wrong.
    Internal,
}

impl ErrorKind {
    /// Maps an error onto its [`ErrorKind`].
    ///
    /// # Errors
    /// Returns [`ErrorKind::Internal`] for errors that carry no more specific
    /// classification.
    pub fn classify(error: &CoreError) -> Self {
        match error {
            CoreError::Config(ConfigError::Invalid { .. })
            | CoreError::Config(ConfigError::Parse(_))
            | CoreError::Config(ConfigError::NotFound(_)) => ErrorKind::InvalidInput,
            CoreError::Config(ConfigError::Io(_)) => ErrorKind::Io,
            CoreError::Engine(EngineError::LimitExceeded { .. })
            | CoreError::Media(MediaError::QueueFull { .. }) => ErrorKind::ResourceExhausted,
            CoreError::Engine(EngineError::Protocol { .. }) => ErrorKind::ProtocolViolation,
            CoreError::Engine(EngineError::NotRunning { .. })
            | CoreError::Engine(EngineError::AlreadyRunning { .. })
            | CoreError::Session(_) => ErrorKind::InvalidState,
            CoreError::Engine(EngineError::Start { .. })
            | CoreError::Engine(EngineError::Stopped { .. }) => ErrorKind::Internal,
            CoreError::Media(MediaError::Unsupported(_)) => ErrorKind::Unsupported,
            CoreError::Media(MediaError::Rejected(_)) => ErrorKind::InvalidInput,
            CoreError::Media(MediaError::Closed) => ErrorKind::InvalidState,
            CoreError::Network(NetworkError::Io(_)) => ErrorKind::Io,
            CoreError::Network(NetworkError::Bind { .. }) => ErrorKind::ResourceExhausted,
            CoreError::Network(NetworkError::NoInterface(_))
            | CoreError::Network(NetworkError::PeerOutsideSubnet(_)) => ErrorKind::InvalidInput,
            CoreError::Capability(CapabilityError::Query(_))
            | CoreError::Capability(CapabilityError::Inconsistent(_)) => ErrorKind::Internal,
            CoreError::Aborted(_) => ErrorKind::Timeout,
        }
    }
}

/// A short, stable, numeric code for an [`ErrorKind`].
///
/// The mapping is part of the public contract of the UniFFI bridge and must not
/// be renumbered without a major version bump.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ErrorCode {
    /// The input was invalid.
    InvalidInput = 1,
    /// A configured limit was reached.
    LimitExceeded = 2,
    /// The peer violated the protocol.
    ProtocolViolation = 3,
    /// A resource was exhausted.
    ResourceExhausted = 4,
    /// The operation timed out.
    Timeout = 5,
    /// The feature is unsupported on this platform.
    Unsupported = 6,
    /// The engine was in the wrong state.
    InvalidState = 7,
    /// An I/O operation failed.
    Io = 8,
    /// An unspecified internal failure.
    Internal = 9,
}

impl ErrorCode {
    /// The numeric value of this code, as passed across the FFI bridge.
    #[must_use]
    pub const fn as_i32(self) -> i32 {
        self as i32
    }
}

impl From<ErrorKind> for ErrorCode {
    fn from(value: ErrorKind) -> Self {
        match value {
            ErrorKind::InvalidInput => ErrorCode::InvalidInput,
            ErrorKind::LimitExceeded => ErrorCode::LimitExceeded,
            ErrorKind::ProtocolViolation => ErrorCode::ProtocolViolation,
            ErrorKind::ResourceExhausted => ErrorCode::ResourceExhausted,
            ErrorKind::Timeout => ErrorCode::Timeout,
            ErrorKind::Unsupported => ErrorCode::Unsupported,
            ErrorKind::InvalidState => ErrorCode::InvalidState,
            ErrorKind::Io => ErrorCode::Io,
            ErrorKind::Internal => ErrorCode::Internal,
        }
    }
}

/// Formats an error and its full source chain into a single line.
///
/// WHY: the native shells log the error chain; a multi-line `Debug` rendering
/// makes log aggregation painful, while a single `Display` line loses the
/// cause. This helper produces a compact, single-line chain.
pub fn describe(error: &dyn std::error::Error) -> String {
    let mut out = error.to_string();
    let mut source = error.source();
    while let Some(cause) = source {
        out.push_str(" -> ");
        out.push_str(&cause.to_string());
        source = cause.source();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_kind_of_invalid_config_is_invalid_input() {
        let error = CoreError::Config(ConfigError::Invalid {
            field: "receiver.port".into(),
            reason: "must be below 65536".into(),
        });
        assert_eq!(ErrorKind::classify(&error), ErrorKind::InvalidInput);
    }

    #[test]
    fn error_kind_of_queue_full_is_resource_exhausted() {
        let error = CoreError::Media(MediaError::QueueFull { capacity: 90 });
        assert_eq!(ErrorKind::classify(&error), ErrorKind::ResourceExhausted);
    }

    #[test]
    fn error_codes_are_stable() {
        // These numbers are part of the UniFFI contract. Changing one is a breaking change.
        assert_eq!(
            ErrorCode::from(ErrorKind::InvalidInput),
            ErrorCode::InvalidInput
        );
        assert_eq!(ErrorCode::InvalidInput.as_i32(), 1);
        assert_eq!(ErrorCode::Internal.as_i32(), 9);
    }

    #[test]
    fn describe_includes_the_cause_chain() {
        let error = CoreError::Network(NetworkError::Bind {
            address: "0.0.0.0:7000".into(),
            reason: "address in use".into(),
        });
        let text = describe(&error);
        assert!(text.contains("could not bind"), "{text}");
        assert!(text.contains("address in use"), "{text}");
    }
}
