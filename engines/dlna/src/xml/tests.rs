#![allow(clippy::expect_used, clippy::unwrap_used)]

use super::*;

#[test]
fn a_simple_document_is_parsed() {
    let document = XmlDocument::parse("<root><child>text</child></root>").expect("valid");
    assert_eq!(document.root().local_name(), "root");
    let child = document.root().child("child").expect("the child exists");
    assert_eq!(child.text(), "text");
}

#[test]
fn namespace_prefixes_are_stripped_from_the_local_name() {
    let document = XmlDocument::parse(
        r#"<s:Envelope xmlns:s="urn:schemas"><s:Body><u:Play xmlns:u="urn:u"/></s:Body></s:Envelope>"#,
    )
    .expect("valid");
    assert_eq!(document.root().local_name(), "Envelope");
    let body = document.root().child("Body").expect("the body exists");
    assert_eq!(
        body.child("Play").expect("the action exists").local_name(),
        "Play"
    );
}

#[test]
fn attributes_are_read() {
    let document = XmlDocument::parse(r#"<root a="1" ns:b="2"><child/></root>"#).expect("valid");
    assert_eq!(document.root().attribute("a"), Some("1"));
    assert_eq!(document.root().attribute("b"), Some("2"));
    assert!(document.root().attribute("c").is_none());
}

#[test]
fn a_doctype_is_rejected() {
    // This is the XXE attack: an external entity that points at a file.
    let attack = r#"<?xml version="1.0"?>
<!DOCTYPE foo [ <!ENTITY xxe SYSTEM "file:///etc/passwd"> ]>
<s:Envelope xmlns:s="urn:x"><s:Body><x>&xxe;</x></s:Body></s:Envelope>"#;
    let error = XmlDocument::parse(attack).expect_err("the doctype is rejected");
    assert_eq!(error, XmlError::DoctypeForbidden);
}

#[test]
fn a_doctype_in_lower_case_is_rejected() {
    let error = XmlDocument::parse("<!doctype html><html/>").expect_err("rejected");
    assert_eq!(error, XmlError::DoctypeForbidden);
}

#[test]
fn an_entity_declaration_is_rejected() {
    let error =
        XmlDocument::parse("<!ENTITY x \"y\"><root/>").expect_err("the declaration is rejected");
    assert_eq!(error, XmlError::EntityForbidden);
}

#[test]
fn an_unknown_entity_is_rejected() {
    let error = XmlDocument::parse("<root>&xxe;</root>").expect_err("the entity is rejected");
    assert_eq!(error, XmlError::UnknownEntity("xxe".to_string()));
}

#[test]
fn an_external_entity_reference_is_rejected() {
    let error = XmlDocument::parse("<root>&file;</root>").expect_err("the entity is rejected");
    assert_eq!(error, XmlError::UnknownEntity("file".to_string()));
}

#[test]
fn the_predefined_entities_are_expanded() {
    let document = XmlDocument::parse("<root>&lt;&gt;&amp;&quot;&apos;</root>").expect("valid");
    assert_eq!(document.root().text(), "<>&\"'");
}

#[test]
fn a_numeric_character_reference_is_expanded() {
    let document = XmlDocument::parse("<root>&#65;&#x42;</root>").expect("valid");
    assert_eq!(document.root().text(), "AB");
}

#[test]
fn a_surrogate_character_reference_is_rejected() {
    let error = XmlDocument::parse("<root>&#xD800;</root>").expect_err("rejected");
    assert!(matches!(error, XmlError::UnknownEntity(_)));
}

#[test]
fn an_unterminated_entity_is_rejected() {
    let error = XmlDocument::parse("<root>&amp</root>").expect_err("rejected");
    assert!(matches!(error, XmlError::Malformed(_)));
}

#[test]
fn a_document_that_is_too_deep_is_rejected() {
    let deep = format!(
        "{}<a/>{}",
        "<a>".repeat(MAX_XML_DEPTH + 4),
        "</a>".repeat(MAX_XML_DEPTH + 4)
    );
    let error = XmlDocument::parse(&deep).expect_err("rejected");
    assert!(matches!(error, XmlError::DepthExceeded { .. }));
}

#[test]
fn a_document_with_too_many_elements_is_rejected() {
    let many = format!("<root>{}</root>", "<a/>".repeat(MAX_XML_NODES + 10));
    let error = XmlDocument::parse(&many).expect_err("rejected");
    assert!(matches!(error, XmlError::TooManyNodes { .. }));
}

#[test]
fn an_oversized_document_is_rejected() {
    let big = format!("<root>{}</root>", "x".repeat(MAX_XML_BYTES + 1));
    let error = XmlDocument::parse(&big).expect_err("rejected");
    assert_eq!(
        error,
        XmlError::TooLarge {
            limit: MAX_XML_BYTES
        }
    );
}

#[test]
fn a_mismatched_end_tag_is_rejected() {
    let error = XmlDocument::parse("<a><b></c></a>").expect_err("rejected");
    assert!(matches!(error, XmlError::Malformed(_)));
}

#[test]
fn a_second_root_element_is_rejected() {
    let error = XmlDocument::parse("<a/><b/>").expect_err("rejected");
    assert!(matches!(error, XmlError::Malformed(_)));
}

#[test]
fn text_outside_the_root_element_is_rejected() {
    let error = XmlDocument::parse("text<a/>").expect_err("rejected");
    assert!(matches!(error, XmlError::Malformed(_)));
}

#[test]
fn an_unclosed_element_is_rejected() {
    let error = XmlDocument::parse("<a><b>").expect_err("rejected");
    assert!(matches!(error, XmlError::Malformed(_)));
}

#[test]
fn an_unquoted_attribute_value_is_rejected() {
    let error = XmlDocument::parse("<a b=c/>").expect_err("rejected");
    assert!(matches!(error, XmlError::Malformed(_)));
}

#[test]
fn a_comment_is_skipped() {
    let document = XmlDocument::parse("<root><!-- a comment --><a/></root>").expect("valid");
    assert_eq!(document.root().children().count(), 1);
}

#[test]
fn a_cdata_section_is_read_as_text() {
    let document = XmlDocument::parse("<root><![CDATA[a < b]]></root>").expect("valid");
    assert_eq!(document.root().text(), "a < b");
}

#[test]
fn a_processing_instruction_is_skipped() {
    let document = XmlDocument::parse("<?xml version=\"1.0\"?><root/>").expect("valid");
    assert_eq!(document.root().local_name(), "root");
}

#[test]
fn a_descendant_is_found() {
    let document = XmlDocument::parse("<a><b><c>deep</c></b></a>").expect("valid");
    assert_eq!(
        document.root().descendant_text("c").as_deref(),
        Some("deep")
    );
    assert!(document.root().descendant_text("d").is_none());
}
