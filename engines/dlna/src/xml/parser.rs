//! The scanner of the XML reader.
//!
//! WHY: the reader has to be strict, and a strict reader is a scanner that
//! knows exactly what it accepts: elements, attributes, text, CDATA
//! sections, comments and processing instructions — and nothing else. The
//! scanner is where that knowledge lives.
//!
//! HOW: the scanner works on bytes and never materialises a token before it
//! has checked its length. A document type is rejected the moment it is
//! seen, so that no entity can ever be declared.

use super::{
    MAX_XML_BYTES, MAX_XML_DEPTH, MAX_XML_NODES, XmlDocument, XmlElement, XmlError, XmlNode,
    decode_entities, local_part,
};

/// The scanner.
pub(super) struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
    nodes: usize,
}

impl<'a> Parser<'a> {
    pub(super) fn new(input: &'a str) -> Result<Self, XmlError> {
        if input.len() > MAX_XML_BYTES {
            return Err(XmlError::TooLarge {
                limit: MAX_XML_BYTES,
            });
        }
        Ok(Self {
            bytes: input.as_bytes(),
            pos: 0,
            nodes: 0,
        })
    }

    pub(super) fn parse(mut self) -> Result<XmlDocument, XmlError> {
        self.skip_prolog()?;
        let root = self.parse_element(1)?;
        self.skip_trailing()?;
        Ok(XmlDocument { root })
    }

    /// Skips the prolog: whitespace, comments and processing instructions.
    fn skip_prolog(&mut self) -> Result<(), XmlError> {
        loop {
            self.skip_whitespace();
            if self.starts_with("<!--") {
                self.skip_comment()?;
            } else if self.starts_with("<?") {
                self.skip_processing_instruction()?;
            } else if self.starts_with("<!") {
                return Err(self.reject_declaration());
            } else {
                return Ok(());
            }
        }
    }

    /// Rejects everything that may follow the root element except noise.
    fn skip_trailing(&mut self) -> Result<(), XmlError> {
        loop {
            self.skip_whitespace();
            if self.pos >= self.bytes.len() {
                return Ok(());
            }
            if self.starts_with("<!--") {
                self.skip_comment()?;
            } else if self.starts_with("<?") {
                self.skip_processing_instruction()?;
            } else {
                return Err(XmlError::Malformed(
                    "there is content after the root element".to_string(),
                ));
            }
        }
    }

    /// Parses one element and everything below it.
    fn parse_element(&mut self, depth: usize) -> Result<XmlElement, XmlError> {
        if depth > MAX_XML_DEPTH {
            return Err(XmlError::DepthExceeded {
                limit: MAX_XML_DEPTH,
            });
        }
        self.expect(b'<')?;
        let (qname, local_name) = self.parse_name()?;
        self.nodes += 1;
        if self.nodes > MAX_XML_NODES {
            return Err(XmlError::TooManyNodes {
                limit: MAX_XML_NODES,
            });
        }

        let mut attributes = Vec::new();
        loop {
            self.skip_whitespace();
            if self.starts_with("/>") {
                self.pos += 2;
                return Ok(XmlElement {
                    qname,
                    local_name,
                    attributes,
                    children: Vec::new(),
                });
            }
            if self.starts_with(">") {
                self.pos += 1;
                break;
            }
            let (attr_name, _) = self.parse_name()?;
            self.skip_whitespace();
            self.expect(b'=')?;
            self.skip_whitespace();
            let value = self.parse_attribute_value()?;
            attributes.push((attr_name, value));
        }

        let mut children = Vec::new();
        loop {
            if self.pos >= self.bytes.len() {
                return Err(XmlError::Malformed("the element is not closed".to_string()));
            }
            if self.starts_with("</") {
                self.pos += 2;
                let (end_name, _) = self.parse_name()?;
                if end_name != qname {
                    return Err(XmlError::Malformed(format!(
                        "the end tag `{end_name}` does not match the start tag `{qname}`"
                    )));
                }
                self.skip_whitespace();
                self.expect(b'>')?;
                return Ok(XmlElement {
                    qname,
                    local_name,
                    attributes,
                    children,
                });
            }
            if self.starts_with("<!--") {
                self.skip_comment()?;
                continue;
            }
            if self.starts_with("<?") {
                self.skip_processing_instruction()?;
                continue;
            }
            if self.starts_with("<![CDATA[") {
                let text = self.parse_cdata()?;
                children.push(XmlNode::Text(text));
                continue;
            }
            if self.starts_with("<!") {
                return Err(self.reject_declaration());
            }
            if self.starts_with("<") {
                let child = self.parse_element(depth + 1)?;
                children.push(XmlNode::Element(child));
                continue;
            }
            let text = self.parse_text()?;
            children.push(XmlNode::Text(text));
        }
    }

    /// Turns a `<!...` construct into the matching error.
    ///
    /// WHY: a DOCTYPE is the entry point of every entity based attack, so it
    /// gets its own error code — the log has to show *why* a document was
    /// rejected.
    fn reject_declaration(&self) -> XmlError {
        let rest = &self.bytes[self.pos..];
        let head: String = rest
            .iter()
            .take(16)
            .map(|b| char::from(*b).to_ascii_lowercase())
            .collect();
        if head.starts_with("<!doctype") {
            XmlError::DoctypeForbidden
        } else if head.starts_with("<!entity") {
            XmlError::EntityForbidden
        } else {
            XmlError::Malformed("markup declarations are not allowed".to_string())
        }
    }

    /// Parses an element or attribute name.
    fn parse_name(&mut self) -> Result<(String, String), XmlError> {
        let start = self.pos;
        while self.pos < self.bytes.len() && is_name_byte(self.bytes[self.pos]) {
            self.pos += 1;
        }
        if self.pos == start {
            return Err(XmlError::Malformed("a name was expected".to_string()));
        }
        let qname = self.text(start, self.pos)?;
        let local_name = local_part(&qname).to_string();
        Ok((qname, local_name))
    }

    /// Parses a quoted attribute value.
    fn parse_attribute_value(&mut self) -> Result<String, XmlError> {
        let quote = match self.bytes.get(self.pos) {
            Some(b'"') => b'"',
            Some(b'\'') => b'\'',
            _ => {
                return Err(XmlError::Malformed(
                    "an attribute value must be quoted".to_string(),
                ));
            }
        };
        self.pos += 1;
        let start = self.pos;
        while self.pos < self.bytes.len() && self.bytes[self.pos] != quote {
            if self.bytes[self.pos] == b'<' {
                return Err(XmlError::Malformed(
                    "an attribute value must not contain `<`".to_string(),
                ));
            }
            self.pos += 1;
        }
        if self.pos >= self.bytes.len() {
            return Err(XmlError::Malformed(
                "the attribute value is not closed".to_string(),
            ));
        }
        let raw = self.text(start, self.pos)?;
        self.pos += 1;
        decode_entities(&raw)
    }

    /// Parses character data.
    fn parse_text(&mut self) -> Result<String, XmlError> {
        let start = self.pos;
        while self.pos < self.bytes.len() && self.bytes[self.pos] != b'<' {
            self.pos += 1;
        }
        let raw = self.text(start, self.pos)?;
        decode_entities(&raw)
    }

    /// Parses a CDATA section. Its content is literal.
    fn parse_cdata(&mut self) -> Result<String, XmlError> {
        self.pos += "<![CDATA[".len();
        let start = self.pos;
        while self.pos < self.bytes.len() && !self.starts_with("]]>") {
            self.pos += 1;
        }
        if self.pos >= self.bytes.len() {
            return Err(XmlError::Malformed(
                "the CDATA section is not closed".to_string(),
            ));
        }
        let text = self.text(start, self.pos)?;
        self.pos += 3;
        Ok(text)
    }

    /// Skips a comment.
    fn skip_comment(&mut self) -> Result<(), XmlError> {
        self.pos += "<!--".len();
        while self.pos < self.bytes.len() && !self.starts_with("-->") {
            self.pos += 1;
        }
        if self.pos >= self.bytes.len() {
            return Err(XmlError::Malformed("the comment is not closed".to_string()));
        }
        self.pos += 3;
        Ok(())
    }

    /// Skips a processing instruction.
    fn skip_processing_instruction(&mut self) -> Result<(), XmlError> {
        self.pos += "<?".len();
        while self.pos < self.bytes.len() && !self.starts_with("?>") {
            self.pos += 1;
        }
        if self.pos >= self.bytes.len() {
            return Err(XmlError::Malformed(
                "the processing instruction is not closed".to_string(),
            ));
        }
        self.pos += 2;
        Ok(())
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn starts_with(&self, needle: &str) -> bool {
        self.bytes[self.pos..].starts_with(needle.as_bytes())
    }

    fn expect(&mut self, byte: u8) -> Result<(), XmlError> {
        if self.bytes.get(self.pos) == Some(&byte) {
            self.pos += 1;
            Ok(())
        } else {
            Err(XmlError::Malformed(format!(
                "`{}` was expected",
                char::from(byte)
            )))
        }
    }

    fn text(&self, start: usize, end: usize) -> Result<String, XmlError> {
        std::str::from_utf8(&self.bytes[start..end])
            .map(str::to_string)
            .map_err(|_| XmlError::Malformed("the document is not valid UTF-8".to_string()))
    }
}

/// Whether a byte may appear in an XML name.
fn is_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b':' | b'.') || byte >= 0x80
}
