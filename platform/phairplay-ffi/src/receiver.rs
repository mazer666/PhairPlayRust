//! The facade of the receiver that the native shells talk to.
//!
//! WHAT: [`Receiver`] is the object the host creates, asks for the status
//! of, configures, and hands a callback to. It is the exported surface of
//! the bridge — a thin layer over [`ReceiverCore`].
//!
//! WHY: A host must not be able to break the receiver. Every method of the
//! facade therefore returns a structured [`FfiError`] and never panics: an
//! unexpected panic is caught and reported as an internal failure, because
//! a crash of the bridge would take the whole application of the user with
//! it.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_ffi::dto::FfiAppConfig;
//! use phairplay_ffi::receiver::Receiver;
//! use phairplay_core::config::AppConfig;
//!
//! let receiver = Receiver::new(FfiAppConfig::from(&AppConfig::default()))
//!     .unwrap_or_else(|error| panic!("the receiver must start: {error}"));
//! let status = receiver.status();
//! assert_eq!(status.len(), 4);
//! ```

use std::sync::Arc;

use phairplay_core::config::AppConfig;
use phairplay_core::protocol::ProtocolKind;

use crate::callbacks::{PlatformAudioSink, PlatformVideoSink, ReceiverEventSink};
use crate::dto::{
    FfiAppConfig, FfiBuildInfo, FfiDeviceIdentity, FfiMediaCapabilities, FfiProtocolStatus,
    FfiSessionSnapshot, FfiStreamStats,
};
use crate::error::FfiError;
use crate::listener::CallbackEventListener;
use crate::supervisor::ReceiverCore;

/// The receiver of PhairPlayRust, as the native shells see it.
#[derive(uniffi::Object)]
pub struct Receiver {
    core: ReceiverCore,
}

impl std::fmt::Debug for Receiver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Receiver")
            .field("uptime_ms", &self.core.uptime_ms())
            .finish_non_exhaustive()
    }
}

#[uniffi::export]
impl Receiver {
    /// Creates a receiver with the given configuration.
    ///
    /// # Errors
    /// Returns [`FfiError`] when the configuration is invalid or when the
    /// runtime of the bridge cannot be created.
    #[uniffi::constructor]
    pub fn new(config: FfiAppConfig) -> Result<Arc<Self>, FfiError> {
        guard(|| {
            let core = AppConfig::try_from(&config)?;
            Ok(Arc::new(Receiver {
                core: ReceiverCore::new(core)?,
            }))
        })
    }

    /// The status of every protocol.
    pub fn status(&self) -> Vec<FfiProtocolStatus> {
        guard(|| {
            Ok(self
                .core
                .status()
                .iter()
                .map(FfiProtocolStatus::from)
                .collect())
        })
        .unwrap_or_else(|error| {
            // The status of the receiver is always answerable: a failure
            // here means that the state could not be read, and the
            // honest answer is an empty list rather than a crash.
            tracing::error!("the status of the receiver could not be read: {error}");
            Vec::new()
        })
    }

    /// The status of one protocol.
    ///
    /// # Errors
    /// Returns [`FfiError`] when the name is not a known protocol.
    pub fn protocol_status(&self, protocol: String) -> Result<FfiProtocolStatus, FfiError> {
        guard(|| Ok(self.core.ffi_status(parse_protocol(&protocol)?)))
    }

    /// Starts the engine of a protocol.
    ///
    /// # Errors
    /// Returns [`FfiError`] when the name is not a known protocol, when the
    /// protocol is disabled, when no engine is registered for it, or when
    /// the engine refuses to start.
    pub fn start_protocol(&self, protocol: String) -> Result<FfiProtocolStatus, FfiError> {
        guard(|| {
            let kind = parse_protocol(&protocol)?;
            Ok(FfiProtocolStatus::from(&self.core.start_protocol(kind)?))
        })
    }

    /// Stops the engine of a protocol.
    ///
    /// # Errors
    /// Returns [`FfiError`] when the name is not a known protocol, when no
    /// engine is registered for it, or when the engine refuses to stop.
    pub fn stop_protocol(&self, protocol: String) -> Result<FfiProtocolStatus, FfiError> {
        guard(|| {
            let kind = parse_protocol(&protocol)?;
            Ok(FfiProtocolStatus::from(&self.core.stop_protocol(kind)?))
        })
    }

    /// Replaces the configuration of the receiver.
    ///
    /// # Errors
    /// Returns [`FfiError`] when the configuration is invalid.
    pub fn set_config(&self, config: FfiAppConfig) -> Result<(), FfiError> {
        guard(|| {
            let config = AppConfig::try_from(&config)?;
            self.core.set_config(config)
        })
    }

    /// The configuration of the receiver.
    pub fn config(&self) -> FfiAppConfig {
        guard(|| Ok(FfiAppConfig::from(&self.core.config())))
            .unwrap_or_else(|_| FfiAppConfig::from(&AppConfig::default()))
    }

    /// The live sessions of the receiver.
    pub fn sessions(&self) -> Vec<FfiSessionSnapshot> {
        guard(|| Ok(self.core.ffi_sessions())).unwrap_or_else(|_| Vec::new())
    }

    /// The counters of one protocol.
    ///
    /// # Errors
    /// Returns [`FfiError`] when the name is not a known protocol.
    pub fn stats(&self, protocol: String) -> Result<FfiStreamStats, FfiError> {
        guard(|| Ok(self.core.ffi_stats(parse_protocol(&protocol)?)))
    }

    /// The identity the receiver advertises to senders.
    pub fn identity(&self) -> FfiDeviceIdentity {
        guard(|| Ok(FfiDeviceIdentity::from(&self.core.identity()))).unwrap_or_else(|_| {
            FfiDeviceIdentity::from(&phairplay_core::identity::DeviceIdentity::default())
        })
    }

    /// The media capabilities of the receiver.
    pub fn capabilities(&self) -> FfiMediaCapabilities {
        guard(|| Ok(FfiMediaCapabilities::from(&self.core.capabilities()))).unwrap_or_default()
    }

    /// Sets the media capabilities of the receiver.
    ///
    /// # Errors
    /// Returns [`FfiError`] when the capabilities cannot be applied.
    pub fn set_capabilities(&self, capabilities: FfiMediaCapabilities) -> Result<(), FfiError> {
        guard(|| {
            self.core
                .set_capabilities(phairplay_core::capabilities::MediaCapabilities::from(
                    &capabilities,
                ));
            Ok(())
        })
    }

    /// The identification of the build that carries the bridge.
    pub fn build_info(&self) -> FfiBuildInfo {
        FfiBuildInfo::from(&phairplay_core::build_info::BuildInfo::current())
    }

    /// Registers the callback that receives the live events.
    pub fn set_event_sink(&self, sink: Box<dyn ReceiverEventSink>) {
        guard(|| {
            self.core.set_listener(CallbackEventListener::new(sink));
            Ok(())
        })
        .unwrap_or_else(|error| {
            tracing::error!("the event sink could not be registered: {error}");
        });
    }

    /// Removes the callback that receives the live events.
    pub fn clear_event_sink(&self) {
        self.core.clear_listener();
    }

    /// Registers the decoder of the platform for the video frames.
    pub fn set_video_sink(&self, sink: Box<dyn PlatformVideoSink>) {
        guard(|| {
            self.core.set_video_sink(Arc::from(sink));
            Ok(())
        })
        .unwrap_or_else(|error| {
            tracing::error!("the video sink could not be registered: {error}");
        });
    }

    /// Registers the decoder of the platform for the audio packets.
    pub fn set_audio_sink(&self, sink: Box<dyn PlatformAudioSink>) {
        guard(|| {
            self.core.set_audio_sink(Arc::from(sink));
            Ok(())
        })
        .unwrap_or_else(|error| {
            tracing::error!("the audio sink could not be registered: {error}");
        });
    }
}

impl Receiver {
    /// Registers the engine of a protocol.
    ///
    /// WHY: The engine implementations live in the `engines/*` crates of the
    /// workspace. They are Rust types, so they cannot be created by a host
    /// over the bridge; a host application that embeds the bridge as a
    /// library registers them here.
    pub fn register_engine(&self, engine: Arc<dyn phairplay_core::protocol::ProtocolEngine>) {
        self.core.register_engine(engine);
    }

    /// The supervisor behind the facade.
    ///
    /// WHY: The dashboard uses the supervisor directly, so that the panel
    /// and the native shells watch the same state.
    #[must_use]
    pub fn core(&self) -> &ReceiverCore {
        &self.core
    }
}

/// Parses the name of a protocol.
fn parse_protocol(name: &str) -> Result<ProtocolKind, FfiError> {
    name.parse::<ProtocolKind>()
        .map_err(|_| FfiError::invalid_input(format!("unknown protocol `{name}`")))
}

/// Runs an exported method and turns a panic into a structured failure.
///
/// WHY: The bridge is the border between a Rust library and an application
/// that cannot survive a crash of the library. A panic that is not caught
/// here ends the process of the user; a panic that is caught here ends one
/// call and is reported like every other failure.
fn guard<T>(call: impl FnOnce() -> Result<T, FfiError>) -> Result<T, FfiError> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(call)) {
        Ok(result) => result,
        Err(payload) => {
            let details = panic_details(&payload);
            tracing::error!("the receiver caught a panic: {details}");
            Err(FfiError::internal(format!(
                "the receiver failed while handling the request: {details}"
            )))
        }
    }
}

/// Extracts the message of a caught panic.
fn panic_details(payload: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "the failure carried no message".to_string()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;
    use crate::test_support::{Must, MustErr};
    use phairplay_core::config::AppConfig;

    #[test]
    fn an_unknown_protocol_is_rejected_with_the_code_of_the_core() {
        let receiver = Receiver::new(FfiAppConfig::from(&AppConfig::default()))
            .must("the receiver must start");
        let error = receiver
            .protocol_status("telepathy".to_string())
            .must_err("the protocol must be rejected");
        assert_eq!(
            error.code(),
            crate::error::FfiErrorCode::InvalidInput.as_i32()
        );
        assert!(error.details().contains("telepathy"), "{error}");
    }

    #[test]
    fn the_status_covers_every_protocol() {
        let receiver = Receiver::new(FfiAppConfig::from(&AppConfig::default()))
            .must("the receiver must start");
        let status = receiver.status();
        assert_eq!(status.len(), 4);
        let names: Vec<&str> = status.iter().map(|entry| entry.protocol.as_str()).collect();
        assert_eq!(names, vec!["airplay", "miracast", "cast", "dlna"]);
    }

    #[test]
    fn starting_a_protocol_without_an_engine_is_an_unsupported_failure() {
        let receiver = Receiver::new(FfiAppConfig::from(&AppConfig::default()))
            .must("the receiver must start");
        let error = receiver
            .start_protocol("airplay".to_string())
            .must_err("there is no engine for the protocol");
        assert_eq!(
            error.code(),
            crate::error::FfiErrorCode::Unsupported.as_i32()
        );
        assert!(error.details().contains("airplay"), "{error}");
    }

    #[test]
    fn the_configuration_survives_the_bridge() {
        let mut config = AppConfig::default();
        config.device.name = "Attic".into();
        let receiver = Receiver::new(FfiAppConfig::from(&config)).must("the receiver must start");
        let read_back = receiver.config();
        assert_eq!(read_back.device.name, "Attic");
        assert_eq!(read_back, FfiAppConfig::from(&config));
    }

    #[test]
    fn an_invalid_configuration_is_rejected() {
        let mut config = FfiAppConfig::from(&AppConfig::default());
        config.limits.video_queue_frames = 0;
        let error = Receiver::new(config).must_err("the configuration must be rejected");
        assert_eq!(
            error.code(),
            crate::error::FfiErrorCode::InvalidInput.as_i32()
        );
    }
}
