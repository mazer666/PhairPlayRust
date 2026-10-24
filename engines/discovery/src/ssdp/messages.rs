//! The messages of the SSDP client.
//!
//! WHY: the answers of the network come from arbitrary devices. The codec
//! therefore rejects everything that is not an answer, checks the number
//! and the length of the headers, and never repairs a message.
//!
//! HOW: the codec is pure — it turns text into messages — and is tested
//! without a network.

use super::{
    MAX_SSDP_HEADER_BYTES, MAX_SSDP_HEADER_COUNT, MAX_SSDP_MESSAGE_BYTES, MULTICAST_ADDRESS,
    SSDP_PORT,
};

/// An `M-SEARCH` query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SsdpSearchQuery {
    /// The search target.
    pub search_target: String,
    /// The maximum number of seconds a device may take to answer.
    pub mx: u16,
}

impl SsdpSearchQuery {
    /// A query for the given search target.
    #[must_use]
    pub fn new(search_target: &str) -> Self {
        Self {
            search_target: search_target.to_string(),
            mx: 1,
        }
    }

    /// Sets the `MX` header.
    #[must_use]
    pub fn with_mx(mut self, mx: u16) -> Self {
        self.mx = mx;
        self
    }

    /// The query as it goes on the wire.
    #[must_use]
    pub fn to_message(&self) -> String {
        format!(
            "M-SEARCH * HTTP/1.1\r\nHOST: {MULTICAST_ADDRESS}:{SSDP_PORT}\r\n\
             MAN: \"ssdp:discover\"\r\nMX: {}\r\nST: {}\r\n\r\n",
            self.mx, self.search_target
        )
    }
}

/// A `200 OK` answer to an `M-SEARCH` query.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SsdpResponse {
    /// The URL of the device description.
    pub location: String,
    /// The unique service name.
    pub usn: String,
    /// The search target the device answered.
    pub search_target: String,
    /// The server string of the device.
    pub server: String,
    /// The `CACHE-CONTROL` header of the answer.
    pub cache_control: String,
}

impl SsdpResponse {
    /// Parses an answer.
    ///
    /// WHY: the answer comes from an arbitrary device on the LAN. A message
    /// that is not an answer, that carries more headers than the limit
    /// allows, or that has an over-long header line is *rejected* — it is
    /// never repaired, because a repaired answer points the control point
    /// at a location the sender did not announce.
    #[must_use]
    pub fn parse(message: &str) -> Option<Self> {
        if message.len() > MAX_SSDP_MESSAGE_BYTES {
            return None;
        }
        let (status, headers) = parse_message(message)?;
        if !status.to_ascii_uppercase().starts_with("HTTP/1.1 200") {
            return None;
        }
        let mut response = Self::default();
        for (key, value) in headers {
            match key.to_ascii_lowercase().as_str() {
                "location" => response.location = value,
                "usn" => response.usn = value,
                "st" => response.search_target = value,
                "server" => response.server = value,
                "cache-control" => response.cache_control = value,
                _ => {}
            }
        }
        Some(response)
    }
}

/// A `NOTIFY` message — a device that appeared or disappeared.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SsdpNotification {
    /// The unique service name.
    pub usn: String,
    /// The notification target.
    pub notification_type: String,
    /// Whether the device announced itself (`ssdp:alive`) or left
    /// (`ssdp:byebye`).
    pub alive: bool,
    /// The URL of the device description, if the device announced one.
    pub location: Option<String>,
}

impl SsdpNotification {
    /// Parses a notification.
    #[must_use]
    pub fn parse(message: &str) -> Option<Self> {
        if message.len() > MAX_SSDP_MESSAGE_BYTES {
            return None;
        }
        let (status, headers) = parse_message(message)?;
        if !status.to_ascii_uppercase().starts_with("NOTIFY ") {
            return None;
        }
        let mut notification = Self::default();
        let mut nts = String::new();
        for (key, value) in headers {
            match key.to_ascii_lowercase().as_str() {
                "nts" => nts = value,
                "nt" => notification.notification_type = value,
                "usn" => notification.usn = value,
                "location" => notification.location = Some(value),
                _ => {}
            }
        }
        match nts.to_ascii_lowercase().as_str() {
            "ssdp:alive" => notification.alive = true,
            "ssdp:byebye" => notification.alive = false,
            _ => return None,
        }
        Some(notification)
    }
}

/// An event the browser observed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SsdpEvent {
    /// A device answered a query.
    Response(SsdpResponse),
    /// A device announced itself or its departure.
    Notification(SsdpNotification),
}

/// Splits an SSDP message into its status line and its headers.
fn parse_message(message: &str) -> Option<(String, Vec<(String, String)>)> {
    let mut lines = message.split("\r\n");
    let status = lines.next()?.trim().to_string();
    if status.is_empty() {
        return None;
    }
    let mut headers = Vec::new();
    let mut total = 0usize;
    for line in lines {
        if line.is_empty() {
            break;
        }
        if headers.len() >= MAX_SSDP_HEADER_COUNT {
            return None;
        }
        if line.len() > MAX_SSDP_HEADER_BYTES {
            return None;
        }
        total += line.len();
        if total > MAX_SSDP_HEADER_BYTES * MAX_SSDP_HEADER_COUNT {
            return None;
        }
        let (key, value) = line.split_once(':')?;
        headers.push((key.trim().to_string(), value.trim().to_string()));
    }
    Some((status, headers))
}
