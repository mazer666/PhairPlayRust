// WHAT: the XML property-list decoder.
// WHY: AirPlay clients that do not speak the binary format still send
//      `<plist>` documents, and the receiver must accept them without
//      pulling in a general purpose XML engine. The decoder is therefore
//      written against the eight element types the format defines and
//      rejects everything else.
// EXAMPLE: the document
//          `<plist version="1.0"><dict><key>a</key><integer>1</integer></dict></plist>`
//          decodes into a dictionary with one entry.

use crate::base64;
use crate::date::rfc3339_to_apple_epoch;
use crate::error::PlistError;
use crate::limits::PlistLimits;
use crate::value::Value;
use crate::xml::reader::{Event, XmlReader};

/// Decodes an XML property list.
///
/// # Errors
///
/// Returns an error when the document is malformed, when it uses an
/// element the format does not define, or when it exceeds one of the
/// configured limits.
pub(crate) fn decode(input: &str, limits: &PlistLimits) -> Result<Value, PlistError> {
    let mut parser = XmlParser {
        reader: XmlReader::new(input, limits.max_string_len),
        limits,
    };
    parser.parse_document()
}

/// The recursive-descent parser over the XML event stream.
struct XmlParser<'a> {
    reader: XmlReader<'a>,
    limits: &'a PlistLimits,
}

impl XmlParser<'_> {
    /// Parses the whole document and returns the root value.
    fn parse_document(&mut self) -> Result<Value, PlistError> {
        let root = self
            .next_significant_event()?
            .ok_or(PlistError::Truncated { offset: 0 })?;
        let name = match root {
            Event::Start { name, self_closing } => {
                if name != "plist" {
                    return Err(PlistError::UnexpectedXmlElement { name });
                }
                if self_closing {
                    return Ok(Value::Dictionary(Vec::new()));
                }
                name
            }
            other => {
                return Err(PlistError::UnexpectedXmlElement {
                    name: describe(&other),
                });
            }
        };

        let value = self.parse_child_element(0)?;

        let closing = self
            .next_significant_event()?
            .ok_or(PlistError::Truncated { offset: 0 })?;
        match closing {
            Event::End { name: closed } if closed == name => {}
            other => {
                return Err(PlistError::UnexpectedXmlElement {
                    name: describe(&other),
                });
            }
        }

        if self.next_significant_event()?.is_some() {
            return Err(PlistError::TrailingXmlContent);
        }
        Ok(value)
    }

    /// Parses the single value element that a container holds.
    ///
    /// `depth` is the number of containers that enclose the element.
    fn parse_child_element(&mut self, depth: usize) -> Result<Value, PlistError> {
        let event = self
            .next_significant_event()?
            .ok_or(PlistError::Truncated { offset: 0 })?;
        match event {
            Event::Start { name, self_closing } => self.parse_value(&name, self_closing, depth),
            other => Err(PlistError::UnexpectedXmlElement {
                name: describe(&other),
            }),
        }
    }

    /// Parses a value element that has already been announced by a start tag.
    ///
    /// `depth` is the number of containers that enclose the element. The
    /// limit is checked here, so that a self-closing container counts as
    /// well.
    fn parse_value(
        &mut self,
        name: &str,
        self_closing: bool,
        depth: usize,
    ) -> Result<Value, PlistError> {
        if depth > self.limits.max_depth {
            return Err(PlistError::DepthLimitExceeded {
                limit: self.limits.max_depth,
            });
        }
        match (name, self_closing) {
            ("dict", false) => self.parse_dict(depth),
            ("dict", true) => Ok(Value::Dictionary(Vec::new())),
            ("array", false) => self.parse_array(depth),
            ("array", true) => Ok(Value::Array(Vec::new())),
            ("string", true) => Ok(Value::String(String::new())),
            ("string", false) => Ok(Value::String(self.read_text("string")?)),
            ("integer", true) => Err(PlistError::MissingXmlChild {
                name: "integer".into(),
                expected: "text",
            }),
            ("integer", false) => {
                let text = self.read_text("integer")?;
                let number = text
                    .trim()
                    .parse::<i64>()
                    .map_err(|_| PlistError::Xml(format!("invalid integer '{text}'")))?;
                Ok(Value::Integer(number))
            }
            ("real", true) => Err(PlistError::MissingXmlChild {
                name: "real".into(),
                expected: "text",
            }),
            ("real", false) => {
                let text = self.read_text("real")?;
                let number = text
                    .trim()
                    .parse::<f64>()
                    .map_err(|_| PlistError::Xml(format!("invalid real '{text}'")))?;
                Ok(Value::Real(number))
            }
            ("true", true) => Ok(Value::Boolean(true)),
            ("true", false) => {
                let text = self.read_text("true")?;
                if !text.trim().is_empty() {
                    return Err(PlistError::Xml("the <true> element carries text".into()));
                }
                Ok(Value::Boolean(true))
            }
            ("false", true) => Ok(Value::Boolean(false)),
            ("false", false) => {
                let text = self.read_text("false")?;
                if !text.trim().is_empty() {
                    return Err(PlistError::Xml("the <false> element carries text".into()));
                }
                Ok(Value::Boolean(false))
            }
            ("data", true) => Err(PlistError::MissingXmlChild {
                name: "data".into(),
                expected: "base64",
            }),
            ("data", false) => {
                let text = self.read_text("data")?;
                let bytes = base64::decode(&text)?;
                if bytes.len() > self.limits.max_data_len {
                    return Err(PlistError::DataTooLarge {
                        length: bytes.len() as u64,
                        limit: self.limits.max_data_len,
                    });
                }
                Ok(Value::Data(bytes))
            }
            ("date", true) => Err(PlistError::MissingXmlChild {
                name: "date".into(),
                expected: "timestamp",
            }),
            ("date", false) => {
                let text = self.read_text("date")?;
                let seconds = rfc3339_to_apple_epoch(text.trim())
                    .ok_or_else(|| PlistError::Xml(format!("invalid date '{text}'")))?;
                Ok(Value::Date(seconds))
            }
            _ => Err(PlistError::UnexpectedXmlElement {
                name: name.to_string(),
            }),
        }
    }

    /// Parses the body of a `<dict>` element.
    fn parse_dict(&mut self, depth: usize) -> Result<Value, PlistError> {
        let mut entries: Vec<(String, Value)> = Vec::new();
        loop {
            let event = self
                .next_event()?
                .ok_or(PlistError::Truncated { offset: 0 })?;
            match event {
                Event::End { name } if name == "dict" => break,
                Event::Text(text) if text.trim().is_empty() => continue,
                Event::Start {
                    name,
                    self_closing: _,
                } if name == "key" => {
                    if entries.len() >= self.limits.max_container_len {
                        return Err(PlistError::ContainerTooLarge {
                            length: entries.len() as u64 + 1,
                            limit: self.limits.max_container_len,
                        });
                    }
                    let key = self.read_text("key")?;
                    let value = self.parse_child_element(depth + 1)?;
                    entries.push((key, value));
                }
                other => {
                    return Err(PlistError::UnexpectedXmlElement {
                        name: describe(&other),
                    });
                }
            }
        }
        Ok(Value::Dictionary(entries))
    }

    /// Parses the body of an `<array>` element.
    fn parse_array(&mut self, depth: usize) -> Result<Value, PlistError> {
        let mut items: Vec<Value> = Vec::new();
        loop {
            let event = self
                .next_event()?
                .ok_or(PlistError::Truncated { offset: 0 })?;
            match event {
                Event::End { name } if name == "array" => break,
                Event::Text(text) if text.trim().is_empty() => continue,
                Event::Start { name, self_closing } => {
                    if items.len() >= self.limits.max_container_len {
                        return Err(PlistError::ContainerTooLarge {
                            length: items.len() as u64 + 1,
                            limit: self.limits.max_container_len,
                        });
                    }
                    items.push(self.parse_value(&name, self_closing, depth + 1)?);
                }
                other => {
                    return Err(PlistError::UnexpectedXmlElement {
                        name: describe(&other),
                    });
                }
            }
        }
        Ok(Value::Array(items))
    }

    /// Collects the character data of a leaf element.
    fn read_text(&mut self, name: &str) -> Result<String, PlistError> {
        let mut text = String::new();
        loop {
            let event = self
                .next_event()?
                .ok_or(PlistError::Truncated { offset: 0 })?;
            match event {
                Event::Text(chunk) => {
                    if text.len() + chunk.len() > self.limits.max_string_len {
                        return Err(PlistError::StringTooLong {
                            length: (text.len() + chunk.len()) as u64,
                            limit: self.limits.max_string_len,
                        });
                    }
                    text.push_str(&chunk);
                }
                Event::End { name: closed } if closed == name => return Ok(text),
                other => {
                    return Err(PlistError::UnexpectedXmlElement {
                        name: describe(&other),
                    });
                }
            }
        }
    }

    /// Reads the next event from the reader.
    fn next_event(&mut self) -> Result<Option<Event>, PlistError> {
        self.reader.next_event()
    }

    /// Reads the next event, skipping whitespace-only character data.
    ///
    /// WHY: a property list is allowed to be indented, and the
    ///      whitespace between two elements carries no value.
    fn next_significant_event(&mut self) -> Result<Option<Event>, PlistError> {
        loop {
            match self.next_event()? {
                Some(Event::Text(text)) if text.trim().is_empty() => continue,
                other => return Ok(other),
            }
        }
    }
}

/// Describes an event for an error message.
fn describe(event: &Event) -> String {
    match event {
        Event::Start { name, .. } => name.clone(),
        Event::End { name } => name.clone(),
        Event::Text(text) => format!("text '{}'", text.trim()),
    }
}

#[cfg(test)]
#[path = "decode_tests.rs"]
mod tests;
