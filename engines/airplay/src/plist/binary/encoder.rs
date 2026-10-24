//! The encoder of the binary property list.

use crate::error::AirPlayError;
use crate::plist::value::PlistValue;

use super::{int_size, write_int, MAGIC, MAX_OBJECTS, TRAILER_SIZE};

pub(super) enum RawObject {
    Boolean(bool),
    Integer(Vec<u8>),
    Real(f64),
    Data(Vec<u8>),
    Text(String),
    Array(Vec<usize>),
    Dictionary(Vec<(usize, usize)>),
}

fn build(value: &PlistValue, objects: &mut Vec<RawObject>) -> usize {
    match value {
        PlistValue::Boolean(inner) => {
            objects.push(RawObject::Boolean(*inner));
            objects.len() - 1
        }
        PlistValue::Integer(inner) => {
            objects.push(RawObject::Integer(integer_bytes(*inner)));
            objects.len() - 1
        }
        PlistValue::Real(inner) => {
            objects.push(RawObject::Real(*inner));
            objects.len() - 1
        }
        PlistValue::Data(inner) => {
            objects.push(RawObject::Data(inner.clone()));
            objects.len() - 1
        }
        PlistValue::String(inner) => {
            objects.push(RawObject::Text(inner.clone()));
            objects.len() - 1
        }
        PlistValue::Array(items) => {
            let refs = items.iter().map(|item| build(item, objects)).collect();
            objects.push(RawObject::Array(refs));
            objects.len() - 1
        }
        PlistValue::Dictionary(entries) => {
            let refs = entries
                .iter()
                .map(|(key, item)| {
                    let key_index = {
                        objects.push(RawObject::Text(key.clone()));
                        objects.len() - 1
                    };
                    (key_index, build(item, objects))
                })
                .collect();
            objects.push(RawObject::Dictionary(refs));
            objects.len() - 1
        }
    }
}

fn integer_bytes(value: i64) -> Vec<u8> {
    if let Ok(small) = i8::try_from(value) {
        return vec![small as u8];
    }
    if let Ok(small) = i16::try_from(value) {
        return small.to_be_bytes().to_vec();
    }
    if let Ok(small) = i32::try_from(value) {
        return small.to_be_bytes().to_vec();
    }
    value.to_be_bytes().to_vec()
}

fn write_object(out: &mut Vec<u8>, object: &RawObject, ref_size: usize) {
    match object {
        RawObject::Boolean(value) => out.push(if *value { 0x09 } else { 0x08 }),
        RawObject::Integer(bytes) => {
            let exponent = match bytes.len() {
                1 => 0u8,
                2 => 1,
                4 => 2,
                _ => 3,
            };
            out.push(0x10 | exponent);
            out.extend_from_slice(bytes);
        }
        RawObject::Real(value) => {
            out.push(0x23);
            out.extend_from_slice(&value.to_be_bytes());
        }
        RawObject::Data(bytes) => {
            write_length_prefixed(out, 0x40, bytes.len());
            out.extend_from_slice(bytes);
        }
        RawObject::Text(text) => {
            if text.is_ascii() {
                write_length_prefixed(out, 0x50, text.len());
                out.extend_from_slice(text.as_bytes());
            } else {
                let units: Vec<u16> = text.encode_utf16().collect();
                write_length_prefixed(out, 0x60, units.len());
                for unit in units {
                    out.extend_from_slice(&unit.to_be_bytes());
                }
            }
        }
        RawObject::Array(refs) => {
            write_length_prefixed(out, 0xA0, refs.len());
            for reference in refs {
                write_int(out, *reference as u64, ref_size);
            }
        }
        RawObject::Dictionary(entries) => {
            write_length_prefixed(out, 0xD0, entries.len());
            for (key, value) in entries {
                write_int(out, *key as u64, ref_size);
                write_int(out, *value as u64, ref_size);
            }
        }
    }
}

fn write_length_prefixed(out: &mut Vec<u8>, marker: u8, length: usize) {
    if length < 0x0F {
        out.push(marker | length as u8);
        return;
    }
    out.push(marker | 0x0F);
    let size = int_size(length as u64);
    out.push(0x10 | (size as u8).trailing_zeros() as u8);
    write_int(out, length as u64, size);
}

///
/// # Errors
/// Returns [`AirPlayError::LimitExceeded`] when the document would exceed
/// `max_bytes`.
pub fn encode(value: &PlistValue) -> Result<Vec<u8>, AirPlayError> {
    encode_with_limit(value, usize::MAX)
}

/// Encodes a value as a binary property list, with a size limit.
///
/// # Errors
/// Returns [`AirPlayError::LimitExceeded`] when the document would exceed
/// `max_bytes`.
pub fn encode_with_limit(
    value: &PlistValue,
    max_bytes: usize,
) -> Result<Vec<u8>, AirPlayError> {
    let mut objects: Vec<RawObject> = Vec::new();
    let root = build(value, &mut objects);
    if objects.len() as u64 > MAX_OBJECTS {
        return Err(AirPlayError::limit_exceeded("too many plist objects"));
    }

    let mut body: Vec<u8> = Vec::new();
    body.extend_from_slice(MAGIC);
    let ref_size = int_size(objects.len() as u64 - 1);
    let mut offsets: Vec<u64> = Vec::with_capacity(objects.len());
    for object in &objects {
        offsets.push(body.len() as u64);
        write_object(&mut body, object, ref_size);
    }

    let offset_size = int_size(offsets.iter().copied().max().unwrap_or(0));
    let offset_table_offset = body.len() as u64;
    for offset in &offsets {
        write_int(&mut body, *offset, offset_size);
    }

    let mut trailer = vec![0u8; TRAILER_SIZE];
    trailer[6] = offset_size as u8;
    trailer[7] = ref_size as u8;
    trailer[8..16].copy_from_slice(&(objects.len() as u64).to_be_bytes());
    trailer[16..24].copy_from_slice(&(root as u64).to_be_bytes());
    trailer[24..32].copy_from_slice(&offset_table_offset.to_be_bytes());
    body.extend_from_slice(&trailer);

    if body.len() > max_bytes {
        return Err(AirPlayError::limit_exceeded(format!(
            "plist of {} bytes exceeds the {max_bytes} byte limit",
            body.len()
        )));
    }
    Ok(body)
}
