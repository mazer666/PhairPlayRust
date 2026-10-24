#![allow(clippy::expect_used, clippy::unwrap_used)]

use super::*;

fn identity() -> DeviceIdentity {
    let mut identity = DeviceIdentity::with_name("Living Room");
    identity.device_id = "0f9a7c11-2222-3333-4444-555566667777".to_string();
    identity
}

#[test]
fn the_airplay_record_carries_the_reference_constants() {
    let record = airplay_txt(&identity());
    assert_eq!(record.get("model"), Some(AIRPLAY_MODEL));
    assert_eq!(record.get("features"), Some(AIRPLAY_FEATURES));
    assert_eq!(record.get("srcvers"), Some(AIRPLAY_SERVER_VERSION));
    assert_eq!(record.get("vv"), Some(AIRPLAY_PROTOCOL_VERSION));
    assert_eq!(record.get("flags"), Some(AIRPLAY_FLAGS));
}

#[test]
fn the_airplay_device_id_is_stable_and_in_colon_notation() {
    let record = airplay_txt(&identity());
    assert_eq!(record.get("deviceid"), Some("0f:9a:7c:11:22:22"));
    let again = airplay_txt(&identity());
    assert_eq!(record.get("deviceid"), again.get("deviceid"));
}

#[test]
fn the_airplay_name_is_stripped_of_control_characters() {
    let mut identity = identity();
    identity.name = "Living\u{0} Room\n".to_string();
    let record = airplay_txt(&identity);
    assert_eq!(record.get("name"), Some("Living Room"));
}

#[test]
fn the_raop_record_carries_the_mandatory_version() {
    let record = raop_txt(&identity());
    assert_eq!(record.get("vn"), Some(RAOP_VERSION));
    assert_eq!(record.get("cn"), Some(RAOP_CIPHER_NUMBERS));
    assert_eq!(record.get("et"), Some(RAOP_ENCRYPTION_TYPES));
    assert_eq!(record.get("md"), Some(RAOP_METADATA_TYPES));
    assert_eq!(record.get("tp"), Some(RAOP_TRANSPORT));
}

#[test]
fn the_cast_record_carries_the_device_id_and_name() {
    let record = cast_txt(&identity());
    assert_eq!(
        record.get("id"),
        Some("0f9a7c11-2222-3333-4444-555566667777")
    );
    assert_eq!(record.get("fn"), Some("Living Room"));
    assert_eq!(record.get("ca"), Some(CAST_CAPABILITIES));
}

#[test]
fn the_miracast_record_carries_the_friendly_name() {
    let record = miracast_txt(&identity());
    assert_eq!(record.get("friendlyname"), Some("Living Room"));
    assert_eq!(record.get("txtvers"), Some(MIRACAST_TXT_VERSION));
}

#[test]
fn a_record_key_with_an_equals_sign_is_rejected() {
    let mut record = TxtRecord::new();
    record.set("bad=key", "value");
    assert!(record.entries().iter().all(|(k, _)| !k.contains('=')));
    assert!(record.get("badkey").is_some());
}

#[test]
fn a_record_value_with_control_characters_is_stripped() {
    let mut record = TxtRecord::new();
    record.set("k", "a\u{0}b\u{7f}c");
    assert_eq!(record.get("k"), Some("abc"));
}

#[test]
fn setting_the_same_key_twice_replaces_the_value() {
    let mut record = TxtRecord::new();
    record.set("k", "one");
    record.set("k", "two");
    assert_eq!(record.get("k"), Some("two"));
    assert_eq!(record.len(), 1);
}

#[test]
fn an_empty_record_is_empty() {
    let record = TxtRecord::new();
    assert!(record.is_empty());
    assert_eq!(record.len(), 0);
}

#[test]
fn the_audio_channel_record_is_advertised() {
    assert_eq!(airplay_txt(&identity()).get("ch"), Some(AUDIO_CHANNELS));
    assert_eq!(raop_txt(&identity()).get("ch"), Some(AUDIO_CHANNELS));
}
