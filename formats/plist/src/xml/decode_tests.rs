use super::*;

fn decode_ok(input: &str) -> Value {
    decode(input, &PlistLimits::default()).unwrap_or_else(|err| panic!("{err}"))
}

fn decode_err(input: &str) -> PlistError {
    match decode(input, &PlistLimits::default()) {
        Ok(value) => panic!("expected an error, got {value:?}"),
        Err(err) => err,
    }
}

#[test]
fn decodes_an_empty_dictionary() {
    let value = decode_ok(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0"><dict></dict></plist>"#,
    );
    assert_eq!(value, Value::Dictionary(Vec::new()));
}

#[test]
fn decodes_every_leaf_type() {
    let value = decode_ok(
        r#"<plist version="1.0"><dict>
            <key>s</key><string>text</string>
            <key>i</key><integer>-17</integer>
            <key>r</key><real>1.5</real>
            <key>t</key><true/>
            <key>f</key><false></false>
            <key>d</key><data>aGk=</data>
            <key>dt</key><date>2001-01-01T00:00:00Z</date>
        </dict></plist>"#,
    );
    assert_eq!(value.get("s"), Some(&Value::String("text".into())));
    assert_eq!(value.get("i"), Some(&Value::Integer(-17)));
    assert_eq!(value.get("r"), Some(&Value::Real(1.5)));
    assert_eq!(value.get("t"), Some(&Value::Boolean(true)));
    assert_eq!(value.get("f"), Some(&Value::Boolean(false)));
    assert_eq!(value.get("d"), Some(&Value::Data(b"hi".to_vec())));
    assert_eq!(value.get("dt"), Some(&Value::Date(0.0)));
}

#[test]
fn decodes_nested_arrays() {
    let value = decode_ok(
        "<plist version=\"1.0\"><array><array><integer>1</integer></array><array/></array></plist>",
    );
    let items = value.as_array().map(<[Value]>::to_vec).unwrap_or_default();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].as_array().map(<[Value]>::len), Some(1));
    assert_eq!(items[1].as_array().map(<[Value]>::len), Some(0));
}

#[test]
fn rejects_an_unknown_root_element() {
    assert!(matches!(
        decode_err("<html><body/></html>"),
        PlistError::UnexpectedXmlElement { .. }
    ));
}

#[test]
fn rejects_an_unknown_value_element() {
    assert!(matches!(
        decode_err("<plist><dict><key>a</key><colour>red</colour></dict></plist>"),
        PlistError::UnexpectedXmlElement { .. }
    ));
}

#[test]
fn rejects_content_after_the_root_element() {
    assert!(matches!(
        decode_err("<plist><dict/></plist><plist/>"),
        PlistError::TrailingXmlContent
    ));
}

#[test]
fn rejects_a_truncated_document() {
    assert!(matches!(
        decode_err("<plist><dict><key>a</key>"),
        PlistError::Truncated { .. }
    ));
}

#[test]
fn rejects_an_internal_dtd_subset() {
    assert!(matches!(
        decode_err("<!DOCTYPE plist [ <!ENTITY x \"y\"> ]><plist><dict/></plist>"),
        PlistError::XmlInternalSubset
    ));
}

#[test]
fn rejects_a_non_numeric_integer() {
    assert!(matches!(
        decode_err("<plist><dict><key>a</key><integer>abc</integer></dict></plist>"),
        PlistError::Xml(_)
    ));
}

#[test]
fn rejects_a_bad_date() {
    assert!(matches!(
        decode_err("<plist><dict><key>a</key><date>yesterday</date></dict></plist>"),
        PlistError::Xml(_)
    ));
}

#[test]
fn enforces_the_depth_limit() {
    let limits = PlistLimits {
        max_depth: 2,
        ..PlistLimits::default()
    };
    let mut document = String::new();
    for _ in 0..8 {
        document.push_str("<array>");
    }
    document.push_str("<integer>1</integer>");
    for _ in 0..8 {
        document.push_str("</array>");
    }
    let error = match decode(&format!("<plist>{document}</plist>"), &limits) {
        Ok(_) => panic!("expected the depth limit to bite"),
        Err(err) => err,
    };
    assert!(matches!(error, PlistError::DepthLimitExceeded { limit: 2 }));
}
