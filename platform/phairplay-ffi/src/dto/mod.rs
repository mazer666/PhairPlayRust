//! The data transfer objects of the bridge.
//!
//! WHAT: The immutable snapshots that travel from the core to the host:
//! protocol statuses, session snapshots, stream counters, the identity of
//! the receiver, the configuration and the live events.
//!
//! WHY: The bridge hands out *copies*, never references into the core. A
//! host that holds a reference would see it change under its feet, and a
//! reference that outlives the runtime would be a use-after-free. A snapshot
//! is cheap, complete and safe to keep.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_ffi::dto::{FfiProtocolKind, FfiProtocolState, FfiProtocolStatus};
//!
//! let status = FfiProtocolStatus {
//!     protocol: FfiProtocolKind::Cast,
//!     state: FfiProtocolState::Advertising,
//!     enabled: true,
//!     tcp_port: Some(8009),
//!     udp_port: None,
//!     peer_count: 0,
//!     last_error: None,
//! };
//!
//! assert_eq!(status.protocol.as_str(), "cast");
//! ```

pub mod config;
pub mod identity;
pub mod media;
pub mod protocol;
pub mod session;
pub mod stats;

pub use config::{
    FfiAppConfig, FfiDeviceConfig, FfiLimitsConfig, FfiPerformanceConfig, FfiProtocolConfig,
};
pub use identity::{FfiBuildInfo, FfiDeviceIdentity};
pub use media::{
    FfiAudioCodec, FfiAudioFormat, FfiEncodedAudioFrame, FfiEncodedVideoFrame,
    FfiMediaCapabilities, FfiPassthroughCodec, FfiVideoCodec, FfiVideoFormat,
};
pub use protocol::{FfiProtocolKind, FfiProtocolState, FfiProtocolStatus};
pub use session::{FfiReceiverEvent, FfiSessionSnapshot};
pub use stats::FfiStreamStats;
