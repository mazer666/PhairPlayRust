
use super::*;

#[test]
fn a_dictionary_answers_lookups() {
    let document = PlistValue::dictionary(vec![("a".into(), PlistValue::Integer(1))]);
    assert_eq!(document.get("a").and_then(PlistValue::as_integer), Some(1));
    assert!(document.get("b").is_none());
}

#[test]
fn the_accessors_only_answer_their_own_type() {
    let value = PlistValue::String("text".into());
    assert_eq!(value.as_str(), Some("text"));
    assert!(value.as_integer().is_none());
    assert!(value.as_data().is_none());
    assert!(value.as_array().is_none());
}

#[test]
fn a_nested_value_can_be_reached() {
    let document = PlistValue::dictionary(vec![(
        "streams".into(),
        PlistValue::array(vec![PlistValue::dictionary(vec![(
            "type".into(),
            PlistValue::Integer(110),
        )])]),
    )]);
    let streams = document.get("streams").and_then(PlistValue::as_array);
    let first = streams.and_then(|items| items.first());
    assert_eq!(
        first.and_then(|item| item.get("type")).and_then(PlistValue::as_integer),
        Some(110)
    );
}
