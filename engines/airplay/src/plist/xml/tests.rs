
use super::*;

use crate::error::AirPlayError;

fn round_trip(value: &PlistValue) {
    let encoded = encode(value);
    let text = String::from_utf8(encoded).expect("utf-8");
    let decoded = decode(&text, 64 * 1024).expect("decodable");
    assert_eq!(&decoded, value);
}

#[test]
fn a_dictionary_round_trips() {
    round_trip(&PlistValue::dictionary(vec![
        ("name".into(), PlistValue::String("PhairPlay".into())),
        ("port".into(), PlistValue::Integer(7000)),
        ("ratio".into(), PlistValue::Real(0.5)),
        ("ready".into(), PlistValue::Boolean(false)),
    ]));
}

#[test]
fn data_round_trips() {
    round_trip(&PlistValue::dictionary(vec![(
        "pk".into(),
        PlistValue::Data(vec![0x00, 0x01, 0xFE, 0xFF]),
    )]));
}

#[test]
fn an_array_round_trips() {
    round_trip(&PlistValue::array(vec![
        PlistValue::Integer(1),
        PlistValue::String("two".into()),
    ]));
}

#[test]
fn entities_are_resolved() {
    let document = PlistValue::String("a & b < c".into());
    round_trip(&document);
}

#[test]
fn a_document_with_a_doctype_is_decoded() {
    let text = "<?xml version=\"1.0\"?>\n<!DOCTYPE plist PUBLIC \"x\" \"y\">\n\
<plist version=\"1.0\"><dict><key>a</key><integer>1</integer></dict></plist>";
    let value = decode(text, 64 * 1024).expect("decodable");
    assert_eq!(
        value.get("a").and_then(PlistValue::as_integer),
        Some(1)
    );
}

#[test]
fn a_malformed_document_is_rejected() {
    let error = decode("<plist><dict><key>a</key>", 64 * 1024).expect_err("must fail");
    assert!(matches!(error, AirPlayError::Parse { .. }));
}

#[test]
fn an_unknown_element_is_rejected() {
    let text = "<plist><dict><key>a</key><banana>1</banana></dict></plist>";
    let error = decode(text, 64 * 1024).expect_err("must fail");
    assert!(matches!(error, AirPlayError::Parse { .. }));
}

#[test]
fn an_oversized_document_is_rejected() {
    let text = "<plist><dict><key>a</key><string>xxxx</string></dict></plist>";
    let error = decode(text, 16).expect_err("must fail");
    assert!(matches!(error, AirPlayError::LimitExceeded { .. }));
}

#[test]
fn a_deeply_nested_document_is_rejected() {
    let mut text = String::new();
    for _ in 0..64 {
        text.push_str("<array>");
    }
    for _ in 0..64 {
        text.push_str("</array>");
    }
    let error = decode(&text, 64 * 1024).expect_err("must fail");
    assert!(matches!(error, AirPlayError::LimitExceeded { .. }));
}

#[test]
fn a_dictionary_without_a_key_is_rejected() {
    let text = "<plist><dict><string>value</string></dict></plist>";
    let error = decode(text, 64 * 1024).expect_err("must fail");
    assert!(matches!(error, AirPlayError::Parse { .. }));
}
