//! The URLs of the GENA callbacks.
//!
//! WHY: the callback URL is the one piece of the protocol that makes the
//! renderer *call out*. It is therefore parsed by hand, strictly, and
//! without ever resolving a name — a name would allow a DNS rebinding and
//! a resolver side channel.
//!
//! HOW: only the `http` scheme with a literal IPv4 host is accepted. The
//! caller decides whether the address is private and whether it is in the
//! local subnet.

use std::net::{Ipv4Addr, SocketAddr};

/// A parsed HTTP URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpUrl {
    /// The scheme, in lower case.
    pub scheme: String,
    /// The host, exactly as it was written.
    pub host: String,
    /// The port, if the URL carried one.
    pub port: Option<u16>,
    /// The path, at least `/`.
    pub path: String,
}

impl HttpUrl {
    /// The address the URL points to.
    #[must_use]
    pub fn socket_addr(&self) -> Option<SocketAddr> {
        let address = self.host.parse::<Ipv4Addr>().ok()?;
        Some(SocketAddr::from((address, self.port.unwrap_or(80))))
    }
}

/// Parses an `http://host[:port][/path]` URL.
///
/// WHY: the URL comes from an untrusted header, so the parser is strict:
/// a missing scheme, a credential in the authority, a fragment or a
/// malformed port is an error — never a best guess.
pub fn parse_http_url(raw: &str) -> Result<HttpUrl, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err("the URL is empty".to_string());
    }
    if raw.len() > 2048 {
        return Err("the URL is too long".to_string());
    }
    let Some((scheme, rest)) = raw.split_once("://") else {
        return Err("the URL has no scheme".to_string());
    };
    let scheme = scheme.to_ascii_lowercase();
    if scheme.is_empty()
        || !scheme
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'-' | b'.'))
    {
        return Err("the scheme is malformed".to_string());
    }
    let (authority, path) = match rest.find(['/', '?', '#']) {
        Some(index) => (&rest[..index], &rest[index..]),
        None => (rest, ""),
    };
    if authority.contains('@') {
        return Err("the URL carries credentials".to_string());
    }
    if path.contains('#') {
        return Err("the URL carries a fragment".to_string());
    }
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => {
            let parsed = port
                .parse::<u16>()
                .map_err(|_| "the port is malformed".to_string())?;
            if parsed == 0 {
                return Err("the port is zero".to_string());
            }
            (host, Some(parsed))
        }
        None => (authority, None),
    };
    if host.is_empty() {
        return Err("the host is empty".to_string());
    }
    if host.contains(['[', ']']) {
        return Err("the host is not a plain address".to_string());
    }
    let path = if path.is_empty() { "/" } else { path };
    Ok(HttpUrl {
        scheme,
        host: host.to_string(),
        port,
        path: path.to_string(),
    })
}

/// Whether an address is in the private range of RFC 1918.
///
/// WHY: the loopback range (`127/8`) and the link-local range (`169.254/16`,
/// which contains the cloud metadata address) are *not* private ranges.
/// A control point has no business on either of them.
#[must_use]
pub fn is_private_ipv4(address: Ipv4Addr) -> bool {
    let octets = address.octets();
    octets[0] == 10
        || (octets[0] == 172 && (16..=31).contains(&octets[1]))
        || (octets[0] == 192 && octets[1] == 168)
}

/// Whether two addresses are in the same /24.
#[must_use]
pub fn same_subnet(left: Ipv4Addr, right: Ipv4Addr) -> bool {
    let left = left.octets();
    let right = right.octets();
    left[0] == right[0] && left[1] == right[1] && left[2] == right[2]
}
