//! The Rust backend of the PhairPlayRust dashboard.
//!
//! WHAT: The HTTP and WebSocket server that the dashboard of PhairPlayRust
//! talks to. It shows the state of the receiver and lets the user switch
//! the protocols on and off.
//!
//! WHY: The dashboard is a **local control panel, not a network service**.
//! The server binds to the loopback interface and to nothing else: a
//! receiver that can be reached from the network would be a device on the
//! local network that anybody can switch off. The privacy promise of
//! PhairPlayRust — no account, no telemetry, no advertising, no cloud —
//! would end at the first port that is open to the network.
//!
//! Every answer of the API is read from the receiver itself: the sessions
//! come from the session registry of the core, the counters from the
//! metrics of the core, the status from the engines. The API invents
//! nothing.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_web::api::router;
//! use phairplay_web::state::AppState;
//!
//! let state = AppState::with_default_configuration()
//!     .unwrap_or_else(|error| panic!("the state must start: {error}"));
//! let app = router(state);
//! ```

#![warn(missing_docs)]

pub mod api;
pub mod assets;
pub mod dto;
pub mod error;
pub mod events;
pub mod server;
pub mod state;

#[cfg(test)]
mod test_support;

pub use api::router;
pub use error::ApiError;
pub use server::{LOOPBACK_HOST, serve};
pub use state::AppState;
