//! The Samsung Smart View engine of PhairPlayRust.
//!
//! WHAT: this crate implements the **discovery layer and the handshake** of
//! Samsung Smart View — the screen mirroring of the Galaxy phones and
//! tablets. It announces the receiver over SSDP with the search target
//! `urn:samsung.com:service:MultiScreenService:1`, serves the description
//! document that the `LOCATION` header points at, answers the HTTP API below
//! `/api/v2/` and accepts the WebSocket upgrade of the channel.
//!
//! WHY: a Galaxy device finds a Smart View target in three steps, and all
//! three are documented by Samsung ("Device Discovery, Authentication, and
//! Pairing", Samsung Smart TV Legacy documentation):
//!
//! 1. the sender sends an `M-SEARCH` for
//!    `urn:samsung.com:service:MultiScreenService:1` and the device answers
//!    with `HTTP/1.1 200 OK` and a `LOCATION` header;
//! 2. the sender fetches the description document at that location and reads
//!    the display name out of it;
//! 3. the sender posts its MAC address, its IP address and its device name
//!    to the device.
//!
//! A receiver that does not answer the search is invisible; a receiver that
//! answers but serves no description document is found and then dropped. The
//! three steps are therefore implemented here as they are documented.
//!
//! HOW: the crate is a set of small, testable pieces — a pure SSDP codec
//! ([`ssdp`]), a pure description builder ([`description`]), a bounded HTTP
//! server with a WebSocket handshake ([`api`]) and a state machine
//! ([`engine`]) that ties them together. Every piece can be tested without a
//! network, and the end-to-end tests talk to a real socket.
//!
//! ## The honest limit
//!
//! The **transport of the picture of the screen** of a Galaxy device is
//! proprietary and not publicly documented. The Smart View SDK is a closed
//! framework of `.jar` libraries. Everything else on the channel **is**
//! implemented, because it is not proprietary in any way that matters:
//!
//! - the channel is served under the name that the sender gives it —
//!   `ws://<host>/api/v2/channels/<name>`, which is the URL that the client
//!   library of Samsung builds;
//! - the greeting of the receiver is the event `ms.channel.connect`, and the
//!   events `ms.channel.ready`, `ms.channel.unauthorized` and the
//!   heartbeat `__ping` are understood;
//! - the **keys of the remote control** are read — the method
//!   `ms.remote.control` with the key in the parameter `DataOfCmd`.
//!
//! A proprietary format is not an unimplementable one. The Miracast
//! receiver, the Google Cast receiver and the AirPlay receiver of this
//! project are implementations of formats that nobody documents either.
//! The limit is therefore **one** thing: the picture of the screen. A
//! Galaxy device finds the receiver, connects to it, holds the channel open
//! and can drive it with its remote — and no picture of the screen flows.
//!
//! Smart View needs **no** HDCP protection and **no** decryption: it is an
//! open Samsung feature on Galaxy devices, not a DRM pipeline like FairPlay.
//! Nothing in this crate invents a restriction that the protocol does not
//! have.
//!
//! ## The modules
//!
//! | Module | What it does |
//! |---|---|
//! | [`ssdp`] | The announcement: the answer to an `M-SEARCH`, the `NOTIFY` alive and the `NOTIFY` byebye. |
//! | [`description`] | The description document the `LOCATION` header points at: display name, model, base URL, capabilities. |
//! | [`api`] | The HTTP API below `/api/v2/`: the API root, the resource description, the base URL, the handshake of the sender and the WebSocket upgrade. |
//! | [`lifecycle`] | The public state of the engine: the state, the ports and the reason of the last failure. |
//! | [`session`] | The single-session rule: the peer that announced itself and the channel that is open. |
//! | [`engine`] | The implementation of the trait `ProtocolEngine` of the core: the state machine and the lifecycle. |
//! | [`error`] | The errors of the crate. |
//!
//! ## Example
//!
//! ```
//! use phairplay_core::protocol::ProtocolEngine;
//! use phairplay_smartview::engine::{SmartViewConfig, SmartViewEngine};
//!
//! let engine = SmartViewEngine::new(SmartViewConfig::default());
//! assert_eq!(engine.kind().as_str(), "smartview");
//! assert_eq!(
//!     engine.status().state,
//!     phairplay_core::protocol::ProtocolState::Idle
//! );
//! ```

#![warn(missing_docs)]

pub mod api;
pub mod description;
pub mod engine;
pub mod error;
pub mod lifecycle;
pub mod session;
pub mod ssdp;

pub use api::{ApiLimits, ApiRequest, ApiResponse, ApiServer};
pub use description::{DeviceDescription, description_document};
pub use engine::{SmartViewConfig, SmartViewEngine};
pub use error::SmartViewError;
pub use session::{PeerInfo, SessionRegistry};
pub use ssdp::{
    MAX_PACKET_BYTES, MULTICAST_ADDRESS, SEARCH_TARGET, SSDP_PORT, SsdpResponder,
    is_smartview_search, notify_alive, notify_byebye, search_response,
};
