//! The binary property list codec (`bplist00`).
//!
//! WHY: the AirPlay 2 handshake is a binary property list conversation —
//! `GET /info` answers with one, the mirroring `SETUP` request is one. The
//! codec is therefore part of the attack surface: a peer controls the offset
//! table, the object references and the declared lengths. Every length is
//! checked against the buffer before it is used, the recursion depth is
//! bounded, and a document that references itself is rejected instead of
//! looping forever.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_airplay::plist::{PlistValue, binary};
//!
//! let document = PlistValue::dictionary(vec![("vv".into(), PlistValue::Integer(2))]);
//! let encoded = binary::encode(&document).expect("encodable");
//! assert!(encoded.starts_with(b"bplist00"));
//! let decoded = binary::decode(&encoded, 1024 * 1024).expect("decodable");
//! assert_eq!(decoded, document);
//! ```

use crate::plist::value::PlistValue;

/// The magic of a binary property list.
pub const MAGIC: &[u8; 8] = b"bplist00";

/// The size of the trailer at the end of a binary property list.
pub(super) const TRAILER_SIZE: usize = 32;

/// The maximum nesting depth the decoder accepts.
pub(super) const MAX_DEPTH: u32 = 32;

/// The maximum number of objects the decoder accepts.
pub(super) const MAX_OBJECTS: u64 = 1_000_000;

mod decoder;
mod encoder;

pub use decoder::decode;
pub use encoder::{encode, encode_with_limit};

/// The number of bytes an integer of the given magnitude needs.
pub(super) fn int_size(value: u64) -> usize {
    if value <= u64::from(u8::MAX) {
        1
    } else if value <= u64::from(u16::MAX) {
        2
    } else if value <= u64::from(u32::MAX) {
        4
    } else {
        8
    }
}

/// Writes an integer in the given size.
pub(super) fn write_int(out: &mut Vec<u8>, value: u64, size: usize) {
    let bytes = value.to_be_bytes();
    out.extend_from_slice(&bytes[8 - size..]);
}

/// Reads an integer of the given size.
pub(super) fn read_int(bytes: &[u8]) -> u64 {
    let mut buffer = [0u8; 8];
    let len = bytes.len().min(8);
    buffer[8 - len..].copy_from_slice(&bytes[..len]);
    u64::from_be_bytes(buffer)
}

impl PlistValue {
    /// A short textual form, used when a dictionary key is not a string.
    fn to_debug_string(&self) -> String {
        match self {
            Self::String(value) => value.clone(),
            Self::Integer(value) => value.to_string(),
            Self::Real(value) => value.to_string(),
            Self::Boolean(value) => value.to_string(),
            Self::Data(value) => format!("<{} bytes>", value.len()),
            Self::Array(value) => format!("<{} items>", value.len()),
            Self::Dictionary(value) => format!("<{} entries>", value.len()),
        }
    }
}

#[cfg(test)]
mod tests;
