//! GENA: the General Event Notification Architecture of UPnP.
//!
//! WHY: a control point subscribes to the events of a service and the
//! renderer then *calls back* into the control point whenever the state
//! changes. That callback is a URL the renderer connects to — and a URL
//! that comes straight from an untrusted device on the LAN. Without a
//! guard, a control point can make the renderer connect to the cloud
//! metadata service of the network (`169.254.169.254`), to a loopback
//! service of the device, or to any host of the internet. That is a
//! server-side request forgery, and on a receiver that is supposed to be
//! privacy-first it is the worst possible bug.
//!
//! HOW: the guard in this module accepts a callback **only** when
//!
//! * the scheme is `http` (no `file:`, no `gopher:`, no `https:` to a
//!   different trust domain),
//! * the host is a *literal* IPv4 address — a name is never resolved, so
//!   there is no DNS rebinding and no resolver side channel,
//! * the address is in the private range of RFC 1918 (`10/8`, `172.16/12`,
//!   `192.168/16`) — which excludes the link-local metadata address
//!   `169.254.169.254` and the loopback address `127.0.0.1`,
//! * the address is in the same /24 as the receiver itself, and
//! * the URL carries no credentials and no fragment.
//!
//! `::1` is rejected twice over: it is not an IPv4 literal, and it is not
//! in a private range.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_dlna::gena::GenaEventManager;
//! use std::net::Ipv4Addr;
//!
//! let manager = GenaEventManager::new(Ipv4Addr::new(192, 168, 1, 5));
//!
//! // A control point on the same subnet may subscribe ...
//! assert!(manager.is_allowed_callback("http://192.168.1.20:8080/events"));
//! // ... the cloud metadata service may not.
//! assert!(!manager.is_allowed_callback("http://169.254.169.254/latest/meta-data/"));
//! ```

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, TcpStream};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use phairplay_core::privacy::mask_url;

/// The timeout of a GENA notification.
///
/// WHY: the deadline of a protocol step belongs to one place. Its single
/// owner is [`phairplay_core::config::timeouts::GENA_NOTIFY_TIMEOUT_MS`].
pub const NOTIFY_TIMEOUT: Duration =
    Duration::from_millis(phairplay_core::config::timeouts::GENA_NOTIFY_TIMEOUT_MS);

/// The lifetime of a subscription, in seconds.
///
/// WHY: the UPnP specification gives the advertisement lifetime and the
/// duration of a GENA subscription the same value, so both are one constant:
/// [`phairplay_core::config::timeouts::UPNP_MAX_AGE_SECONDS`].
pub const SUBSCRIPTION_TIMEOUT_SECONDS: u32 =
    phairplay_core::config::timeouts::UPNP_MAX_AGE_SECONDS;

/// The maximum size of the body of a notification.
pub const MAX_NOTIFY_BODY_BYTES: usize = 16 * 1024;

/// The link-local address of the cloud metadata service.
pub const METADATA_ADDRESS: Ipv4Addr = Ipv4Addr::new(169, 254, 169, 254);

/// The reasons a subscription can be refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenaError {
    /// The `CALLBACK` header is missing.
    MissingCallback,
    /// The `CALLBACK` header is malformed.
    MalformedCallback(String),
    /// The callback points at an address the renderer may not call.
    ForbiddenCallback(String),
    /// The `SID` header is missing.
    MissingSid,
    /// The subscription is unknown.
    UnknownSid,
}

impl std::fmt::Display for GenaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingCallback => write!(f, "the CALLBACK header is missing"),
            Self::MalformedCallback(reason) => {
                write!(f, "the CALLBACK header is malformed: {reason}")
            }
            Self::ForbiddenCallback(url) => write!(
                f,
                "the callback {url} is not a private IPv4 address of the local subnet"
            ),
            Self::MissingSid => write!(f, "the SID header is missing"),
            Self::UnknownSid => write!(f, "the subscription is unknown"),
        }
    }
}

impl std::error::Error for GenaError {}

mod url;

pub use url::{HttpUrl, is_private_ipv4, parse_http_url, same_subnet};

/// One event subscription.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subscription {
    /// The subscription identifier.
    pub sid: String,
    /// The callback URL.
    pub callback: String,
    /// The service path the subscription belongs to.
    pub service_path: String,
    /// The sequence number of the next notification.
    pub sequence: u32,
}

/// The answer to a `SUBSCRIBE` request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenaSubscription {
    /// The subscription identifier.
    pub sid: String,
    /// The lifetime of the subscription, in seconds.
    pub timeout_seconds: u32,
}

/// Sends a notification to a callback.
///
/// WHY: the trait is the seam that makes the event path testable without a
/// network, and it keeps the *decision* (which subscriptions exist, which
/// events are published) separate from the *transport*.
pub trait Notifier: Send + Sync + std::fmt::Debug {
    /// Delivers one notification.
    ///
    /// # Errors
    /// Returns a human-readable reason when the callback could not be
    /// reached.
    fn notify(&self, callback: &str, sid: &str, sequence: u32, body: &str) -> Result<(), String>;
}

/// The notifier that speaks HTTP over a TCP socket.
#[derive(Debug, Default)]
pub struct HttpNotifier;

impl Notifier for HttpNotifier {
    fn notify(&self, callback: &str, sid: &str, sequence: u32, body: &str) -> Result<(), String> {
        let url = parse_http_url(callback).map_err(|error| error.to_string())?;
        let address = url
            .socket_addr()
            .ok_or_else(|| "the callback has no usable address".to_string())?;
        let mut stream = TcpStream::connect_timeout(&address, NOTIFY_TIMEOUT)
            .map_err(|error| error.to_string())?;
        stream
            .set_read_timeout(Some(NOTIFY_TIMEOUT))
            .map_err(|error| error.to_string())?;
        stream
            .set_write_timeout(Some(NOTIFY_TIMEOUT))
            .map_err(|error| error.to_string())?;
        let request = format!(
            "NOTIFY {path} HTTP/1.1\r\nHOST: {host}:{port}\r\nCONTENT-TYPE: text/xml; charset=\"utf-8\"\r\n\
             NT: upnp:event\r\nNTS: upnp:propchange\r\nSID: {sid}\r\nSEQ: {sequence}\r\n\
             CONTENT-LENGTH: {length}\r\n\r\n{body}",
            path = url.path,
            host = url.host,
            port = url.port.unwrap_or(80),
            sid = sid,
            sequence = sequence,
            length = body.len(),
            body = body,
        );
        stream
            .write_all(request.as_bytes())
            .map_err(|error| error.to_string())?;
        let _ = stream.flush();
        // The status line is read so that the connection ends cleanly.
        let mut response = [0u8; 64];
        let _ = stream.read(&mut response);
        Ok(())
    }
}

/// The registry of event subscriptions.
#[derive(Debug)]
pub struct GenaEventManager {
    local_ip: Ipv4Addr,
    subscriptions: Mutex<HashMap<String, Subscription>>,
    counter: AtomicU64,
    notifier: Arc<dyn Notifier>,
}

impl GenaEventManager {
    /// A manager for a receiver with the given local address.
    #[must_use]
    pub fn new(local_ip: Ipv4Addr) -> Self {
        Self {
            local_ip,
            subscriptions: Mutex::new(HashMap::new()),
            counter: AtomicU64::new(0),
            notifier: Arc::new(HttpNotifier),
        }
    }

    /// A manager that uses a different notifier.
    #[must_use]
    pub fn with_notifier(local_ip: Ipv4Addr, notifier: Arc<dyn Notifier>) -> Self {
        Self {
            local_ip,
            subscriptions: Mutex::new(HashMap::new()),
            counter: AtomicU64::new(0),
            notifier,
        }
    }

    /// The local address of the receiver.
    #[must_use]
    pub fn local_ip(&self) -> Ipv4Addr {
        self.local_ip
    }

    /// Whether a callback URL may be called.
    ///
    /// This is the SSRF guard. See the module documentation for the rules.
    #[must_use]
    pub fn is_allowed_callback(&self, uri: &str) -> bool {
        let Ok(url) = parse_http_url(uri) else {
            return false;
        };
        if url.scheme != "http" {
            return false;
        }
        // A name is never resolved: only a literal address is accepted.
        let Ok(address) = url.host.parse::<Ipv4Addr>() else {
            return false;
        };
        if address == METADATA_ADDRESS {
            return false;
        }
        if !is_private_ipv4(address) {
            return false;
        }
        same_subnet(self.local_ip, address)
    }

    /// Registers a subscription.
    ///
    /// # Errors
    /// Returns [`GenaError`] when the callback header is missing, malformed
    /// or points at an address the renderer may not call.
    pub fn subscribe(
        &self,
        callback_header: Option<&str>,
        service_path: &str,
    ) -> Result<GenaSubscription, GenaError> {
        let raw = callback_header.ok_or(GenaError::MissingCallback)?.trim();
        if raw.is_empty() {
            return Err(GenaError::MissingCallback);
        }
        // The header may list several callbacks; the first one is used.
        let candidate = raw
            .split_whitespace()
            .next()
            .ok_or(GenaError::MissingCallback)?;
        let inner = candidate
            .strip_prefix('<')
            .and_then(|value| value.strip_suffix('>'))
            .ok_or_else(|| {
                GenaError::MalformedCallback(
                    "the callback is not enclosed in angle brackets".into(),
                )
            })?;
        if !self.is_allowed_callback(inner) {
            return Err(GenaError::ForbiddenCallback(inner.to_string()));
        }
        let sid = self.next_sid();
        let subscription = Subscription {
            sid: sid.clone(),
            callback: inner.to_string(),
            service_path: service_path.to_string(),
            sequence: 0,
        };
        self.subscriptions
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(sid.clone(), subscription);
        Ok(GenaSubscription {
            sid,
            timeout_seconds: SUBSCRIPTION_TIMEOUT_SECONDS,
        })
    }

    /// Removes a subscription.
    ///
    /// # Errors
    /// Returns [`GenaError::MissingSid`] when the header is missing and
    /// [`GenaError::UnknownSid`] when the subscription is not known.
    pub fn unsubscribe(&self, sid: Option<&str>) -> Result<(), GenaError> {
        let sid = sid.ok_or(GenaError::MissingSid)?;
        let removed = self
            .subscriptions
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(sid);
        if removed.is_some() {
            Ok(())
        } else {
            Err(GenaError::UnknownSid)
        }
    }

    /// The number of active subscriptions.
    #[must_use]
    pub fn subscription_count(&self) -> usize {
        self.subscriptions
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .len()
    }

    /// Publishes a state change to every subscription of a service.
    ///
    /// WHY: a control point that is not told about a state change keeps
    /// showing the old one. The reference implementation publishes the
    /// `LastChange` of the service that changed; this one does the same and
    /// increments the sequence number, which the specification requires.
    pub fn notify_subscribers(&self, service_path: &str, last_change: &str) {
        let body = format!(
            "<e:propertyset xmlns:e=\"urn:schemas-upnp-org:event-1-0\">\
             <e:property><LastChange>{}</LastChange></e:property></e:propertyset>",
            escape_xml(last_change)
        );
        if body.len() > MAX_NOTIFY_BODY_BYTES {
            return;
        }
        let targets: Vec<(String, String, u32)> = {
            let mut subscriptions = self.subscriptions.lock().unwrap_or_else(|e| e.into_inner());
            let mut targets = Vec::new();
            for subscription in subscriptions.values_mut() {
                if subscription.service_path == service_path {
                    targets.push((
                        subscription.callback.clone(),
                        subscription.sid.clone(),
                        subscription.sequence,
                    ));
                    subscription.sequence = subscription.sequence.wrapping_add(1);
                }
            }
            targets
        };
        for (callback, sid, sequence) in targets {
            if let Err(error) = self.notifier.notify(&callback, &sid, sequence, &body) {
                // WHY the masked URL: the callback carries the address of the
                // control point and often a token in its path. The log needs
                // to name the peer, not to identify it.
                tracing::warn!(callback = %mask_url(&callback), reason = %error, "the GENA notification failed");
            }
        }
    }

    /// Generates the next subscription identifier.
    ///
    /// WHY: the identifier has to be unique and has to look like a UUID,
    /// because the specification demands the `uuid:` prefix and control
    /// points store the identifier as an opaque string.
    fn next_sid(&self) -> String {
        let counter = self.counter.fetch_add(1, Ordering::Relaxed);
        let random: u64 = rand::random();
        let time_low = counter as u32;
        let time_mid = (random >> 48) as u16;
        let time_high = 0x4000 | (((random >> 32) as u16) & 0x0fff);
        let clock_seq = 0x8000 | (((random >> 16) as u16) & 0x3fff);
        let node = random as u32;
        format!("uuid:{time_low:08x}-{time_mid:04x}-{time_high:04x}-{clock_seq:04x}-{node:08x}")
    }
}

impl crate::soap::EventSink for GenaEventManager {
    fn publish(&self, service_path: &str, last_change: &str) {
        self.notify_subscribers(service_path, last_change);
    }
}

/// Escapes a text for use in an XML document.
fn escape_xml(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests;
