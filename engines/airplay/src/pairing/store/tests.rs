#![allow(clippy::expect_used, clippy::unwrap_used)]

use super::*;

#[test]
fn a_controller_can_be_stored_and_read_back() {
    let store = InMemoryPairingStore::new();
    store.add_controller("mac-1", &[7u8; 32]).expect("store");
    assert_eq!(
        store.controller("mac-1").expect("query"),
        Some(vec![7u8; 32])
    );
    assert!(store.has_any_controller().expect("query"));
}

#[test]
fn an_unknown_controller_is_not_found() {
    let store = InMemoryPairingStore::new();
    assert_eq!(store.controller("mac-2").expect("query"), None);
    assert!(!store.has_any_controller().expect("query"));
}

#[test]
fn a_controller_can_be_removed() {
    let store = InMemoryPairingStore::new();
    store.add_controller("mac-1", &[7u8; 32]).expect("store");
    store.remove_controller("mac-1").expect("remove");
    assert!(!store.has_any_controller().expect("query"));
}

#[test]
fn the_failed_attempt_counter_grows_and_can_be_reset() {
    let store = InMemoryPairingStore::new();
    assert_eq!(store.failed_attempts().expect("query"), 0);
    assert_eq!(store.record_failed_attempt().expect("record"), 1);
    assert_eq!(store.record_failed_attempt().expect("record"), 2);
    store.reset_failed_attempts().expect("reset");
    assert_eq!(store.failed_attempts().expect("query"), 0);
}
