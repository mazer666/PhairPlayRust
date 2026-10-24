// WHAT: the single error type of the property-list codec.
// WHY: every rejection of untrusted input must be explainable, and the
//      receiver stack reports the reason to the peer log. One enum keeps
//      the error surface small and comparable.
// EXAMPLE: a binary plist whose trailer announces 2^40 objects produces
//          `PlistError::TooManyObjects { count, limit }` instead of an
//          allocation of unknown size.

use thiserror::Error;

/// Everything that can go wrong while reading or writing a property list.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PlistError {
    /// The input starts with neither `bplist00` nor an XML property list.
    #[error("input is neither a binary nor an XML property list")]
    UnknownFormat,

    /// The input ended before a structure was complete.
    #[error("property list is truncated at offset {offset}")]
    Truncated {
        /// Byte offset at which the input ran out.
        offset: usize,
    },

    /// The binary header is not `bplist00`.
    #[error("binary property list header is not 'bplist00'")]
    BadBinaryMagic,

    /// The 32-byte trailer could not be interpreted.
    #[error("binary property list trailer is malformed: {reason}")]
    BadTrailer {
        /// Human readable reason.
        reason: &'static str,
    },

    /// The trailer announces an offset or reference width outside `1..=8`.
    #[error("binary property list announces an illegal {what} width of {value}")]
    BadWidth {
        /// Which width it was: `offset` or `object reference`.
        what: &'static str,
        /// The announced width in bytes.
        value: u8,
    },

    /// The trailer announces more objects than the configured limit allows.
    #[error("binary property list announces {count} objects, the limit is {limit}")]
    TooManyObjects {
        /// The announced object count.
        count: u64,
        /// The configured limit.
        limit: usize,
    },

    /// An object index points outside the object table.
    #[error("object reference {index} is outside the object table of {count} objects")]
    ObjectIndexOutOfRange {
        /// The offending index.
        index: u64,
        /// The number of objects in the table.
        count: usize,
    },

    /// The object marker nibble does not name a known object type.
    #[error("object marker 0x{marker:02x} is not a known property-list object type")]
    UnknownObjectType {
        /// The raw marker byte.
        marker: u8,
    },

    /// A length field announces more bytes than the input contains.
    #[error("length field announces {length} bytes but only {remaining} remain")]
    LengthExceedsInput {
        /// The announced length.
        length: u64,
        /// The number of bytes actually available.
        remaining: usize,
    },

    /// A container is larger than the configured limit.
    #[error("container of {length} elements exceeds the limit of {limit}")]
    ContainerTooLarge {
        /// The announced element count.
        length: u64,
        /// The configured limit.
        limit: usize,
    },

    /// A string is longer than the configured limit.
    #[error("string of {length} bytes exceeds the limit of {limit}")]
    StringTooLong {
        /// The announced length in bytes.
        length: u64,
        /// The configured limit.
        limit: usize,
    },

    /// A data blob is longer than the configured limit.
    #[error("data blob of {length} bytes exceeds the limit of {limit}")]
    DataTooLarge {
        /// The announced length in bytes.
        length: u64,
        /// The configured limit.
        limit: usize,
    },

    /// The nesting is deeper than the configured limit.
    #[error("property list nests deeper than the limit of {limit}")]
    DepthLimitExceeded {
        /// The configured limit.
        limit: usize,
    },

    /// The object graph contains a cycle.
    #[error("property list object {index} refers to itself through a cycle")]
    CyclicReference {
        /// The object index at which the cycle was detected.
        index: u64,
    },

    /// The expansion of shared references exceeded the node budget.
    #[error("property list expands to more than {limit} nodes")]
    NodeBudgetExceeded {
        /// The configured budget.
        limit: usize,
    },

    /// An integer does not fit into the supported range.
    #[error("integer does not fit into a signed 64-bit value")]
    IntegerOutOfRange,

    /// A string is not valid UTF-8 or UTF-16.
    #[error("string is not valid {encoding}")]
    InvalidTextEncoding {
        /// The encoding that failed: `UTF-8` or `UTF-16`.
        encoding: &'static str,
    },

    /// The XML document is malformed.
    #[error("XML error: {0}")]
    Xml(String),

    /// The XML document declares an internal DTD subset.
    ///
    /// WHY: an internal subset is the entry point of the billion-laughs
    ///      entity-expansion attack. The decoder never expands entities, so
    ///      it refuses the document instead of silently ignoring it.
    #[error("XML document declares an internal DTD subset, which is not accepted")]
    XmlInternalSubset,

    /// The XML document contains an element the decoder does not know.
    #[error("unexpected XML element <{name}>")]
    UnexpectedXmlElement {
        /// The element name.
        name: String,
    },

    /// The XML document is missing a required child element.
    #[error("XML element <{name}> is missing a <{expected}> child")]
    MissingXmlChild {
        /// The element that lacks the child.
        name: String,
        /// The child that was expected.
        expected: &'static str,
    },

    /// The XML document has trailing content after the root element.
    #[error("XML document has content after the root element")]
    TrailingXmlContent,

    /// The value cannot be represented in the requested encoding.
    #[error("cannot encode the value: {0}")]
    Encode(String),

    /// The object type is defined by Apple but this codec does not support it.
    #[error("the property-list object {description} is not supported")]
    UnsupportedObject {
        /// What the object was.
        description: &'static str,
    },
}

impl PlistError {
    /// Returns `true` when the error is a resource-limit rejection.
    ///
    /// WHY: the engine layer maps limit rejections onto a dedicated
    ///      transport error, so it must be able to recognise them.
    ///
    /// EXAMPLE: `PlistError::TooManyObjects { .. }.is_limit_exceeded()` is `true`.
    pub fn is_limit_exceeded(&self) -> bool {
        matches!(
            self,
            PlistError::TooManyObjects { .. }
                | PlistError::ContainerTooLarge { .. }
                | PlistError::StringTooLong { .. }
                | PlistError::DataTooLarge { .. }
                | PlistError::DepthLimitExceeded { .. }
                | PlistError::NodeBudgetExceeded { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    #[test]
    fn limit_errors_are_recognised() {
        assert!(PlistError::TooManyObjects { count: 1, limit: 2 }.is_limit_exceeded());
        assert!(PlistError::DepthLimitExceeded { limit: 3 }.is_limit_exceeded());
        assert!(!PlistError::UnknownFormat.is_limit_exceeded());
        assert!(!PlistError::BadBinaryMagic.is_limit_exceeded());
    }

    #[test]
    fn messages_name_the_offending_value() {
        let err = PlistError::LengthExceedsInput {
            length: 4_000_000_000,
            remaining: 12,
        };
        let text = err.to_string();
        assert!(text.contains("4000000000"), "{text}");
        assert!(text.contains("12"), "{text}");
    }
}
