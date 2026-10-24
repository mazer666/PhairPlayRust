//! The hardening of the XML property-list decoder of the AirPlay engine.
//!
//! WHY: the XML property list is one of the inputs a hostile device on the
//! local network can push into the receiver. The decoder of the AirPlay
//! engine is a *hand-written* scanner — it does not use the decoder of the
//! `formats/plist` crate — and a hand-written scanner has to prove its own
//! guards. The binary decoder of the same crate has a test for a reference
//! cycle, for an absurd object count and for a trailer that points outside
//! the buffer; the XML decoder had **no test that an external entity is
//! never resolved**.
//!
//! HOW: every test here feeds the decoder the hostile document itself — a
//! `DOCTYPE` with an internal subset, an external entity, a document that
//! is too large, a document that nests too deeply — and asserts the typed
//! outcome. Nothing here checks that "nothing crashed": every test names
//! the error it expects.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use phairplay_airplay::error::AirPlayError;
use phairplay_airplay::plist::PlistValue;
use phairplay_airplay::plist::xml::decode;

/// The limit that the tests use for a document of a normal size.
const NORMAL_LIMIT: usize = 64 * 1024;

/// The XML declaration and the wrapper of a property list.
fn document(body: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<plist version=\"1.0\">\n{body}\n</plist>\n"
    )
}

#[test]
fn an_external_entity_is_never_resolved() {
    // The XXE attack, executed on the decoder of the AirPlay engine.
    //
    // The document declares an entity that points at a file of the host and
    // then uses it. A decoder that resolves the entity reads the file; a
    // decoder that refuses the document says so. What must not happen is
    // that the content of the file ends up in the value.
    let text = "<?xml version=\"1.0\"?>\n<!DOCTYPE plist [ <!ENTITY xxe SYSTEM \"file:///etc/passwd\"> ]>\n\
         <plist version=\"1.0\"><dict><key>a</key><string>&xxe;</string></dict></plist>";
    match decode(text, NORMAL_LIMIT) {
        Err(error) => {
            // The document is refused. The reason has to name the parser.
            assert!(
                matches!(error, AirPlayError::Parse { .. }),
                "the document must be refused as malformed: {error}"
            );
        }
        Ok(value) => {
            // The document was accepted: the entity reference must then be
            // *literal*. A resolved entity would carry the content of the
            // file of the host.
            let text = value
                .get("a")
                .and_then(PlistValue::as_str)
                .unwrap_or_default()
                .to_string();
            assert!(
                !text.contains("root:"),
                "the content of a file of the host must never appear: {text}"
            );
            assert!(
                text.contains("&xxe;"),
                "an entity that was not declared must stay literal: {text}"
            );
        }
    }
}

#[test]
fn an_entity_that_points_at_a_network_service_is_never_resolved() {
    // The same attack, but against a service of the local network instead
    // of a file of the host.
    let text = "<?xml version=\"1.0\"?>\n<!DOCTYPE plist [ <!ENTITY xxe SYSTEM \"http://169.254.169.254/latest/meta-data/\"> ]>\n\
         <plist version=\"1.0\"><dict><key>a</key><string>&xxe;</string></dict></plist>";
    match decode(text, NORMAL_LIMIT) {
        Err(error) => assert!(
            matches!(error, AirPlayError::Parse { .. }),
            "the document must be refused as malformed: {error}"
        ),
        Ok(value) => {
            let text = value
                .get("a")
                .and_then(PlistValue::as_str)
                .unwrap_or_default()
                .to_string();
            assert!(
                !text.contains("meta-data"),
                "the answer of the metadata service must never appear: {text}"
            );
        }
    }
}

#[test]
fn a_doctype_with_an_internal_subset_is_refused() {
    // An internal subset is where an entity is declared. A decoder that
    // accepts a subset has to resolve the declarations in it — and then the
    // guard of the external entities is the only thing that is left.
    let text = "<?xml version=\"1.0\"?>\n<!DOCTYPE plist [ <!ENTITY x \"y\"> ]>\n\
                <plist version=\"1.0\"><dict><key>a</key><integer>1</integer></dict></plist>";
    let error = decode(text, NORMAL_LIMIT).expect_err("the internal subset must be refused");
    assert!(
        matches!(error, AirPlayError::Parse { .. }),
        "the internal subset must be refused as malformed: {error}"
    );
}

#[test]
fn the_plain_doctype_of_apple_is_accepted() {
    // The legacy form that Apple itself writes carries a `DOCTYPE` with a
    // public identifier and no internal subset. A receiver that refuses it
    // would refuse every modern sender — so the guard must be narrow.
    let text = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
                <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \
                \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
                <plist version=\"1.0\"><dict><key>a</key><integer>1</integer></dict></plist>";
    let value = decode(text, NORMAL_LIMIT).expect("the legacy document is decodable");
    assert_eq!(value.get("a").and_then(PlistValue::as_integer), Some(1));
}

#[test]
fn a_document_that_is_too_large_is_refused_before_it_is_parsed() {
    let body = format!(
        "<dict><key>a</key><string>{}</string></dict>",
        "x".repeat(4096)
    );
    let text = document(&body);
    let error = decode(&text, 128).expect_err("the document must be refused");
    assert!(
        matches!(error, AirPlayError::LimitExceeded { .. }),
        "the document must be refused by the limit: {error}"
    );
}

#[test]
fn a_document_that_nests_too_deeply_is_refused() {
    let mut text = String::new();
    for _ in 0..64 {
        text.push_str("<array>");
    }
    for _ in 0..64 {
        text.push_str("</array>");
    }
    let error = decode(&text, NORMAL_LIMIT).expect_err("the document must be refused");
    assert!(
        matches!(error, AirPlayError::LimitExceeded { .. }),
        "the document must be refused by the limit: {error}"
    );
}

#[test]
fn a_truncated_document_is_refused() {
    let text = "<plist version=\"1.0\"><dict><key>a</key>";
    let error = decode(text, NORMAL_LIMIT).expect_err("the document must be refused");
    assert!(
        matches!(error, AirPlayError::Parse { .. }),
        "the document must be refused as malformed: {error}"
    );
}

#[test]
fn a_dictionary_without_a_key_is_refused() {
    let text = "<plist version=\"1.0\"><dict><string>value</string></dict></plist>";
    let error = decode(text, NORMAL_LIMIT).expect_err("the document must be refused");
    assert!(
        matches!(error, AirPlayError::Parse { .. }),
        "the document must be refused as malformed: {error}"
    );
}

#[test]
fn an_unknown_element_is_refused() {
    let text = "<plist version=\"1.0\"><dict><key>a</key><banana>1</banana></dict></plist>";
    let error = decode(text, NORMAL_LIMIT).expect_err("the document must be refused");
    assert!(
        matches!(error, AirPlayError::Parse { .. }),
        "the document must be refused as malformed: {error}"
    );
}
