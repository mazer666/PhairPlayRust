//! The accept loop of the API server.
//!
//! WHY: the accept loop is the piece that owns the listening socket. It
//! has to end *immediately* when the engine stops — and not only when the
//! next connection arrives — so that the port is free again and a receiver
//! that is restarted right away does not fail with "address already in
//! use".
//!
//! HOW: the accept and the shutdown signal are raced, every connection is
//! handled in its own task, and the task of the accept loop is aborted when
//! the server is dropped.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_smartview::api::ApiServer;
//!
//! // The server is bound by the engine; this is only the type.
//! let _ = std::mem::size_of::<ApiServer>();
//! ```

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tokio::net::TcpListener;
use tokio::sync::watch;
use tokio::task::JoinHandle;

use crate::api::{ApiContext, handle_connection};

#[derive(Debug)]
/// The listening socket of the API and the task that accepts the
/// connections.
pub struct ApiServer {
    local_addr: SocketAddr,
    worker: Option<JoinHandle<()>>,
    running: Arc<AtomicBool>,
    shutdown: watch::Sender<bool>,
}

impl ApiServer {
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
        F: FnOnce(u16) -> Arc<ApiContext>,
    {
        let listener = TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], port))).await?;
        let local_port = listener
            .local_addr()
            .map(|address| address.port())
            .unwrap_or(port);
        let context = context(local_port);
        let running = Arc::new(AtomicBool::new(true));
        let worker_running = Arc::clone(&running);
        let (shutdown, mut shutdown_rx) = watch::channel(false);
        let worker = tokio::spawn(async move {
            loop {
                if !worker_running.load(Ordering::Relaxed) {
                    break;
                }
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
    pub async fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        let _ = self.shutdown.send(true);
        if let Some(worker) = self.worker.take() {
            let _ = tokio::time::timeout(Duration::from_secs(2), worker).await;
        }
    }
}

impl Drop for ApiServer {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        let _ = self.shutdown.send(true);
        if let Some(worker) = self.worker.take() {
            worker.abort();
        }
    }
}
