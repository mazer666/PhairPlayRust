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

use phairplay_core::ssdp as core_ssdp;

/// The IPv4 multicast group SSDP uses.
pub use core_ssdp::MULTICAST_ADDRESS;

/// The responder of the process.
///
/// WHY it lives in the core: the receiver announces more than one service
/// over the same multicast group, and Linux allows one socket per group.
/// Two responders would fight over the port and the receiver would lose a
/// protocol because of a socket option. The core owns one listener, and
/// every service registers itself with it.
pub use core_ssdp::SsdpResponder;

/// The reasons the SSDP socket could not be opened.
pub use core_ssdp::SsdpError;

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

/// The value of the `MAN` header of a search that asks for an answer.
const MAN_DISCOVER: &str = "ssdp:discover";

/// Whether a message is an `M-SEARCH` that this receiver should answer.
///
/// WHY: the receiver answers only the searches that ask for the MultiScreen
/// service and for the wildcard. Answering every search would put the
/// receiver into the device list of every control point on the network,
/// including the ones that have nothing to do with Samsung.
///
/// WHY the wildcard counts: the UPnP device architecture says that a query
/// for `ssdp:all` is answered once per service type of the device. The
/// framework of Samsung is a service type of this receiver, so a control
/// point that asks for everything has to see it. Without that, the
/// framework is found only by a sender that knows its name — and a sender
/// that does not know it never finds the receiver.
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
    let target = requested_target(message);
    target.eq_ignore_ascii_case(SEARCH_TARGET) || target.eq_ignore_ascii_case(core_ssdp::ST_ALL)
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

/// The description of the receiver as a service of the shared responder.
///
/// WHY the service carries the rule: the listener of the core asks every
/// service whether a message is a search *it* answers. The rule of Samsung —
/// the `MAN` header may be missing, a `Content-Length` means the datagram
/// is malformed — belongs to the receiver, not to the listener.
#[must_use]
pub fn service(location: &str, udn: &str) -> core_ssdp::SsdpService {
    core_ssdp::SsdpService {
        filter: core_ssdp::SearchFilter::new(is_smartview_search),
        primary_target: SEARCH_TARGET.to_string(),
        search_targets: vec![SEARCH_TARGET.to_string()],
        notification_targets: vec![SEARCH_TARGET.to_string()],
        location: location.to_string(),
        usn: udn.to_string(),
        server: SERVER.to_string(),
    }
}

#[cfg(test)]
mod tests;
