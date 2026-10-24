// WHAT: the XML property-list encoder.
// WHY: the legacy AirPlay endpoints (`GET /playback-info` and friends)
//      answer with `text/x-apple-plist+xml`, so the receiver has to be
//      able to produce that document as well as the binary one.
// EXAMPLE: `encode(&Value::dictionary([("a".into(), Value::Integer(1))]))`
//          produces a `<plist>` document with one dictionary entry.

use crate::base64;
use crate::date::apple_epoch_to_rfc3339;
use crate::error::PlistError;
use crate::value::Value;

/// The header that precedes every generated document.
const HEADER: &str = concat!(
    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
    "<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" ",
    "\"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n",
    "<plist version=\"1.0\">\n",
);

/// Encodes a value as an XML property list.
///
/// # Errors
///
/// Returns an error when the value cannot be represented in XML, which
/// happens for a non-finite real and for a date outside the range the
/// calendar can express.
pub(crate) fn encode(value: &Value) -> Result<Vec<u8>, PlistError> {
    let mut out = String::new();
    out.push_str(HEADER);
    write_value(&mut out, value, 0)?;
    out.push_str("</plist>\n");
    Ok(out.into_bytes())
}

/// Writes one value at the given indentation depth.
fn write_value(out: &mut String, value: &Value, depth: usize) -> Result<(), PlistError> {
    match value {
        Value::Boolean(flag) => {
            let tag = if *flag { "true" } else { "false" };
            push_indent(out, depth);
            out.push('<');
            out.push_str(tag);
            out.push_str("/>\n");
        }
        Value::Integer(number) => {
            push_indent(out, depth);
            out.push_str("<integer>");
            out.push_str(&number.to_string());
            out.push_str("</integer>\n");
        }
        Value::Real(number) => {
            if !number.is_finite() {
                return Err(PlistError::Encode(format!(
                    "the real {number} cannot be written to XML"
                )));
            }
            push_indent(out, depth);
            out.push_str("<real>");
            out.push_str(&format_real(*number));
            out.push_str("</real>\n");
        }
        Value::String(text) => {
            push_indent(out, depth);
            out.push_str("<string>");
            escape_into(out, text);
            out.push_str("</string>\n");
        }
        Value::Data(bytes) => {
            push_indent(out, depth);
            out.push_str("<data>");
            out.push_str(&base64::encode(bytes));
            out.push_str("</data>\n");
        }
        Value::Date(seconds) => {
            let text = apple_epoch_to_rfc3339(*seconds).ok_or_else(|| {
                PlistError::Encode(format!("the date {seconds} cannot be written"))
            })?;
            push_indent(out, depth);
            out.push_str("<date>");
            out.push_str(&text);
            out.push_str("</date>\n");
        }
        Value::Array(items) => {
            if items.is_empty() {
                push_indent(out, depth);
                out.push_str("<array/>\n");
                return Ok(());
            }
            push_indent(out, depth);
            out.push_str("<array>\n");
            for item in items {
                write_value(out, item, depth + 1)?;
            }
            push_indent(out, depth);
            out.push_str("</array>\n");
        }
        Value::Dictionary(entries) => {
            if entries.is_empty() {
                push_indent(out, depth);
                out.push_str("<dict/>\n");
                return Ok(());
            }
            push_indent(out, depth);
            out.push_str("<dict>\n");
            for (key, item) in entries {
                push_indent(out, depth + 1);
                out.push_str("<key>");
                escape_into(out, key);
                out.push_str("</key>\n");
                write_value(out, item, depth + 1)?;
            }
            push_indent(out, depth);
            out.push_str("</dict>\n");
        }
    }
    Ok(())
}

/// Appends `depth` tab characters.
fn push_indent(out: &mut String, depth: usize) {
    for _ in 0..depth {
        out.push('\t');
    }
}

/// Formats a real so that it reads back as a real, not as an integer.
fn format_real(number: f64) -> String {
    let text = format!("{number:?}");
    if text.contains('.') || text.contains('e') || text.contains('E') {
        text
    } else {
        format!("{text}.0")
    }
}

/// Escapes the five XML predefined entities.
fn escape_into(out: &mut String, text: &str) {
    for character in text.chars() {
        match character {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            other => out.push(other),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;
    use crate::limits::PlistLimits;
    use crate::xml::decode::decode as decode_xml;

    fn round_trip(value: &Value) -> Value {
        let bytes = encode(value).unwrap_or_else(|err| panic!("{err}"));
        let text = String::from_utf8(bytes).unwrap_or_else(|err| panic!("{err}"));
        decode_xml(&text, &PlistLimits::default()).unwrap_or_else(|err| panic!("{err}"))
    }

    #[test]
    fn encodes_a_dictionary() {
        let value = Value::dictionary([
            ("name", Value::String("Living Room".into())),
            ("volume", Value::Integer(42)),
        ]);
        let bytes = encode(&value).unwrap_or_default();
        let text = String::from_utf8(bytes).unwrap_or_default();
        assert!(text.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(text.contains("<key>name</key>"));
        assert!(text.contains("<string>Living Room</string>"));
        assert!(text.trim_end().ends_with("</plist>"));
    }

    #[test]
    fn round_trips_every_value_type() {
        let value = Value::dictionary([
            ("s", Value::String("text".into())),
            ("i", Value::Integer(-17)),
            ("r", Value::Real(1.5)),
            ("t", Value::Boolean(true)),
            ("f", Value::Boolean(false)),
            ("d", Value::Data(vec![0, 1, 2, 255])),
            ("dt", Value::Date(1.5)),
        ]);
        assert_eq!(round_trip(&value), value);
    }

    #[test]
    fn round_trips_a_whole_real_as_a_real() {
        let value = Value::Real(2.0);
        assert_eq!(round_trip(&value), value);
    }

    #[test]
    fn escapes_markup_characters() {
        let value = Value::String("a & b < c > d \" e".into());
        let bytes = encode(&value).unwrap_or_default();
        let text = String::from_utf8(bytes).unwrap_or_default();
        assert!(text.contains("a &amp; b &lt; c &gt; d &quot; e"), "{text}");
        assert_eq!(round_trip(&value), value);
    }

    #[test]
    fn rejects_a_non_finite_real() {
        let error = match encode(&Value::Real(f64::NAN)) {
            Ok(_) => panic!("expected an error"),
            Err(err) => err,
        };
        assert!(matches!(error, PlistError::Encode(_)));
    }

    #[test]
    fn round_trips_empty_containers() {
        assert_eq!(
            round_trip(&Value::Array(Vec::new())),
            Value::Array(Vec::new())
        );
        assert_eq!(
            round_trip(&Value::Dictionary(Vec::new())),
            Value::Dictionary(Vec::new())
        );
    }
}
