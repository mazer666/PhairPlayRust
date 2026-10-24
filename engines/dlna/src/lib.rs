//! The UPnP / DLNA MediaRenderer engine of PhairPlayRust.
//!
//! WHAT: this crate implements a DLNA media renderer — the protocol that
//! "Cast to Device" of Windows, the renderer list of VLC and every mobile
//! UPnP control point talk to. It announces the renderer over SSDP, serves
//! the UPnP description documents, answers the SOAP actions of the three
//! services (`ConnectionManager`, `AVTransport`, `RenderingControl`) and
//! notifies the subscribers over GENA.
//!
//! WHY: a renderer is the most exposed surface of the receiver. Every
//! request comes from an arbitrary device of the local network, and one of
//! the GENA features — the callback into the control point — is a request
//! *out* of the renderer. Both directions are hardened here:
//!
//! * the XML reader rejects a document type, so no entity can be declared
//!   and no external document can be read ([`xml`]);
//! * the GENA guard accepts a callback only from a literal private IPv4
//!   address of the local subnet, so the renderer cannot be turned into a
//!   proxy for the cloud metadata service ([`gena`]);
//! * every read is bounded — request line, header count, header length,
//!   body length — and every blocking read has a deadline ([`http`]);
//! * a second control point is **rejected** while the first one streams; it
//!   is never queued ([`state`], [`soap`]).
//!
//! HOW: the crate is a set of small, testable pieces — a pure XML reader, a
//! pure message codec, a pure state machine — that the [`engine::DlnaEngine`]
//! wires together. Every piece can be tested without a network.
//!
//! ## Example
//!
//! ```
//! use phairplay_dlna::engine::{DlnaConfig, DlnaEngine};
//! use phairplay_core::protocol::ProtocolEngine;
//!
//! let engine = DlnaEngine::new(DlnaConfig::default());
//! assert_eq!(engine.kind().as_str(), "dlna");
//! assert_eq!(engine.status().state, phairplay_core::protocol::ProtocolState::Idle);
//! ```

#![warn(missing_docs)]

pub mod description;
pub mod engine;
pub mod gena;
pub mod http;
pub mod reporter;
pub mod server;
pub mod soap;
pub mod ssdp;
pub mod state;
pub mod xml;

pub use description::{device_description, service_description};
pub use engine::{DlnaConfig, DlnaEngine};
pub use gena::{GenaError, GenaEventManager};
pub use http::{HttpLimits, HttpRequest, HttpResponse, Method};
pub use reporter::SessionReporter;
pub use server::{HttpServer, ServerContext};
pub use soap::{SoapDispatcher, SoapRequest, SoapResponse};
pub use ssdp::{SsdpResponder, is_renderer_search, notify_alive, notify_byebye, search_response};
pub use state::{RendererState, TransportState};
pub use xml::{XmlDocument, XmlElement, XmlError};
