// WHAT: a pull parser for the small subset of XML that a property list
//      uses.
// WHY: an XML property list arrives from the network, so the parser must
//      never expand entities, never fetch anything and never recurse
//      without a limit. A hand-written scanner keeps that guarantee
//      visible in the code instead of hiding it behind a dependency.
// EXAMPLE: the reader turns `<dict><key>a</key></dict>` into a sequence
//          of start, start, text, end, end events.

use crate::error::PlistError;

/// One event of the XML event stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Event {
    /// An opening tag. `self_closing` is set for tags of the form `<x/>`.
    Start {
        /// The element name.
        name: String,
        /// Whether the tag closed itself.
        self_closing: bool,
    },
    /// A closing tag.
    End {
        /// The element name.
        name: String,
    },
    /// Character data between tags, with entities already resolved.
    Text(String),
}

/// A scanner over an XML document.
pub(crate) struct XmlReader<'a> {
    input: &'a str,
    position: usize,
    max_text_len: usize,
}

impl<'a> XmlReader<'a> {
    /// Creates a reader over `input`.
    ///
    /// `max_text_len` bounds the size of a single text node so that a
    /// hostile peer cannot make the decoder accumulate an unbounded
    /// string.
    pub(crate) fn new(input: &'a str, max_text_len: usize) -> Self {
        Self {
            input,
            position: 0,
            max_text_len,
        }
    }

    /// Returns the next event, or `None` at the end of the document.
    pub(crate) fn next_event(&mut self) -> Result<Option<Event>, PlistError> {
        loop {
            let rest = &self.input[self.position..];
            if rest.is_empty() {
                return Ok(None);
            }
            if rest.starts_with('<') {
                if let Some(event) = self.read_markup()? {
                    return Ok(Some(event));
                }
                continue;
            }
            let end = rest
                .find('<')
                .map_or(self.input.len(), |index| self.position + index);
            let raw = &self.input[self.position..end];
            self.position = end;
            let text = decode_entities(raw)?;
            if text.len() > self.max_text_len {
                return Err(PlistError::StringTooLong {
                    length: text.len() as u64,
                    limit: self.max_text_len,
                });
            }
            return Ok(Some(Event::Text(text)));
        }
    }

    /// Reads one markup construct. Returns `None` when the construct was
    /// a comment or a processing instruction that carries no data.
    fn read_markup(&mut self) -> Result<Option<Event>, PlistError> {
        let rest = &self.input[self.position..];
        if rest.starts_with("<!--") {
            let end = self.input[self.position..]
                .find("-->")
                .ok_or_else(|| PlistError::Xml("unterminated comment".into()))?;
            self.position += end + 3;
            return Ok(None);
        }
        if rest.starts_with("<?") {
            let end = self.input[self.position..]
                .find("?>")
                .ok_or_else(|| PlistError::Xml("unterminated processing instruction".into()))?;
            self.position += end + 2;
            return Ok(None);
        }
        if rest.starts_with("<![CDATA[") {
            let start = self.position + "<![CDATA[".len();
            let end = self.input[start..]
                .find("]]>")
                .ok_or_else(|| PlistError::Xml("unterminated CDATA section".into()))?;
            let text = self.input[start..start + end].to_string();
            self.position = start + end + 3;
            if text.len() > self.max_text_len {
                return Err(PlistError::StringTooLong {
                    length: text.len() as u64,
                    limit: self.max_text_len,
                });
            }
            return Ok(Some(Event::Text(text)));
        }
        if rest.starts_with("<!DOCTYPE") {
            self.skip_doctype()?;
            return Ok(None);
        }
        if rest.starts_with("</") {
            let end = rest
                .find('>')
                .ok_or_else(|| PlistError::Xml("unterminated end tag".into()))?;
            let name = rest[2..end].trim().to_string();
            if name.is_empty() {
                return Err(PlistError::Xml("end tag without a name".into()));
            }
            self.position += end + 1;
            return Ok(Some(Event::End { name }));
        }

        let end = rest
            .find('>')
            .ok_or_else(|| PlistError::Xml("unterminated start tag".into()))?;
        let body = &rest[1..end];
        let self_closing = body.ends_with('/');
        let body = body.trim_end_matches('/');
        let name = body
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .to_string();
        if name.is_empty() {
            return Err(PlistError::Xml("start tag without a name".into()));
        }
        self.position += end + 1;
        Ok(Some(Event::Start { name, self_closing }))
    }

    /// Skips a `<!DOCTYPE …>` declaration.
    ///
    /// WHY: a doctype with an internal subset is the entry point of the
    ///      billion-laughs entity-expansion attack. This decoder resolves
    ///      only the five predefined entities and numeric references, so
    ///      it refuses a document that carries an internal subset instead
    ///      of ignoring the risk.
    fn skip_doctype(&mut self) -> Result<(), PlistError> {
        let rest = &self.input[self.position..];
        let end = rest
            .find('>')
            .ok_or_else(|| PlistError::Xml("unterminated doctype".into()))?;
        if rest[..end].contains('[') {
            return Err(PlistError::XmlInternalSubset);
        }
        self.position += end + 1;
        Ok(())
    }
}

/// Resolves the five predefined entities and numeric character references.
fn decode_entities(raw: &str) -> Result<String, PlistError> {
    if !raw.contains('&') {
        return Ok(raw.to_string());
    }
    let mut out = String::with_capacity(raw.len());
    let mut rest = raw;
    while let Some(index) = rest.find('&') {
        out.push_str(&rest[..index]);
        let tail = &rest[index + 1..];
        let end = tail
            .find(';')
            .ok_or_else(|| PlistError::Xml("unterminated entity reference".into()))?;
        let entity = &tail[..end];
        let replacement = match entity {
            "amp" => "&".to_string(),
            "lt" => "<".to_string(),
            "gt" => ">".to_string(),
            "quot" => "\"".to_string(),
            "apos" => "'".to_string(),
            _ => decode_numeric_entity(entity)?,
        };
        out.push_str(&replacement);
        rest = &tail[end + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

/// Decodes `&#NN;` and `&#xNN;` character references.
fn decode_numeric_entity(entity: &str) -> Result<String, PlistError> {
    let digits = entity
        .strip_prefix('#')
        .ok_or_else(|| PlistError::Xml(format!("unknown entity &{entity};")))?;
    let code = if let Some(hex) = digits
        .strip_prefix('x')
        .or_else(|| digits.strip_prefix('X'))
    {
        u32::from_str_radix(hex, 16)
            .map_err(|_| PlistError::Xml(format!("invalid character reference &{entity};")))?
    } else {
        digits
            .parse::<u32>()
            .map_err(|_| PlistError::Xml(format!("invalid character reference &{entity};")))?
    };
    let character = char::from_u32(code)
        .ok_or_else(|| PlistError::Xml(format!("invalid character reference &{entity};")))?;
    Ok(character.to_string())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    fn events(input: &str) -> Vec<Event> {
        let mut reader = XmlReader::new(input, 1024);
        let mut out = Vec::new();
        while let Some(event) = reader.next_event().unwrap_or(None) {
            out.push(event);
        }
        out
    }

    #[test]
    fn reads_elements_and_text() {
        let collected = events("<a>hi</a>");
        assert_eq!(
            collected,
            vec![
                Event::Start {
                    name: "a".into(),
                    self_closing: false
                },
                Event::Text("hi".into()),
                Event::End { name: "a".into() },
            ]
        );
    }

    #[test]
    fn reads_self_closing_elements() {
        let collected = events("<true/>");
        assert_eq!(
            collected,
            vec![Event::Start {
                name: "true".into(),
                self_closing: true
            }]
        );
    }

    #[test]
    fn skips_comments_and_processing_instructions() {
        let collected = events("<?xml version=\"1.0\"?><!-- hi --><a/>");
        assert_eq!(
            collected,
            vec![Event::Start {
                name: "a".into(),
                self_closing: true
            }]
        );
    }

    #[test]
    fn reads_cdata_verbatim() {
        let collected = events("<a><![CDATA[<raw>]]></a>");
        assert_eq!(collected[1], Event::Text("<raw>".into()));
    }

    #[test]
    fn resolves_entities() {
        let collected = events("<a>&lt;&amp;&gt;&#65;&#x42;</a>");
        assert_eq!(collected[1], Event::Text("<&>AB".into()));
    }

    #[test]
    fn rejects_an_internal_dtd_subset() {
        let mut reader = XmlReader::new("<!DOCTYPE plist [ <!ENTITY x \"y\"> ]>", 1024);
        assert!(matches!(
            reader.next_event(),
            Err(PlistError::XmlInternalSubset)
        ));
    }

    #[test]
    fn skips_a_plain_doctype() {
        let collected = events(
            "<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://example.invalid/d.dtd\"><a/>",
        );
        assert_eq!(
            collected,
            vec![Event::Start {
                name: "a".into(),
                self_closing: true
            }]
        );
    }

    #[test]
    fn rejects_an_unknown_entity() {
        let mut reader = XmlReader::new("<a>&nbsp;</a>", 1024);
        // The first event is the start tag; the text with the unknown
        // entity follows.
        assert!(reader.next_event().is_ok());
        assert!(reader.next_event().is_err());
    }

    #[test]
    fn rejects_unterminated_markup() {
        let mut reader = XmlReader::new("<a", 1024);
        assert!(reader.next_event().is_err());
    }

    #[test]
    fn enforces_the_text_limit() {
        let mut reader = XmlReader::new("<a>0123456789</a>", 4);
        assert!(reader.next_event().is_ok());
        assert!(matches!(
            reader.next_event(),
            Err(PlistError::StringTooLong { .. })
        ));
    }
}
