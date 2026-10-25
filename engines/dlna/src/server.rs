//! The HTTP server of the DLNA control plane.
//!
//! WHAT: the server answers four kinds of request — the device description,
//! the service descriptions, the SOAP actions and the GENA subscriptions.
//!
//! WHY: every one of those requests comes from a device on the LAN. The
//! server therefore never reads more than the configured limits, never
//! waits longer than the idle timeout, and never lets a second control
//! point take the renderer away from the first one.
//!
//! HOW: the accept loop is a task; every connection is handled in its own
//! task so that a slow control point cannot block the others. The
//! connection counter feeds the `peer_count` of the engine status.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_dlna::server::Lifecycle;
//! use phairplay_core::protocol::ProtocolState;
//!
//! let lifecycle = Lifecycle::new();
//! assert_eq!(lifecycle.state(), ProtocolState::Idle);
//! ```

use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use phairplay_core::privacy::mask_socket_addr;
use phairplay_core::protocol::{PeerLimit, ProtocolState};
use tokio::io::BufReader;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch;
use tokio::task::JoinHandle;

use crate::gena::GenaEventManager;
use crate::http::{HttpLimits, HttpResponse, Method, read_request, write_response};
use crate::soap::SoapDispatcher;
use crate::state::RendererState;

/// The description path of the device document.
pub const DESCRIPTION_PATH: &str = "/description.xml";

/// The runtime state the engine and the server share.
///
/// WHY: the engine status has to be readable from any thread while a
/// transition is in flight. The state therefore lives behind one mutex and
/// is copied out for the status snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lifecycle {
    state: ProtocolState,
    tcp_port: Option<u16>,
    udp_port: Option<u16>,
    connections: u32,
    last_error: Option<String>,
    enabled: bool,
}

impl Default for Lifecycle {
    fn default() -> Self {
        Self::new()
    }
}

impl Lifecycle {
    /// The state of a receiver that has not been started.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: ProtocolState::Idle,
            tcp_port: None,
            udp_port: None,
            connections: 0,
            last_error: None,
            enabled: true,
        }
    }

    /// The current protocol state.
    #[must_use]
    pub fn state(&self) -> ProtocolState {
        self.state
    }

    /// Sets the protocol state.
    pub fn set_state(&mut self, state: ProtocolState) {
        self.state = state;
    }

    /// The TCP port of the description server.
    #[must_use]
    pub fn tcp_port(&self) -> Option<u16> {
        self.tcp_port
    }

    /// Sets the TCP port of the description server.
    pub fn set_tcp_port(&mut self, port: Option<u16>) {
        self.tcp_port = port;
    }

    /// The UDP port of the SSDP responder.
    #[must_use]
    pub fn udp_port(&self) -> Option<u16> {
        self.udp_port
    }

    /// Sets the UDP port of the SSDP responder.
    pub fn set_udp_port(&mut self, port: Option<u16>) {
        self.udp_port = port;
    }

    /// The number of connections that are currently open.
    #[must_use]
    pub fn connections(&self) -> u32 {
        self.connections
    }

    /// Records that a connection was opened.
    pub fn connection_opened(&mut self) {
        self.connections = self.connections.saturating_add(1);
    }

    /// Records that a connection was closed.
    pub fn connection_closed(&mut self) {
        self.connections = self.connections.saturating_sub(1);
    }

    /// The reason of the last failure.
    #[must_use]
    pub fn last_error(&self) -> Option<String> {
        self.last_error.clone()
    }

    /// Records the reason of the last failure.
    pub fn set_last_error(&mut self, reason: Option<String>) {
        self.last_error = reason;
    }

    /// Whether the protocol is enabled.
    #[must_use]
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Sets whether the protocol is enabled.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

/// Everything the request handler needs.
#[derive(Debug)]
pub struct ServerContext {
    /// The friendly name of the renderer.
    pub friendly_name: String,
    /// The unique device name of the renderer.
    pub udn: String,
    /// The URL of the device description.
    pub presentation_url: String,
    /// The state of the renderer.
    pub state: Arc<RendererState>,
    /// The event manager.
    pub events: Arc<GenaEventManager>,
    /// The SOAP dispatcher.
    pub dispatcher: Arc<SoapDispatcher>,
    /// The limits of the HTTP reader.
    pub limits: HttpLimits,
    /// The shared lifecycle.
    pub lifecycle: Arc<Mutex<Lifecycle>>,
    /// The rule that says how many peers the renderer serves at the same
    /// time.
    ///
    /// WHY the rule is here: the rule that says how many peers a protocol
    /// serves has **one** owner — the list
    /// `session.multi_peer_protocols` of the configuration — and the engine
    /// asks it instead of carrying a number of its own.
    pub peer_limit: PeerLimit,
}

/// The HTTP server of the control plane.
#[derive(Debug)]
pub struct HttpServer {
    local_addr: SocketAddr,
    worker: Option<JoinHandle<()>>,
    running: Arc<AtomicBool>,
    shutdown: watch::Sender<bool>,
}

impl HttpServer {
    /// Binds the server and starts the accept loop.
    ///
    /// WHY: the context is built by a closure and not passed in, because
    /// the context carries the port of the server — and the port is only
    /// known after the socket is bound. A port of `0` picks a free one.
    ///
    /// # Errors
    /// Returns the I/O error when the socket cannot be bound.
    pub async fn bind<F>(port: u16, context: F) -> std::io::Result<Self>
    where
        F: FnOnce(u16) -> Arc<ServerContext>,
    {
        let listener = TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], port))).await?;
        let local_port = listener.local_addr().map(|a| a.port()).unwrap_or(port);
        let context = context(local_port);
        let running = Arc::new(AtomicBool::new(true));
        let worker_running = Arc::clone(&running);
        let (shutdown, mut shutdown_rx) = watch::channel(false);
        let worker = tokio::spawn(async move {
            loop {
                if !worker_running.load(Ordering::Relaxed) {
                    break;
                }
                // The accept and the shutdown signal are raced, so that the
                // loop ends *immediately* when the engine stops — and not
                // only when the next connection arrives.
                let accepted = tokio::select! {
                    biased;
                    _ = shutdown_rx.changed() => break,
                    accepted = listener.accept() => accepted,
                };
                let Ok((stream, peer)) = accepted else {
                    if !worker_running.load(Ordering::Relaxed) {
                        break;
                    }
                    continue;
                };
                let context = Arc::clone(&context);
                tokio::spawn(async move {
                    handle_connection(stream, peer, context).await;
                });
            }
        });
        Ok(Self {
            local_addr: SocketAddr::from(([0, 0, 0, 0], local_port)),
            worker: Some(worker),
            running,
            shutdown,
        })
    }

    /// The port the server listens on.
    #[must_use]
    pub fn local_port(&self) -> u16 {
        self.local_addr.port()
    }

    /// The address the server listens on.
    #[must_use]
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Stops the accept loop and releases the socket.
    ///
    /// WHY: the port has to be free the moment the engine reports that it
    /// has stopped. An aborted task would keep the listener alive until the
    /// runtime drops it, and a receiver that is restarted immediately would
    /// fail with "address already in use".
    pub async fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        let _ = self.shutdown.send(true);
        if let Some(worker) = self.worker.take() {
            let _ = tokio::time::timeout(std::time::Duration::from_secs(2), worker).await;
        }
    }
}

impl Drop for HttpServer {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        let _ = self.shutdown.send(true);
        if let Some(worker) = self.worker.take() {
            worker.abort();
        }
    }
}

mod routing;

use routing::route;

/// Handles one client connection.
async fn handle_connection(stream: TcpStream, peer: SocketAddr, context: Arc<ServerContext>) {
    // WHY the second client is refused: DLNA is a single-session protocol.
    // The second client is told so instead of being queued — a queued
    // client waits for a session that may never end.
    //
    // WHY the limit comes from the context: the rule has **one** owner —
    // the list `session.multi_peer_protocols` of the configuration.
    let refused = {
        let mut lifecycle = context.lifecycle.lock().unwrap_or_else(|e| e.into_inner());
        if lifecycle.connections() as usize >= context.peer_limit.max_peers() {
            true
        } else {
            lifecycle.connection_opened();
            false
        }
    };
    if refused {
        tracing::warn!(peer = %mask_socket_addr(&peer), "the renderer refused a second client");
        // WHY the answer and not a closed socket: a client that gets
        // nothing cannot tell a busy renderer from a broken one. The status
        // `503 Service Unavailable` says exactly that the renderer is busy.
        let response = HttpResponse::text(
            503,
            "Service Unavailable",
            "the renderer serves one client at a time",
        );
        let mut stream = stream;
        let _ = write_response(&mut stream, &response, true).await;
        return;
    }
    let mut reader = BufReader::new(stream);
    let request = match read_request(&mut reader, &context.limits, peer).await {
        Ok(request) => request,
        Err(error) => {
            let response = HttpResponse::text(error.status(), error.reason(), &error.to_string());
            let mut stream = reader.into_inner();
            let _ = write_response(&mut stream, &response, true).await;
            context
                .lifecycle
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .connection_closed();
            return;
        }
    };
    let response = route(&request, &context);
    let include_body = request.method != Method::Head;
    let mut stream = reader.into_inner();
    let _ = write_response(&mut stream, &response, include_body).await;
    context
        .lifecycle
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .connection_closed();
}

#[cfg(test)]
mod tests;
