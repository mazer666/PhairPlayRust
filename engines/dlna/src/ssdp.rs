//! The SSDP side of the DLNA discovery: the `NOTIFY` announcements and the
//! answers to `M-SEARCH` queries.
//!
//! WHY: a DLNA control point finds a renderer over SSDP, not over DNS-SD.
//! The renderer therefore has to announce itself on the multicast group and
//! has to answer the queries of the control points that are already there.
//!
//! HOW: the socket lives on its own thread because the multicast socket has
//! to be joined with the operating system's multicast API, which the
//! asynchronous runtime does not expose. Every read has a deadline, so that
//! the thread notices a `stop()` within half a second.
//!
//! The message codec is pure and is unit tested without a network.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_dlna::ssdp::{notify_alive, search_response};
//!
//! let alive = notify_alive("http://10.0.0.5:8200/description.xml", "uuid:abc", "upnp:rootdevice");
//! assert!(alive.starts_with("NOTIFY * HTTP/1.1"));
//! assert!(alive.contains("NTS: ssdp:alive"));
//!
//! let response = search_response("http://10.0.0.5:8200/description.xml", "uuid:abc", "ssdp:all");
//! assert!(response.starts_with("HTTP/1.1 200 OK"));
//! ```

use std::io;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

/// The IPv4 multicast group SSDP uses.
pub const MULTICAST_ADDRESS: Ipv4Addr = Ipv4Addr::new(239, 255, 255, 250);

/// The port SSDP uses.
///
/// WHY: the value is fixed by the UPnP specification. Its single owner is
/// [`phairplay_core::config::ports::SSDP`].
pub const SSDP_PORT: u16 = phairplay_core::config::ports::SSDP;

/// The search target of the root device.
pub const ROOT_DEVICE: &str = "upnp:rootdevice";

/// The search target of a media renderer.
pub const MEDIA_RENDERER: &str = "urn:schemas-upnp-org:device:MediaRenderer:1";

/// The search target that matches every device.
pub const ST_ALL: &str = "ssdp:all";

/// The server string the renderer announces.
pub const SERVER: &str = "Linux/1.0 UPnP/1.0 PhairPlayRust/1.0";

/// The lifetime of an announcement, in seconds.
///
/// WHY: the UPnP specification gives the advertisement lifetime and the
/// duration of a GENA subscription the same value, so both are one constant:
/// [`phairplay_core::config::timeouts::UPNP_MAX_AGE_SECONDS`].
pub const CACHE_CONTROL_MAX_AGE: u32 = phairplay_core::config::timeouts::UPNP_MAX_AGE_SECONDS;

/// The maximum size of an SSDP packet the renderer reads.
///
/// WHY: the same cap applies to the SSDP browser of the discovery crate.
/// Its single owner is
/// [`phairplay_core::config::limits::SSDP_MESSAGE_BYTES`].
pub const MAX_PACKET_BYTES: usize = phairplay_core::config::limits::SSDP_MESSAGE_BYTES;

/// The interval the reader thread wakes up in.
const READ_POLL_INTERVAL: Duration = Duration::from_millis(500);

/// The reasons the SSDP socket could not be opened.
#[derive(Debug, thiserror::Error)]
pub enum SsdpError {
    /// The socket could not be bound.
    #[error("the SSDP socket could not be bound: {0}")]
    Bind(String),

    /// The multicast group could not be joined.
    #[error("the SSDP multicast group could not be joined: {0}")]
    Multicast(String),
}

/// Whether a message is an `M-SEARCH` that this renderer should answer.
///
/// WHY: the renderer answers only the queries that ask for a device it
/// actually is. Answering every query would put the renderer into the
/// device list of every control point on the network, including the ones
/// that have nothing to do with media rendering.
#[must_use]
pub fn is_renderer_search(message: &str) -> bool {
    let normalized = message.to_ascii_lowercase();
    if !normalized.starts_with("m-search ") {
        return false;
    }
    if !normalized.contains("man: \"ssdp:discover\"") {
        return false;
    }
    matches!(
        requested_target(message).as_str(),
        ST_ALL | ROOT_DEVICE | MEDIA_RENDERER
    )
}

/// The search target of an `M-SEARCH` message.
///
/// An unknown target falls back to the media renderer type, because that is
/// the only device this renderer has.
#[must_use]
pub fn requested_target(message: &str) -> String {
    for line in message.split("\r\n").flat_map(|part| part.split('\n')) {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        if key.trim().eq_ignore_ascii_case("st") {
            let target = value.trim();
            if target.eq_ignore_ascii_case(ST_ALL) {
                return MEDIA_RENDERER.to_string();
            }
            if target.is_empty() {
                return MEDIA_RENDERER.to_string();
            }
            return target.to_string();
        }
    }
    MEDIA_RENDERER.to_string()
}

/// The answer to an `M-SEARCH` query.
#[must_use]
pub fn search_response(location: &str, udn: &str, search_target: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\nCACHE-CONTROL: max-age={CACHE_CONTROL_MAX_AGE}\r\nEXT:\r\n\
         LOCATION: {location}\r\nSERVER: {SERVER}\r\nST: {search_target}\r\nUSN: {udn}::{search_target}\r\n\r\n"
    )
}

/// The announcement that the renderer has appeared.
#[must_use]
pub fn notify_alive(location: &str, udn: &str, notification_type: &str) -> String {
    format!(
        "NOTIFY * HTTP/1.1\r\nHOST: {MULTICAST_ADDRESS}:{SSDP_PORT}\r\n\
         CACHE-CONTROL: max-age={CACHE_CONTROL_MAX_AGE}\r\nLOCATION: {location}\r\n\
         NT: {notification_type}\r\nNTS: ssdp:alive\r\nSERVER: {SERVER}\r\nUSN: {udn}::{notification_type}\r\n\r\n"
    )
}

/// The announcement that the renderer has left.
#[must_use]
pub fn notify_byebye(udn: &str, notification_type: &str) -> String {
    format!(
        "NOTIFY * HTTP/1.1\r\nHOST: {MULTICAST_ADDRESS}:{SSDP_PORT}\r\nNT: {notification_type}\r\n\
         NTS: ssdp:byebye\r\nUSN: {udn}::{notification_type}\r\n\r\n"
    )
}

/// The notification targets the renderer announces itself under.
#[must_use]
pub fn notification_targets(udn: &str) -> Vec<String> {
    vec![
        ROOT_DEVICE.to_string(),
        MEDIA_RENDERER.to_string(),
        udn.to_string(),
    ]
}

/// The SSDP responder of the renderer.
#[derive(Debug)]
pub struct SsdpResponder {
    socket: Arc<UdpSocket>,
    running: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
    local_port: u16,
}

impl SsdpResponder {
    /// Binds the responder socket and starts the reader thread.
    ///
    /// # Errors
    /// Returns [`SsdpError::Bind`] when the socket cannot be bound and
    /// [`SsdpError::Multicast`] when the group cannot be joined.
    pub fn bind(port: u16, location: String, udn: String) -> Result<Self, SsdpError> {
        let socket = UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, port)))
            .map_err(|error| SsdpError::Bind(error.to_string()))?;
        socket
            .set_read_timeout(Some(READ_POLL_INTERVAL))
            .map_err(|error| SsdpError::Bind(error.to_string()))?;
        socket
            .join_multicast_v4(&MULTICAST_ADDRESS, &Ipv4Addr::UNSPECIFIED)
            .map_err(|error| SsdpError::Multicast(error.to_string()))?;
        let local_port = socket
            .local_addr()
            .map(|address| address.port())
            .unwrap_or(port);
        let shared = Arc::new(socket);
        let running = Arc::new(AtomicBool::new(true));
        let reader_socket = Arc::clone(&shared);
        let reader_running = Arc::clone(&running);
        let worker = std::thread::Builder::new()
            .name("phairplay-dlna-ssdp".to_string())
            .spawn(move || {
                let mut buffer = vec![0u8; MAX_PACKET_BYTES];
                while reader_running.load(Ordering::Relaxed) {
                    match reader_socket.recv_from(&mut buffer) {
                        Ok((read, from)) => {
                            let message = String::from_utf8_lossy(&buffer[..read]).to_string();
                            if is_renderer_search(&message) {
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
            .map_err(|error| SsdpError::Bind(error.to_string()))?;

        Ok(Self {
            socket: shared,
            running,
            worker: Some(worker),
            local_port,
        })
    }

    /// The port the responder listens on.
    #[must_use]
    pub fn local_port(&self) -> u16 {
        self.local_port
    }

    /// Announces the renderer on the multicast group.
    ///
    /// # Errors
    /// Returns the I/O error of the send.
    pub fn send_alive(&self, location: &str, udn: &str) -> io::Result<()> {
        for target in notification_targets(udn) {
            let message = notify_alive(location, udn, &target);
            self.socket.send_to(
                message.as_bytes(),
                SocketAddr::from((MULTICAST_ADDRESS, SSDP_PORT)),
            )?;
        }
        Ok(())
    }

    /// Announces that the renderer has left.
    ///
    /// # Errors
    /// Returns the I/O error of the send.
    pub fn send_byebye(&self, udn: &str) -> io::Result<()> {
        for target in notification_targets(udn) {
            let message = notify_byebye(udn, &target);
            self.socket.send_to(
                message.as_bytes(),
                SocketAddr::from((MULTICAST_ADDRESS, SSDP_PORT)),
            )?;
        }
        Ok(())
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

#[cfg(test)]
mod tests;
