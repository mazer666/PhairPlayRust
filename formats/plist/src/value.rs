// WHAT: the value model of a property list.
// WHY: AirPlay and Google Cast exchange plists whose contents are
//      dictionaries, arrays, strings, numbers, booleans, data blobs and
//      dates. A single owned, order-preserving tree keeps the codec
//      free of lifetimes and keeps the encoders deterministic.
// EXAMPLE: a `Value::Dictionary` keeps its keys in insertion order, so
//          encoding the same value twice produces the same bytes.

use std::collections::BTreeMap;

/// A property-list value.
///
/// The variants mirror the eight types the Apple property-list format
/// defines. Dictionaries keep their keys in insertion order; a lookup
/// helper is provided for callers that do not care about the order.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// A boolean.
    Boolean(bool),
    /// A signed integer.
    Integer(i64),
    /// A floating point number.
    Real(f64),
    /// A text string.
    String(String),
    /// A blob of bytes.
    Data(Vec<u8>),
    /// A point in time, in seconds since the Apple epoch (2001-01-01 UTC).
    Date(f64),
    /// An ordered list of values.
    Array(Vec<Value>),
    /// An ordered list of key/value pairs.
    Dictionary(Vec<(String, Value)>),
}

impl Value {
    /// Returns the value as a dictionary, if it is one.
    ///
    /// EXAMPLE: `Value::Dictionary(v).as_dictionary()` is `Some(&v)`.
    pub fn as_dictionary(&self) -> Option<&[(String, Value)]> {
        match self {
            Value::Dictionary(entries) => Some(entries),
            _ => None,
        }
    }

    /// Returns the value as an array, if it is one.
    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            Value::Array(items) => Some(items),
            _ => None,
        }
    }

    /// Returns the value as a string slice, if it is one.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(text) => Some(text),
            _ => None,
        }
    }

    /// Returns the value as a boolean, if it is one.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Boolean(flag) => Some(*flag),
            _ => None,
        }
    }

    /// Returns the value as an integer, if it is one.
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Value::Integer(number) => Some(*number),
            _ => None,
        }
    }

    /// Returns the value as a real number, if it is one.
    ///
    /// An integer is *not* converted: a plist that says `<integer>` stays
    /// an integer, because the two types are distinguished on the wire.
    pub fn as_real(&self) -> Option<f64> {
        match self {
            Value::Real(number) => Some(*number),
            _ => None,
        }
    }

    /// Returns the value as a data blob, if it is one.
    pub fn as_data(&self) -> Option<&[u8]> {
        match self {
            Value::Data(bytes) => Some(bytes),
            _ => None,
        }
    }

    /// Returns the value as a date, in seconds since the Apple epoch.
    pub fn as_date(&self) -> Option<f64> {
        match self {
            Value::Date(seconds) => Some(*seconds),
            _ => None,
        }
    }

    /// Looks a key up in a dictionary.
    ///
    /// WHY: the dictionary is an ordered list, so a lookup has to scan it.
    ///      The tables used by the receiver hold a handful of keys, and a
    ///      scan keeps the representation simple and order-preserving.
    ///
    /// EXAMPLE: `Value::Dictionary(entries).get("name")` returns the value.
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Dictionary(entries) => entries
                .iter()
                .find(|(name, _)| name == key)
                .map(|(_, value)| value),
            _ => None,
        }
    }

    /// Builds a dictionary from an ordered map of entries.
    ///
    /// EXAMPLE: `Value::dictionary([("a".into(), Value::Boolean(true))])`.
    pub fn dictionary<I, K>(entries: I) -> Value
    where
        I: IntoIterator<Item = (K, Value)>,
        K: Into<String>,
    {
        Value::Dictionary(
            entries
                .into_iter()
                .map(|(key, value)| (key.into(), value))
                .collect(),
        )
    }

    /// Returns the number of value nodes in the tree.
    ///
    /// WHY: the encoder uses it to size its object table before it starts
    ///      to write, and the tests use it to check that a decoded tree is
    ///      the size the peer announced.
    pub fn node_count(&self) -> usize {
        match self {
            Value::Array(items) => 1 + items.iter().map(Value::node_count).sum::<usize>(),
            Value::Dictionary(entries) => {
                1 + entries
                    .iter()
                    .map(|(_, value)| value.node_count())
                    .sum::<usize>()
            }
            _ => 1,
        }
    }

    /// Returns the nesting depth of the tree; a leaf has depth 1.
    pub fn depth(&self) -> usize {
        match self {
            Value::Array(items) => 1 + items.iter().map(Value::depth).max().unwrap_or(0),
            Value::Dictionary(entries) => {
                1 + entries
                    .iter()
                    .map(|(_, value)| value.depth())
                    .max()
                    .unwrap_or(0)
            }
            _ => 1,
        }
    }

    /// Returns the dictionary entries sorted by key, for callers that need
    /// a canonical order.
    pub fn sorted_dictionary(&self) -> BTreeMap<&str, &Value> {
        match self {
            Value::Dictionary(entries) => entries
                .iter()
                .map(|(key, value)| (key.as_str(), value))
                .collect(),
            _ => BTreeMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Value {
        Value::dictionary([
            ("name", Value::String("Living Room".into())),
            ("volume", Value::Integer(42)),
            ("muted", Value::Boolean(false)),
        ])
    }

    #[test]
    fn accessors_report_the_matching_variant_only() {
        assert_eq!(sample().as_str(), None);
        assert_eq!(Value::String("x".into()).as_str(), Some("x"));
        assert_eq!(Value::Integer(7).as_integer(), Some(7));
        assert_eq!(Value::Real(1.5).as_real(), Some(1.5));
        assert_eq!(Value::Boolean(true).as_bool(), Some(true));
        assert_eq!(Value::Data(vec![1, 2]).as_data(), Some(&[1u8, 2][..]));
        assert_eq!(Value::Date(0.0).as_date(), Some(0.0));
        assert_eq!(Value::Array(vec![]).as_array(), Some(&[][..]));
    }

    #[test]
    fn integers_are_not_silently_turned_into_reals() {
        assert_eq!(Value::Integer(3).as_real(), None);
    }

    #[test]
    fn get_finds_a_key_in_a_dictionary() {
        assert_eq!(sample().get("volume"), Some(&Value::Integer(42)));
        assert_eq!(sample().get("missing"), None);
        assert_eq!(Value::Integer(1).get("volume"), None);
    }

    #[test]
    fn dictionary_preserves_insertion_order() {
        let entries = sample()
            .as_dictionary()
            .map(<[(String, Value)]>::to_vec)
            .unwrap_or_default();
        let keys: Vec<&str> = entries.iter().map(|(key, _)| key.as_str()).collect();
        assert_eq!(keys, vec!["name", "volume", "muted"]);
    }

    #[test]
    fn node_count_and_depth_describe_the_tree() {
        let tree = Value::Array(vec![
            Value::Integer(1),
            Value::Array(vec![Value::Integer(2)]),
        ]);
        assert_eq!(tree.node_count(), 4);
        assert_eq!(tree.depth(), 3);
        assert_eq!(Value::Integer(1).node_count(), 1);
        assert_eq!(Value::Integer(1).depth(), 1);
    }

    #[test]
    fn sorted_dictionary_orders_keys() {
        let tree = Value::dictionary([("b", Value::Integer(1)), ("a", Value::Integer(2))]);
        let sorted = tree.sorted_dictionary();
        let keys: Vec<&str> = sorted.keys().copied().collect();
        assert_eq!(keys, vec!["a", "b"]);
        assert!(Value::Integer(0).sorted_dictionary().is_empty());
    }
}
