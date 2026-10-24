//! The WebSocket channel of the live events.
//!
//! WHAT: The endpoint `GET /api/events`. It upgrades the connection to a
//! WebSocket and writes every event of the receiver into it as a JSON
//! object.
//!
//! WHY: The dashboard shows the live sessions without asking the server
//! again and again. A poll every second would be a small load, but it
//! would also show a state that is already a second old. The channel
//! carries the event itself, so the dashboard sees the moment a sender
//! appears.
//!
//! The channel is one way: the server writes, the dashboard reads. A
//! client that sends something is ignored — the panel is not a remote
//! control.
//!
//! EXAMPLE:
//!
//! ```text
//! {"type":"session_started","session":{"id":1,"protocol":"airplay", ... }}
//! {"type":"protocol_status_changed","status":{"protocol":"cast","state":"advertising", ... }}
//! ```

use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::Response;
use tokio::sync::broadcast::error::RecvError;

use crate::state::AppState;

/// `GET /api/events` — the channel of the live events.
pub async fn events(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| write_events(socket, state))
}

/// Writes the events of the receiver into the socket.
async fn write_events(mut socket: WebSocket, state: AppState) {
    let mut events = state.subscribe();
    loop {
        tokio::select! {
            event = events.recv() => {
                match event {
                    Ok(event) => {
                        let text = match serde_json::to_string(&event) {
                            Ok(text) => text,
                            Err(error) => {
                                tracing::error!("an event could not be serialised: {error}");
                                continue;
                            }
                        };
                        if socket.send(Message::Text(text.into())).await.is_err() {
                            break;
                        }
                    }
                    // WHY: a client that is too slow misses the oldest
                    // events. The receiver must not wait for it.
                    Err(RecvError::Lagged(_)) => continue,
                    Err(RecvError::Closed) => break,
                }
            }
            incoming = socket.recv() => {
                match incoming {
                    // The client closed the connection.
                    None => break,
                    Some(Ok(Message::Close(_))) => break,
                    // Everything else is ignored: the panel is a reader.
                    Some(Ok(_)) => {}
                    Some(Err(error)) => {
                        tracing::debug!("the event channel ended: {error}");
                        break;
                    }
                }
            }
        }
    }
}
