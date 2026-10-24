//! The identity this receiver advertises to senders.
//!
//! WHY: senders identify the receiver by name. A name that is too long, that
//! contains control characters, or that changes on every start makes the
//! device churn through the sender's remembered-device list. The identity is
//! therefore persisted, sanitised and stable.

use serde::{Deserialize, Serialize};

/// The maximum length of a DNS-SD service instance name.
pub const MAX_SERVICE_NAME_LENGTH: usize = 63;

/// The identity of this receiver.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceIdentity {
    /// The user-visible device name, e.g. "Living Room".
    pub name: String,
    /// The hardware model, e.g. "PhairPlay Rust Receiver".
    pub model: String,
    /// The manufacturer string reported to senders.
    pub manufacturer: String,
    /// The protocol version reported to senders.
    pub protocol_version: String,
    /// A stable, opaque identifier for this installation.
    pub device_id: String,
}

impl Default for DeviceIdentity {
    fn default() -> Self {
        Self {
            name: "PhairPlay".to_string(),
            model: "PhairPlay Rust Receiver".to_string(),
            manufacturer: "PhairPlayRust".to_string(),
            protocol_version: "1.0".to_string(),
            device_id: "00000000-0000-0000-0000-000000000000".to_string(),
        }
    }
}

impl DeviceIdentity {
    /// Creates an identity with the given device name.
    #[must_use]
    pub fn with_name(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Self::default()
        }
    }

    /// The sanitised DNS-SD service instance name.
    ///
    /// WHY: an mDNS daemon rejects a name that contains control characters,
    /// and an unhandled rejection inside the registration path takes the
    /// whole receiver down — the device silently vanishes from every sender's
    /// picker. Sanitising here means no engine ever has to deal with it.
    #[must_use]
    pub fn service_instance_name(&self) -> String {
        sanitize_service_name(&self.name, "PhairPlay")
    }
}

/// Sanitises a display name for use as a DNS-SD service instance name.
///
/// - Control and non-printable characters are removed.
/// - The result is trimmed and capped at [`MAX_SERVICE_NAME_LENGTH`].
/// - If nothing usable is left, `fallback` is returned.
#[must_use]
pub fn sanitize_service_name(raw: &str, fallback: &str) -> String {
    let printable: String = raw
        .chars()
        .filter(|c| (' '..='~').contains(c))
        .collect::<String>();
    let trimmed = printable.trim();
    let truncated = trimmed
        .chars()
        .take(MAX_SERVICE_NAME_LENGTH)
        .collect::<String>();
    let result = truncated.trim();
    if result.is_empty() {
        fallback
            .chars()
            .take(MAX_SERVICE_NAME_LENGTH)
            .collect::<String>()
            .trim()
            .to_string()
    } else {
        result.to_string()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    #[test]
    fn control_characters_are_removed() {
        let name = sanitize_service_name("Living\u{0} Room\n", "PhairPlay");
        assert_eq!(name, "Living Room");
    }

    #[test]
    fn names_are_capped_at_the_dns_sd_limit() {
        let long = "A".repeat(200);
        let name = sanitize_service_name(&long, "PhairPlay");
        assert_eq!(name.chars().count(), MAX_SERVICE_NAME_LENGTH);
    }

    #[test]
    fn an_unusable_name_falls_back() {
        let name = sanitize_service_name("\u{1}\u{2}\u{3}", "PhairPlay");
        assert_eq!(name, "PhairPlay");
    }

    #[test]
    fn a_name_that_is_only_whitespace_falls_back() {
        let name = sanitize_service_name("   ", "PhairPlay");
        assert_eq!(name, "PhairPlay");
    }

    #[test]
    fn identity_uses_the_sanitised_name_for_the_service_instance() {
        let identity = DeviceIdentity::with_name("Kitchen\u{0} TV");
        assert_eq!(identity.service_instance_name(), "Kitchen TV");
    }
}
