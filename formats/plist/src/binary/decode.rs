// WHAT: the decoder for the binary property-list format (`bplist00`).
// WHY: the binary format is the one the AirPlay 2 handshake uses, and it
//      is a classic attack surface: a peer controls the trailer, the
//      offset table, every object marker and every length field. The
//      decoder therefore checks every length against the bytes that are
//      actually present, bounds the object count and the nesting depth,
//      and refuses an object graph that refers back to itself.
// EXAMPLE: a trailer that announces `0xFFFF_FFFF_FFFF_FFFF` objects is
//          rejected with `PlistError::TooManyObjects` before a single
//          byte is allocated for the offset table.

use super::objects::BinaryDecoder;
use crate::binary::MAGIC;
use crate::error::PlistError;
use crate::limits::PlistLimits;
use crate::value::Value;

/// The size of the trailer at the end of a binary property list.
const TRAILER_LEN: usize = 32;

/// The largest value an offset or an object reference may use.
const MAX_WIDTH: u8 = 8;

/// Decodes a binary property list.
///
/// # Errors
///
/// Returns an error when the header is wrong, when the trailer announces
/// impossible values, or when the object graph violates one of the
/// configured limits.
pub(crate) fn decode(input: &[u8], limits: &PlistLimits) -> Result<Value, PlistError> {
    if input.len() < MAGIC.len() + TRAILER_LEN {
        return Err(PlistError::Truncated {
            offset: input.len(),
        });
    }
    if &input[..MAGIC.len()] != MAGIC {
        return Err(PlistError::BadBinaryMagic);
    }

    let trailer = &input[input.len() - TRAILER_LEN..];
    let offset_size = trailer[6];
    let ref_size = trailer[7];
    if !(1..=MAX_WIDTH).contains(&offset_size) {
        return Err(PlistError::BadWidth {
            what: "offset",
            value: offset_size,
        });
    }
    if !(1..=MAX_WIDTH).contains(&ref_size) {
        return Err(PlistError::BadWidth {
            what: "object reference",
            value: ref_size,
        });
    }

    let num_objects = read_be_u64(&trailer[8..16]);
    let top_object = read_be_u64(&trailer[16..24]);
    let table_offset = read_be_u64(&trailer[24..32]);

    if num_objects == 0 {
        return Err(PlistError::BadTrailer {
            reason: "the object table is empty",
        });
    }
    if num_objects > limits.max_objects as u64 {
        return Err(PlistError::TooManyObjects {
            count: num_objects,
            limit: limits.max_objects,
        });
    }
    if top_object >= num_objects {
        return Err(PlistError::ObjectIndexOutOfRange {
            index: top_object,
            count: num_objects as usize,
        });
    }

    let table_offset = usize::try_from(table_offset).map_err(|_| PlistError::BadTrailer {
        reason: "the offset table does not fit into the address space",
    })?;
    let table_bytes = (num_objects as usize)
        .checked_mul(offset_size as usize)
        .ok_or(PlistError::BadTrailer {
            reason: "the offset table size overflows",
        })?;
    let table_end = table_offset
        .checked_add(table_bytes)
        .ok_or(PlistError::BadTrailer {
            reason: "the offset table size overflows",
        })?;
    let body_end = input.len() - TRAILER_LEN;
    if table_offset < MAGIC.len() || table_end > body_end {
        return Err(PlistError::BadTrailer {
            reason: "the offset table lies outside the file",
        });
    }

    let mut offsets = Vec::with_capacity(num_objects as usize);
    for index in 0..num_objects as usize {
        let start = table_offset + index * offset_size as usize;
        let raw = &input[start..start + offset_size as usize];
        let offset = read_be_u64(raw);
        let offset = usize::try_from(offset).map_err(|_| PlistError::BadTrailer {
            reason: "an object offset does not fit into the address space",
        })?;
        if offset < MAGIC.len() || offset >= table_offset {
            return Err(PlistError::BadTrailer {
                reason: "an object offset points outside the object region",
            });
        }
        offsets.push(offset);
    }

    let mut decoder = BinaryDecoder {
        input,
        offsets,
        ref_size: ref_size as usize,
        limits,
        region_end: table_offset,
        cache: vec![None; num_objects as usize],
        in_progress: vec![false; num_objects as usize],
        nodes_used: 0,
    };
    decoder.decode_object(top_object, 0)
}

/// Reads a big endian `u64` from a slice.
fn read_be_u64(raw: &[u8]) -> u64 {
    let mut value = 0_u64;
    for byte in raw {
        value = (value << 8) | *byte as u64;
    }
    value
}

#[cfg(test)]
#[path = "decode_tests.rs"]
mod tests;
