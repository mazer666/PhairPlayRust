use super::*;
use crate::nal::NalUnitType;

/// Unwraps a NAL unit, failing the test when there is none.
fn expect_unit(units: Vec<NalUnit>) -> NalUnit {
    match units.into_iter().next() {
        Some(unit) => unit,
        None => panic!("expected a complete NAL unit"),
    }
}

/// Builds the payload of a fragmentation unit.
fn fragment(indicator: u8, start: bool, end: bool, nal_type: u8, data: &[u8]) -> Vec<u8> {
    let mut payload = Vec::with_capacity(data.len() + 2);
    payload.push(indicator);
    let mut fu_header = nal_type & 0x1f;
    if start {
        fu_header |= 0x80;
    }
    if end {
        fu_header |= 0x40;
    }
    payload.push(fu_header);
    payload.extend_from_slice(data);
    payload
}

#[test]
fn reassembles_a_fragmented_nal_unit() {
    let mut reassembler = FuAReassembler::new();
    assert!(
        reassembler
            .push(&fragment(0x7c, true, false, 5, &[1, 2, 3]))
            .unwrap_or_default()
            .is_empty()
    );
    assert!(
        reassembler
            .push(&fragment(0x7c, false, false, 5, &[4, 5]))
            .unwrap_or_default()
            .is_empty()
    );
    let unit = reassembler
        .push(&fragment(0x7c, false, true, 5, &[6]))
        .unwrap_or_default();
    let unit = expect_unit(unit);
    assert_eq!(unit.nal_type, NalUnitType::SliceIdr);
    assert_eq!(unit.payload, vec![1, 2, 3, 4, 5, 6]);
    assert!(reassembler.is_idle());
}

#[test]
fn passes_a_single_nal_unit_through() {
    let mut reassembler = FuAReassembler::new();
    let unit = reassembler.push(&[0x67, 1, 2]).unwrap_or_default();
    let unit = expect_unit(unit);
    assert_eq!(unit.nal_type, NalUnitType::Sps);
    assert_eq!(unit.payload, vec![1, 2]);
}

#[test]
fn drops_a_continuation_without_a_start() {
    let mut reassembler = FuAReassembler::new();
    assert!(
        reassembler
            .push(&fragment(0x7c, false, true, 5, &[1]))
            .unwrap_or_default()
            .is_empty()
    );
    assert!(reassembler.is_idle());
}

#[test]
fn drops_the_previous_fragment_when_a_new_one_starts() {
    let mut reassembler = FuAReassembler::new();
    let _ = reassembler.push(&fragment(0x7c, true, false, 5, &[1, 2]));
    assert!(!reassembler.is_idle());
    let _ = reassembler.push(&fragment(0x7c, true, false, 5, &[3, 4]));
    let unit = reassembler
        .push(&fragment(0x7c, false, true, 5, &[5]))
        .unwrap_or_default();
    let unit = expect_unit(unit);
    assert_eq!(unit.payload, vec![3, 4, 5]);
}

#[test]
fn rejects_a_reserved_bit() {
    let mut reassembler = FuAReassembler::new();
    let payload = vec![0x7c, 0x25, 0x00];
    assert!(matches!(
        reassembler.push(&payload),
        Err(H264Error::InvalidRtp(_))
    ));
}

#[test]
fn rejects_a_too_short_fragment() {
    let mut reassembler = FuAReassembler::new();
    assert!(matches!(
        reassembler.push(&[0x7c]),
        Err(H264Error::InvalidRtp(_))
    ));
}

#[test]
fn rejects_an_empty_payload() {
    let mut reassembler = FuAReassembler::new();
    assert!(matches!(
        reassembler.push(&[]),
        Err(H264Error::InvalidRtp(_))
    ));
}

#[test]
fn rejects_a_fragment_that_is_larger_than_the_limit() {
    let mut reassembler = FuAReassembler::new();
    let payload = fragment(
        0x7c,
        true,
        false,
        5,
        &vec![0u8; MAX_REASSEMBLED_NAL_SIZE + 1],
    );
    assert!(matches!(
        reassembler.push(&payload),
        Err(H264Error::InvalidRtp(_))
    ));
}

#[test]
fn rejects_a_reassembled_unit_that_is_larger_than_the_limit() {
    let mut reassembler = FuAReassembler::new();
    let half = vec![0u8; MAX_REASSEMBLED_NAL_SIZE / 2 + 1];
    let _ = reassembler.push(&fragment(0x7c, true, false, 5, &half));
    let error = match reassembler.push(&fragment(0x7c, false, true, 5, &half)) {
        Ok(_) => panic!("expected the size limit to bite"),
        Err(err) => err,
    };
    assert!(matches!(error, H264Error::InvalidRtp(_)), "{error:?}");
}

#[test]
fn rejects_an_unknown_payload_type() {
    let mut reassembler = FuAReassembler::new();
    assert!(matches!(
        reassembler.push(&[0x1f, 0x00]),
        Err(H264Error::InvalidRtp(_))
    ));
}

#[test]
fn unpacks_an_aggregation_packet() {
    let mut reassembler = FuAReassembler::new();
    let mut payload = vec![0x78];
    payload.extend_from_slice(&2u16.to_be_bytes());
    payload.extend_from_slice(&[0x67, 0x42]);
    payload.extend_from_slice(&1u16.to_be_bytes());
    payload.extend_from_slice(&[0x68]);
    let units = reassembler.push(&payload).unwrap_or_default();
    assert_eq!(units.len(), 2);
    assert_eq!(units[0].nal_type, NalUnitType::Sps);
    assert_eq!(units[0].payload, vec![0x42]);
    assert_eq!(units[1].nal_type, NalUnitType::Pps);
}

#[test]
fn keeps_the_packet_that_arrives_after_an_aggregation_packet() {
    // WHY: a queue of the units of an aggregation packet would
    //      have to swallow the next packet, and that packet would be
    //      lost. Every packet has to be decoded.
    let mut reassembler = FuAReassembler::new();
    let mut aggregation = vec![0x78];
    aggregation.extend_from_slice(&2u16.to_be_bytes());
    aggregation.extend_from_slice(&[0x67, 0x42]);
    aggregation.extend_from_slice(&1u16.to_be_bytes());
    aggregation.extend_from_slice(&[0x68]);
    let units = reassembler.push(&aggregation).unwrap_or_default();
    assert_eq!(units.len(), 2);

    let units = reassembler.push(&[0x65, 0x99]).unwrap_or_default();
    assert_eq!(units.len(), 1);
    // The header byte 0x65 names the type five, an IDR picture.
    assert_eq!(units[0].nal_type, NalUnitType::SliceIdr);
    assert_eq!(units[0].payload, vec![0x99]);
}

#[test]
fn rejects_an_aggregation_packet_with_a_bad_length() {
    let mut reassembler = FuAReassembler::new();
    let payload = vec![0x78, 0x00, 0x08, 0x67];
    assert!(matches!(
        reassembler.push(&payload),
        Err(H264Error::InvalidRtp(_))
    ));
}
