//! The protobuf wire format, reduced to what the CastV2 envelope needs.
//!
//! WHY: the Google Cast protocol frames every message as a serialised
//! `CastMessage` protobuf. Pulling a full protobuf runtime into the receiver
//! for nine fields would be a large dependency for a small amount of work —
//! and it would put a code generator into the build of a privacy-first
//! receiver. This module implements the wire format directly: varints, tags
//! and length-delimited fields, with every length checked against a limit
//! before a single byte is copied.
//!
//! WHAT: [`ProtoWriter`] serialises, [`ProtoReader`] parses. Both refuse
//! malformed input instead of repairing it, which is the rule the build
//! contract sets for every network input.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_cast::ProtoWriter;
//!
//! let mut writer = ProtoWriter::new();
//! writer.write_string_field(4, "urn:x-cast:com.google.cast.receiver");
//! assert!(!writer.finish().is_empty());
//! ```

use std::fmt;

/// The protobuf wire type of a varint field.
pub const WIRE_VARINT: u8 = 0;
/// The protobuf wire type of a fixed 64-bit field.
pub const WIRE_64BIT: u8 = 1;
/// The protobuf wire type of a length-delimited field.
pub const WIRE_LENGTH_DELIMITED: u8 = 2;
/// The protobuf wire type of a fixed 32-bit field.
pub const WIRE_32BIT: u8 = 5;

/// The maximum number of bytes a varint may occupy.
pub const MAX_VARINT_BYTES: usize = 10;

/// The maximum number of bytes a single length-delimited field may carry.
///
/// WHY: a length-delimited field is the one place where a peer can name a
/// size. Without a ceiling, a four-byte header could ask for four gigabytes
/// of allocation on the local network.
pub const MAX_FIELD_BYTES: usize = 1024 * 1024;

/// A malformed protobuf message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtoError {
    /// The message ended in the middle of a field.
    Truncated,
    /// A varint was longer than the limit of the variable integer.
    VarintTooLong,
    /// The wire type of a field is not supported by the CastV2 envelope.
    UnsupportedWireType(u8),
    /// A length-delimited field declared more bytes than the limit allows.
    FieldTooLarge(usize),
    /// A string field did not contain valid UTF-8.
    InvalidUtf8,
    /// A field number of zero was read; protobuf reserves it.
    InvalidFieldNumber,
}

impl fmt::Display for ProtoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated => f.write_str("protobuf message is truncated"),
            Self::VarintTooLong => {
                write!(f, "protobuf varint exceeds {MAX_VARINT_BYTES} bytes")
            }
            Self::UnsupportedWireType(wire) => {
                write!(f, "protobuf wire type {wire} is not supported")
            }
            Self::FieldTooLarge(size) => write!(
                f,
                "protobuf field of {size} bytes exceeds the limit of {MAX_FIELD_BYTES} bytes"
            ),
            Self::InvalidUtf8 => f.write_str("protobuf string field is not valid UTF-8"),
            Self::InvalidFieldNumber => f.write_str("protobuf field number zero is reserved"),
        }
    }
}

impl std::error::Error for ProtoError {}

/// Serialises protobuf fields into a byte buffer.
#[derive(Debug, Default)]
pub struct ProtoWriter {
    buffer: Vec<u8>,
}

impl ProtoWriter {
    /// Creates an empty writer.
    #[must_use]
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    /// Appends a raw varint.
    pub fn write_varint(&mut self, mut value: u64) {
        loop {
            let byte = (value & 0x7f) as u8;
            value >>= 7;
            if value == 0 {
                self.buffer.push(byte);
                break;
            }
            self.buffer.push(byte | 0x80);
        }
    }

    /// Appends a field tag.
    pub fn write_tag(&mut self, field: u32, wire_type: u8) {
        self.write_varint(u64::from(field) << 3 | u64::from(wire_type));
    }

    /// Appends a varint field.
    pub fn write_varint_field(&mut self, field: u32, value: u64) {
        self.write_tag(field, WIRE_VARINT);
        self.write_varint(value);
    }

    /// Appends a length-delimited field.
    ///
    /// # Errors
    /// Returns [`ProtoError::FieldTooLarge`] when `bytes` is longer than
    /// the limit of a field. The limit is checked *before* the length prefix
    /// is written, so an oversized payload is never announced on the wire.
    pub fn write_bytes_field(&mut self, field: u32, bytes: &[u8]) -> Result<(), ProtoError> {
        if bytes.len() > MAX_FIELD_BYTES {
            return Err(ProtoError::FieldTooLarge(bytes.len()));
        }
        self.write_tag(field, WIRE_LENGTH_DELIMITED);
        self.write_varint(bytes.len() as u64);
        self.buffer.extend_from_slice(bytes);
        Ok(())
    }

    /// Appends a UTF-8 string field.
    ///
    /// # Errors
    /// Returns [`ProtoError::FieldTooLarge`] when the string is longer than
    /// the limit of a field.
    pub fn write_string_field(&mut self, field: u32, value: &str) -> Result<(), ProtoError> {
        self.write_bytes_field(field, value.as_bytes())
    }

    /// The bytes written so far.
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.buffer
    }

    /// Consumes the writer and returns the encoded message.
    #[must_use]
    pub fn finish(self) -> Vec<u8> {
        self.buffer
    }
}

/// Parses protobuf fields from a byte slice.
#[derive(Debug)]
pub struct ProtoReader<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> ProtoReader<'a> {
    /// Creates a reader over `data`.
    #[must_use]
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, position: 0 }
    }

    /// Whether every byte has been consumed.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.position >= self.data.len()
    }

    /// The number of bytes consumed so far.
    #[must_use]
    pub fn position(&self) -> usize {
        self.position
    }

    /// Reads a varint.
    ///
    /// # Errors
    /// Returns [`ProtoError::Truncated`] when the buffer ends inside the
    /// varint and [`ProtoError::VarintTooLong`] when the varint does not
    /// terminate within the limit of the variable integer.
    pub fn read_varint(&mut self) -> Result<u64, ProtoError> {
        let mut result: u64 = 0;
        let mut shift = 0u32;
        for index in 0..MAX_VARINT_BYTES {
            let byte = *self
                .data
                .get(self.position + index)
                .ok_or(ProtoError::Truncated)?;
            result |= u64::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                self.position += index + 1;
                return Ok(result);
            }
            shift += 7;
        }
        Err(ProtoError::VarintTooLong)
    }

    /// Reads a field tag.
    ///
    /// # Errors
    /// Returns [`ProtoError::InvalidFieldNumber`] for the reserved field
    /// number zero and any error of [`ProtoReader::read_varint`].
    pub fn read_tag(&mut self) -> Result<(u32, u8), ProtoError> {
        let key = self.read_varint()?;
        let field = (key >> 3) as u32;
        if field == 0 {
            return Err(ProtoError::InvalidFieldNumber);
        }
        let wire_type = (key & 0x07) as u8;
        Ok((field, wire_type))
    }

    /// Reads a length-delimited field.
    ///
    /// # Errors
    /// Returns [`ProtoError::Truncated`] when the buffer is shorter than the
    /// declared length, [`ProtoError::FieldTooLarge`] when the declared length
    /// exceeds the limit of a field and any error of
    /// [`ProtoReader::read_varint`].
    pub fn read_bytes(&mut self) -> Result<&'a [u8], ProtoError> {
        let length = self.read_varint()? as usize;
        if length > MAX_FIELD_BYTES {
            return Err(ProtoError::FieldTooLarge(length));
        }
        let end = self
            .position
            .checked_add(length)
            .ok_or(ProtoError::Truncated)?;
        if end > self.data.len() {
            return Err(ProtoError::Truncated);
        }
        let slice = &self.data[self.position..end];
        self.position = end;
        Ok(slice)
    }

    /// Reads a UTF-8 string field.
    ///
    /// # Errors
    /// Returns [`ProtoError::InvalidUtf8`] when the bytes are not valid UTF-8
    /// and any error of [`ProtoReader::read_bytes`].
    pub fn read_string(&mut self) -> Result<&'a str, ProtoError> {
        let bytes = self.read_bytes()?;
        std::str::from_utf8(bytes).map_err(|_| ProtoError::InvalidUtf8)
    }

    /// Skips a field the reader does not know.
    ///
    /// # Errors
    /// Returns [`ProtoError::UnsupportedWireType`] for a wire type that
    /// cannot be skipped and any error of [`ProtoReader::read_varint`].
    pub fn skip_field(&mut self, wire_type: u8) -> Result<(), ProtoError> {
        match wire_type {
            WIRE_VARINT => {
                self.read_varint()?;
            }
            WIRE_64BIT => {
                if self.position + 8 > self.data.len() {
                    return Err(ProtoError::Truncated);
                }
                self.position += 8;
            }
            WIRE_LENGTH_DELIMITED => {
                self.read_bytes()?;
            }
            WIRE_32BIT => {
                if self.position + 4 > self.data.len() {
                    return Err(ProtoError::Truncated);
                }
                self.position += 4;
            }
            other => return Err(ProtoError::UnsupportedWireType(other)),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    #[test]
    fn a_varint_round_trips() {
        let mut writer = ProtoWriter::new();
        writer.write_varint(0);
        writer.write_varint(1);
        writer.write_varint(300);
        writer.write_varint(u64::MAX);
        let bytes = writer.finish();

        let mut reader = ProtoReader::new(&bytes);
        assert_eq!(reader.read_varint().expect("varint"), 0);
        assert_eq!(reader.read_varint().expect("varint"), 1);
        assert_eq!(reader.read_varint().expect("varint"), 300);
        assert_eq!(reader.read_varint().expect("varint"), u64::MAX);
        assert!(reader.is_empty());
    }

    #[test]
    fn a_truncated_varint_is_rejected() {
        let mut reader = ProtoReader::new(&[0x80]);
        assert_eq!(reader.read_varint(), Err(ProtoError::Truncated));
    }

    #[test]
    fn a_varint_longer_than_ten_bytes_is_rejected() {
        // Eleven continuation bytes: a peer trying to make us spin.
        let bytes = [0xffu8; 11];
        let mut reader = ProtoReader::new(&bytes);
        assert_eq!(reader.read_varint(), Err(ProtoError::VarintTooLong));
    }

    #[test]
    fn a_string_field_round_trips() {
        let mut writer = ProtoWriter::new();
        writer
            .write_string_field(4, "urn:x-cast:com.google.cast.media")
            .expect("string");
        let bytes = writer.finish();

        let mut reader = ProtoReader::new(&bytes);
        let (field, wire) = reader.read_tag().expect("tag");
        assert_eq!(field, 4);
        assert_eq!(wire, WIRE_LENGTH_DELIMITED);
        assert_eq!(
            reader.read_string().expect("string"),
            "urn:x-cast:com.google.cast.media"
        );
    }

    #[test]
    fn a_field_longer_than_the_limit_is_rejected_before_it_is_written() {
        // A hostile peer announces a 16 MiB payload. The writer must refuse
        // it instead of announcing the length on the wire.
        let payload = vec![0u8; MAX_FIELD_BYTES + 1];
        let mut writer = ProtoWriter::new();
        assert_eq!(
            writer.write_bytes_field(7, &payload),
            Err(ProtoError::FieldTooLarge(MAX_FIELD_BYTES + 1))
        );
        assert!(writer.as_slice().is_empty());
    }

    #[test]
    fn a_declared_length_beyond_the_buffer_is_rejected() {
        // 0x7f says "127 bytes follow", the buffer holds two.
        let bytes = [0x7fu8, 0x01, 0x02];
        let mut reader = ProtoReader::new(&bytes);
        assert_eq!(reader.read_bytes(), Err(ProtoError::Truncated));
    }

    #[test]
    fn a_declared_length_beyond_the_limit_is_rejected() {
        let mut writer = ProtoWriter::new();
        writer.write_varint((MAX_FIELD_BYTES + 1) as u64);
        let bytes = writer.finish();
        let mut reader = ProtoReader::new(&bytes);
        assert_eq!(
            reader.read_bytes(),
            Err(ProtoError::FieldTooLarge(MAX_FIELD_BYTES + 1))
        );
    }

    #[test]
    fn an_invalid_utf8_string_is_rejected() {
        let mut writer = ProtoWriter::new();
        writer.write_bytes_field(6, &[0xff, 0xfe]).expect("bytes");
        let bytes = writer.finish();
        let mut reader = ProtoReader::new(&bytes);
        let (_, wire) = reader.read_tag().expect("tag");
        assert_eq!(wire, WIRE_LENGTH_DELIMITED);
        assert_eq!(reader.read_string(), Err(ProtoError::InvalidUtf8));
    }

    #[test]
    fn an_unknown_field_is_skipped_not_repaired() {
        // Field 9, wire type 2, three bytes of payload. The reader must step
        // over it and land on the next field.
        let mut writer = ProtoWriter::new();
        writer.write_bytes_field(9, b"abc").expect("bytes");
        writer.write_varint_field(3, 42);
        let bytes = writer.finish();

        let mut reader = ProtoReader::new(&bytes);
        let (field, wire) = reader.read_tag().expect("tag");
        assert_eq!(field, 9);
        reader.skip_field(wire).expect("skip");
        let (field, wire) = reader.read_tag().expect("tag");
        assert_eq!(field, 3);
        assert_eq!(wire, WIRE_VARINT);
        assert_eq!(reader.read_varint().expect("varint"), 42);
    }

    #[test]
    fn an_unsupported_wire_type_is_rejected() {
        // Wire type 3 (start group) is not used by the CastV2 envelope.
        let bytes = [0x0bu8];
        let mut reader = ProtoReader::new(&bytes);
        let (_, wire) = reader.read_tag().expect("tag");
        assert_eq!(
            reader.skip_field(wire),
            Err(ProtoError::UnsupportedWireType(3))
        );
    }

    #[test]
    fn the_field_number_zero_is_reserved() {
        let bytes = [0x00u8];
        let mut reader = ProtoReader::new(&bytes);
        assert_eq!(reader.read_tag(), Err(ProtoError::InvalidFieldNumber));
    }
}
