//! The value model of Apple property lists.
//!
//! WHY: the AirPlay 2 handshake exchanges structured data — the receiver's
//! capabilities, the sender's stream description, the pairing parameters —
//! as property lists. A typed value model keeps the encoders and decoders
//! free of `Any`, and keeps the *shape* of a document visible in the type
//! system.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_airplay::plist::PlistValue;
//!
//! let document = PlistValue::Dictionary(vec![
//!     ("name".into(), PlistValue::String("Living Room".into())),
//!     ("port".into(), PlistValue::Integer(7000)),
//! ]);
//! assert_eq!(document.get("name").and_then(PlistValue::as_str), Some("Living Room"));
//! ```

/// One value of a property list.
#[derive(Debug, Clone, PartialEq)]
pub enum PlistValue {
    /// A string.
    String(String),
    /// A signed integer.
    Integer(i64),
    /// A floating point number.
    Real(f64),
    /// A boolean.
    Boolean(bool),
    /// A byte string.
    Data(Vec<u8>),
    /// An array of values.
    Array(Vec<PlistValue>),
    /// A dictionary of values, in insertion order.
    Dictionary(Vec<(String, PlistValue)>),
}

impl PlistValue {
    /// Builds a dictionary from a list of entries.
    #[must_use]
    pub fn dictionary(entries: Vec<(String, PlistValue)>) -> Self {
        Self::Dictionary(entries)
    }

    /// Builds an array.
    #[must_use]
    pub fn array(values: Vec<PlistValue>) -> Self {
        Self::Array(values)
    }

    /// The string value, if the value is a string.
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    /// The integer value, if the value is an integer.
    #[must_use]
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Self::Integer(value) => Some(*value),
            _ => None,
        }
    }

    /// The floating point value, if the value is a real number.
    #[must_use]
    pub fn as_real(&self) -> Option<f64> {
        match self {
            Self::Real(value) => Some(*value),
            _ => None,
        }
    }

    /// The boolean value, if the value is a boolean.
    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Boolean(value) => Some(*value),
            _ => None,
        }
    }

    /// The byte string, if the value is data.
    #[must_use]
    pub fn as_data(&self) -> Option<&[u8]> {
        match self {
            Self::Data(value) => Some(value),
            _ => None,
        }
    }

    /// The array, if the value is an array.
    #[must_use]
    pub fn as_array(&self) -> Option<&[PlistValue]> {
        match self {
            Self::Array(values) => Some(values),
            _ => None,
        }
    }

    /// The dictionary, if the value is a dictionary.
    #[must_use]
    pub fn as_dictionary(&self) -> Option<&[(String, PlistValue)]> {
        match self {
            Self::Dictionary(entries) => Some(entries),
            _ => None,
        }
    }

    /// Looks a key up in a dictionary.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&PlistValue> {
        self.as_dictionary()?
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value)
    }
}

impl From<&str> for PlistValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<String> for PlistValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<i64> for PlistValue {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<bool> for PlistValue {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

#[cfg(test)]
mod tests;
