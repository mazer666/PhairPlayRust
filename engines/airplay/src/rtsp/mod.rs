//! The RTSP / HTTP-like control channel of AirPlay.
//!
//! WHY: an AirPlay sender opens one TCP connection and speaks RTSP on it —
//! `OPTIONS`, `ANNOUNCE`, `SETUP`, `RECORD`, … — and then, for the AirPlay 2
//! handshake, switches to an HTTP-like dialect on the *same* port (`GET
//! /info`, `POST /pair-setup`, `PUT /photo`). Both dialects share one
//! request/response model, one reader and one router, so that a peer can
//! never smuggle a request past the limits by switching the dialect.
//!
//! | Module | Responsibility |
//! |---|---|
//! | [`request`] | The typed request: method, URI, headers, body. |
//! | [`response`] | The typed response and its wire serialisation. |
//! | [`reader`] | The bounded, deadline-driven reader. |
//! | [`router`] | The dispatch of a request onto the session state. |

pub mod http;
pub mod reader;
pub mod request;
pub mod response;
pub mod router;

pub use reader::{RtspRequestReader, StreamBuffer};
pub use router::{RouteContext, route};
pub use request::{RtspMethod, RtspRequest};
pub use response::{RtspResponse, SERVER_TOKEN};
