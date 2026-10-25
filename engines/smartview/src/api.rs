//! The HTTP API of the Smart View receiver.
//!
//! WHAT: the HTTP surface below `/api/v2/` that a Galaxy device talks to
//! after it has found the receiver over SSDP: the API root, the resource
//! description, the base URL, the handshake of the sender and the WebSocket
//! upgrade of the channel.
//!
//! WHY: Samsung documents the root of the API as
//! `http://<IP>:8001/api/v2/` and documents that the sender posts its MAC
//! address, its IP address and its device name to the device. The three
//! steps of the discovery and the handshake therefore have to be served —
//! a receiver that is found but does not answer is useless.
//!
//! HOW: the server is a small, hand-written HTTP/1.1 server. It was **not**
//! built on a web framework: an engine crate must not drag a web stack
//! into every binary of the workspace (the UniFFI library and the native
//! shells link the engines), and the DLNA engine already showed that a
//! bounded reader is a few hundred lines. Every read has a limit and a
//! deadline, and the answer to a request that violates one is a status code
//! and not a crash.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_smartview::api::{ApiLimits, Method};
//!
//! let limits = ApiLimits::default();
//! assert!(limits.max_body_bytes > 0);
//! assert_eq!(Method::parse("GET"), Method::Get);
//! ```

use std::net::SocketAddr;
use std::sync::Arc;

use phairplay_core::protocol::ProtocolKind;
use phairplay_core::session::{SessionEndReason, SessionEvent, SessionSnapshot};
use tokio::io::{AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

use crate::description::DeviceDescription;
use crate::error::SmartViewError;
use crate::session::SessionRegistry;

pub mod channel;
pub mod message;
mod reader;
mod routes;
mod server;
mod types;

pub use channel::{Frame, accept_key};
pub use message::{ChannelEvent, ChannelMessage};
pub use reader::read_request;
pub use routes::{
    RouteOutcome, is_channel_path, is_websocket_upgrade, route, upgrade_response,
};
pub use server::ApiServer;
pub use types::{ApiLimits, ApiRequest, ApiResponse, Method};

/// The root of the API.
pub const API_ROOT_PATH: &str = "/api/v2/";

/// The path of the resource description.
pub const RESOURCES_PATH: &str = "/api/v2/resources";

/// The path of the base URL of the receiver.
pub const BASE_URL_PATH: &str = "/api/v2/baseurl";

/// The prefix of the path of a WebSocket channel.
///
/// WHY a prefix and not a fixed path: the client of Samsung builds the URL
/// of the channel from the base URL of the service and the name of the
/// channel — `ws://<host>/api/v2/channels/<name>`. The name of the channel
/// is chosen by the sender, so the receiver has to accept every name under
/// this prefix and not one fixed name.
///
/// EXAMPLE: the channel `samsung.remote.control` of the remote control of a
/// Samsung television is `ws://192.168.1.10:8001/api/v2/channels/samsung.remote.control`.
pub const CHANNEL_PATH_PREFIX: &str = "/api/v2/channels/";

/// The path of the WebSocket channel.
///
/// WHY it still exists: the name of the channel that a sender uses for the
/// picture of the screen of a device is not documented, and a sender that
/// asks for it without a name finds this path. It is the same handler as
/// every other channel.
pub const WEBSOCKET_PATH: &str = "/api/v2/websocket";

/// Everything the request handler needs.
#[derive(Debug)]
pub struct ApiContext {
    /// The description document of the receiver.
    pub description: DeviceDescription,
    /// The base URL of the API.
    pub base_url: String,
    /// The URL of the description document.
    pub description_url: String,
    /// The registry of the one session the receiver serves.
    pub sessions: Arc<SessionRegistry>,
    /// The channel the session events are reported on.
    pub events: tokio::sync::mpsc::Sender<SessionEvent>,
    /// The limits of the reader.
    pub limits: ApiLimits,
}

/// Handles one client connection.
async fn handle_connection(stream: TcpStream, peer: SocketAddr, context: Arc<ApiContext>) {
    let mut reader = BufReader::new(stream);
    let request = match read_request(&mut reader, &context.limits, peer).await {
        Ok(request) => request,
        Err(error) => {
            let response = error_response(&error);
            let mut stream = reader.into_inner();
            let _ = write_response(&mut stream, &response, true).await;
            return;
        }
    };
    match route(&request, &context) {
        RouteOutcome::Response(response) => {
            let include_body = request.method != Method::Head;
            let mut stream = reader.into_inner();
            let _ = write_response(&mut stream, &response, include_body).await;
        }
        RouteOutcome::Upgrade { key, session } => {
            let response = upgrade_response(&key);
            {
                let stream = reader.get_mut();
                if write_response(stream, &response, false).await.is_err() {
                    context.sessions.close(session.id);
                    return;
                }
            }
            report_started(&context, &session).await;

            // WHY the event comes first: the client library of Samsung waits
            // for the event `ms.channel.connect` before it says anything on
            // the channel. The event carries the token of the session and the
            // name of the channel.
            let event = message::ChannelEvent::Connect {
                token: session.id.to_string(),
            };
            if let Err(error) = channel::write_event(reader.get_mut(), &event).await {
                context.sessions.close(session.id);
                report_ended(&context, session.id, false).await;
                return;
            }

            let outcome = channel::hold_channel(&mut reader, &context.limits).await;
            context.sessions.close(session.id);
            report_ended(&context, session.id, outcome.is_ok()).await;
        }
    }
}

/// The answer to a request that could not be read.
fn error_response(error: &SmartViewError) -> ApiResponse {
    let (status, reason) = match error {
        SmartViewError::LimitExceeded(_) => (413, "Payload Too Large"),
        SmartViewError::UriTooLong(_) => (414, "URI Too Long"),
        SmartViewError::HeadersTooLarge(_) => (431, "Request Header Fields Too Large"),
        SmartViewError::Protocol(_) | SmartViewError::Request(_) | SmartViewError::Handshake(_) => {
            (400, "Bad Request")
        }
        SmartViewError::Ssdp(_) | SmartViewError::Description(_) => (500, "Internal Server Error"),
    };
    ApiResponse::json(
        status,
        reason,
        &serde_json::json!({ "error": error.to_string() }),
    )
}

/// Reports the start of a session.
async fn report_started(context: &ApiContext, session: &crate::session::ActiveSession) {
    let event = SessionEvent::Started {
        session: SessionSnapshot {
            id: session.id,
            protocol: ProtocolKind::SmartView,
            sender_name: session.peer.name.clone(),
            sender_address: Some(session.address.to_string()),
            video: None,
            audio: None,
            started_at: session.started_at,
        },
    };
    // WHY: the event is reported without blocking. A full channel of the
    // supervisor must not stall the connection handler — that would give a
    // peer a way to hold the receiver busy.
    if context.events.try_send(event).is_err() {
        tracing::warn!("the session event channel is full; the start was not reported");
    }
}

/// Reports the end of a session.
async fn report_ended(context: &ApiContext, id: phairplay_core::session::SessionId, clean: bool) {
    let reason = if clean {
        SessionEndReason::SenderDisconnected
    } else {
        SessionEndReason::Failed("the channel broke".to_string())
    };
    let event = SessionEvent::Ended {
        session: id,
        reason,
    };
    if context.events.try_send(event).is_err() {
        tracing::warn!("the session event channel is full; the end was not reported");
    }
}

/// Writes a response to a client connection.
///
/// # Errors
/// Returns the I/O error of the write.
pub async fn write_response(
    stream: &mut TcpStream,
    response: &ApiResponse,
    include_body: bool,
) -> std::io::Result<()> {
    let mut head = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n",
        response.status,
        response.reason,
        response.content_type,
        response.body.len()
    );
    // WHY: the answer of the WebSocket upgrade carries its own
    // `Connection: Upgrade` header. A second `Connection: close` would tell
    // the sender to close the channel it has just opened.
    if !response
        .extra_headers
        .iter()
        .any(|(name, _)| name.eq_ignore_ascii_case("connection"))
    {
        head.push_str("Connection: close\r\n");
    }
    for (name, value) in &response.extra_headers {
        head.push_str(&format!("{name}: {value}\r\n"));
    }
    head.push_str("\r\n");
    stream.write_all(head.as_bytes()).await?;
    if include_body {
        stream.write_all(&response.body).await?;
    }
    stream.flush().await
}

#[cfg(test)]
mod tests;
