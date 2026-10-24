//! The life of one sender.
//!
//! WHY: the engine owns the socket and the state machine; the peer module
//! owns the conversation with a single sender. The split keeps both small
//! enough to be read at once — and the conversation with a peer on the local
//! network is exactly the place where a mistake hurts.
//!
//! HOW: the peer reads request after request from one connection, routes
//! each one, and writes the answer back. The session begins with `PLAY` and
//! ends with `TEARDOWN` or with the closing of the connection.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use phairplay_core::protocol::{EngineContext, ProtocolKind, ProtocolState};
use phairplay_core::session::{SessionEndReason, SessionEvent, SessionId, SessionSnapshot};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::engine::Inner;
use crate::rtsp::{self, RtspRequest};

/// The answer to a sender that is refused because a session is already active.
const REFUSAL: &str = "RTSP/1.0 503 Service Unavailable\r\nCSeq: 0\r\n\r\n";

/// The answer to a sender that sends a malformed request.
const BAD_REQUEST: &str = "RTSP/1.0 400 Bad Request\r\nCSeq: 0\r\n\r\n";

/// The answer to a sender that uses a method the sink does not implement.
const NOT_IMPLEMENTED: &str = "RTSP/1.0 501 Not Implemented\r\nCSeq: 0\r\n\r\n";

/// Answers a refused sender with `503 Service Unavailable`.
pub(crate) async fn refuse(mut stream: TcpStream) {
    let _ = stream.write_all(REFUSAL.as_bytes()).await;
    let _ = stream.flush().await;
    let _ = stream.shutdown().await;
}

/// Serves one sender from the first request to the last.
pub(crate) async fn run_peer(
    inner: Arc<Inner>,
    mut stream: TcpStream,
    ctx: EngineContext,
    address: SocketAddr,
) {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 4096];
    let mut session: Option<SessionId> = None;
    let mut shutdown = inner.shutdown_signal();

    'peer: loop {
        // The read ends when the sender sends something, when the sender says
        // nothing for too long, or when the engine stops.
        let outcome = tokio::select! {
            biased;
            _ = shutdown.changed() => ReadOutcome::Shutdown,
            result = tokio::time::timeout(idle_timeout(), stream.read(&mut chunk)) => {
                match result {
                    Ok(Ok(0)) => ReadOutcome::Eof,
                    Ok(Ok(count)) => ReadOutcome::Data(count),
                    Ok(Err(error)) => {
                        tracing::debug!(%error, "miracast could not read from the sender");
                        ReadOutcome::Error
                    }
                    Err(_) => {
                        tracing::debug!("miracast closed a sender that said nothing");
                        ReadOutcome::Idle
                    }
                }
            }
        };
        let read = match outcome {
            ReadOutcome::Data(count) => count,
            ReadOutcome::Eof | ReadOutcome::Error | ReadOutcome::Idle | ReadOutcome::Shutdown => {
                break;
            }
        };
        buffer.extend_from_slice(&chunk[..read]);

        if buffer.len() > rtsp::MAX_MESSAGE_BYTES {
            tracing::warn!("miracast refused a message that is too large");
            break;
        }

        // The sender sends request after request; each one ends with an empty
        // line. A partial request stays in the buffer until the rest arrives.
        while let Some(end) = find_header_end(&buffer) {
            let request_bytes: Vec<u8> = buffer.drain(..end).collect();
            let request = match rtsp::read_request(&request_bytes) {
                Ok(request) => request,
                Err(error) => {
                    tracing::warn!(%error, "miracast refused a malformed request");
                    let _ = stream.write_all(BAD_REQUEST.as_bytes()).await;
                    let _ = stream.flush().await;
                    return;
                }
            };

            let response = match rtsp::route(&request) {
                Ok(response) => response,
                Err(error) => {
                    tracing::debug!(%error, "miracast refused a method");
                    let _ = stream.write_all(NOT_IMPLEMENTED.as_bytes()).await;
                    let _ = stream.flush().await;
                    return;
                }
            };

            if request.method == "PLAY" && session.is_none() {
                let id = next_session(&inner);
                session = Some(id);
                let sender_name = sender_name_from(&request);
                register(&inner, id, sender_name.clone().unwrap_or_default());
                set_state(&inner, ProtocolState::Streaming);
                let _ = ctx
                    .events
                    .send(SessionEvent::Started {
                        session: SessionSnapshot {
                            id,
                            protocol: ProtocolKind::Miracast,
                            sender_name: sender_name.clone(),
                            sender_address: Some(address.ip().to_string()),
                            video: None,
                            audio: None,
                            started_at: std::time::Instant::now(),
                        },
                    })
                    .await;
            }

            if stream
                .write_all(&response.to_bytes(request.cseq))
                .await
                .is_err()
            {
                break;
            }
            let _ = stream.flush().await;

            if request.method == "TEARDOWN" {
                // WHY: the label is needed — a plain `break` would only leave
                // the inner loop over the buffered requests, and the session
                // would stay open until the idle timeout fired.
                break 'peer;
            }
        }
    }

    if let Some(id) = session {
        unregister(&inner, id);
        let next = if inner.is_running() {
            ProtocolState::Advertising
        } else {
            ProtocolState::Stopped
        };
        set_state(&inner, next);
        let _ = ctx
            .events
            .send(SessionEvent::Ended {
                session: id,
                reason: SessionEndReason::SenderDisconnected,
            })
            .await;
    }
}

/// The next session identifier of this engine.
fn next_session(inner: &Arc<Inner>) -> SessionId {
    SessionId(inner.next_session_index())
}

/// Records a sender that connected.
fn register(inner: &Arc<Inner>, session: SessionId, name: String) {
    if let Ok(mut sessions) = inner.sessions.lock() {
        sessions.insert(session, name);
    }
}

/// Forgets a sender that disconnected.
fn unregister(inner: &Arc<Inner>, session: SessionId) {
    if let Ok(mut sessions) = inner.sessions.lock() {
        sessions.remove(&session);
    }
}

/// Writes the state of the engine and refreshes the number of senders.
fn set_state(inner: &Arc<Inner>, state: ProtocolState) {
    if let Ok(mut engine_state) = inner.state.lock() {
        engine_state.state = Some(state);
        engine_state.peer_count = inner.sessions.lock().map(|s| s.len()).unwrap_or(0) as u32;
    }
}

/// The name of the sender, taken from the `User-Agent` header.
fn sender_name_from(request: &RtspRequest) -> Option<String> {
    request
        .headers
        .get("User-Agent")
        .map(|value| value.split_whitespace().next().unwrap_or(value).to_string())
        .filter(|value| !value.is_empty())
}

/// The end of the header block of a request, including the empty line.
fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|index| index + 4)
}

/// The time the engine waits before it reports a stalled peer.
///
/// WHY: the deadline is the idle timeout of the *performance budget* of the
/// core, not a number of this engine. The engine reads it from the
/// configuration, so the value has exactly one owner.
fn idle_timeout() -> Duration {
    phairplay_core::config::PerformanceConfig::default().idle_timeout()
}

/// How a read from the sender ended.
enum ReadOutcome {
    /// The sender sent data.
    Data(usize),
    /// The sender closed the connection.
    Eof,
    /// The connection failed.
    Error,
    /// The sender said nothing for too long.
    Idle,
    /// The engine stopped.
    Shutdown,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    #[test]
    fn the_header_end_is_found() {
        let buffer = b"OPTIONS x RTSP/1.0\r\nCSeq: 1\r\n\r\n";
        assert_eq!(find_header_end(buffer), Some(buffer.len()));
    }

    #[test]
    fn an_incomplete_request_has_no_header_end() {
        assert_eq!(find_header_end(b"OPTIONS x RTSP/1.0\r\n"), None);
    }

    #[test]
    fn the_sender_name_is_taken_from_the_user_agent() {
        let mut headers = std::collections::HashMap::new();
        headers.insert("User-Agent".to_string(), "Windows 11 Miracast".to_string());
        let request = RtspRequest {
            method: "PLAY".to_string(),
            uri: "rtsp://x/".to_string(),
            cseq: 1,
            headers,
        };
        assert_eq!(sender_name_from(&request).as_deref(), Some("Windows"));
    }

    #[test]
    fn a_request_without_a_user_agent_has_no_name() {
        let request = RtspRequest {
            method: "PLAY".to_string(),
            uri: "rtsp://x/".to_string(),
            cseq: 1,
            headers: std::collections::HashMap::new(),
        };
        assert!(sender_name_from(&request).is_none());
    }
}
