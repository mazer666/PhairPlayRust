#![allow(clippy::expect_used, clippy::unwrap_used)]

use super::*;

use crate::error::AirPlayError;

fn phase_one(version: u8, mode: u8) -> [u8; 16] {
    let mut request = [0u8; 16];
    request[4] = version;
    request[14] = mode;
    request
}

#[test]
fn the_first_phase_answers_with_142_bytes() {
    let mut session = FairPlaySession::new();
    let answer = session.setup(&phase_one(VERSION_V3, 0)).expect("phase 1");
    assert_eq!(answer.len(), SETUP_REPLY_BYTES);
    assert_eq!(&answer[..4], b"FPLY");
}

#[test]
fn every_mode_has_its_own_answer() {
    let mut session = FairPlaySession::new();
    let answers: Vec<Vec<u8>> = (0..4)
        .map(|mode| {
            session
                .setup(&phase_one(VERSION_V3, mode))
                .expect("phase 1")
        })
        .collect();
    for (index, answer) in answers.iter().enumerate() {
        for (other, other_answer) in answers.iter().enumerate() {
            assert_eq!(answer == other_answer, index == other);
        }
    }
}

#[test]
fn the_second_version_patches_the_mode_into_the_answer() {
    let mut session = FairPlaySession::new();
    let answer = session.setup(&phase_one(VERSION_V2, 2)).expect("phase 1");
    assert_eq!(answer.len(), SETUP_REPLY_BYTES);
    assert_eq!(answer[V2_MODE_OFFSET], 2);
    assert_eq!(&answer[..4], b"FPLY");
}

#[test]
fn the_negotiated_version_is_remembered() {
    let mut session = FairPlaySession::new();
    let _ = session.setup(&phase_one(VERSION_V2, 0)).expect("phase 1");
    assert_eq!(session.negotiated_version(), VERSION_V2);
}

#[test]
fn a_request_of_the_wrong_size_is_rejected() {
    let mut session = FairPlaySession::new();
    let error = session.setup(&[0u8; 8]).expect_err("must fail");
    assert!(matches!(error, AirPlayError::Parse { .. }));
}

#[test]
fn an_unsupported_version_is_rejected() {
    let mut session = FairPlaySession::new();
    let error = session.setup(&phase_one(0x09, 0)).expect_err("must fail");
    assert!(matches!(error, AirPlayError::Parse { .. }));
}

#[test]
fn an_invalid_mode_is_rejected() {
    let mut session = FairPlaySession::new();
    let error = session
        .setup(&phase_one(VERSION_V3, 9))
        .expect_err("must fail");
    assert!(matches!(error, AirPlayError::Parse { .. }));
}

#[test]
fn the_second_phase_answers_with_32_bytes() {
    let mut session = FairPlaySession::new();
    let mut request = [0u8; HANDSHAKE_REQUEST_BYTES];
    request[4] = VERSION_V3;
    request[HANDSHAKE_ECHO_OFFSET..].copy_from_slice(&[0xAB; 20]);
    let answer = session.handshake(&request).expect("phase 2");
    assert_eq!(answer.len(), HANDSHAKE_REPLY_BYTES);
    assert_eq!(answer[4], VERSION_V3);
    assert_eq!(&answer[12..], &[0xAB; 20]);
}

#[test]
fn the_second_version_echoes_its_own_version() {
    let mut session = FairPlaySession::new();
    let mut request = [0u8; HANDSHAKE_REQUEST_BYTES];
    request[4] = VERSION_V2;
    let answer = session.handshake(&request).expect("phase 2");
    assert_eq!(answer[4], VERSION_V2);
}

#[test]
fn a_second_phase_request_of_the_wrong_size_is_rejected() {
    let mut session = FairPlaySession::new();
    let error = session.handshake(&[0u8; 32]).expect_err("must fail");
    assert!(matches!(error, AirPlayError::Parse { .. }));
}

#[test]
fn the_key_cannot_be_unwrapped_without_the_handshake() {
    let session = FairPlaySession::new();
    let error = session.decrypt(&[0u8; 72]).expect_err("must fail");
    assert!(matches!(error, AirPlayError::InvalidState { .. }));
}

#[test]
fn the_key_decryption_reports_the_product_boundary() {
    let mut session = FairPlaySession::new();
    let mut request = [0u8; HANDSHAKE_REQUEST_BYTES];
    request[4] = VERSION_V3;
    let _ = session.handshake(&request).expect("phase 2");
    let error = session.decrypt(&[0u8; 72]).expect_err("must fail");
    assert!(matches!(error, AirPlayError::Unsupported { .. }), "{error}");
}

#[test]
fn a_wrapped_key_of_the_wrong_size_is_rejected() {
    let mut session = FairPlaySession::new();
    let mut request = [0u8; HANDSHAKE_REQUEST_BYTES];
    request[4] = VERSION_V3;
    let _ = session.handshake(&request).expect("phase 2");
    let error = session.decrypt(&[0u8; 16]).expect_err("must fail");
    assert!(matches!(error, AirPlayError::Parse { .. }));
}

#[test]
fn the_hex_decoder_rejects_an_odd_length() {
    assert!(decode_hex("abc").is_none());
}
