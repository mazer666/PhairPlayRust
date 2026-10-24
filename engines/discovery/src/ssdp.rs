//! The SSDP side of service discovery: sending `M-SEARCH` queries and
//! interpreting what the network answers.
//!
//! WHY: some senders discover a DLNA renderer over SSDP instead of over
//! DNS-SD. The receiver therefore has to be able to *ask* the network who
//! is out there, and it has to be able to understand the answers — an
//! unknown device on the LAN is free to send anything at all.
//!
//! HOW: the message codec is pure and testable; [`SsdpBrowser`] owns the
//! socket and the reader thread. The queue that feeds the consumer is
//! bounded twice — by the number of events *and* by a byte budget — because
//! a chatty neighbour must not be able to grow the queue without limit.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_discovery::ssdp::{SsdpSearchQuery, SsdpResponse};
//!
//! let query = SsdpSearchQuery::new("ssdp:all").with_mx(2);
//! assert!(query.to_message().starts_with("M-SEARCH * HTTP/1.1"));
//!
//! let response = SsdpResponse::parse(
//!     "HTTP/1.1 200 OK\r\nLOCATION: http://192.168.1.5:8200/description.xml\r\n\
//!      USN: uuid:abc::upnp:rootdevice\r\n\r\n",
//! )
//! .expect("a valid response");
//! assert_eq!(response.location, "http://192.168.1.5:8200/description.xml");
//! ```

use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, TrySendError, sync_channel};
use std::thread::JoinHandle;
use std::time::Duration;

/// The IPv4 multicast group SSDP uses.
pub const MULTICAST_ADDRESS: Ipv4Addr = Ipv4Addr::new(239, 255, 255, 250);

/// The port SSDP uses.
///
/// WHY: the value is fixed by the UPnP specification. Its single owner is
/// [`phairplay_core::config::ports::SSDP`].
pub const SSDP_PORT: u16 = phairplay_core::config::ports::SSDP;

/// The search target that matches every device.
pub const ST_ALL: &str = "ssdp:all";

/// The search target of the root device.
pub const ST_ROOT_DEVICE: &str = "upnp:rootdevice";

/// The search target of a media renderer.
pub const ST_MEDIA_RENDERER: &str = "urn:schemas-upnp-org:device:MediaRenderer:1";

/// The maximum size of an SSDP message this crate reads or writes.
///
/// WHY: the same cap applies to the DLNA renderer, which answers on the
/// same multicast group. Its single owner is
/// [`phairplay_core::config::limits::SSDP_MESSAGE_BYTES`].
pub const MAX_SSDP_MESSAGE_BYTES: usize = phairplay_core::config::limits::SSDP_MESSAGE_BYTES;

/// The maximum number of headers in one SSDP message.
pub const MAX_SSDP_HEADER_COUNT: usize = 32;

/// The maximum length of one SSDP header line.
pub const MAX_SSDP_HEADER_BYTES: usize = 1024;

/// The number of events the browser queues before it starts dropping.
pub const DEFAULT_EVENT_QUEUE_CAPACITY: usize = 64;

/// The byte budget of the event queue.
pub const DEFAULT_EVENT_QUEUE_BYTES: usize = 64 * 1024;

/// Failures of the SSDP client.
#[derive(Debug, thiserror::Error)]
pub enum SsdpError {
    /// The socket could not be bound.
    #[error("the SSDP socket could not be bound: {0}")]
    Bind(String),

    /// The socket could not be configured for multicast.
    #[error("the SSDP socket could not be configured: {0}")]
    Configure(String),

    /// The query could not be sent.
    #[error("the SSDP query could not be sent: {0}")]
    Send(String),
}

mod messages;

pub use messages::{SsdpEvent, SsdpNotification, SsdpResponse, SsdpSearchQuery};

/// The SSDP client: it asks the network which devices are out there.
///
/// WHY: the browser runs on its own thread with a blocking socket, because
/// the multicast socket has to be joined with the operating system's
/// multicast API — which the asynchronous runtime does not expose.
#[derive(Debug)]
pub struct SsdpBrowser {
    socket: Arc<UdpSocket>,
    events: Receiver<SsdpEvent>,
    running: Arc<AtomicBool>,
    queued_bytes: Arc<AtomicUsize>,
    dropped: Arc<AtomicUsize>,
    worker: Option<JoinHandle<()>>,
}

impl SsdpBrowser {
    /// Binds the browser socket.
    ///
    /// # Errors
    /// Returns [`SsdpError::Bind`] when the socket cannot be bound and
    /// [`SsdpError::Configure`] when the multicast options cannot be set.
    pub fn new() -> Result<Self, SsdpError> {
        Self::with_limits(DEFAULT_EVENT_QUEUE_CAPACITY, DEFAULT_EVENT_QUEUE_BYTES)
    }

    /// Binds the browser socket with an explicit queue budget.
    ///
    /// # Errors
    /// Returns [`SsdpError::Bind`] when the socket cannot be bound and
    /// [`SsdpError::Configure`] when the multicast options cannot be set.
    pub fn with_limits(capacity: usize, byte_budget: usize) -> Result<Self, SsdpError> {
        let socket = UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)))
            .map_err(|error| SsdpError::Bind(error.to_string()))?;
        socket
            .set_read_timeout(Some(Duration::from_millis(500)))
            .map_err(|error| SsdpError::Configure(error.to_string()))?;
        socket
            .join_multicast_v4(&MULTICAST_ADDRESS, &Ipv4Addr::UNSPECIFIED)
            .map_err(|error| SsdpError::Configure(error.to_string()))?;

        let (sender, events) = sync_channel(capacity);
        let running = Arc::new(AtomicBool::new(true));
        let queued_bytes = Arc::new(AtomicUsize::new(0));
        let dropped = Arc::new(AtomicUsize::new(0));

        let reader_socket = socket
            .try_clone()
            .map_err(|error| SsdpError::Bind(error.to_string()))?;
        let reader_running = Arc::clone(&running);
        let reader_bytes = Arc::clone(&queued_bytes);
        let reader_dropped = Arc::clone(&dropped);
        let worker = std::thread::Builder::new()
            .name("phairplay-ssdp-browser".to_string())
            .spawn(move || {
                let mut buffer = vec![0u8; MAX_SSDP_MESSAGE_BYTES];
                while reader_running.load(Ordering::Relaxed) {
                    match reader_socket.recv_from(&mut buffer) {
                        Ok((read, _)) => {
                            let text = String::from_utf8_lossy(&buffer[..read]).to_string();
                            let event = if SsdpResponse::parse(&text).is_some() {
                                SsdpResponse::parse(&text).map(SsdpEvent::Response)
                            } else {
                                SsdpNotification::parse(&text).map(SsdpEvent::Notification)
                            };
                            if let Some(event) = event {
                                push_bounded(
                                    &sender,
                                    event,
                                    &reader_bytes,
                                    &reader_dropped,
                                    byte_budget,
                                );
                            }
                        }
                        Err(error) => match error.kind() {
                            std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut => {}
                            _ => break,
                        },
                    }
                }
            })
            .map_err(|error| SsdpError::Bind(error.to_string()))?;

        Ok(Self {
            socket: Arc::new(socket),
            events,
            running,
            queued_bytes,
            dropped,
            worker: Some(worker),
        })
    }

    /// Sends an `M-SEARCH` query to the multicast group.
    ///
    /// # Errors
    /// Returns [`SsdpError::Send`] when the datagram cannot be sent.
    pub fn search(&self, search_target: &str) -> Result<(), SsdpError> {
        self.send(&SsdpSearchQuery::new(search_target))
    }

    /// Sends a prepared query.
    ///
    /// # Errors
    /// Returns [`SsdpError::Send`] when the datagram cannot be sent.
    pub fn send(&self, query: &SsdpSearchQuery) -> Result<(), SsdpError> {
        let message = query.to_message();
        if message.len() > MAX_SSDP_MESSAGE_BYTES {
            return Err(SsdpError::Send(
                "the query exceeds the message limit".to_string(),
            ));
        }
        self.socket
            .send_to(
                message.as_bytes(),
                SocketAddr::from((MULTICAST_ADDRESS, SSDP_PORT)),
            )
            .map_err(|error| SsdpError::Send(error.to_string()))?;
        Ok(())
    }

    /// Waits for the next event.
    #[must_use]
    pub fn recv_timeout(&self, timeout: Duration) -> Option<SsdpEvent> {
        self.events.recv_timeout(timeout).ok()
    }

    /// The number of events that were dropped because the queue was full.
    #[must_use]
    pub fn dropped(&self) -> usize {
        self.dropped.load(Ordering::Relaxed)
    }

    /// The number of bytes currently queued.
    #[must_use]
    pub fn queued_bytes(&self) -> usize {
        self.queued_bytes.load(Ordering::Relaxed)
    }

    /// Stops the reader thread.
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

impl Drop for SsdpBrowser {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Pushes an event into a queue that is bounded by count *and* by bytes.
fn push_bounded(
    sender: &SyncSender<SsdpEvent>,
    event: SsdpEvent,
    queued_bytes: &AtomicUsize,
    dropped: &AtomicUsize,
    byte_budget: usize,
) {
    let size = estimate_size(&event);
    let current = queued_bytes.load(Ordering::Relaxed);
    if current.saturating_add(size) > byte_budget {
        dropped.fetch_add(1, Ordering::Relaxed);
        return;
    }
    match sender.try_send(event) {
        Ok(()) => {
            queued_bytes.fetch_add(size, Ordering::Relaxed);
        }
        Err(TrySendError::Full(_)) | Err(TrySendError::Disconnected(_)) => {
            dropped.fetch_add(1, Ordering::Relaxed);
        }
    }
}

/// The approximate memory footprint of an event.
fn estimate_size(event: &SsdpEvent) -> usize {
    match event {
        SsdpEvent::Response(response) => {
            response.location.len()
                + response.usn.len()
                + response.search_target.len()
                + response.server.len()
                + response.cache_control.len()
        }
        SsdpEvent::Notification(notification) => {
            notification.usn.len()
                + notification.notification_type.len()
                + notification.location.as_deref().map_or(0, str::len)
        }
    }
}

#[cfg(test)]
mod tests;
