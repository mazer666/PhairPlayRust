#![allow(clippy::expect_used, clippy::unwrap_used)]

use super::*;

use crate::error::AirPlayError;

fn round_trip(value: &PlistValue) {
    let encoded = encode(value).expect("encodable");
    let decoded = decode(&encoded, 1024 * 1024).expect("decodable");
    assert_eq!(&decoded, value);
}

#[test]
fn a_dictionary_round_trips() {
    round_trip(&PlistValue::dictionary(vec![
        ("name".into(), PlistValue::String("PhairPlay".into())),
        ("port".into(), PlistValue::Integer(7000)),
        ("ratio".into(), PlistValue::Real(0.5)),
        ("ready".into(), PlistValue::Boolean(true)),
    ]));
}

#[test]
fn an_array_round_trips() {
    round_trip(&PlistValue::array(vec![
        PlistValue::Integer(1),
        PlistValue::Integer(2),
        PlistValue::Integer(3),
    ]));
}

#[test]
fn data_round_trips() {
    round_trip(&PlistValue::dictionary(vec![(
        "pk".into(),
        PlistValue::Data(vec![0x01, 0x02, 0x03, 0xFF]),
    )]));
}

#[test]
fn a_nested_structure_round_trips() {
    round_trip(&PlistValue::dictionary(vec![(
        "streams".into(),
        PlistValue::array(vec![PlistValue::dictionary(vec![(
            "type".into(),
            PlistValue::Integer(110),
        )])]),
    )]));
}

#[test]
fn a_long_string_uses_the_extended_length() {
    let long = "x".repeat(1000);
    round_trip(&PlistValue::String(long));
}

#[test]
fn a_non_ascii_string_round_trips() {
    round_trip(&PlistValue::String("Küche — Wohnzimmer".into()));
}

#[test]
fn a_document_without_the_magic_is_rejected() {
    let error = decode(b"not a plist at all........", 1024).expect_err("must fail");
    assert!(matches!(error, AirPlayError::Parse { .. }));
}

#[test]
fn a_truncated_document_is_rejected() {
    let mut encoded = encode(&PlistValue::Integer(1)).expect("encodable");
    encoded.truncate(12);
    let error = decode(&encoded, 1024).expect_err("must fail");
    assert!(matches!(error, AirPlayError::Parse { .. }));
}

#[test]
fn a_trailer_that_points_outside_the_buffer_is_rejected() {
    let mut encoded = encode(&PlistValue::Integer(1)).expect("encodable");
    let trailer = encoded.len() - TRAILER_SIZE;
    // The offset table offset points behind the end of the document.
    encoded[trailer + 24..trailer + 32].copy_from_slice(&u64::MAX.to_be_bytes());
    let error = decode(&encoded, 1024).expect_err("must fail");
    assert!(matches!(error, AirPlayError::Parse { .. }), "{error}");
}

#[test]
fn a_document_that_declares_an_absurd_object_count_is_rejected() {
    let mut encoded = encode(&PlistValue::Integer(1)).expect("encodable");
    let trailer = encoded.len() - TRAILER_SIZE;
    encoded[trailer + 8..trailer + 16].copy_from_slice(&u64::MAX.to_be_bytes());
    let error = decode(&encoded, 1024).expect_err("must fail");
    assert!(
        matches!(error, AirPlayError::LimitExceeded { .. }),
        "{error}"
    );
}

#[test]
fn a_document_with_a_reference_cycle_is_rejected() {
    let mut encoded = encode(&PlistValue::array(vec![PlistValue::Integer(1)])).expect("encodable");
    // The array holds one element; point it at the array itself.
    let trailer = encoded.len() - TRAILER_SIZE;
    let offset_size = encoded[trailer + 6] as usize;
    let ref_size = encoded[trailer + 7] as usize;
    let offset_table = u64::from_be_bytes(
        encoded[trailer + 24..trailer + 32]
            .try_into()
            .unwrap_or([0; 8]),
    ) as usize;
    let root = u64::from_be_bytes(
        encoded[trailer + 16..trailer + 24]
            .try_into()
            .unwrap_or([0; 8]),
    );
    let array_offset =
        read_int(&encoded[offset_table + offset_size..offset_table + offset_size * 2]);
    let mut buffer = vec![0u8; ref_size];
    let root_bytes = root.to_be_bytes();
    buffer.copy_from_slice(&root_bytes[8 - ref_size..]);
    encoded[array_offset as usize + 1..array_offset as usize + 1 + ref_size]
        .copy_from_slice(&buffer);
    let error = decode(&encoded, 1024).expect_err("must fail");
    assert!(matches!(error, AirPlayError::Parse { .. }), "{error}");
}

#[test]
fn an_oversized_document_is_rejected_by_the_limit() {
    let document = PlistValue::String("x".repeat(4096));
    let error = encode_with_limit(&document, 128).expect_err("must fail");
    assert!(matches!(error, AirPlayError::LimitExceeded { .. }));
}
