#![allow(clippy::expect_used, clippy::unwrap_used)]

use super::*;

#[test]
fn the_airplay_pair_is_complete() {
    // A receiver that publishes only one of the two is invisible to
    // some senders — this test pins the pair.
    assert!(is_known_service_type(AIRPLAY_TCP));
    assert!(is_known_service_type(RAOP_TCP));
    // WHY: the count is compared with the core, and not with a literal —
    // a new protocol must not break a test that is about the AirPlay pair.
    assert_eq!(
        ALL_SERVICE_TYPES.len(),
        phairplay_core::config::service_types::ALL.len()
    );
}

#[test]
fn the_smart_view_type_is_known() {
    // WHY: a Galaxy phone looks for exactly this type. A receiver that does
    // not publish it is invisible to every Samsung sender.
    assert!(is_known_service_type(SAMSUNG_MSF_TCP));
}

#[test]
fn an_unknown_service_type_is_not_known() {
    assert!(!is_known_service_type("_printer._tcp"));
    assert!(!is_known_service_type(""));
}

#[test]
fn the_raop_name_carries_the_mac_prefix() {
    let name = raop_instance_name("aabbccddeeff", "Living Room");
    assert_eq!(name, "AABBCCDDEEFF@Living Room");
}

#[test]
fn the_raop_name_is_capped_at_the_dns_sd_limit() {
    let long = "x".repeat(200);
    let name = raop_instance_name("aabbccddeeff", &long);
    assert_eq!(name.chars().count(), MAX_SERVICE_NAME_LENGTH);
    assert!(name.starts_with("AABBCCDDEEFF@"));
}

#[test]
fn the_raop_name_drops_a_second_separator() {
    let name = raop_instance_name("aabbccddeeff", "Room@Two");
    assert_eq!(name.matches('@').count(), 1);
    assert_eq!(name, "AABBCCDDEEFF@RoomTwo");
}

#[test]
fn the_raop_name_strips_control_characters() {
    let name = raop_instance_name("aabbccddeeff", "Living\u{0} Room\n");
    assert_eq!(name, "AABBCCDDEEFF@Living Room");
}

#[test]
fn a_mac_with_colons_is_normalised() {
    assert_eq!(normalise_mac("aa:bb:cc:dd:ee:ff"), "AABBCCDDEEFF");
}

#[test]
fn a_short_mac_is_padded_to_twelve_digits() {
    assert_eq!(normalise_mac("aabb"), "AABB00000000");
}

#[test]
fn a_pseudo_mac_is_stable_and_twelve_digits_long() {
    let id = "0f9a7c11-2222-3333-4444-555566667777";
    assert_eq!(pseudo_mac(id), "0F9A7C112222");
    assert_eq!(pseudo_mac(id), pseudo_mac(id));
    assert_eq!(pseudo_mac(id).len(), MAC_HEX_LENGTH);
}

#[test]
fn an_empty_device_id_yields_a_zero_mac() {
    assert_eq!(pseudo_mac(""), "000000000000");
}

#[test]
fn the_mac_is_formatted_in_colon_notation() {
    assert_eq!(format_mac("AABBCCDDEEFF"), "aa:bb:cc:dd:ee:ff");
    assert_eq!(format_mac("aabbccddeeff"), "aa:bb:cc:dd:ee:ff");
}
