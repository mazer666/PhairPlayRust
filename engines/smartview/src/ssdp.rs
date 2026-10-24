//! The SSDP announcement of the Smart View receiver.
//!
//! WHAT: the part of the discovery that a Galaxy device sees: the answer to
//! an `M-SEARCH` for `urn:samsung.com:service:MultiScreenService:1`, the
//! `NOTIFY` that announces the receiver and the `NOTIFY` that takes it away
//! again.
//!
//! WHY: Samsung documents the search in "Device Discovery, Authentication,
//! and Pairing": a sender looks for
//! `ST: urn:samsung.com:service:MultiScreenService:1`, and a device that
//! offers the MultiScreen service answers with `HTTP/1.1 200 OK` and a
//! `LOCATION:` header that points at the description document. A receiver
//! that stays silent is invisible; a receiver that answers with a
//! `LOCATION` that leads nowhere is found and then dropped.
//!
//! HOW: the codec is pure and is unit tested without a network; the socket
//! lives on its own thread because the multicast socket has to be joined
//! with the operating system's multicast API, which the asynchronous runtime
//! does not expose. Every read has a deadline, so the thread notices a
//! `stop()` within half a second.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_smartview::ssdp::{notify_alive, search_response};
//!
//! let response = search_response(
//!     "http://10.0.0.5:8001/description.xml",
//!     "uuid:abc",
//!     "urn:samsung.com:service:MultiScreenService:1",
//! );
//! assert!(response.starts_with("HTTP/1.1 200 OK"));
//! assert!(response.contains("LOCATION: http://10.0.0.5:8001/description.xml"));
//!
//! let alive = notify_alive(
//!     "http://10.0.0.5:8001/description.xml",
//!     "uuid:abc",
//!     "urn:samsung.com:service:MultiScreenService:1",
//! );
//! assert!(alive.contains("NTS: ssdp:alive"));
//! ```

use std::io;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

use socket2::{Domain, Protocol, Socket, Type};

/// The IPv4 multicast group SSDP uses.
pub const MULTICAST_ADDRESS: Ipv4Addr = Ipv4Addr::new(239, 255, 255, 250);

/// The port SSDP uses.
///
/// WHY: the value is fixed by the UPnP specification. Its single owner is
/// [`phairplay_core::config::ports::SSDP`].
pub const SSDP_PORT: u16 = phairplay_core::config::ports::SSDP;

/// The search target of the Samsung MultiScreen service.
///
/// WHY: this is the target Samsung documents in "Device Discovery,
/// Authentication, and Pairing". A receiver that answers a different target
/// is not found by a Galaxy device.
pub const SEARCH_TARGET: &str = "urn:samsung.com:service:MultiScreenService:1";

/// The server string the receiver announces.
pub const SERVER: &str = "Linux/1.0 UPnP/1.0 PhairPlayRust/1.0";

/// The lifetime of an announcement, in seconds.
///
/// WHY: the value is fixed by the UPnP specification. Its single owner is
/// [`phairplay_core::config::timeouts::UPNP_MAX_AGE_SECONDS`].
pub const CACHE_CONTROL_MAX_AGE: u32 = phairplay_core::config::timeouts::UPNP_MAX_AGE_SECONDS;

/// The maximum size of an SSDP packet the receiver reads.
///
/// WHY: the same cap applies to the SSDP browser of the discovery crate.
/// Its single owner is
/// [`phairplay_core::config::limits::SSDP_MESSAGE_BYTES`].
pub const MAX_PACKET_BYTES: usize = phairplay_core::config::limits::SSDP_MESSAGE_BYTES;

/// The interval the reader thread wakes up in.
const READ_POLL_INTERVAL: Duration = Duration::from_millis(500);

/// The value of the `MAN` header of a search that asks for an answer.
const MAN_DISCOVER: &str = "ssdp:discover";

/// Whether a message is an `M-SEARCH` that this receiver should answer.
///
/// WHY: the receiver answers only the searches that ask for the MultiScreen
/// service. Answering every search would put the receiver into the device
/// list of every control point on the network, including the ones that have
/// nothing to do with Samsung.
///
/// A search without a `MAN` header is answered as well — some Samsung
/// senders omit it — but a search with a *different* `MAN` value is not.
///
/// A search that carries a `Content-Length` is **not** answered either: an
/// `M-SEARCH` is a request without a body, and a datagram that announces a
/// length it does not carry is malformed. A malformed datagram is refused,
/// not repaired.
#[must_use]
pub fn is_smartview_search(message: &str) -> bool {
    if !message.to_ascii_lowercase().starts_with("m-search ") {
        return false;
    }
    if !has_discover_man(message) {
        return false;
    }
    if has_header(message, "content-length") {
        return false;
    }
    requested_target(message).eq_ignore_ascii_case(SEARCH_TARGET)
}

/// The search target of an `M-SEARCH` message.
///
/// An unknown target is returned unchanged, so that the caller can decide
/// what to do with it.
#[must_use]
pub fn requested_target(message: &str) -> String {
    header_value(message, "st").unwrap_or_default()
}

/// The value of a header of an SSDP message.
///
/// WHY: the SSDP header names are case-insensitive, and a sender that
/// writes `St:` instead of `ST:` is not broken. The value is trimmed, and
/// the quotes that some senders put around the target are removed.
#[must_use]
pub fn header_value(message: &str, name: &str) -> Option<String> {
    for line in message.split("\r\n").flat_map(|part| part.split('\n')) {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        if key.trim().eq_ignore_ascii_case(name) {
            let value = value.trim().trim_matches('"').trim();
            return Some(value.to_string());
        }
    }
    None
}

/// Whether the search asks for an answer.
fn has_discover_man(message: &str) -> bool {
    match header_value(message, "man") {
        Some(value) => value.eq_ignore_ascii_case(MAN_DISCOVER),
        None => true,
    }
}

/// Whether a message carries a header.
fn has_header(message: &str, name: &str) -> bool {
    message
        .split("\r\n")
        .flat_map(|part| part.split('\n'))
        .any(|line| {
            line.split_once(':')
                .is_some_and(|(key, _)| key.trim().eq_ignore_ascii_case(name))
        })
}

/// The answer to an `M-SEARCH` query.
#[must_use]
pub fn search_response(location: &str, udn: &str, search_target: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\nCACHE-CONTROL: max-age={CACHE_CONTROL_MAX_AGE}\r\nEXT:\r\n\
         LOCATION: {location}\r\nSERVER: {SERVER}\r\nST: {search_target}\r\nUSN: {udn}::{search_target}\r\n\r\n"
    )
}

/// The announcement that the receiver has appeared.
#[must_use]
pub fn notify_alive(location: &str, udn: &str, notification_type: &str) -> String {
    format!(
        "NOTIFY * HTTP/1.1\r\nHOST: {MULTICAST_ADDRESS}:{SSDP_PORT}\r\n\
         CACHE-CONTROL: max-age={CACHE_CONTROL_MAX_AGE}\r\nLOCATION: {location}\r\n\
         NT: {notification_type}\r\nNTS: ssdp:alive\r\nSERVER: {SERVER}\r\nUSN: {udn}::{notification_type}\r\n\r\n"
    )
}

/// The announcement that the receiver has left.
#[must_use]
pub fn notify_byebye(udn: &str, notification_type: &str) -> String {
    format!(
        "NOTIFY * HTTP/1.1\r\nHOST: {MULTICAST_ADDRESS}:{SSDP_PORT}\r\nNT: {notification_type}\r\n\
         NTS: ssdp:byebye\r\nUSN: {udn}::{notification_type}\r\n\r\n"
    )
}

/// The SSDP responder of the Smart View receiver.
#[derive(Debug)]
pub struct SsdpResponder {
    socket: Arc<UdpSocket>,
    running: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
    local_port: u16,
    announcement_target: SocketAddr,
}

impl SsdpResponder {
    /// Binds the responder socket and starts the reader thread.
    ///
    /// # Errors
    /// Returns [`crate::error::SmartViewError::Ssdp`] when the socket cannot
    /// be bound or the multicast group cannot be joined.
    pub fn bind(
        port: u16,
        location: String,
        udn: String,
    ) -> Result<Self, crate::error::SmartViewError> {
        let socket = bind_multicast_socket(port)?;
        let local_port = socket
            .local_addr()
            .map(|address| address.port())
            .unwrap_or(port);
        let shared = Arc::new(socket);
        let running = Arc::new(AtomicBool::new(true));
        let reader_socket = Arc::clone(&shared);
        let reader_running = Arc::clone(&running);
        let worker = std::thread::Builder::new()
            .name("phairplay-smartview-ssdp".to_string())
            .spawn(move || {
                let mut buffer = vec![0u8; MAX_PACKET_BYTES];
                while reader_running.load(Ordering::Relaxed) {
                    match reader_socket.recv_from(&mut buffer) {
                        Ok((read, from)) => {
                            let message = String::from_utf8_lossy(&buffer[..read]).to_string();
                            if is_smartview_search(&message) {
                                let target = requested_target(&message);
                                let response = search_response(&location, &udn, &target);
                                let _ = reader_socket.send_to(response.as_bytes(), from);
                            }
                        }
                        Err(error) => match error.kind() {
                            io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut => {}
                            _ => break,
                        },
                    }
                }
            })
            .map_err(|error| crate::error::SmartViewError::Ssdp(error.to_string()))?;

        Ok(Self {
            socket: shared,
            running,
            worker: Some(worker),
            local_port,
            announcement_target: SocketAddr::from((MULTICAST_ADDRESS, local_port)),
        })
    }

    /// The port the responder listens on.
    #[must_use]
    pub fn local_port(&self) -> u16 {
        self.local_port
    }

    /// The address the announcements are sent to.
    ///
    /// WHY: the announcement belongs to the group and to the port the
    /// responder listens on. In the configuration of a receiver that is the
    /// well-known SSDP port 1900 — the port a sender searches. A receiver
    /// that is configured on another port announces on that port, so that
    /// the pair "the port I listen on" and "the port I announce on" can
    /// never disagree.
    #[must_use]
    pub fn announcement_target(&self) -> SocketAddr {
        self.announcement_target
    }

    /// Announces the receiver on the multicast group.
    ///
    /// # Errors
    /// Returns the I/O error of the send.
    pub fn send_alive(&self, location: &str, udn: &str) -> io::Result<()> {
        let message = notify_alive(location, udn, SEARCH_TARGET);
        self.socket
            .send_to(message.as_bytes(), self.announcement_target)
            .map(|_| ())
    }

    /// Announces that the receiver has left.
    ///
    /// # Errors
    /// Returns the I/O error of the send.
    pub fn send_byebye(&self, udn: &str) -> io::Result<()> {
        let message = notify_byebye(udn, SEARCH_TARGET);
        self.socket
            .send_to(message.as_bytes(), self.announcement_target)
            .map(|_| ())
    }

    /// Stops the reader thread.
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

impl Drop for SsdpResponder {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Binds the multicast socket and joins the SSDP group.
///
/// WHY: `SO_REUSEADDR` is set before the bind. Without it, a second
/// responder on the same host — the DLNA engine of the same receiver, for
/// example — cannot bind the port at all, and the receiver loses a
/// protocol. The standard library cannot set the option, and the only way
/// to set it without `unsafe` is a crate that wraps the socket API.
fn bind_multicast_socket(port: u16) -> Result<UdpSocket, crate::error::SmartViewError> {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))
        .map_err(|error| crate::error::SmartViewError::Ssdp(error.to_string()))?;
    socket
        .set_reuse_address(true)
        .map_err(|error| crate::error::SmartViewError::Ssdp(error.to_string()))?;
    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, port));
    socket
        .bind(&address.into())
        .map_err(|error| crate::error::SmartViewError::Ssdp(error.to_string()))?;
    socket
        .join_multicast_v4(&MULTICAST_ADDRESS, &Ipv4Addr::UNSPECIFIED)
        .map_err(|error| crate::error::SmartViewError::Ssdp(error.to_string()))?;
    let socket: UdpSocket = socket.into();
    socket
        .set_read_timeout(Some(READ_POLL_INTERVAL))
        .map_err(|error| crate::error::SmartViewError::Ssdp(error.to_string()))?;
    Ok(socket)
}

#[cfg(test)]
mod tests;
