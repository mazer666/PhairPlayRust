//! The decoder of the XML property list.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;

use crate::error::AirPlayError;
use crate::plist::value::PlistValue;

use super::MAX_DEPTH;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Event {
    /// An opening tag with its name and whether it is self-closing.
    Start(String, bool),
    /// A closing tag.
    End(String),
    /// Character data between tags.
    Text(String),
}

/// Decodes an XML property list.
///
/// # Errors
/// Returns [`AirPlayError::Parse`] when the document is not a well-formed
/// property list and [`AirPlayError::LimitExceeded`] when it is too large or
/// nests too deeply.
pub fn decode(text: &str, max_bytes: usize) -> Result<PlistValue, AirPlayError> {
    if text.len() > max_bytes {
        return Err(AirPlayError::limit_exceeded(format!(
            "plist of {} bytes exceeds the {max_bytes} byte limit",
            text.len()
        )));
    }
    let events = tokenize(text);
    let mut cursor = 0usize;
    skip_whitespace(&events, &mut cursor);
    // The document is wrapped in a `plist` element.
    if let Some(Event::Start(name, _)) = events.get(cursor) {
        if name == "plist" {
            cursor += 1;
            skip_whitespace(&events, &mut cursor);
        }
    }
    let value = parse_value(&events, &mut cursor, 0)?;
    Ok(value)
}

/// Skips the whitespace between two elements.
fn skip_whitespace(events: &[Event], cursor: &mut usize) {
    while let Some(Event::Text(text)) = events.get(*cursor) {
        if !text.trim().is_empty() {
            return;
        }
        *cursor += 1;
    }
}

/// Splits the document into tags and text.
fn tokenize(text: &str) -> Vec<Event> {
    let bytes = text.as_bytes();
    let mut events = Vec::new();
    let mut position = 0usize;
    let mut text_start = 0usize;
    while position < bytes.len() {
        if bytes[position] != b'<' {
            position += 1;
            continue;
        }
        if text_start < position {
            events.push(Event::Text(text[text_start..position].to_string()));
        }
        if text[position..].starts_with("<!--") {
            match text[position..].find("-->") {
                Some(offset) => position += offset + 3,
                None => break,
            }
            text_start = position;
            continue;
        }
        if text[position..].starts_with("<?") {
            match text[position..].find("?>") {
                Some(offset) => position += offset + 2,
                None => break,
            }
            text_start = position;
            continue;
        }
        if text[position..].starts_with("<!") {
            match text[position..].find('>') {
                Some(offset) => position += offset + 1,
                None => break,
            }
            text_start = position;
            continue;
        }
        let Some(end) = text[position..].find('>') else {
            break;
        };
        let raw = &text[position + 1..position + end];
        position += end + 1;
        text_start = position;
        if let Some(name) = raw.strip_prefix('/') {
            events.push(Event::End(name.trim().to_string()));
        } else {
            let trimmed = raw.trim_end();
            let self_closing = trimmed.ends_with('/');
            let name = trimmed.trim_end_matches('/').trim();
            // The element name ends at the first whitespace; the rest are
            // attributes.
            let name = name.split_whitespace().next().unwrap_or_default();
            events.push(Event::Start(name.to_string(), self_closing));
        }
    }
    if text_start < text.len() {
        events.push(Event::Text(text[text_start..].to_string()));
    }
    events
}

/// Parses one value at the cursor.
fn parse_value(
    events: &[Event],
    cursor: &mut usize,
    depth: u32,
) -> Result<PlistValue, AirPlayError> {
    if depth > MAX_DEPTH {
        return Err(AirPlayError::limit_exceeded("plist nests too deeply"));
    }
    skip_whitespace(events, cursor);
    let Some(Event::Start(name, self_closing)) = events.get(*cursor).cloned() else {
        return Err(AirPlayError::parse("expected a plist element"));
    };
    *cursor += 1;
    match name.as_str() {
        "true" => Ok(PlistValue::Boolean(true)),
        "false" => Ok(PlistValue::Boolean(false)),
        "string" => {
            let text = read_text(events, cursor, &name, self_closing)?;
            Ok(PlistValue::String(unescape(&text)))
        }
        "integer" => {
            let text = read_text(events, cursor, &name, self_closing)?;
            let number = text
                .trim()
                .parse::<i64>()
                .map_err(|_| AirPlayError::parse("plist integer is malformed"))?;
            Ok(PlistValue::Integer(number))
        }
        "real" => {
            let text = read_text(events, cursor, &name, self_closing)?;
            let number = text
                .trim()
                .parse::<f64>()
                .map_err(|_| AirPlayError::parse("plist real is malformed"))?;
            Ok(PlistValue::Real(number))
        }
        "data" => {
            let text = read_text(events, cursor, &name, self_closing)?;
            let bytes = BASE64
                .decode(text.trim())
                .map_err(|_| AirPlayError::parse("plist data is not valid Base64"))?;
            Ok(PlistValue::Data(bytes))
        }
        "date" => {
            let text = read_text(events, cursor, &name, self_closing)?;
            let number = text
                .trim()
                .parse::<f64>()
                .map_err(|_| AirPlayError::parse("plist date is malformed"))?;
            Ok(PlistValue::Real(number))
        }
        "array" => {
            if self_closing {
                return Ok(PlistValue::Array(Vec::new()));
            }
            let mut items = Vec::new();
            loop {
                skip_whitespace(events, cursor);
                match events.get(*cursor) {
                    Some(Event::End(end)) if end == "array" => {
                        *cursor += 1;
                        break;
                    }
                    None => return Err(AirPlayError::parse("unterminated plist array")),
                    _ => items.push(parse_value(events, cursor, depth + 1)?),
                }
            }
            Ok(PlistValue::Array(items))
        }
        "dict" => {
            if self_closing {
                return Ok(PlistValue::Dictionary(Vec::new()));
            }
            let mut entries = Vec::new();
            loop {
                skip_whitespace(events, cursor);
                match events.get(*cursor) {
                    Some(Event::End(end)) if end == "dict" => {
                        *cursor += 1;
                        break;
                    }
                    None => return Err(AirPlayError::parse("unterminated plist dictionary")),
                    Some(Event::Start(name, self_closing)) if name == "key" => {
                        *cursor += 1;
                        let key = read_text(events, cursor, name, *self_closing)?;
                        let value = parse_value(events, cursor, depth + 1)?;
                        entries.push((unescape(&key), value));
                    }
                    Some(_) => {
                        return Err(AirPlayError::parse(
                            "plist dictionary expects a key element",
                        ));
                    }
                }
            }
            Ok(PlistValue::Dictionary(entries))
        }
        other => Err(AirPlayError::parse(format!(
            "unsupported plist element `{other}`"
        ))),
    }
}

/// Reads the character data of an element.
fn read_text(
    events: &[Event],
    cursor: &mut usize,
    name: &str,
    self_closing: bool,
) -> Result<String, AirPlayError> {
    if self_closing {
        return Ok(String::new());
    }
    let mut text = String::new();
    loop {
        match events.get(*cursor) {
            Some(Event::Text(chunk)) => {
                text.push_str(chunk);
                *cursor += 1;
            }
            Some(Event::End(end)) if end == name => {
                *cursor += 1;
                return Ok(text);
            }
            Some(Event::End(_)) => {
                return Err(AirPlayError::parse(format!(
                    "plist element `{name}` is closed by another element"
                )));
            }
            Some(Event::Start(_, _)) => {
                return Err(AirPlayError::parse(format!(
                    "plist element `{name}` may not contain elements"
                )));
            }
            None => return Err(AirPlayError::parse("unterminated plist element")),
        }
    }
}

/// Resolves the five XML entities.
fn unescape(text: &str) -> String {
    if !text.contains('&') {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(index) = rest.find('&') {
        out.push_str(&rest[..index]);
        let tail = &rest[index..];
        let Some(end) = tail.find(';') else {
            out.push('&');
            rest = &tail[1..];
            continue;
        };
        let entity = &tail[1..end];
        match entity {
            "amp" => out.push('&'),
            "lt" => out.push('<'),
            "gt" => out.push('>'),
            "quot" => out.push('"'),
            "apos" => out.push('\''),
            other => {
                out.push('&');
                out.push_str(other);
                out.push(';');
            }
        }
        rest = &tail[end + 1..];
    }
    out.push_str(rest);
    out
}
