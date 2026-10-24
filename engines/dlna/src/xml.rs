//! A deliberately small XML reader for the UPnP control plane.
//!
//! WHY: a SOAP body arrives from an arbitrary device on the LAN. A parser
//! that resolves DTDs hands that device a file reader (XXE), a parser that
//! expands entities hands it a memory bomb (the "billion laughs" attack),
//! and a parser that recurses without a limit hands it a stack overflow.
//! The reference implementation solved this with the flags of the Java XML
//! factory; in Rust the reliable answer is a parser that simply *cannot*
//! do any of those things.
//!
//! HOW: this parser accepts elements, attributes, text, CDATA sections,
//! comments and processing instructions — and nothing else:
//!
//! * a `<!DOCTYPE` declaration is **rejected**, so no entity can ever be
//!   declared and no external document can ever be referenced;
//! * the only entities that are expanded are the five that XML defines
//!   itself plus numeric character references — an unknown entity is an
//!   error, not a literal;
//! * the nesting depth and the number of elements are bounded;
//! * the size of the document is bounded.
//!
//! A parser that rejects is the only parser that is safe here: a repaired
//! document is a document the sender did not send.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_dlna::xml::XmlDocument;
//!
//! let document = XmlDocument::parse(
//!     r#"<s:Envelope xmlns:s="urn:x"><s:Body><u:Play/></s:Body></s:Envelope>"#,
//! )
//! .expect("a well-formed document");
//! assert_eq!(document.root().local_name(), "Envelope");
//! ```

/// The maximum size of a document this parser accepts.
pub const MAX_XML_BYTES: usize = 64 * 1024;

/// The maximum nesting depth this parser accepts.
pub const MAX_XML_DEPTH: usize = 32;

/// The maximum number of elements this parser accepts.
pub const MAX_XML_NODES: usize = 4096;

/// The reasons a document can be rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XmlError {
    /// The document is larger than [`MAX_XML_BYTES`].
    TooLarge {
        /// The limit that was exceeded.
        limit: usize,
    },

    /// The document carries a `<!DOCTYPE>` declaration.
    DoctypeForbidden,

    /// The document carries an `<!ENTITY>` declaration.
    EntityForbidden,

    /// The document references an entity that XML does not define.
    UnknownEntity(String),

    /// The document nests deeper than [`MAX_XML_DEPTH`].
    DepthExceeded {
        /// The limit that was exceeded.
        limit: usize,
    },

    /// The document has more elements than [`MAX_XML_NODES`].
    TooManyNodes {
        /// The limit that was exceeded.
        limit: usize,
    },

    /// The document is not well-formed.
    Malformed(String),
}

impl std::fmt::Display for XmlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooLarge { limit } => {
                write!(f, "the XML document exceeds the limit of {limit} bytes")
            }
            Self::DoctypeForbidden => write!(
                f,
                "the XML document carries a DOCTYPE declaration, which is not allowed"
            ),
            Self::EntityForbidden => write!(
                f,
                "the XML document carries an entity declaration, which is not allowed"
            ),
            Self::UnknownEntity(name) => {
                write!(f, "the XML document references the unknown entity `{name}`")
            }
            Self::DepthExceeded { limit } => {
                write!(f, "the XML document nests deeper than {limit} levels")
            }
            Self::TooManyNodes { limit } => {
                write!(f, "the XML document has more than {limit} elements")
            }
            Self::Malformed(reason) => write!(f, "the XML document is malformed: {reason}"),
        }
    }
}

impl std::error::Error for XmlError {}

/// One element of a parsed document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlElement {
    qname: String,
    local_name: String,
    attributes: Vec<(String, String)>,
    children: Vec<XmlNode>,
}

/// A node of a parsed document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XmlNode {
    /// A child element.
    Element(XmlElement),
    /// A run of character data.
    Text(String),
}

impl XmlElement {
    /// The name of the element without its namespace prefix.
    #[must_use]
    pub fn local_name(&self) -> &str {
        &self.local_name
    }

    /// The name of the element including its namespace prefix.
    #[must_use]
    pub fn qualified_name(&self) -> &str {
        &self.qname
    }

    /// The value of an attribute, matched without the namespace prefix.
    #[must_use]
    pub fn attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(key, _)| local_part(key) == name)
            .map(|(_, value)| value.as_str())
    }

    /// The direct child elements.
    pub fn children(&self) -> impl Iterator<Item = &XmlElement> {
        self.children.iter().filter_map(|node| match node {
            XmlNode::Element(element) => Some(element),
            XmlNode::Text(_) => None,
        })
    }

    /// The first direct child element with the given local name.
    #[must_use]
    pub fn child(&self, local_name: &str) -> Option<&XmlElement> {
        self.children().find(|e| e.local_name() == local_name)
    }

    /// The character data of the direct text children, concatenated.
    #[must_use]
    pub fn text(&self) -> String {
        let mut out = String::new();
        for node in &self.children {
            if let XmlNode::Text(text) = node {
                out.push_str(text);
            }
        }
        out
    }

    /// The first descendant element with the given local name and its text.
    #[must_use]
    pub fn descendant_text(&self, local_name: &str) -> Option<String> {
        if self.local_name == local_name {
            return Some(self.text());
        }
        for child in self.children() {
            if let Some(text) = child.descendant_text(local_name) {
                return Some(text);
            }
        }
        None
    }
}

/// A parsed document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlDocument {
    root: XmlElement,
}

impl XmlDocument {
    /// Parses a document.
    ///
    /// # Errors
    /// Returns [`XmlError`] when the document is too large, too deep, too
    /// rich in elements, carries a DTD or references an unknown entity.
    pub fn parse(input: &str) -> Result<Self, XmlError> {
        Parser::new(input)?.parse()
    }

    /// The root element.
    #[must_use]
    pub fn root(&self) -> &XmlElement {
        &self.root
    }
}

mod parser;

use parser::Parser;

/// Expands the entities XML defines itself.
///
/// WHY: this is the whole XXE defence. A document that wants an external
/// entity has to declare it, and a declaration is a DOCTYPE — which the
/// parser rejects before it ever gets here. An entity that is not one of
/// the five predefined ones is an error, never a literal: silently passing
/// `&xxe;` through would let the *application* resolve it later.
///
/// The part of a qualified name after the colon.
fn local_part(name: &str) -> &str {
    name.rsplit_once(':').map_or(name, |(_, local)| local)
}

/// `&xxe;` through would let the *application* resolve it later.
fn decode_entities(raw: &str) -> Result<String, XmlError> {
    if !raw.contains('&') {
        return Ok(raw.to_string());
    }
    let mut out = String::with_capacity(raw.len());
    let mut rest = raw;
    while let Some(index) = rest.find('&') {
        out.push_str(&rest[..index]);
        let tail = &rest[index + 1..];
        let Some(end) = tail.find(';') else {
            return Err(XmlError::Malformed(
                "an entity reference is not terminated".to_string(),
            ));
        };
        let name = &tail[..end];
        match name {
            "amp" => out.push('&'),
            "lt" => out.push('<'),
            "gt" => out.push('>'),
            "quot" => out.push('"'),
            "apos" => out.push('\''),
            _ => {
                let character = parse_character_reference(name)
                    .ok_or_else(|| XmlError::UnknownEntity(name.to_string()))?;
                out.push(character);
            }
        }
        rest = &tail[end + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

/// Parses `&#NN;` and `&#xHH;`.
fn parse_character_reference(name: &str) -> Option<char> {
    let digits = name.strip_prefix('#')?;
    let value = if let Some(hex) = digits
        .strip_prefix('x')
        .or_else(|| digits.strip_prefix('X'))
    {
        u32::from_str_radix(hex, 16).ok()?
    } else {
        digits.parse::<u32>().ok()?
    };
    let scalar = char::from_u32(value)?;
    // A surrogate is not a character; XML forbids it in a document.
    if scalar.is_control() && scalar != '\t' && scalar != '\n' && scalar != '\r' {
        return None;
    }
    Some(scalar)
}

#[cfg(test)]
mod tests;
