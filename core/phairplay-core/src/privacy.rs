//! The masking of the personal data in the log.
//!
//! WHY: the log of a receiver that runs on a home network is a file that
//! outlives the session. The name of the sender — the name of a phone, of a
//! laptop, often the name of its owner — and the address of the device are
//! personal data. The security policy of the project demands that they are
//! masked in every log line, so that a log that is copied into a bug report
//! does not tell the reader whose phone was streaming.
//!
//! HOW: the masking is a *pseudonymisation*, not an anonymisation.
//!
//! * A **name** becomes `sender#<digest>`: the first four bytes of the
//!   SHA-256 digest of the name, in hexadecimal. Two log lines about the
//!   same device carry the same pseudonym, so a session can still be
//!   followed through the log, but the name itself is not readable.
//! * An **address** keeps its network prefix and loses its host part:
//!   `192.168.1.42` becomes `192.168.1.x`, `2001:db8:1:2::3` becomes
//!   `2001:db8:1:2::x`. The prefix answers the question a log has to
//!   answer — "was this the neighbour or the guest network?" — without
//!   naming the device.
//!
//! The honest limits of the promise:
//!
//! * A name that is drawn from a small dictionary can be recovered by
//!   brute force from its pseudonym. The pseudonym is therefore a *stable
//!   correlation handle*, not a secret.
//! * The masking protects the log. It does not protect the events of a
//!   session: the dashboard shows the name of the sender on purpose,
//!   because that is what the user of the receiver wants to see.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_core::privacy::{mask_ip, mask_name, mask_socket_addr};
//! use std::net::{IpAddr, SocketAddr};
//!
//! assert_eq!(mask_name("Anna's iPhone"), mask_name("Anna's iPhone"));
//! assert!(!mask_name("Anna's iPhone").contains("Anna"));
//!
//! let address: IpAddr = "192.168.1.42".parse().expect("an address");
//! assert_eq!(mask_ip(&address), "192.168.1.x");
//!
//! let peer: SocketAddr = "192.168.1.42:7000".parse().expect("an address");
//! assert_eq!(mask_socket_addr(&peer), "192.168.1.x:7000");
//! ```

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use sha2::{Digest, Sha256};

/// The number of hexadecimal characters of the pseudonym of a name.
const PSEUDONYM_LENGTH: usize = 8;

/// Masks the name of a device, of a sender or of a control point.
///
/// WHY: the name is the piece of data that identifies a *person* — an
///      "Anna's iPhone" is in the log of every other receiver of the world.
///      The pseudonym keeps the log readable ("the same device appears
///      again") without keeping the name.
///
/// # Example
///
/// ```
/// use phairplay_core::privacy::mask_name;
///
/// let masked = mask_name("Living Room");
/// assert!(masked.starts_with("device#"));
/// assert!(!masked.contains("Living"));
/// ```
#[must_use]
pub fn mask_name(name: &str) -> String {
    if name.is_empty() {
        return String::new();
    }
    let digest = Sha256::digest(name.as_bytes());
    let mut pseudonym = String::with_capacity("device#".len() + PSEUDONYM_LENGTH);
    pseudonym.push_str("device#");
    for byte in digest.iter().take(PSEUDONYM_LENGTH / 2) {
        pseudonym.push_str(&format!("{byte:02x}"));
    }
    pseudonym
}

/// Masks an IP address.
///
/// WHY: the host part of an address identifies the device; the network
///      prefix identifies the network. A log needs the second one far more
///      often than the first one.
///
/// # Example
///
/// ```
/// use phairplay_core::privacy::mask_ip;
/// use std::net::IpAddr;
///
/// let address: IpAddr = "10.1.2.3".parse().expect("an address");
/// assert_eq!(mask_ip(&address), "10.1.2.x");
/// ```
#[must_use]
pub fn mask_ip(address: &IpAddr) -> String {
    match address {
        IpAddr::V4(address) => mask_ipv4(address),
        IpAddr::V6(address) => mask_ipv6(address),
    }
}

/// Masks the host part of an IPv4 address.
fn mask_ipv4(address: &Ipv4Addr) -> String {
    let octets = address.octets();
    format!("{}.{}.{}.x", octets[0], octets[1], octets[2])
}

/// Masks the host part of an IPv6 address.
///
/// WHY: the prefix of a link-local address (`fe80::/10`) or of a unique
///      local address (`fc00::/7`) identifies the network, the rest
///      identifies the interface. Only the first four groups are kept.
fn mask_ipv6(address: &Ipv6Addr) -> String {
    // An IPv4 address that is mapped into IPv6 is an IPv4 address.
    if let Some(mapped) = address.to_ipv4() {
        return mask_ipv4(&mapped);
    }
    let groups = address.segments();
    format!(
        "{:x}:{:x}:{:x}:{:x}::x",
        groups[0], groups[1], groups[2], groups[3]
    )
}

/// Masks a socket address.
///
/// WHY: the port is not personal data and it is the part that tells a
///      reader which service of the receiver was involved. It is kept.
///
/// # Example
///
/// ```
/// use phairplay_core::privacy::mask_socket_addr;
/// use std::net::SocketAddr;
///
/// let peer: SocketAddr = "192.168.1.42:7000".parse().expect("an address");
/// assert_eq!(mask_socket_addr(&peer), "192.168.1.x:7000");
/// ```
#[must_use]
pub fn mask_socket_addr(address: &SocketAddr) -> String {
    format!("{}:{}", mask_ip(&address.ip()), address.port())
}

/// Masks the host of a URL.
///
/// WHY: a URL that the receiver is about to call carries the address of the
///      peer. It is masked like every other address; a name is masked like
///      every other name.
///
/// # Example
///
/// ```
/// use phairplay_core::privacy::mask_host;
///
/// assert_eq!(mask_host("192.168.1.42"), "192.168.1.x");
/// assert!(mask_host("nas.local").starts_with("device#"));
/// ```
#[must_use]
pub fn mask_host(host: &str) -> String {
    let trimmed = host.trim();
    if let Ok(address) = trimmed.parse::<IpAddr>() {
        return mask_ip(&address);
    }
    mask_name(trimmed)
}

/// Masks an optional name.
///
/// WHY: a missing name is normal — not every sender announces one — and the
///      log must not turn that into an empty field that looks like a bug.
#[must_use]
pub fn mask_optional_name(name: Option<&str>) -> String {
    match name {
        Some(name) => mask_name(name),
        None => "unknown".to_string(),
    }
}

/// Masks the host of a URL and drops its path.
///
/// WHY: a URL that the receiver is about to call carries the address of the
///      peer *and* often a token in its path. The log needs to say which
///      peer it was, and nothing else.
///
/// # Example
///
/// ```
/// use phairplay_core::privacy::mask_url;
///
/// let masked = mask_url("http://192.168.1.42:8080/events/secret-token");
/// assert!(masked.contains("192.168.1.x"), "{masked}");
/// assert!(!masked.contains("secret-token"), "{masked}");
/// ```
#[must_use]
pub fn mask_url(url: &str) -> String {
    let trimmed = url.trim();
    let Some((scheme, rest)) = trimmed.split_once("://") else {
        // Not a URL: it is a name or an opaque value.
        return mask_name(trimmed);
    };
    let authority_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..authority_end];
    // The port is not personal data and it names the service.
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) if port.chars().all(|c| c.is_ascii_digit()) => (host, Some(port)),
        _ => (authority, None),
    };
    match port {
        Some(port) => format!("{scheme}://{}:{port}/…", mask_host(host)),
        None => format!("{scheme}://{}/…", mask_host(host)),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    #[test]
    fn a_name_is_not_readable_in_its_pseudonym() {
        let masked = mask_name("Anna's iPhone");
        assert!(masked.starts_with("device#"), "{masked}");
        assert!(!masked.contains("Anna"), "{masked}");
        assert!(!masked.contains("iPhone"), "{masked}");
    }

    #[test]
    fn the_same_name_always_gets_the_same_pseudonym() {
        assert_eq!(mask_name("Anna's iPhone"), mask_name("Anna's iPhone"));
    }

    #[test]
    fn two_different_names_get_two_different_pseudonyms() {
        assert_ne!(mask_name("Anna's iPhone"), mask_name("Bob's Laptop"));
    }

    #[test]
    fn an_empty_name_stays_empty() {
        assert_eq!(mask_name(""), "");
    }

    #[test]
    fn the_host_part_of_an_ipv4_address_is_masked() {
        let address: IpAddr = "192.168.1.42".parse().expect("an address");
        let masked = mask_ip(&address);
        assert_eq!(masked, "192.168.1.x");
        assert!(!masked.contains("42"), "{masked}");
    }

    #[test]
    fn the_host_part_of_an_ipv6_address_is_masked() {
        let address: IpAddr = "2001:db8:85a3:1234:5678:9abc:def0:1"
            .parse()
            .expect("an address");
        let masked = mask_ip(&address);
        assert!(masked.starts_with("2001:db8:85a3:1234"), "{masked}");
        assert!(!masked.contains("def0"), "{masked}");
    }

    #[test]
    fn the_loopback_address_is_masked_as_well() {
        let address: IpAddr = "127.0.0.1".parse().expect("an address");
        assert_eq!(mask_ip(&address), "127.0.0.x");
    }

    #[test]
    fn the_metadata_address_is_masked_as_well() {
        let address: IpAddr = "169.254.169.254".parse().expect("an address");
        let masked = mask_ip(&address);
        assert_eq!(masked, "169.254.169.x");
        assert!(!masked.ends_with(".254"), "{masked}");
    }

    #[test]
    fn the_port_of_a_socket_address_is_kept() {
        let peer: SocketAddr = "192.168.1.42:7000".parse().expect("an address");
        let masked = mask_socket_addr(&peer);
        assert_eq!(masked, "192.168.1.x:7000");
        assert!(!masked.contains("42"), "{masked}");
    }

    #[test]
    fn a_host_that_is_an_address_is_masked_like_an_address() {
        assert_eq!(mask_host("192.168.1.42"), "192.168.1.x");
    }

    #[test]
    fn a_host_that_is_a_name_is_masked_like_a_name() {
        let masked = mask_host("nas.local");
        assert!(masked.starts_with("device#"), "{masked}");
        assert!(!masked.contains("nas"), "{masked}");
    }

    #[test]
    fn a_missing_name_becomes_unknown() {
        assert_eq!(mask_optional_name(None), "unknown");
        assert!(mask_optional_name(Some("Anna")).starts_with("device#"));
    }

    #[test]
    fn the_host_of_a_url_is_masked_and_the_path_is_dropped() {
        let masked = mask_url("http://192.168.1.42:8080/events/secret-token");
        assert_eq!(masked, "http://192.168.1.x:8080/…");
        assert!(!masked.contains("secret-token"), "{masked}");
    }

    #[test]
    fn a_url_without_a_port_is_masked_too() {
        let masked = mask_url("http://192.168.1.42/events");
        assert_eq!(masked, "http://192.168.1.x/…");
    }

    #[test]
    fn a_url_with_a_name_is_masked_like_a_name() {
        let masked = mask_url("http://control-point.local:8080/events");
        assert!(masked.starts_with("http://device#"), "{masked}");
        assert!(!masked.contains("control-point"), "{masked}");
    }

    #[test]
    fn a_value_that_is_not_a_url_is_masked_like_a_name() {
        let masked = mask_url("some-opaque-token");
        assert!(masked.starts_with("device#"), "{masked}");
    }
}
