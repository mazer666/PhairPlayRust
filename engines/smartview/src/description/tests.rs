//! The tests of the description document.
//!
//! WHY: the document is read by a parser on the phone, and the name in it
//! comes from the configuration of the receiver. A name with a control
//! character or with a markup character must not break the document.

#![allow(clippy::expect_used, clippy::unwrap_used)]
use super::*;
use phairplay_core::capabilities::{AudioCodec, VideoCodec};

/// A description of a receiver.
fn description() -> DeviceDescription {
    DeviceDescription::new(
        "Living Room",
        "PhairPlay Rust Receiver",
        "uuid:abc",
        "http://10.0.0.5:8001/api/v2/",
    )
}

#[test]
fn the_description_document_is_built_by_the_free_function_too() {
    // WHY: the builder and the function are two doors to the same
    // document — they may not answer differently.
    let document = description_document(&description());
    assert!(
        document.contains("<friendlyName>Living Room</friendlyName>"),
        "{document}"
    );
    assert_eq!(document, description().to_xml());
}

#[test]
fn the_document_carries_the_facts_a_sender_reads() {
    let document = description().to_xml();
    assert!(
        document.contains("<friendlyName>Living Room</friendlyName>"),
        "{document}"
    );
    assert!(
        document.contains("<modelName>PhairPlay Rust Receiver</modelName>"),
        "{document}"
    );
    assert!(document.contains("<UDN>uuid:abc</UDN>"), "{document}");
    assert!(
        document.contains("<baseURL>http://10.0.0.5:8001/api/v2/</baseURL>"),
        "{document}"
    );
}

#[test]
fn the_document_names_the_service_it_describes() {
    let document = description().to_xml();
    assert!(document.contains(NAMESPACE), "{document}");
    assert!(
        document.contains("<specVersion><major>1</major>"),
        "{document}"
    );
}

#[test]
fn a_control_character_in_the_name_never_reaches_the_document() {
    // WHY: XML 1.0 has no escape for a control character. A name that
    // carries one would produce a document that no parser accepts — and the
    // device would disappear from the list of the phone when the user taps
    // it.
    let hostile = DeviceDescription::new(
        "Living\u{0} Room\u{7}\u{1b}",
        "PhairPlay Rust Receiver",
        "uuid:abc",
        "http://10.0.0.5:8001/api/v2/",
    );
    let document = hostile.to_xml();
    assert!(!document.contains('\u{0}'), "{document}");
    assert!(!document.contains('\u{7}'), "{document}");
    assert!(!document.contains('\u{1b}'), "{document}");
    assert!(
        document.contains("<friendlyName>Living Room</friendlyName>"),
        "{document}"
    );
}

#[test]
fn a_markup_character_in_the_name_is_escaped() {
    let hostile = DeviceDescription::new(
        "<script>alert(1)</script>",
        "PhairPlay Rust Receiver",
        "uuid:abc",
        "http://10.0.0.5:8001/api/v2/",
    );
    let document = hostile.to_xml();
    assert!(!document.contains("<script>"), "{document}");
    assert!(document.contains("&lt;script&gt;"), "{document}");
}

#[test]
fn an_ampersand_in_the_base_url_is_escaped() {
    let hostile = DeviceDescription::new(
        "Living Room",
        "PhairPlay Rust Receiver",
        "uuid:abc",
        "http://10.0.0.5:8001/api/v2/?a=1&b=2",
    );
    let document = hostile.to_xml();
    assert!(document.contains("a=1&amp;b=2"), "{document}");
}

#[test]
fn the_capabilities_are_listed_as_mime_types() {
    let mut capabilities = MediaCapabilities::empty();
    capabilities.video.insert(VideoCodec::H264);
    capabilities.audio.insert(AudioCodec::AacLc);
    let document = description().with_capabilities(&capabilities).to_xml();
    assert!(
        document.contains("<capability kind=\"video\" value=\"video/avc\"/>"),
        "{document}"
    );
    assert!(
        document.contains("<capability kind=\"audio\" value=\"audio/mp4a-latm\"/>"),
        "{document}"
    );
}

#[test]
fn a_receiver_without_capabilities_lists_none() {
    let document = description().to_xml();
    assert!(document.contains("<capabilities>"), "{document}");
    assert!(document.contains("</capabilities>"), "{document}");
    assert!(!document.contains("<capability "), "{document}");
}

#[test]
fn the_text_sanitiser_keeps_the_characters_that_are_allowed() {
    let text = "a\tb\nc\rd e";
    assert_eq!(sanitize_xml_text(text), text);
}

#[test]
fn the_text_sanitiser_removes_the_characters_that_xml_forbids() {
    let text = "a\u{0}b\u{c}c\u{1b}d\u{fffe}e";
    assert_eq!(sanitize_xml_text(text), "abcde");
}

#[test]
fn the_escaping_covers_the_five_markup_characters() {
    assert_eq!(escape_xml("&"), "&amp;");
    assert_eq!(escape_xml("<"), "&lt;");
    assert_eq!(escape_xml(">"), "&gt;");
    assert_eq!(escape_xml("\""), "&quot;");
    assert_eq!(escape_xml("'"), "&apos;");
}
