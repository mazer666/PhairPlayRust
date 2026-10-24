//! The SSDP responder of the process.
//!
//! WHAT: one listener on the SSDP multicast group, and a registry of the
//! services that it answers for.
//!
//! WHY: the receiver speaks two protocols that are discovered over SSDP —
//! DLNA and Samsung Smart View — and **Linux lets only one socket join a
//! multicast group**. Two responders, one group, one port: the second bind
//! fails with `Address already in use`, and the receiver loses a protocol
//! because of a socket option. The way out is the way every real UPnP stack
//! takes it: **one listener, many services**. The services register
//! themselves, and the listener answers for each of them only for the
//! search targets it announced.
//!
//! HOW: the responder of the process is created once, on the first call of
//! [`shared`]. It binds the multicast socket with `SO_REUSEADDR` — the
//! standard library cannot set the option, and the only way to set it
//! without `unsafe` is a crate that wraps the socket API — and starts one
//! reader thread. The thread reads the datagrams of the group and answers
//! the `M-SEARCH` queries of the services that are registered.
//!
//! EXAMPLE:
//!
//! ```no_run
//! use phairplay_core::ssdp::{SsdpResponder, SsdpService};
//!
//! # fn main() -> Result<(), phairplay_core::ssdp::SsdpError> {
//! let responder = SsdpResponder::shared()?;
//! let service = SsdpService {
//!     filter: phairplay_core::ssdp::SearchFilter::new(|message| {
//!         message.starts_with("M-SEARCH ")
//!     }),
//!     primary_target: "urn:samsung.com:service:MultiScreenService:1".to_string(),
//!     search_targets: vec!["urn:samsung.com:service:MultiScreenService:1".to_string()],
//!     notification_targets: vec!["urn:samsung.com:service:MultiScreenService:1".to_string()],
//!     location: "http://192.168.1.10:8001/description.xml".to_string(),
//!     usn: "uuid:phairplay".to_string(),
//!     server: "PhairPlayRust/1.0".to_string(),
//! };
//! responder.add_service(service.clone());
//! responder.announce(&service)?;
//! # Ok(())
//! # }
//! ```

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::JoinHandle;
use std::time::Duration;

use socket2::{Domain, Protocol, Socket, Type};

use crate::config::ports::SSDP;

/// The multicast group of SSDP.
pub const MULTICAST_ADDRESS: Ipv4Addr = Ipv4Addr::new(239, 255, 255, 250);

/// The search target that asks for every device.
pub const ST_ALL: &str = "ssdp:all";

/// The number of seconds a control point may cache an answer.
pub const CACHE_CONTROL_MAX_AGE: u32 = 1800;

/// How often the reader thread checks whether it is still wanted.
const READ_POLL_INTERVAL: Duration = Duration::from_millis(250);

/// The largest datagram the reader accepts.
const MAX_DATAGRAM_BYTES: usize = 8 * 1024;

/// The failures of the SSDP responder.
#[derive(Debug, thiserror::Error)]
pub enum SsdpError {
    /// The socket could not be bound.
    #[error("the SSDP socket could not be bound: {0}")]
    Bind(String),

    /// The multicast group could not be joined.
    #[error("the SSDP multicast group could not be joined: {0}")]
    Multicast(String),

    /// A datagram could not be sent.
    #[error("the SSDP datagram could not be sent: {0}")]
    Send(String),
}

/// The rule that decides whether a message is a search a service answers.
///
/// WHY: the protocols differ in what they accept. A UPnP control point
/// always sends the `MAN` header; some Samsung senders omit it, and a
/// datagram that announces a length it does not carry is malformed. The
/// rule belongs to the service, not to the listener — the listener only
/// knows that it has to ask.
#[derive(Clone)]
pub struct SearchFilter(Arc<dyn Fn(&str) -> bool + Send + Sync>);

impl SearchFilter {
    /// A filter from a rule.
    #[must_use]
    pub fn new(rule: impl Fn(&str) -> bool + Send + Sync + 'static) -> Self {
        Self(Arc::new(rule))
    }

    /// Whether the message is a search this service answers.
    #[must_use]
    pub fn accepts(&self, message: &str) -> bool {
        (self.0)(message)
    }
}

impl std::fmt::Debug for SearchFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SearchFilter(<rule>)")
    }
}

impl PartialEq for SearchFilter {
    fn eq(&self, _other: &Self) -> bool {
        // WHY: two filters are never compared. The registry keys a service by
        // its unique name, and the rule itself is a closure that cannot be
        // compared.
        true
    }
}

impl Eq for SearchFilter {}

/// One service that the receiver announces over SSDP.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SsdpService {
    /// The rule that decides whether a message is a search of this service.
    pub filter: SearchFilter,
    /// The type the service is.
    ///
    /// WHY: a query for `ssdp:all` is answered with the type of the
    /// service, not with the wildcard — the answer has to tell the control
    /// point *what* it found.
    pub primary_target: String,
    /// The search targets the service answers for.
    ///
    /// WHY: a device that answers for a target it does not have appears in
    /// the device list of every control point on the network.
    pub search_targets: Vec<String>,
    /// The notification targets the service announces itself under.
    pub notification_targets: Vec<String>,
    /// The address of the description document of the service.
    pub location: String,
    /// The unique name of the service.
    pub usn: String,
    /// The server string of the answer.
    pub server: String,
}

impl SsdpService {
    /// The type the service answers a search target with.
    ///
    /// WHY: `None` means that the service does not answer for this target
    /// at all. A device that answers for a type it does not have appears in
    /// the device list of every control point on the network.
    #[must_use]
    pub fn answer_target(&self, search_target: &str) -> Option<String> {
        if search_target.eq_ignore_ascii_case(ST_ALL) {
            return Some(self.primary_target.clone());
        }
        self.search_targets
            .iter()
            .find(|target| target.eq_ignore_ascii_case(search_target))
            .cloned()
    }

    /// Whether the service answers for a search target.
    #[must_use]
    pub fn answers(&self, search_target: &str) -> bool {
        self.answer_target(search_target).is_some()
    }
}

/// The SSDP responder of the process.
///
/// WHY the responder is shared: the receiver announces two services over the
/// same multicast group, and Linux allows one socket per group. The services
/// register themselves; the listener answers for each of them separately.
#[derive(Debug)]
pub struct SsdpResponder {
    socket: Arc<UdpSocket>,
    services: Arc<Mutex<HashMap<String, SsdpService>>>,
    running: Arc<AtomicBool>,
    worker: Mutex<Option<JoinHandle<()>>>,
    local_port: u16,
}

impl SsdpResponder {
    /// Binds the responder socket and starts the reader thread.
    ///
    /// # Errors
    /// Returns [`SsdpError::Bind`] when the socket cannot be bound and
    /// [`SsdpError::Multicast`] when the group cannot be joined.
    pub fn bind(port: u16) -> Result<Self, SsdpError> {
        let socket = bind_multicast_socket(port)?;
        let local_port = socket
            .local_addr()
            .map(|address| address.port())
            .unwrap_or(port);
        let socket = Arc::new(socket);
        let services: Arc<Mutex<HashMap<String, SsdpService>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let running = Arc::new(AtomicBool::new(true));

        let reader_socket = Arc::clone(&socket);
        let reader_services = Arc::clone(&services);
        let reader_running = Arc::clone(&running);
        let worker = std::thread::Builder::new()
            .name("phairplay-ssdp".to_string())
            .spawn(move || {
                let mut buffer = [0_u8; MAX_DATAGRAM_BYTES];
                while reader_running.load(Ordering::SeqCst) {
                    match reader_socket.recv_from(&mut buffer) {
                        Ok((count, from)) => {
                            let message = String::from_utf8_lossy(&buffer[..count]);
                            answer_search(&reader_services, &reader_socket, &message, from);
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
            socket,
            services,
            running,
            worker: Mutex::new(Some(worker)),
            local_port,
        })
    }

    /// The responder of this process.
    ///
    /// WHY: there is one receiver per process, and therefore one SSDP
    /// listener. The services of the protocols register themselves here.
    ///
    /// # Errors
    /// Returns [`SsdpError::Bind`] when the socket cannot be bound and
    /// [`SsdpError::Multicast`] when the group cannot be joined.
    pub fn shared() -> Result<Arc<Self>, SsdpError> {
        static RESPONDER: OnceLock<Mutex<Option<Arc<SsdpResponder>>>> = OnceLock::new();
        let cell = RESPONDER.get_or_init(|| Mutex::new(None));
        let mut guard = cell
            .lock()
            .map_err(|_| SsdpError::Bind("the responder lock is poisoned".to_string()))?;
        if let Some(existing) = guard.as_ref() {
            return Ok(Arc::clone(existing));
        }
        let responder = Arc::new(Self::bind(SSDP)?);
        *guard = Some(Arc::clone(&responder));
        Ok(responder)
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
        SocketAddr::from((MULTICAST_ADDRESS, self.local_port))
    }

    /// Adds a service to the registry.
    ///
    /// WHY: the service is keyed by its unique name, so that a service
    /// that is started twice replaces itself and does not answer twice.
    pub fn add_service(&self, service: SsdpService) {
        if let Ok(mut services) = self.services.lock() {
            services.insert(service.usn.clone(), service);
        }
    }

    /// Removes a service from the registry.
    pub fn remove_service(&self, usn: &str) {
        if let Ok(mut services) = self.services.lock() {
            services.remove(usn);
        }
    }

    /// The services that are currently registered.
    #[must_use]
    pub fn services(&self) -> Vec<SsdpService> {
        self.services
            .lock()
            .map(|services| services.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Announces a service as alive.
    ///
    /// # Errors
    /// Returns [`SsdpError::Send`] when the datagram cannot be sent.
    pub fn announce(&self, service: &SsdpService) -> Result<(), SsdpError> {
        for target in &service.notification_targets {
            let message = notify_alive(&service.location, &service.usn, target, &service.server);
            self.send_to_group(&message)?;
        }
        Ok(())
    }

    /// Announces that a service has left.
    ///
    /// # Errors
    /// Returns [`SsdpError::Send`] when the datagram cannot be sent.
    pub fn withdraw(&self, service: &SsdpService) -> Result<(), SsdpError> {
        for target in &service.notification_targets {
            let message = notify_byebye(&service.usn, target);
            self.send_to_group(&message)?;
        }
        Ok(())
    }

    /// Sends a datagram to the multicast group.
    fn send_to_group(&self, message: &str) -> Result<(), SsdpError> {
        self.socket
            .send_to(message.as_bytes(), self.announcement_target())
            .map(|_| ())
            .map_err(|error| SsdpError::Send(error.to_string()))
    }

    /// Stops the reader thread.
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        if let Ok(mut worker) = self.worker.lock() {
            if let Some(handle) = worker.take() {
                let _ = handle.join();
            }
        }
    }
}

impl Drop for SsdpResponder {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Answers a search query for every registered service that it concerns.
fn answer_search(
    services: &Arc<Mutex<HashMap<String, SsdpService>>>,
    socket: &UdpSocket,
    message: &str,
    from: SocketAddr,
) {
    if !is_search(message) {
        return;
    }
    let target = requested_target(message);
    let candidates = match services.lock() {
        Ok(services) => services.values().cloned().collect::<Vec<_>>(),
        Err(_) => return,
    };
    for service in candidates {
        // WHY the filter comes first: the rule of the protocol decides
        // whether the message is a search at all — a Samsung sender may
        // omit the `MAN` header, a UPnP control point never does.
        if !service.filter.accepts(message) {
            continue;
        }
        let Some(answer_target) = service.answer_target(&target) else {
            continue;
        };
        let answer = search_response(
            &service.location,
            &service.usn,
            &answer_target,
            &service.server,
        );
        let _ = socket.send_to(answer.as_bytes(), from);
    }
}

/// Binds the multicast socket and joins the SSDP group.
///
/// WHY: `SO_REUSEADDR` is set before the bind. Without it the socket holds
/// the port exclusively, and a second responder on the same host cannot bind
/// it at all. The standard library cannot set the option, and the only way to
/// set it without `unsafe` is a crate that wraps the socket API.
fn bind_multicast_socket(port: u16) -> Result<UdpSocket, SsdpError> {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))
        .map_err(|error| SsdpError::Bind(error.to_string()))?;
    socket
        .set_reuse_address(true)
        .map_err(|error| SsdpError::Bind(error.to_string()))?;
    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, port));
    socket
        .bind(&address.into())
        .map_err(|error| SsdpError::Bind(error.to_string()))?;
    let socket: UdpSocket = socket.into();
    socket
        .join_multicast_v4(&MULTICAST_ADDRESS, &Ipv4Addr::UNSPECIFIED)
        .map_err(|error| SsdpError::Multicast(error.to_string()))?;
    socket
        .set_read_timeout(Some(READ_POLL_INTERVAL))
        .map_err(|error| SsdpError::Bind(error.to_string()))?;
    Ok(socket)
}

/// Whether a message is an `M-SEARCH` query.
#[must_use]
pub fn is_search(message: &str) -> bool {
    let normalized = message.to_ascii_lowercase();
    normalized.starts_with("m-search ") && normalized.contains("man: \"ssdp:discover\"")
}

/// The search target of an `M-SEARCH` message.
///
/// WHY the empty case: a query without a target asks for every device, and a
/// service that has a target of its own answers for it.
#[must_use]
pub fn requested_target(message: &str) -> String {
    for line in message.split("\r\n").flat_map(|part| part.split('\n')) {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        if key.trim().eq_ignore_ascii_case("st") {
            let target = value.trim();
            if target.is_empty() {
                return ST_ALL.to_string();
            }
            return target.to_string();
        }
    }
    ST_ALL.to_string()
}

/// The answer to an `M-SEARCH` query.
#[must_use]
pub fn search_response(location: &str, usn: &str, search_target: &str, server: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\nCACHE-CONTROL: max-age={CACHE_CONTROL_MAX_AGE}\r\nEXT:\r\n\
         LOCATION: {location}\r\nSERVER: {server}\r\nST: {search_target}\r\nUSN: {usn}::{search_target}\r\n\r\n"
    )
}

/// The announcement that a service has appeared.
#[must_use]
pub fn notify_alive(location: &str, usn: &str, notification_type: &str, server: &str) -> String {
    format!(
        "NOTIFY * HTTP/1.1\r\nHOST: {MULTICAST_ADDRESS}:{SSDP}\r\n\
         CACHE-CONTROL: max-age={CACHE_CONTROL_MAX_AGE}\r\nLOCATION: {location}\r\n\
         NT: {notification_type}\r\nNTS: ssdp:alive\r\nSERVER: {server}\r\nUSN: {usn}::{notification_type}\r\n\r\n"
    )
}

/// The announcement that a service has left.
#[must_use]
pub fn notify_byebye(usn: &str, notification_type: &str) -> String {
    format!(
        "NOTIFY * HTTP/1.1\r\nHOST: {MULTICAST_ADDRESS}:{SSDP}\r\nNT: {notification_type}\r\n\
         NTS: ssdp:byebye\r\nUSN: {usn}::{notification_type}\r\n\r\n"
    )
}

/// The address of the group, for a caller that wants to name it.
#[must_use]
pub fn group_address() -> IpAddr {
    IpAddr::V4(MULTICAST_ADDRESS)
}

#[cfg(test)]
mod tests;
