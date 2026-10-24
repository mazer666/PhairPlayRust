//! Apple property-list codec — XML **and** binary (`bplist00`), both ways.
//!
//! WHAT: this crate reads and writes the two property-list representations
//!      that the AirPlay 2 handshake and the Google Cast protocol use:
//!      the XML form (`text/x-apple-plist+xml`) and the binary form
//!      (`bplist00`). Both directions are supported for both forms.
//!
//! WHY: the receiver stack has to speak both dialects, and the binary
//!      dialect arrives straight from the network. A property list is
//!      therefore *untrusted input*: every length it announces is checked
//!      against the bytes that are actually present, the number of
//!      objects, the nesting depth and the size of the expanded object
//!      graph are bounded, and an object graph that refers back to itself
//!      is refused instead of being followed. See [`PlistLimits`].
//!
//! EXAMPLE:
//! ```
//! use phairplay_plist::{decode, encode_binary, encode_xml, Value};
//!
//! let value = Value::dictionary([
//!     ("name", Value::String("Living Room".into())),
//!     ("volume", Value::Integer(42)),
//! ]);
//!
//! let binary = encode_binary(&value).unwrap_or_else(|err| panic!("{err}"));
//! assert_eq!(&binary[..8], b"bplist00");
//! assert_eq!(decode(&binary).unwrap_or_else(|err| panic!("{err}")), value);
//!
//! let xml = encode_xml(&value).unwrap_or_else(|err| panic!("{err}"));
//! assert!(xml.starts_with(b"<?xml"));
//! ```

#![warn(missing_docs)]

mod base64;
mod binary;
mod date;
mod error;
mod limits;
mod value;
mod xml;

pub use error::PlistError;
pub use limits::PlistLimits;
pub use value::Value;

use crate::binary::MAGIC;

/// Decodes a property list, detecting the format from the first bytes.
///
/// # Errors
///
/// Returns an error when the input is neither a binary nor an XML
/// property list, or when it violates one of the default limits.
///
/// # Example
///
/// ```
/// use phairplay_plist::{decode, Value};
///
/// let bytes = b"<?xml version=\"1.0\"?><plist version=\"1.0\"><dict><key>a</key><integer>1</integer></dict></plist>";
/// let value = decode(bytes).unwrap_or_else(|err| panic!("{err}"));
/// assert_eq!(value.get("a"), Some(&Value::Integer(1)));
/// ```
pub fn decode(bytes: &[u8]) -> Result<Value, PlistError> {
    decode_with_limits(bytes, &PlistLimits::default())
}

/// Decodes a property list with an explicit set of limits.
///
/// # Errors
///
/// Returns an error when the input is malformed or exceeds a limit.
///
/// # Example
///
/// ```
/// use phairplay_plist::{decode_with_limits, PlistLimits};
///
/// let limits = PlistLimits { max_depth: 0, ..PlistLimits::default() };
/// let bytes = b"<plist><array><array/></array></plist>";
/// assert!(decode_with_limits(bytes, &limits).is_err());
/// ```
pub fn decode_with_limits(bytes: &[u8], limits: &PlistLimits) -> Result<Value, PlistError> {
    if is_binary(bytes) {
        return binary::decode(bytes, limits);
    }
    let text = std::str::from_utf8(bytes)
        .map_err(|_| PlistError::InvalidTextEncoding { encoding: "UTF-8" })?;
    if !text.trim_start().starts_with('<') {
        return Err(PlistError::UnknownFormat);
    }
    xml::decode(text, limits)
}

/// Returns `true` when the bytes start with the binary property-list magic.
///
/// # Example
///
/// ```
/// assert!(phairplay_plist::is_binary(b"bplist00"));
/// assert!(!phairplay_plist::is_binary(b"<?xml"));
/// ```
pub fn is_binary(bytes: &[u8]) -> bool {
    bytes.len() >= MAGIC.len() && &bytes[..MAGIC.len()] == MAGIC
}

/// Encodes a value as an XML property list.
///
/// # Errors
///
/// Returns an error when the value cannot be represented in XML.
///
/// # Example
///
/// ```
/// use phairplay_plist::{encode_xml, Value};
///
/// let bytes = encode_xml(&Value::Boolean(true)).unwrap_or_else(|err| panic!("{err}"));
/// assert!(bytes.windows(7).any(|window| window == b"<true/>"));
/// ```
pub fn encode_xml(value: &Value) -> Result<Vec<u8>, PlistError> {
    xml::encode(value)
}

/// Encodes a value as a binary property list.
///
/// # Errors
///
/// Returns an error when the value nests deeper than the default limit
/// allows.
///
/// # Example
///
/// ```
/// use phairplay_plist::{encode_binary, Value};
///
/// let bytes = encode_binary(&Value::Integer(7)).unwrap_or_else(|err| panic!("{err}"));
/// assert_eq!(&bytes[..8], b"bplist00");
/// ```
pub fn encode_binary(value: &Value) -> Result<Vec<u8>, PlistError> {
    binary::encode(value, &PlistLimits::default())
}

/// Encodes a value as a binary property list with an explicit set of limits.
///
/// # Errors
///
/// Returns an error when the value nests deeper than the limit allows.
///
/// # Example
///
/// ```
/// use phairplay_plist::{encode_binary_with_limits, PlistLimits, Value};
///
/// let limits = PlistLimits { max_depth: 0, ..PlistLimits::default() };
/// let deep = Value::Array(vec![Value::Array(vec![Value::Integer(1)])]);
/// assert!(encode_binary_with_limits(&deep, &limits).is_err());
/// ```
pub fn encode_binary_with_limits(
    value: &Value,
    limits: &PlistLimits,
) -> Result<Vec<u8>, PlistError> {
    binary::encode(value, limits)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    #[test]
    fn detects_the_binary_format() {
        assert!(is_binary(b"bplist00"));
        assert!(!is_binary(b"bplist0"));
        assert!(!is_binary(b"<?xml version=\"1.0\"?>"));
    }

    #[test]
    fn rejects_input_that_is_neither_format() {
        assert!(matches!(
            decode(b"\x00\x01\x02\x03"),
            Err(PlistError::UnknownFormat | PlistError::InvalidTextEncoding { .. })
        ));
    }

    #[test]
    fn rejects_invalid_utf8_in_an_xml_document() {
        let mut bytes = b"<plist><dict>".to_vec();
        bytes.push(0xff);
        bytes.extend_from_slice(b"</dict></plist>");
        assert!(matches!(
            decode(&bytes),
            Err(PlistError::InvalidTextEncoding { encoding: "UTF-8" })
        ));
    }

    #[test]
    fn round_trips_through_both_representations() {
        let value = Value::dictionary([
            ("name", Value::String("Room".into())),
            ("level", Value::Real(0.5)),
            ("flag", Value::Boolean(true)),
            ("blob", Value::Data(vec![1, 2, 3])),
            (
                "list",
                Value::Array(vec![Value::Integer(1), Value::Integer(2)]),
            ),
        ]);
        let binary = encode_binary(&value).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(decode(&binary).unwrap_or_else(|err| panic!("{err}")), value);
        let xml = encode_xml(&value).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(decode(&xml).unwrap_or_else(|err| panic!("{err}")), value);
    }
}
