//! The Miracast (Wi-Fi Display) sink engine of PhairPlayRust.
//!
//! WHY: Miracast is the protocol that lets a Windows laptop or an Android
//! phone show its picture on a television without any cloud service in
//! between. It is the protocol of the local network, like AirPlay — and like
//! AirPlay it needs a real receiver: an RTSP control plane, a negotiated
//! session and a clean teardown.
//!
//! WHAT: the crate implements the WFD (Wi-Fi Display) sink.
//!
//! | Module | Responsibility |
//! |---|---|
//! | [`wfd`] | The WFD information element and its hardened parser. |
//! | [`rtsp`] | The WFD RTSP control plane: the reader, the router, the sink parameters. |
//! | [`engine`] | The state machine and the socket that owns the session. |
//! | [`peer`] | The life of one sender. |
//! | [`error`] | The failures of the crate. |
//!
//! The control plane is RTSP on port 7236. The sender asks with `OPTIONS`
//! what the sink supports, reads the sink's parameters with `GET_PARAMETER`,
//! sets the session up with `SETUP` and starts it with `PLAY`.
//!
//! ## Example
//!
//! ```
//! use phairplay_miracast::{MiracastConfig, MiracastEngine};
//! use phairplay_core::protocol::ProtocolEngine;
//!
//! let engine = MiracastEngine::new(MiracastConfig::default());
//! assert_eq!(engine.kind(), phairplay_core::protocol::ProtocolKind::Miracast);
//! assert_eq!(engine.status().state, phairplay_core::protocol::ProtocolState::Idle);
//! ```
//!
//! ## Scope
//!
//! The crate implements the *control plane* of Miracast. The Wi-Fi Direct
//! group formation needs a system API that only the native shell has, and
//! the RTP media path belongs to the platform layer as well. What the sender
//! sends and what it is told is complete and tested here.
//!
//! This crate is part of the [PhairPlayRust](https://github.com/mazer666/PhairPlayRust)
//! workspace and is published under the AGPL-3.0-or-later licence.
//!
//! The build contract that governs this crate lives in
//! [`docs/design/BUILD-CONTRACT.md`](https://github.com/mazer666/PhairPlayRust/blob/main/docs/design/BUILD-CONTRACT.md).

#![warn(missing_docs)]

mod engine;
mod error;
mod peer;
pub mod rtsp;
pub mod wfd;

pub use engine::{DEFAULT_WFD_PORT, MiracastConfig, MiracastEngine};
pub use error::MiracastError;
pub use rtsp::{MAX_MESSAGE_BYTES, RtspRequest, RtspResponse, sink_parameters};
pub use wfd::{WfdBlock, WfdBlockKind, WfdInformationElement};
