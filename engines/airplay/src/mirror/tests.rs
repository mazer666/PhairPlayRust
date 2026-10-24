
use super::*;

#[test]
fn the_derived_material_has_the_expected_length() {
    let (key, iv) = stream_key_material(&[1u8; 16], &[2u8; 32], 42);
    assert_eq!(key.len(), KEY_BYTES);
    assert_eq!(iv.len(), KEY_BYTES);
}

#[test]
fn the_derivation_is_deterministic() {
    let first = stream_key_material(&[1u8; 16], &[2u8; 32], 42);
    let second = stream_key_material(&[1u8; 16], &[2u8; 32], 42);
    assert_eq!(first, second);
}

#[test]
fn a_different_stream_identifier_yields_a_different_key() {
    let first = stream_key_material(&[1u8; 16], &[2u8; 32], 42);
    let second = stream_key_material(&[1u8; 16], &[2u8; 32], 43);
    assert_ne!(first.0, second.0);
}

#[test]
fn a_different_shared_secret_yields_a_different_key() {
    let first = stream_key_material(&[1u8; 16], &[2u8; 32], 42);
    let second = stream_key_material(&[1u8; 16], &[3u8; 32], 42);
    assert_ne!(first.0, second.0);
}

#[test]
fn the_audio_key_is_the_first_half_of_the_hash() {
    let key = audio_key(&[1u8; 16], &[2u8; 32]);
    let mut hasher = Sha512::new();
    hasher.update([1u8; 16]);
    hasher.update([2u8; 32]);
    assert_eq!(key.as_slice(), &hasher.finalize()[..16]);
}

#[test]
fn avcc_nal_units_become_annex_b() {
    let mut avcc = Vec::new();
    avcc.extend_from_slice(&3u32.to_be_bytes());
    avcc.extend_from_slice(&[0x65, 0xAA, 0xBB]);
    avcc.extend_from_slice(&2u32.to_be_bytes());
    avcc.extend_from_slice(&[0x41, 0xCC]);
    let annex_b = avcc_to_annex_b(&avcc);
    assert_eq!(
        annex_b,
        vec![0, 0, 0, 1, 0x65, 0xAA, 0xBB, 0, 0, 0, 1, 0x41, 0xCC]
    );
}

#[test]
fn a_truncated_avcc_payload_is_stopped_at_the_last_complete_unit() {
    let mut avcc = Vec::new();
    avcc.extend_from_slice(&3u32.to_be_bytes());
    avcc.extend_from_slice(&[0x65, 0xAA, 0xBB]);
    avcc.extend_from_slice(&9u32.to_be_bytes());
    avcc.extend_from_slice(&[0x41]);
    let annex_b = avcc_to_annex_b(&avcc);
    assert_eq!(annex_b, vec![0, 0, 0, 1, 0x65, 0xAA, 0xBB]);
}

#[test]
fn a_zero_length_unit_ends_the_conversion() {
    let mut avcc = Vec::new();
    avcc.extend_from_slice(&0u32.to_be_bytes());
    avcc.extend_from_slice(&[0x41]);
    assert!(avcc_to_annex_b(&avcc).is_empty());
}
