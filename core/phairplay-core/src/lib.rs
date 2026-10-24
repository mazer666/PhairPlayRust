//! # PhairPlayRust core
//!
//! The shared core of the PhairPlayRust receiver stack.
//!
//! PhairPlayRust is a free, open-source, privacy-first receiver for AirPlay 2,
//! Miracast, Google Cast and DLNA/UPnP. It has **no account, no telemetry,
//! no advertising and no cloud dependency** — it only ever speaks to devices
//! on the local network.
//!
//! This crate owns everything that is *not* protocol specific:
//!
//! | Module | Responsibility |
//! |---|---|
//! | [`protocol`] | The [`protocol::ProtocolEngine`] trait every receiver implements. |
//! | [`session`] | Session lifecycle and the events the engines report. |
//! | [`media`] | Encoded-frame abstractions and the sinks that consume them. |
//! | [`capabilities`] | What this device can actually decode. |
//! | [`config`] | The typed configuration model. |
//! | [`identity`] | The identity advertised to senders. |
//! | [`metrics`] | Live counters about a running session. |
//! | [`privacy`] | The masking of the names and the addresses in the log. |
//! | [`queue`] | The media queues, bounded by a count *and* by a byte budget. |
//! | [`error`] | The closed error hierarchy of the whole workspace. |
//!
//! ## Layering
//!
//! ```text
//!   apps/{android,macos,web}      ← native shells and the dashboard
//!            │
//!   platform/phairplay-ffi        ← UniFFI bridge
//!            │
//!   core/phairplay-core           ← this crate: the shared vocabulary
//!         │        │
//!   engines/{airplay,miracast,cast,dlna,discovery}   ← protocol state machines
//!         │
//!   formats/{plist,h264,aac,alac}                     ← wire and media codecs
//! ```
//!
//! Dependencies point strictly downwards. An engine may use the core and the
//! format crates; the core never knows an engine exists; a format crate never
//! knows a protocol exists.
//!
//! ## Example
//!
//! ```
//! use phairplay_core::protocol::{ProtocolKind, ProtocolState, ProtocolStatus};
//!
//! let status = ProtocolStatus {
//!     protocol: ProtocolKind::AirPlay,
//!     state: ProtocolState::Advertising,
//!     enabled: true,
//!     tcp_port: Some(7000),
//!     udp_port: None,
//!     peer_count: 0,
//!     last_error: None,
//! };
//!
//! assert!(status.state.accepts_peers());
//! ```

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/mazer666/PhairPlayRust/main/docs/assets/phairplayrust-logo.svg"
)]

pub mod build_info;
pub mod capabilities;
pub mod config;
pub mod error;
pub mod identity;
pub mod jobs;
pub mod media;
pub mod metrics;
pub mod privacy;
pub mod protocol;
pub mod queue;
pub mod session;
pub mod ssdp;

pub use build_info::BuildInfo;
pub use capabilities::{AudioCodec, MediaCapabilities, PassthroughCodec, VideoCodec};
pub use config::{
    AppConfig, DeviceConfig, EngineLimits, LimitsConfig, PerformanceConfig, ProtocolConfig,
};
pub use error::{CoreError, EngineError, ErrorCode, ErrorKind};
pub use identity::DeviceIdentity;
pub use media::{AudioSink, EncodedAudioFrame, EncodedVideoFrame, VideoSink};
pub use privacy::{mask_ip, mask_name, mask_socket_addr};
pub use protocol::{EngineContext, ProtocolEngine, ProtocolKind, ProtocolState, ProtocolStatus};
pub use queue::{AudioFrameQueue, BoundedFrameQueue, PushOutcome, QueueBudget, VideoFrameQueue};
pub use session::{SessionEvent, SessionId, SessionRegistry, SessionSnapshot};
