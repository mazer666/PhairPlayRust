// WHAT: the encoder for the binary property-list format (`bplist00`).
// WHY: the AirPlay 2 handshake and the Google Cast protocol both answer
//      with binary property lists, so the receiver has to be able to
//      produce them. The encoder writes the same layout that Apple's
//      `PropertyListSerialization` produces: a header, the objects, an
//      offset table and a 32-byte trailer.
// EXAMPLE: `encode(&value, &limits)` returns a byte vector that starts
//          with the eight bytes `bplist00`.

use crate::binary::MAGIC;
use crate::error::PlistError;
use crate::limits::PlistLimits;
use crate::value::Value;

/// The size of the trailer at the end of a binary property list.
const TRAILER_LEN: usize = 32;

/// One object of the object table, in the order the offsets describe.
#[derive(Debug, Clone)]
enum Object {
    /// A boolean.
    Boolean(bool),
    /// A signed integer.
    Integer(i64),
    /// A floating point number.
    Real(f64),
    /// A date, in seconds since the Apple epoch.
    Date(f64),
    /// A data blob.
    Data(Vec<u8>),
    /// A string; the encoder decides between ASCII and UTF-16.
    Text(String),
    /// An array, with the indices of its elements.
    Array(Vec<usize>),
    /// A dictionary, with the indices of its keys and values.
    Dictionary(Vec<(usize, usize)>),
}

/// Encodes a value as a binary property list.
///
/// # Errors
///
/// Returns an error when the value nests deeper than the configured
/// limit allows.
pub(crate) fn encode(value: &Value, limits: &PlistLimits) -> Result<Vec<u8>, PlistError> {
    let mut objects: Vec<Object> = Vec::new();
    let root = collect(value, &mut objects, 0, limits)?;

    let num_objects = objects.len();
    if num_objects == 0 {
        return Err(PlistError::Encode("the value tree is empty".into()));
    }
    let ref_size = width_for((num_objects - 1) as u64);

    let mut bodies: Vec<Vec<u8>> = Vec::with_capacity(num_objects);
    for object in &objects {
        bodies.push(encode_object(object, ref_size));
    }

    let mut offsets: Vec<u64> = Vec::with_capacity(num_objects);
    let mut cursor = MAGIC.len() as u64;
    for body in &bodies {
        offsets.push(cursor);
        cursor += body.len() as u64;
    }
    let table_offset = cursor;
    let offset_size = width_for(*offsets.last().unwrap_or(&0));

    let mut out: Vec<u8> =
        Vec::with_capacity(table_offset as usize + num_objects * offset_size + TRAILER_LEN);
    out.extend_from_slice(MAGIC);
    for body in &bodies {
        out.extend_from_slice(body);
    }
    for offset in &offsets {
        out.extend_from_slice(&offset.to_be_bytes()[8 - offset_size..]);
    }

    let mut trailer = [0_u8; TRAILER_LEN];
    trailer[6] = offset_size as u8;
    trailer[7] = ref_size as u8;
    trailer[8..16].copy_from_slice(&(num_objects as u64).to_be_bytes());
    trailer[16..24].copy_from_slice(&(root as u64).to_be_bytes());
    trailer[24..32].copy_from_slice(&table_offset.to_be_bytes());
    out.extend_from_slice(&trailer);
    Ok(out)
}

/// Flattens a value into the object table and returns the index of the root.
fn collect(
    value: &Value,
    objects: &mut Vec<Object>,
    depth: usize,
    limits: &PlistLimits,
) -> Result<usize, PlistError> {
    if depth > limits.max_depth {
        return Err(PlistError::DepthLimitExceeded {
            limit: limits.max_depth,
        });
    }
    let index = objects.len();
    match value {
        Value::Boolean(flag) => objects.push(Object::Boolean(*flag)),
        Value::Integer(number) => objects.push(Object::Integer(*number)),
        Value::Real(number) => objects.push(Object::Real(*number)),
        Value::Date(seconds) => objects.push(Object::Date(*seconds)),
        Value::Data(bytes) => objects.push(Object::Data(bytes.clone())),
        Value::String(text) => objects.push(Object::Text(text.clone())),
        Value::Array(items) => {
            objects.push(Object::Array(Vec::new()));
            let mut children = Vec::with_capacity(items.len());
            for item in items {
                children.push(collect(item, objects, depth + 1, limits)?);
            }
            if let Object::Array(slot) = &mut objects[index] {
                *slot = children;
            }
        }
        Value::Dictionary(entries) => {
            objects.push(Object::Dictionary(Vec::new()));
            let mut children = Vec::with_capacity(entries.len());
            for (key, item) in entries {
                let key_index = objects.len();
                objects.push(Object::Text(key.clone()));
                let value_index = collect(item, objects, depth + 1, limits)?;
                children.push((key_index, value_index));
            }
            if let Object::Dictionary(slot) = &mut objects[index] {
                *slot = children;
            }
        }
    }
    Ok(index)
}

/// Encodes one object body.
fn encode_object(object: &Object, ref_size: usize) -> Vec<u8> {
    let mut out = Vec::new();
    match object {
        Object::Boolean(flag) => out.push(if *flag { 0x09 } else { 0x08 }),
        Object::Integer(number) => encode_integer(&mut out, *number),
        Object::Real(number) => {
            out.push(0x23);
            out.extend_from_slice(&number.to_be_bytes());
        }
        Object::Date(seconds) => {
            out.push(0x33);
            out.extend_from_slice(&seconds.to_be_bytes());
        }
        Object::Data(bytes) => {
            write_count(&mut out, 0x40, bytes.len() as u64);
            out.extend_from_slice(bytes);
        }
        Object::Text(text) => {
            if text.is_ascii() {
                write_count(&mut out, 0x50, text.len() as u64);
                out.extend_from_slice(text.as_bytes());
            } else {
                let units: Vec<u16> = text.encode_utf16().collect();
                write_count(&mut out, 0x60, units.len() as u64);
                for unit in units {
                    out.extend_from_slice(&unit.to_be_bytes());
                }
            }
        }
        Object::Array(children) => {
            write_count(&mut out, 0xa0, children.len() as u64);
            for child in children {
                write_reference(&mut out, *child as u64, ref_size);
            }
        }
        Object::Dictionary(children) => {
            write_count(&mut out, 0xd0, children.len() as u64);
            for (key, _) in children {
                write_reference(&mut out, *key as u64, ref_size);
            }
            for (_, value) in children {
                write_reference(&mut out, *value as u64, ref_size);
            }
        }
    }
    out
}

/// Writes an integer in the smallest width that still holds it.
fn encode_integer(out: &mut Vec<u8>, number: i64) {
    if (0..=0xff).contains(&number) {
        out.push(0x10);
        out.push(number as u8);
    } else if (0..=0xffff).contains(&number) {
        out.push(0x11);
        out.extend_from_slice(&(number as u16).to_be_bytes());
    } else if (0..=0xffff_ffff).contains(&number) {
        out.push(0x12);
        out.extend_from_slice(&(number as u32).to_be_bytes());
    } else {
        out.push(0x13);
        out.extend_from_slice(&number.to_be_bytes());
    }
}

/// Writes a count field, using the inline integer form when needed.
///
/// The low nibble of the marker holds the count literally. Only counts of
/// fifteen or more need the escape, in which case the marker carries
/// `0xF` and an integer object with the real count follows inline.
fn write_count(out: &mut Vec<u8>, base: u8, count: u64) {
    if count < 0x0f {
        out.push(base | count as u8);
        return;
    }
    out.push(base | 0x0f);
    let mut bytes = [0_u8; 8];
    let mut value = count;
    let mut length = 0_usize;
    while value > 0 {
        bytes[7 - length] = (value & 0xff) as u8;
        value >>= 8;
        length += 1;
    }
    let length = length.max(1);
    out.push(0x10 | (length.ilog2() as u8));
    out.extend_from_slice(&bytes[8 - length..]);
}

/// Writes one object reference.
fn write_reference(out: &mut Vec<u8>, reference: u64, ref_size: usize) {
    out.extend_from_slice(&reference.to_be_bytes()[8 - ref_size..]);
}

/// The number of bytes needed to hold `value` as an unsigned integer.
fn width_for(value: u64) -> usize {
    let mut width = 1_usize;
    while width < 8 && (value >> (width * 8)) != 0 {
        width += 1;
    }
    width
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;
    use crate::binary::decode::decode;

    fn limits() -> PlistLimits {
        PlistLimits::default()
    }

    fn round_trip(value: &Value) -> Value {
        let bytes = encode(value, &limits()).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(&bytes[..8], MAGIC);
        decode(&bytes, &limits()).unwrap_or_else(|err| panic!("{err}"))
    }

    #[test]
    fn writes_the_magic_and_the_trailer() {
        let bytes = encode(&Value::Boolean(true), &limits()).unwrap_or_default();
        assert_eq!(&bytes[..8], MAGIC);
        let trailer = &bytes[bytes.len() - TRAILER_LEN..];
        assert_eq!(trailer[6], 1);
        assert_eq!(trailer[7], 1);
        assert_eq!(
            u64::from_be_bytes(trailer[8..16].try_into().unwrap_or([0; 8])),
            1
        );
    }

    #[test]
    fn round_trips_a_boolean() {
        assert_eq!(round_trip(&Value::Boolean(true)), Value::Boolean(true));
        assert_eq!(round_trip(&Value::Boolean(false)), Value::Boolean(false));
    }

    #[test]
    fn round_trips_integers_of_every_width() {
        for number in [
            0_i64,
            1,
            255,
            256,
            65_535,
            65_536,
            4_294_967_295,
            4_294_967_296,
            i64::MAX,
            i64::MIN,
        ] {
            assert_eq!(round_trip(&Value::Integer(number)), Value::Integer(number));
        }
    }

    #[test]
    fn round_trips_a_real() {
        assert_eq!(round_trip(&Value::Real(1.5)), Value::Real(1.5));
    }

    #[test]
    fn round_trips_a_date() {
        assert_eq!(round_trip(&Value::Date(12.5)), Value::Date(12.5));
    }

    #[test]
    fn round_trips_data_of_every_length() {
        for len in [0_usize, 1, 14, 15, 16, 255, 256, 1000] {
            let data: Vec<u8> = (0..len).map(|index| (index % 251) as u8).collect();
            assert_eq!(round_trip(&Value::Data(data.clone())), Value::Data(data));
        }
    }

    #[test]
    fn round_trips_ascii_and_unicode_strings() {
        assert_eq!(
            round_trip(&Value::String("hello".into())),
            Value::String("hello".into())
        );
        assert_eq!(
            round_trip(&Value::String("grüße 🎵".into())),
            Value::String("grüße 🎵".into())
        );
    }

    #[test]
    fn round_trips_containers() {
        let value = Value::dictionary([
            ("name", Value::String("Room".into())),
            (
                "levels",
                Value::Array(vec![Value::Integer(1), Value::Integer(2)]),
            ),
            (
                "nested",
                Value::dictionary([("flag", Value::Boolean(true))]),
            ),
        ]);
        assert_eq!(round_trip(&value), value);
    }

    #[test]
    fn round_trips_an_array_of_every_length() {
        for len in [0_usize, 1, 2, 3, 7, 15, 16, 17] {
            let items: Vec<Value> = (0..len).map(|index| Value::Integer(index as i64)).collect();
            let value = Value::Array(items);
            assert_eq!(round_trip(&value), value);
        }
    }

    #[test]
    fn rejects_a_tree_that_is_too_deep() {
        let mut limits = limits();
        limits.max_depth = 4;
        let mut value = Value::Integer(1);
        for _ in 0..8 {
            value = Value::Array(vec![value]);
        }
        assert!(matches!(
            encode(&value, &limits),
            Err(PlistError::DepthLimitExceeded { .. })
        ));
    }

    #[test]
    fn width_grows_with_the_value() {
        assert_eq!(width_for(0), 1);
        assert_eq!(width_for(0xff), 1);
        assert_eq!(width_for(0x100), 2);
        assert_eq!(width_for(0x1_0000), 3);
        assert_eq!(width_for(u64::MAX), 8);
    }
}
