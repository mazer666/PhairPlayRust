//! The XML property list codec.
//!
//! WHY: some AirPlay clients — the video playback mode in particular — expect
//! the legacy XML form (`text/x-apple-plist+xml`) instead of the binary one,
//! and the legacy PIN pairing may arrive as XML as well. The codec therefore
//! implements both directions, with the same hardening rules as the binary
//! codec: the document size is bounded, the nesting depth is bounded, and
//! anything that is not a well-formed property list is rejected.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_airplay::plist::{PlistValue, xml};
//!
//! let document = PlistValue::dictionary(vec![("readyToPlay".into(), PlistValue::Boolean(false))]);
//! let encoded = xml::encode(&document);
//! let text = String::from_utf8(encoded).expect("utf-8");
//! assert!(text.contains("<key>readyToPlay</key>"));
//! let decoded = xml::decode(&text, 64 * 1024).expect("decodable");
//! assert_eq!(decoded, document);
//! ```

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;

use crate::plist::value::PlistValue;

/// The maximum nesting depth of an XML property list.
const MAX_DEPTH: u32 = 32;

/// Encodes a value as an XML property list.
#[must_use]
pub fn encode(value: &PlistValue) -> Vec<u8> {
    let mut out = String::with_capacity(512);
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str(
        "<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \
\"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n",
    );
    out.push_str("<plist version=\"1.0\">\n");
    write_value(&mut out, value, 0);
    out.push_str("</plist>\n");
    out.into_bytes()
}

/// Writes one value and its tags.
fn write_value(out: &mut String, value: &PlistValue, depth: u32) {
    let indent = "  ".repeat(depth as usize);
    match value {
        PlistValue::String(text) => {
            out.push_str(&indent);
            out.push_str("<string>");
            out.push_str(&escape(text));
            out.push_str("</string>\n");
        }
        PlistValue::Integer(number) => {
            out.push_str(&indent);
            out.push_str("<integer>");
            out.push_str(&number.to_string());
            out.push_str("</integer>\n");
        }
        PlistValue::Real(number) => {
            out.push_str(&indent);
            out.push_str("<real>");
            out.push_str(&number.to_string());
            out.push_str("</real>\n");
        }
        PlistValue::Boolean(flag) => {
            out.push_str(&indent);
            out.push_str(if *flag { "<true/>\n" } else { "<false/>\n" });
        }
        PlistValue::Data(bytes) => {
            out.push_str(&indent);
            out.push_str("<data>");
            out.push_str(&BASE64.encode(bytes));
            out.push_str("</data>\n");
        }
        PlistValue::Array(items) => {
            out.push_str(&indent);
            out.push_str("<array>\n");
            for item in items {
                write_value(out, item, depth + 1);
            }
            out.push_str(&indent);
            out.push_str("</array>\n");
        }
        PlistValue::Dictionary(entries) => {
            out.push_str(&indent);
            out.push_str("<dict>\n");
            for (key, item) in entries {
                out.push_str(&indent);
                out.push_str("  <key>");
                out.push_str(&escape(key));
                out.push_str("</key>\n");
                write_value(out, item, depth + 2);
            }
            out.push_str(&indent);
            out.push_str("</dict>\n");
        }
    }
}

/// Escapes the five XML entities.
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
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
    out
}

/// One event of the XML token stream.
mod decode;

pub use decode::decode;

#[cfg(test)]
mod tests;
