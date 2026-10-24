//! The UniFFI bridge between the PhairPlayRust core and the native shells.
//!
//! WHAT: The bridge that carries the state of the receiver from the Rust
//! core into Kotlin, Swift and the web dashboard. It owns the data transfer
//! objects, the [`Receiver`] facade and the callback interfaces a host
//! implements.
//!
//! WHY: The core knows everything about protocols and nothing about
//! platforms. The shells know everything about platforms and nothing about
//! protocols. The bridge is the only place where the two meet, and it is
//! deliberately thin: it carries snapshots, it never hands out a reference
//! into the core, and it never panics on the way.
//!
//! The rules the bridge follows:
//!
//! * **No panic.** Every failure leaves the bridge as an [`FfiError`] with
//!   the stable numeric code of `phairplay_core::error::ErrorCode` and a
//!   human-readable explanation. A panic that slips through is caught and
//!   reported as an internal failure, because a crash of the library would
//!   take the application of the user with it.
//! * **No invented state.** The sessions come from the
//!   `phairplay_core::session::SessionRegistry`, the counters from
//!   `phairplay_core::metrics`, the status from the engines themselves.
//! * **No secrets, no network.** The bridge speaks to nobody. It is a
//!   transport, not a service.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_ffi::dto::FfiAppConfig;
//! use phairplay_ffi::receiver::Receiver;
//! use phairplay_core::config::AppConfig;
//!
//! // The host creates the receiver, asks for the status and stops again.
//! let receiver = Receiver::new(FfiAppConfig::from(&AppConfig::default()))
//!     .unwrap_or_else(|error| panic!("the receiver must start: {error}"));
//! let status = receiver.status();
//! assert_eq!(status.len(), 5);
//! ```

#![warn(missing_docs)]

pub mod callbacks;
pub mod composition;
pub mod dto;
pub mod error;
pub mod listener;
pub mod receiver;
pub mod sinks;
pub mod supervisor;

#[cfg(test)]
mod test_support;

pub use callbacks::{PlatformAudioSink, PlatformVideoSink, ReceiverEventSink};
pub use dto::{
    FfiAppConfig, FfiAudioCodec, FfiAudioFormat, FfiBuildInfo, FfiDeviceConfig, FfiDeviceIdentity,
    FfiEncodedAudioFrame, FfiEncodedVideoFrame, FfiLimitsConfig, FfiMediaCapabilities,
    FfiPassthroughCodec, FfiPerformanceConfig, FfiProtocolConfig, FfiProtocolKind,
    FfiProtocolState, FfiProtocolStatus, FfiReceiverEvent, FfiSessionSnapshot, FfiStreamStats,
    FfiVideoCodec, FfiVideoFormat,
};
pub use error::{FfiError, FfiErrorCode};
pub use listener::{CallbackEventListener, EventListener};
pub use receiver::Receiver;
pub use supervisor::ReceiverCore;

uniffi::setup_scaffolding!("phairplay_ffi");
