//! The Google Cast receiver engine of PhairPlayRust.
//!
//! WHY: Google Cast is the protocol that brings the most senders — a phone,
//! a laptop, a speaker — into a living room. It is also the only protocol of
//! the four that a sender may use *several times at once*: a Cast receiver
//! serves every sender that connects, and only the one-peer rule of the
//! build contract is lifted for it.
//!
//! WHAT: the crate implements the CastV2 protocol:
//!
//! | Module | Responsibility |
//! |---|---|
//! | `proto` | The protobuf wire format of the message envelope. |
//! | `message` | The `CastMessage` envelope and the four namespaces. |
//! | `frame` | The length-prefixed framing of the transport. |
//! | `payloads` | The JSON payloads of the control channels. |
//! | `media` | The media channel: load, play, pause, seek, status. |
//! | `session` | The state of one peer and the dispatch of its messages. |
//! | `tls` | The TLS layer with a self-signed certificate. |
//! | `advertisement` | The `_googlecast._tcp` DNS-SD advertisement. |
//! | `peer` | The life of one connection. |
//! | `engine` | The [`phairplay_core::protocol::ProtocolEngine`] implementation. |
//!
//! The transport is TLS on port 8009, the protocol version is CastV2 1.0 and
//! the application the receiver offers is the Google default media receiver
//! (`CC1AD845`).
//!
//! ## Example
//!
//! ```
//! use phairplay_cast::{CastConfig, CastEngine};
//! use phairplay_core::protocol::ProtocolEngine;
//!
//! let engine = CastEngine::new(CastConfig::default());
//! assert_eq!(engine.kind(), phairplay_core::protocol::ProtocolKind::Cast);
//! ```
//!
//! ## Scope
//!
//! The crate implements the *control plane* of Cast: the handshake, the
//! heartbeat, the channel management, the receiver status and the media
//! status. It does not fetch the media a sender names — that needs an HTTP
//! client and demuxers, which belong to the platform layer, not to the
//! protocol engine. What the sender sends and what it is told is complete
//! and tested here.
//!
//! This crate is part of the [PhairPlayRust](https://github.com/mazer666/PhairPlayRust)
//! workspace and is published under the AGPL-3.0-or-later licence.
//!
//! The build contract that governs this crate lives in
//! [`docs/design/BUILD-CONTRACT.md`](https://github.com/mazer666/PhairPlayRust/blob/main/docs/design/BUILD-CONTRACT.md).

#![warn(missing_docs)]

mod advertisement;
mod engine;
mod frame;
mod media;
mod message;
mod payloads;
mod peer;
mod proto;
mod session;
mod tls;

pub use advertisement::{
    AdvertiseError, CastService, DEVICE_ID_LENGTH, GOOGLECAST_SERVICE_TYPE, MdnsRegistrar,
    READY_STATE_CONNECTED, READY_STATE_IDLE, ServiceRegistrar, device_id, txt_properties,
};
pub use engine::{
    CastConfig, CastEngine, CastTls, DEFAULT_HANDSHAKE_TIMEOUT_MS, DEFAULT_MAX_SESSIONS,
    DEFAULT_PORT,
};
pub use frame::{DEFAULT_MAX_MESSAGE_BYTES, FrameError, read_frame, write_frame};
pub use media::{
    MediaCommand, MediaOutcome, MediaParseError, MediaSession, MediaStatusEntry,
    MediaStatusResponse, SeekRequest,
};
pub use message::{
    CastMessage, DEFAULT_MEDIA_APP_ID, DEFAULT_MEDIA_APP_NAME, HEARTBEAT_CHANNEL, MEDIA_CHANNEL,
    Namespace, PayloadType, RECEIVER_CHANNEL, TRANSPORT_CHANNEL,
};
pub use payloads::{
    Artwork, ConnectRequest, ConnectResponse, LoadRequest, MediaItem, MediaMetadata, MediaTrack,
    ReceiverApplication, ReceiverNamespace, ReceiverStatus, ReceiverStatusResponse,
    SetVolumeRequest, VolumeRequest, VolumeStatus, message_type,
};
pub use proto::{ProtoError, ProtoReader, ProtoWriter};
pub use session::{Dispatch, PeerSession, ProtocolViolation, SENDER_CHANNEL, SessionAction};
pub use tls::{TlsError, self_signed};
