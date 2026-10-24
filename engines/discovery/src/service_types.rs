//! The DNS-SD service types PhairPlayRust advertises, and the naming rules
//! that go with them.
//!
//! WHY: the service *type* is what a sender looks for. A receiver that
//! advertises the wrong type is invisible, and a receiver that advertises
//! only half of a type pair is invisible to *some* senders. AirPlay is the
//! prime example: an AirPlay receiver must publish **both** `_airplay._tcp`
//! (the mirroring pipeline) and `_raop._tcp` (the audio pipeline). macOS and
//! iOS refuse to show a device that only publishes `_airplay._tcp` when the
//! user picks the *audio* entry point, and several senders filter the device
//! list on `_raop._tcp` alone.
//!
//! HOW: the service types are constants here so that the discovery crate,
//! the protocol engines and the tests all agree on one spelling. The
//! *instance* name is sanitised with
//! [`phairplay_core::identity::sanitize_service_name`], because an mDNS
//! daemon rejects a name that contains control characters and an unhandled
//! rejection inside the registration path takes the whole receiver down.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_discovery::service_types::{AIRPLAY_TCP, RAOP_TCP, raop_instance_name};
//!
//! assert_eq!(AIRPLAY_TCP, "_airplay._tcp");
//! assert_eq!(RAOP_TCP, "_raop._tcp");
//!
//! // The RAOP instance name is "<MAC hex>@<display name>" and must stay
//! // inside the 63 character limit of a DNS-SD instance name.
//! let name = raop_instance_name("aabbccddeeff", "Living Room");
//! assert_eq!(name, "AABBCCDDEEFF@Living Room");
//! assert!(name.chars().count() <= 63);
//! ```

use phairplay_core::identity::sanitize_service_name;

pub use phairplay_core::identity::MAX_SERVICE_NAME_LENGTH;

/// The AirPlay mirroring service type.
pub const AIRPLAY_TCP: &str = "_airplay._tcp";

/// The RAOP (Remote Audio Output Protocol) service type.
///
/// WHY: it is required *alongside* [`AIRPLAY_TCP`], not instead of it.
pub const RAOP_TCP: &str = "_raop._tcp";

/// The Google Cast service type.
pub const GOOGLECAST_TCP: &str = "_googlecast._tcp";

/// The Miracast / Wi-Fi Display service type.
pub const MIRACAST_TCP: &str = "_miracast._tcp";

/// Every service type this crate knows how to advertise.
pub const ALL_SERVICE_TYPES: [&str; 4] = [AIRPLAY_TCP, RAOP_TCP, GOOGLECAST_TCP, MIRACAST_TCP];

/// The separator between the MAC prefix and the display name in a RAOP
/// service instance name.
const RAOP_SEPARATOR: char = '@';

/// The length of the MAC address in hexadecimal digits, without separators.
const MAC_HEX_LENGTH: usize = 12;

/// Builds the RAOP service instance name.
///
/// WHY: the AirPlay protocol requires the RAOP instance name to be
/// `<MAC address in upper-case hex>@<device name>`. A receiver that
/// advertises a plain name is still found by some senders, but the audio
/// pipeline of the others silently never starts.
///
/// The result is capped at [`MAX_SERVICE_NAME_LENGTH`] characters *in total* —
/// the MAC prefix is not optional, so the display name is what gets
/// shortened. The `@` separator is removed from the display name: a second
/// `@` would make the name ambiguous for the RAOP parser of the sender.
///
/// EXAMPLE:
///
/// ```
/// use phairplay_discovery::service_types::raop_instance_name;
///
/// assert_eq!(raop_instance_name("aa:bb:cc:dd:ee:ff", "Kitchen"), "AABBCCDDEEFF@Kitchen");
/// ```
#[must_use]
pub fn raop_instance_name(mac: &str, display_name: &str) -> String {
    let hex = normalise_mac(mac);
    let cleaned: String = display_name
        .chars()
        .filter(|c| *c != RAOP_SEPARATOR)
        .collect::<String>();
    let name = sanitize_service_name(&cleaned, "PhairPlay");
    let budget = MAX_SERVICE_NAME_LENGTH
        .saturating_sub(hex.len() + 1)
        .min(MAX_SERVICE_NAME_LENGTH);
    let truncated: String = name.chars().take(budget).collect();
    format!("{hex}{RAOP_SEPARATOR}{truncated}")
}

/// Normalises a MAC address into twelve upper-case hexadecimal digits.
///
/// WHY: senders parse the RAOP instance name with a strict pattern. A MAC
/// that still carries colons (`aa:bb:cc:dd:ee:ff`) does not match it, and a
/// MAC that is shorter than six bytes cannot be padded with zeros without
/// inventing an address that may collide with a real one — so the padding is
/// derived from the *stable* device identifier instead.
#[must_use]
pub fn normalise_mac(mac: &str) -> String {
    let digits: String = mac
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .map(|c| c.to_ascii_uppercase())
        .collect();
    if digits.len() >= MAC_HEX_LENGTH {
        digits[..MAC_HEX_LENGTH].to_string()
    } else {
        let mut padded = digits;
        while padded.len() < MAC_HEX_LENGTH {
            padded.push('0');
        }
        padded
    }
}

/// Derives a stable, MAC-shaped identifier from a persistent device id.
///
/// WHY: the platform layer knows the real MAC address, but the core
/// identity does not carry one — and a MAC that changes on every start
/// makes the device churn through the remembered-device list of every
/// sender. The persistent device id is stable, so the first twelve
/// hexadecimal digits of it are used as the MAC-shaped prefix.
///
/// EXAMPLE:
///
/// ```
/// use phairplay_discovery::service_types::pseudo_mac;
///
/// let id = "0f9a7c11-2222-3333-4444-555566667777";
/// assert_eq!(pseudo_mac(id), "0F9A7C112222");
/// ```
#[must_use]
pub fn pseudo_mac(device_id: &str) -> String {
    normalise_mac(device_id)
}

/// Formats a MAC address the way the AirPlay `deviceid` record expects it.
///
/// WHY: the reference implementation reports the `deviceid` TXT record in
/// lower-case colon notation (`aa:bb:cc:dd:ee:ff`). A sender that parses
/// the record strictly rejects the twelve-digit form.
///
/// EXAMPLE:
///
/// ```
/// use phairplay_discovery::service_types::format_mac;
///
/// assert_eq!(format_mac("AABBCCDDEEFF"), "aa:bb:cc:dd:ee:ff");
/// ```
#[must_use]
pub fn format_mac(mac: &str) -> String {
    let hex = normalise_mac(mac);
    hex.as_bytes()
        .chunks(2)
        .map(|pair| String::from_utf8_lossy(pair).to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join(":")
}

/// Whether a service type is one this crate can advertise.
#[must_use]
pub fn is_known_service_type(service_type: &str) -> bool {
    ALL_SERVICE_TYPES.contains(&service_type)
}

#[cfg(test)]
mod tests;
