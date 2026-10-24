//! Apple property lists.
//!
//! WHY: the AirPlay 2 handshake is a property-list conversation. The codec
//! lives in its own module because it is a *format*, not a protocol: it
//! transforms bytes into bytes and knows nothing about AirPlay. Both the
//! binary (`bplist00`) and the legacy XML form are supported, in both
//! directions, with bounded decoders.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_airplay::plist::{PlistValue, decode};
//!
//! let document = PlistValue::dictionary(vec![("vv".into(), PlistValue::Integer(2))]);
//! let encoded = phairplay_airplay::plist::binary::encode(&document).expect("encodable");
//! let decoded = decode(&encoded, 1024 * 1024).expect("decodable");
//! assert_eq!(decoded.get("vv").and_then(PlistValue::as_integer), Some(2));
//! ```

pub mod binary;
pub mod value;
pub mod xml;

pub use value::PlistValue;

use crate::error::AirPlayError;

/// Decodes a property list in either the binary or the XML form.
///
/// # Errors
/// Returns [`AirPlayError::Parse`] when the document is malformed in either
/// form.
pub fn decode(bytes: &[u8], max_bytes: usize) -> Result<PlistValue, AirPlayError> {
    if bytes.starts_with(binary::MAGIC) {
        return binary::decode(bytes, max_bytes);
    }
    let text = std::str::from_utf8(bytes)
        .map_err(|_| AirPlayError::parse("plist is neither binary nor UTF-8"))?;
    xml::decode(text, max_bytes)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    #[test]
    fn a_binary_document_is_decoded_by_the_generic_entry_point() {
        let document = PlistValue::dictionary(vec![("a".into(), PlistValue::Integer(1))]);
        let encoded = binary::encode(&document).expect("encodable");
        assert_eq!(decode(&encoded, 4096).expect("decodable"), document);
    }

    #[test]
    fn an_xml_document_is_decoded_by_the_generic_entry_point() {
        let document = PlistValue::dictionary(vec![("a".into(), PlistValue::Integer(1))]);
        let encoded = xml::encode(&document);
        assert_eq!(decode(&encoded, 4096).expect("decodable"), document);
    }

    #[test]
    fn a_garbage_document_is_rejected() {
        let error = decode(&[0xFFu8; 32], 4096).expect_err("must fail");
        assert!(matches!(error, AirPlayError::Parse { .. }));
    }
}
