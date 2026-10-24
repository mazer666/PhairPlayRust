//! The AirPlay 2 receiver engine of PhairPlayRust.
//!
//! This crate implements the AirPlay 2 *receiver* side: the RTSP control
//! channel, the SDP negotiation, the interleaved RTP transport, the pairing
//! handshakes, the FairPlay key exchange, the now-playing metadata, the
//! photo endpoint, the DACP reverse channel and the NTP timing service.
//!
//! WHY the split into modules: an AirPlay session has five independent
//! vocabularies — RTSP requests, SDP, RTP, the pairing crypto and the
//! metadata formats — and every one of them parses *untrusted* input. Keeping
//! them apart means every parser can be fuzzed, unit-tested and hardened on
//! its own, and the session state machine only ever sees typed values.
//!
//! | Module | Responsibility |
//! |---|---|
//! | [`rtsp`] | Request model, bounded reader, response builder, router. |
//! | [`sdp`] | The `ANNOUNCE` body: codecs, keys, ports. |
//! | [`h264`] | The H.264 SPS parser (resolution from the bitstream). |
//! | [`rtp`] | Interleaved framing and FU-A reassembly. |
//! | [`session`] | The per-connection state machine. |
//! | [`engine`] | The [`phairplay_core::protocol::ProtocolEngine`] implementation. |
//! | [`pairing`] | Ed25519/X25519 pairing and the legacy SRP PIN flow. |
//! | [`fairplay`] | The `fp-setup` exchange and the DRM boundary. |
//! | [`plist`] | Apple property lists (binary and XML). |
//! | [`dmap`] | The DAAP/DMAP now-playing metadata. |
//! | [`dacp`] | The DACP reverse control channel. |
//! | [`timing`] | The Apple NTP timing service. |
//! | [`photo`] | The photo endpoint. |
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_airplay::{AirPlayConfig, AirPlayEngine};
//! use phairplay_core::protocol::{ProtocolEngine, ProtocolKind};
//!
//! let engine = AirPlayEngine::new(AirPlayConfig::default());
//! assert_eq!(engine.kind(), ProtocolKind::AirPlay);
//! assert_eq!(engine.status().state, phairplay_core::protocol::ProtocolState::Idle);
//! ```

#![warn(missing_docs)]

pub mod dacp;
pub mod dmap;
pub mod engine;
pub mod error;
pub mod fairplay;
pub mod h264;
pub mod limits;
pub mod mirror;
pub mod pairing;
pub mod photo;
pub mod plist;
pub mod queue;
pub mod rtp;
pub mod rtsp;
pub mod sdp;
pub mod session;
pub mod timing;

pub use engine::{AirPlayConfig, AirPlayEngine};
pub use error::AirPlayError;
pub use limits::AirPlayLimits;
pub use rtsp::{RtspMethod, RtspRequest, RtspRequestReader, RtspResponse};

pub use rtp::FuAReassembler;
pub use sdp::{SdpSession, parse_sdp};
pub use session::PlaybackState;
