//! The decoder of the binary property list.

use crate::error::AirPlayError;
use crate::plist::value::PlistValue;

use super::{MAGIC, MAX_DEPTH, MAX_OBJECTS, TRAILER_SIZE, read_int};

///
/// # Errors
/// Returns [`AirPlayError::Parse`] when the document is malformed and
/// [`AirPlayError::LimitExceeded`] when it declares more objects than
/// `max_bytes` worth of memory could hold.
pub fn decode(bytes: &[u8], max_bytes: usize) -> Result<PlistValue, AirPlayError> {
    if bytes.len() < MAGIC.len() + TRAILER_SIZE {
        return Err(AirPlayError::parse("plist is too short"));
    }
    if !bytes.starts_with(MAGIC) {
        return Err(AirPlayError::parse("plist does not start with bplist00"));
    }
    let trailer = &bytes[bytes.len() - TRAILER_SIZE..];
    let offset_size = trailer[6] as usize;
    let ref_size = trailer[7] as usize;
    let object_count = u64::from_be_bytes(trailer[8..16].try_into().unwrap_or([0; 8]));
    let top_object = u64::from_be_bytes(trailer[16..24].try_into().unwrap_or([0; 8]));
    let offset_table = u64::from_be_bytes(trailer[24..32].try_into().unwrap_or([0; 8]));

    if object_count == 0 || object_count > MAX_OBJECTS {
        return Err(AirPlayError::limit_exceeded(format!(
            "plist declares {object_count} objects"
        )));
    }
    if !matches!(offset_size, 1 | 2 | 4 | 8) || !matches!(ref_size, 1 | 2 | 4 | 8) {
        return Err(AirPlayError::parse(
            "plist declares an invalid integer size",
        ));
    }
    if top_object >= object_count {
        return Err(AirPlayError::parse("plist root object is out of range"));
    }
    let table_bytes = object_count
        .checked_mul(offset_size as u64)
        .ok_or_else(|| AirPlayError::parse("plist offset table overflows"))?;
    let table_end = offset_table
        .checked_add(table_bytes)
        .ok_or_else(|| AirPlayError::parse("plist offset table overflows"))?;
    if table_end > bytes.len() as u64 || offset_table < MAGIC.len() as u64 {
        return Err(AirPlayError::parse("plist offset table is out of range"));
    }
    if object_count as usize * 16 > max_bytes {
        return Err(AirPlayError::limit_exceeded("plist has too many objects"));
    }

    let mut offsets: Vec<u64> = Vec::with_capacity(object_count as usize);
    for index in 0..object_count as usize {
        let start = offset_table as usize + index * offset_size;
        let offset = read_int(&bytes[start..start + offset_size]);
        if offset as usize >= bytes.len() {
            return Err(AirPlayError::parse("plist object offset is out of range"));
        }
        offsets.push(offset);
    }

    let mut visiting: Vec<u64> = Vec::new();
    decode_object(
        bytes,
        &offsets,
        ref_size,
        top_object,
        0,
        &mut visiting,
        max_bytes,
    )
}

fn decode_object(
    bytes: &[u8],
    offsets: &[u64],
    ref_size: usize,
    index: u64,
    depth: u32,
    visiting: &mut Vec<u64>,
    max_bytes: usize,
) -> Result<PlistValue, AirPlayError> {
    if depth > MAX_DEPTH {
        return Err(AirPlayError::limit_exceeded("plist nests too deeply"));
    }
    if visiting.contains(&index) {
        return Err(AirPlayError::parse("plist contains a reference cycle"));
    }
    let offset = *offsets
        .get(index as usize)
        .ok_or_else(|| AirPlayError::parse("plist object index is out of range"))?
        as usize;
    let marker = *bytes
        .get(offset)
        .ok_or_else(|| AirPlayError::parse("plist object is out of range"))?;
    let kind = marker >> 4;
    let low = (marker & 0x0F) as usize;

    match kind {
        0x0 => match low {
            0 => Ok(PlistValue::String(String::new())),
            8 => Ok(PlistValue::Boolean(false)),
            9 => Ok(PlistValue::Boolean(true)),
            _ => Err(AirPlayError::parse("unsupported plist singleton")),
        },
        0x1 => {
            let size = 1usize << low.min(3);
            let start = offset + 1;
            let end = start + size;
            if end > bytes.len() {
                return Err(AirPlayError::parse("plist integer is out of range"));
            }
            let mut buffer = [0u8; 8];
            buffer[8 - size..].copy_from_slice(&bytes[start..end]);
            let raw = i64::from_be_bytes(buffer);
            Ok(PlistValue::Integer(raw))
        }
        0x2 => {
            let size = if low == 2 { 4 } else { 8 };
            let start = offset + 1;
            let end = start + size;
            if end > bytes.len() {
                return Err(AirPlayError::parse("plist real is out of range"));
            }
            if size == 4 {
                let raw = f32::from_be_bytes(bytes[start..end].try_into().unwrap_or([0; 4]));
                Ok(PlistValue::Real(f64::from(raw)))
            } else {
                let raw = f64::from_be_bytes(bytes[start..end].try_into().unwrap_or([0; 8]));
                Ok(PlistValue::Real(raw))
            }
        }
        0x3 => Err(AirPlayError::parse("plist dates are not supported")),
        0x4 | 0x5 | 0x6 | 0x8 => {
            let (length, data_start) = read_length(bytes, offset, low)?;
            // A UTF-16 string declares its length in code units, not in bytes.
            let byte_length = if kind == 0x6 { length * 2 } else { length };
            let end = data_start + byte_length;
            if end > bytes.len() {
                return Err(AirPlayError::parse("plist payload is out of range"));
            }
            if byte_length > max_bytes {
                return Err(AirPlayError::limit_exceeded("plist payload is too large"));
            }
            let payload = &bytes[data_start..end];
            match kind {
                0x4 => Ok(PlistValue::Data(payload.to_vec())),
                0x5 => Ok(PlistValue::String(
                    String::from_utf8_lossy(payload).into_owned(),
                )),
                0x6 => {
                    let units: Vec<u16> = payload
                        .chunks_exact(2)
                        .map(|pair| u16::from_be_bytes([pair[0], pair[1]]))
                        .collect();
                    Ok(PlistValue::String(String::from_utf16_lossy(&units)))
                }
                _ => Err(AirPlayError::parse("plist uids are not supported")),
            }
        }
        0x7 => {
            let (length, data_start) = read_length(bytes, offset, low)?;
            let end = data_start + length;
            if end > bytes.len() {
                return Err(AirPlayError::parse("plist payload is out of range"));
            }
            let text = std::str::from_utf8(&bytes[data_start..end])
                .map_err(|_| AirPlayError::parse("plist string is not valid UTF-8"))?;
            Ok(PlistValue::String(text.to_string()))
        }
        0xA | 0xC => {
            let (count, data_start) = read_length(bytes, offset, low)?;
            let end = data_start + count * ref_size;
            if end > bytes.len() {
                return Err(AirPlayError::parse("plist array is out of range"));
            }
            visiting.push(index);
            let mut items = Vec::with_capacity(count.min(1024));
            for item in 0..count {
                let start = data_start + item * ref_size;
                let reference = read_int(&bytes[start..start + ref_size]);
                items.push(decode_object(
                    bytes,
                    offsets,
                    ref_size,
                    reference,
                    depth + 1,
                    visiting,
                    max_bytes,
                )?);
            }
            visiting.pop();
            Ok(PlistValue::Array(items))
        }
        0xD => {
            let (count, data_start) = read_length(bytes, offset, low)?;
            let end = data_start + count * ref_size * 2;
            if end > bytes.len() {
                return Err(AirPlayError::parse("plist dictionary is out of range"));
            }
            visiting.push(index);
            let mut entries = Vec::with_capacity(count.min(1024));
            for item in 0..count {
                let entry_start = data_start + item * ref_size * 2;
                let key_reference = read_int(&bytes[entry_start..entry_start + ref_size]);
                let value_reference =
                    read_int(&bytes[entry_start + ref_size..entry_start + ref_size * 2]);
                let key = decode_object(
                    bytes,
                    offsets,
                    ref_size,
                    key_reference,
                    depth + 1,
                    visiting,
                    max_bytes,
                )?;
                let value = decode_object(
                    bytes,
                    offsets,
                    ref_size,
                    value_reference,
                    depth + 1,
                    visiting,
                    max_bytes,
                )?;
                let key = match key {
                    PlistValue::String(key) => key,
                    other => other.to_debug_string(),
                };
                entries.push((key, value));
            }
            visiting.pop();
            Ok(PlistValue::Dictionary(entries))
        }
        _ => Err(AirPlayError::parse("unsupported plist object type")),
    }
}

fn read_length(bytes: &[u8], offset: usize, low: usize) -> Result<(usize, usize), AirPlayError> {
    if low != 0x0F {
        return Ok((low, offset + 1));
    }
    let marker = *bytes
        .get(offset + 1)
        .ok_or_else(|| AirPlayError::parse("plist length marker is out of range"))?;
    if marker >> 4 != 0x1 {
        return Err(AirPlayError::parse("plist length is not an integer"));
    }
    let size = 1usize << (marker & 0x0F).min(3);
    let start = offset + 2;
    let end = start + size;
    if end > bytes.len() {
        return Err(AirPlayError::parse("plist length is out of range"));
    }
    let length = read_int(&bytes[start..end]);
    let length =
        usize::try_from(length).map_err(|_| AirPlayError::parse("plist length overflows"))?;
    Ok((length, end))
}
