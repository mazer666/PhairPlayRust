//! The SSDP side of the DLNA discovery: the `NOTIFY` announcements and the
//! answers to `M-SEARCH` queries.
//!
//! WHY: a DLNA control point finds a renderer over SSDP, not over DNS-SD.
//! The renderer therefore has to announce itself on the multicast group and
//! has to answer the queries of the control points that are already there.
//!
//! WHY the responder lives in the core: the receiver announces **two**
//! services over the same multicast group — this renderer and the Samsung
//! Smart View receiver — and Linux allows only one socket per multicast
//! group. Two responders would fight over the port, and the receiver would
//! lose a protocol because of a socket option. The core therefore owns
//! **one** listener, and every service registers itself with it. See
//! [`phairplay_core::ssdp`].
//!
//! HOW: the message codec of this module is pure and is unit tested
//! without a network.
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

use phairplay_core::ssdp as core_ssdp;

/// The IPv4 multicast group SSDP uses.
pub use core_ssdp::MULTICAST_ADDRESS;

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
pub const ST_ALL: &str = core_ssdp::ST_ALL;

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

/// The reasons the SSDP socket could not be opened.
pub use phairplay_core::ssdp::SsdpError;

/// The responder of the process.
pub use phairplay_core::ssdp::SsdpResponder;

/// Whether a message is an `M-SEARCH` that this renderer should answer.
///
/// WHY: the renderer answers only the queries that ask for a device it
/// actually is. Answering every query would put the renderer into the
/// device list of every control point on the network, including the ones
/// that have nothing to do with media rendering.
#[must_use]
pub fn is_renderer_search(message: &str) -> bool {
    if !core_ssdp::is_search(message) {
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
    let target = core_ssdp::requested_target(message);
    if target.eq_ignore_ascii_case(ST_ALL) || target.is_empty() {
        return MEDIA_RENDERER.to_string();
    }
    target
}

/// The answer to an `M-SEARCH` query.
#[must_use]
pub fn search_response(location: &str, udn: &str, search_target: &str) -> String {
    core_ssdp::search_response(location, udn, search_target, SERVER)
}

/// The announcement that the renderer has appeared.
#[must_use]
pub fn notify_alive(location: &str, udn: &str, notification_type: &str) -> String {
    core_ssdp::notify_alive(location, udn, notification_type, SERVER)
}

/// The announcement that the renderer has left.
#[must_use]
pub fn notify_byebye(udn: &str, notification_type: &str) -> String {
    core_ssdp::notify_byebye(udn, notification_type)
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

/// The description of the renderer as a service of the shared responder.
///
/// WHY: the service carries the targets it answers for and the ones it
/// announces itself under, so that the shared listener can answer for the
/// renderer without knowing anything about UPnP.
#[must_use]
pub fn service(location: &str, udn: &str) -> core_ssdp::SsdpService {
    core_ssdp::SsdpService {
        // WHY the filter: the listener asks every service whether the
        // message is a search *it* answers. The rule of UPnP — the query
        // has to ask for a device this renderer is — belongs to the
        // renderer, not to the listener.
        filter: core_ssdp::SearchFilter::new(is_renderer_search),
        primary_target: MEDIA_RENDERER.to_string(),
        search_targets: vec![ROOT_DEVICE.to_string(), MEDIA_RENDERER.to_string()],
        notification_targets: notification_targets(udn),
        location: location.to_string(),
        usn: udn.to_string(),
        server: SERVER.to_string(),
    }
}

#[cfg(test)]
mod tests;
