#![allow(clippy::expect_used, clippy::unwrap_used)]

use super::*;

/// A 1280x720 High-profile sequence parameter set.
const SPS_720P: [u8; 27] = [
    0x67, 0x64, 0x00, 0x28, 0xac, 0xd9, 0x40, 0x50, 0x05, 0xbb, 0x01, 0x6a, 0x02, 0x02, 0x02, 0x80,
    0x00, 0x00, 0x30, 0x08, 0x00, 0x00, 0x1e, 0x47, 0x81, 0x8c, 0x0b,
];

#[test]
fn a_720p_sps_yields_its_resolution() {
    let info = parse_sps(&SPS_720P).expect("valid sps");
    assert_eq!(info.width, 1280);
    assert_eq!(info.height, 720);
    assert_eq!(info.profile_idc, 100);
    assert_eq!(info.level_idc, 40);
}

#[test]
fn a_start_code_is_stripped_before_parsing() {
    let mut with_start = vec![0u8, 0, 0, 1];
    with_start.extend_from_slice(&SPS_720P);
    let info = parse_sps(&with_start).expect("valid sps");
    assert_eq!((info.width, info.height), (1280, 720));
}

#[test]
fn a_picture_parameter_set_is_not_a_sequence_parameter_set() {
    assert!(parse_sps(&[0x68, 0xce, 0x32, 0x12]).is_none());
}

#[test]
fn an_empty_buffer_is_rejected() {
    assert!(parse_sps(&[]).is_none());
}

#[test]
fn a_truncated_sps_is_rejected() {
    assert!(parse_sps(&SPS_720P[..5]).is_none());
}

#[test]
fn a_run_of_zero_bits_cannot_overflow_the_exp_golomb_reader() {
    // 31 leading zeros is the largest legal prefix; a longer one must be
    // rejected instead of wrapping into a negative size.
    let mut hostile = vec![0x67, 0x64, 0x00, 0x28];
    hostile.extend(std::iter::repeat_n(0u8, 32));
    assert!(parse_sps(&hostile).is_none());
}

#[test]
fn an_all_zero_sps_is_rejected() {
    assert!(parse_sps(&[0u8; 64]).is_none());
}

#[test]
fn the_nal_type_is_read_from_the_first_byte() {
    assert_eq!(nal_type(&SPS_720P), Some(NAL_TYPE_SPS));
    assert_eq!(nal_type(&[0x68, 0xce]), Some(NAL_TYPE_PPS));
    assert_eq!(nal_type(&[]), None);
}

#[test]
fn a_start_code_is_recognised_in_both_lengths() {
    assert_eq!(strip_start_code(&[0, 0, 0, 1, 0x67]), &[0x67]);
    assert_eq!(strip_start_code(&[0, 0, 1, 0x67]), &[0x67]);
    assert_eq!(strip_start_code(&[0x67]), &[0x67]);
}
