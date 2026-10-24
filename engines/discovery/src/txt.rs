//! The DNS-SD TXT records that tell a sender what this receiver can do.
//!
//! WHY: the service *type* only says "there is an AirPlay receiver here".
//! The TXT records carry the actual contract: which AirPlay features are
//! implemented, which codecs are supported, what the device is called. A
//! sender that finds no `features` record assumes the AirPlay 1 subset and
//! never starts the mirroring pipeline.
//!
//! HOW: every record is built from the data the core crate owns — the
//! [`DeviceIdentity`] and the [`MediaCapabilities`] — plus the constants
//! that the reference implementation proved to work with real senders. The
//! values are plain ASCII: RFC 6763 requires a TXT string to consist of
//! printable US-ASCII characters, and a name with a control character in it
//! makes the whole registration fail.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_core::identity::DeviceIdentity;
//! use phairplay_discovery::txt::airplay_txt;
//!
//! let identity = DeviceIdentity::with_name("Living Room");
//! let record = airplay_txt(&identity);
//! assert_eq!(record.get("model"), Some("AppleTV5,3"));
//! ```

use phairplay_core::identity::DeviceIdentity;

use crate::service_types::{format_mac, pseudo_mac};

/// The model string that makes macOS treat the receiver like an Apple TV.
///
/// WHY: without it macOS falls back to the AirPlay 1 photo-only mode and
/// never offers screen mirroring.
pub const AIRPLAY_MODEL: &str = "AppleTV5,3";

/// The AirPlay server version reported to senders.
pub const AIRPLAY_SERVER_VERSION: &str = "220.68";

/// The AirPlay feature bitmask.
///
/// WHY: the bitmask is what tells macOS that mirroring, photo sharing and
/// synchronised audio are available. The value is the one the reference
/// implementation ships and that real senders accept; the individual bit
/// positions are documented in the upstream technical specification, which
/// does not agree with the shipped constant bit for bit (see the delivery
/// notes).
pub const AIRPLAY_FEATURES: &str = "0x5A7FFFF7,0x1E";

/// The AirPlay protocol version (`vv` record).
pub const AIRPLAY_PROTOCOL_VERSION: &str = "2";

/// The AirPlay flags record. `0x4` marks a screen-mirroring receiver.
pub const AIRPLAY_FLAGS: &str = "0x4";

/// The RAOP cipher numbers the receiver supports.
pub const RAOP_CIPHER_NUMBERS: &str = "0,1,2,3";

/// The RAOP encryption types the receiver supports.
pub const RAOP_ENCRYPTION_TYPES: &str = "0,3,5";

/// The RAOP metadata types the receiver supports.
pub const RAOP_METADATA_TYPES: &str = "0,1,2";

/// The RAOP transport.
pub const RAOP_TRANSPORT: &str = "UDP";

/// The RAOP version number. Senders refuse to talk to a receiver that does
/// not advertise it.
pub const RAOP_VERSION: &str = "65537";

/// The Cast capability bitmask: video and audio, 4K not supported.
pub const CAST_CAPABILITIES: &str = "4101";

/// The Cast model identifier.
pub const CAST_MODEL: &str = "0 1 2";

/// The Cast protocol version.
pub const CAST_VERSION: &str = "1.0";

/// The Miracast TXT record version.
pub const MIRACAST_TXT_VERSION: &str = "1";

/// The number of audio channels the receiver advertises (`ch` record).
pub const AUDIO_CHANNELS: &str = "2";

/// An ordered set of DNS-SD TXT key/value pairs.
///
/// WHY: the order of the records is irrelevant to the protocol but relevant
/// to the tests and to the log output, so the record keeps insertion order
/// instead of using a hash map.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TxtRecord {
    entries: Vec<(String, String)>,
}

impl TxtRecord {
    /// An empty record.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds or replaces a record.
    ///
    /// # Panics
    /// Never — a key that is not printable ASCII or that contains `=` is
    /// rejected, because RFC 6763 forbids both and the mDNS daemon would
    /// refuse the whole registration.
    pub fn set(&mut self, key: &str, value: &str) -> &mut Self {
        let key = sanitise_key(key);
        let value = sanitise_value(value);
        match self.entries.iter_mut().find(|(k, _)| *k == key) {
            Some(entry) => entry.1 = value,
            None => self.entries.push((key, value)),
        }
        self
    }

    /// The value of a record.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// All records, in insertion order.
    #[must_use]
    pub fn entries(&self) -> &[(String, String)] {
        &self.entries
    }

    /// The number of records.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the record is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Removes the characters RFC 6763 forbids in a TXT key.
fn sanitise_key(key: &str) -> String {
    key.chars()
        .filter(|c| c.is_ascii_graphic() && *c != '=')
        .collect()
}

/// Removes the characters RFC 6763 forbids in a TXT value.
fn sanitise_value(value: &str) -> String {
    value.chars().filter(|c| (' '..='~').contains(c)).collect()
}

/// Builds the `_airplay._tcp` TXT record.
///
/// WHY: the `features` record is the one that decides whether macOS offers
/// screen mirroring at all; the `deviceid` must stay stable across restarts
/// or the device reappears as a new entry in every sender's device list.
#[must_use]
pub fn airplay_txt(identity: &DeviceIdentity) -> TxtRecord {
    let mac = format_mac(&pseudo_mac(&identity.device_id));
    let mut record = TxtRecord::new();
    record.set("deviceid", &mac);
    record.set("features", AIRPLAY_FEATURES);
    record.set("model", AIRPLAY_MODEL);
    record.set("srcvers", AIRPLAY_SERVER_VERSION);
    record.set("vv", AIRPLAY_PROTOCOL_VERSION);
    record.set("pi", &identity.device_id);
    record.set("flags", AIRPLAY_FLAGS);
    record.set("name", &identity.name);
    // The number of audio channels is what a sender checks before it offers
    // the audio-only entry point.
    record.set("ch", AUDIO_CHANNELS);
    record
}

/// Builds the `_raop._tcp` TXT record.
///
/// WHY: `cn` (cipher numbers) and `et` (encryption types) decide whether
/// the sender may use the encrypted RAOP pipeline; `vn` is mandatory — a
/// sender that does not find a version number refuses the service.
#[must_use]
pub fn raop_txt(identity: &DeviceIdentity) -> TxtRecord {
    let mac = format_mac(&pseudo_mac(&identity.device_id));
    let mut record = TxtRecord::new();
    record.set("cn", RAOP_CIPHER_NUMBERS);
    record.set("da", "true");
    record.set("et", RAOP_ENCRYPTION_TYPES);
    record.set("md", RAOP_METADATA_TYPES);
    record.set("sv", "false");
    record.set("tp", RAOP_TRANSPORT);
    record.set("vn", RAOP_VERSION);
    record.set("vs", AIRPLAY_SERVER_VERSION);
    record.set("am", AIRPLAY_MODEL);
    record.set("deviceid", &mac);
    record.set("ch", AUDIO_CHANNELS);
    record
}

/// Builds the `_googlecast._tcp` TXT record.
#[must_use]
pub fn cast_txt(identity: &DeviceIdentity) -> TxtRecord {
    let mut record = TxtRecord::new();
    record.set("md", CAST_MODEL);
    record.set("id", &identity.device_id);
    record.set("fn", &identity.name);
    record.set("rs", "");
    record.set("ca", CAST_CAPABILITIES);
    record.set("st", "0");
    record.set("bs", "GOOG");
    record.set("vs", CAST_VERSION);
    record.set("ve", "05");
    record
}

/// Builds the `_miracast._tcp` TXT record.
#[must_use]
pub fn miracast_txt(identity: &DeviceIdentity) -> TxtRecord {
    let mut record = TxtRecord::new();
    record.set("friendlyname", &identity.name);
    record.set("manufacturer", &identity.manufacturer);
    record.set("modelname", &identity.model);
    record.set("txtvers", MIRACAST_TXT_VERSION);
    record.set("source", "PhairPlayRust");
    record
}

#[cfg(test)]
mod tests;
