#![allow(clippy::expect_used, clippy::unwrap_used)]

use super::*;

#[test]
fn an_identity_is_stable_across_loads() {
    let store = InMemoryIdentityStore::new();
    let first = PairingIdentity::load_or_create(&store).expect("identity");
    let second = PairingIdentity::load_or_create(&store).expect("identity");
    assert_eq!(first.public_key(), second.public_key());
}

#[test]
fn a_signature_verifies_against_the_public_key() {
    let identity = PairingIdentity::from_seed(&[7u8; 32]);
    let signature = identity.sign(b"hello");
    assert!(PairingIdentity::verify(
        &identity.public_key(),
        b"hello",
        &signature
    ));
}

#[test]
fn a_signature_over_another_message_does_not_verify() {
    let identity = PairingIdentity::from_seed(&[7u8; 32]);
    let signature = identity.sign(b"hello");
    assert!(!PairingIdentity::verify(
        &identity.public_key(),
        b"world",
        &signature
    ));
}

#[test]
fn a_malformed_signature_is_rejected_without_panicking() {
    let identity = PairingIdentity::from_seed(&[7u8; 32]);
    assert!(!PairingIdentity::verify(
        &identity.public_key(),
        b"hello",
        b"too short"
    ));
}

#[test]
fn a_malformed_public_key_is_rejected_without_panicking() {
    assert!(!PairingIdentity::verify(&[0u8; 32], b"hello", &[0u8; 64]));
}

#[test]
fn two_identities_with_different_seeds_differ() {
    let first = PairingIdentity::from_seed(&[1u8; 32]);
    let second = PairingIdentity::from_seed(&[2u8; 32]);
    assert_ne!(first.public_key(), second.public_key());
}
