#![allow(clippy::expect_used, clippy::unwrap_used)]

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
    assert_eq!(value.get("a").and_then(PlistValue::as_integer), Some(1));
}

#[test]
fn a_doctype_with_an_internal_subset_is_refused() {
    // The attack: the internal subset is where an entity is declared, and
    // the subset may hold a `>` inside a quoted value — so a decoder that
    // cuts the declaration at the first `>` reads a different document than
    // the sender sent. The subset is refused instead of being skipped.
    let text = "<?xml version=\"1.0\"?>\n\
<!DOCTYPE plist [ <!ENTITY xxe SYSTEM \"file:///etc/passwd\"> ]>\n\
<plist version=\"1.0\"><dict><key>a</key><string>&xxe;</string></dict></plist>";
    let error = decode(text, 64 * 1024).expect_err("the subset must be refused");
    assert!(
        matches!(error, AirPlayError::Parse { .. }),
        "the subset must be refused as malformed: {error}"
    );
}

#[test]
fn a_doctype_with_a_subset_that_holds_a_greater_than_sign_is_refused() {
    // The parser differential: the `>` of the entity value is not the end
    // of the declaration. A document that hides a `>` inside a quoted value
    // must not be able to smuggle markup past the scanner.
    let text = "<?xml version=\"1.0\"?>\n\
<!DOCTYPE plist [ <!ENTITY x \"a>b\"> ]>\n\
<plist version=\"1.0\"><dict><key>a</key><integer>1</integer></dict></plist>";
    let error = decode(text, 64 * 1024).expect_err("the subset must be refused");
    assert!(
        matches!(error, AirPlayError::Parse { .. }),
        "the subset must be refused as malformed: {error}"
    );
}

#[test]
fn an_unclosed_declaration_is_refused() {
    let text = "<?xml version=\"1.0\"?>\n<!DOCTYPE plist PUBLIC \"x\"\n\
<plist version=\"1.0\"><dict><key>a</key><integer>1</integer></dict></plist>";
    let error = decode(text, 64 * 1024).expect_err("the declaration must be refused");
    assert!(
        matches!(error, AirPlayError::Parse { .. }),
        "the declaration must be refused as malformed: {error}"
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
