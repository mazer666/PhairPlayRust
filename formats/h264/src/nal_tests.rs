use super::*;

#[test]
fn maps_the_known_types() {
    assert_eq!(NalUnitType::from_value(1), NalUnitType::SliceNonIdr);
    assert_eq!(NalUnitType::from_value(5), NalUnitType::SliceIdr);
    assert_eq!(NalUnitType::from_value(7), NalUnitType::Sps);
    assert_eq!(NalUnitType::from_value(8), NalUnitType::Pps);
    assert_eq!(NalUnitType::from_value(9), NalUnitType::Aud);
    assert_eq!(NalUnitType::from_value(24), NalUnitType::Reserved(24));
    assert_eq!(NalUnitType::Sps.value(), 7);
    assert_eq!(NalUnitType::Reserved(24).value(), 24);
}

#[test]
fn classifies_the_types() {
    assert!(NalUnitType::SliceIdr.is_vcl());
    assert!(!NalUnitType::Sps.is_vcl());
    assert!(NalUnitType::Pps.is_parameter_set());
    assert!(!NalUnitType::Aud.is_parameter_set());
}

#[test]
fn splits_a_three_byte_start_code_stream() {
    let units = split_annex_b(&[0, 0, 1, 0x67, 0x42, 0, 0, 1, 0x68, 0xce]);
    assert_eq!(units.len(), 2);
    assert_eq!(units[0].nal_type, NalUnitType::Sps);
    assert_eq!(units[0].payload, vec![0x42]);
    assert_eq!(units[1].nal_type, NalUnitType::Pps);
    assert_eq!(units[1].payload, vec![0xce]);
}

#[test]
fn splits_a_four_byte_start_code_stream() {
    let units = split_annex_b(&[0, 0, 0, 1, 0x67, 0x42]);
    assert_eq!(units.len(), 1);
    assert_eq!(units[0].nal_type, NalUnitType::Sps);
}

#[test]
fn tolerates_leading_zeroes() {
    let units = split_annex_b(&[0, 0, 0, 0, 0, 1, 0x67, 0x42]);
    assert_eq!(units.len(), 1);
    assert_eq!(units[0].payload, vec![0x42]);
}

#[test]
fn ignores_a_trailing_fragment() {
    let units = split_annex_b(&[0, 0, 1, 0x67, 0x42, 0x00]);
    assert_eq!(units.len(), 1);
    assert_eq!(units[0].payload, vec![0x42, 0x00]);
}

#[test]
fn returns_nothing_for_an_empty_stream() {
    assert!(split_annex_b(&[]).is_empty());
    assert!(split_annex_b(&[0, 0, 1]).is_empty());
}

#[test]
fn rejects_the_forbidden_zero_bit() {
    let error = match NalUnit::from_header(0x80, Vec::new()) {
        Ok(_) => panic!("expected an error"),
        Err(err) => err,
    };
    assert!(matches!(error, H264Error::InvalidNalUnit(_)));
}

#[test]
fn builds_the_header_byte() {
    let unit = NalUnit {
        nal_type: NalUnitType::Sps,
        nal_ref_idc: 3,
        payload: vec![1, 2],
    };
    assert_eq!(unit.header(), 0x67);
    assert_eq!(unit.to_bytes(), vec![0x67, 1, 2]);
}

#[test]
fn round_trips_through_annex_b() {
    let unit = NalUnit {
        nal_type: NalUnitType::SliceIdr,
        nal_ref_idc: 3,
        payload: vec![1, 2, 3],
    };
    let stream = to_annex_b(&unit);
    let units = split_annex_b(&stream);
    assert_eq!(units.len(), 1);
    assert_eq!(units[0], unit);
}

#[test]
fn converts_avcc_to_annex_b() {
    let avcc = [0, 0, 0, 2, 0x67, 0x42, 0, 0, 0, 1, 0x68];
    let annex_b = avcc_to_annex_b(&avcc).unwrap_or_default();
    assert_eq!(annex_b, vec![0, 0, 0, 1, 0x67, 0x42, 0, 0, 0, 1, 0x68]);
}

#[test]
fn rejects_an_avcc_length_that_points_past_the_end() {
    let avcc = [0, 0, 0, 8, 0x67, 0x42];
    let error = match avcc_to_annex_b(&avcc) {
        Ok(_) => panic!("expected an error"),
        Err(err) => err,
    };
    assert!(matches!(error, H264Error::InvalidAvcc(_)));
}

#[test]
fn rejects_a_truncated_avcc_length() {
    let avcc = [0, 0, 1];
    assert!(avcc_to_annex_b(&avcc).is_err());
}

#[test]
fn rejects_a_zero_length_avcc_unit() {
    let avcc = [0, 0, 0, 0];
    assert!(avcc_to_annex_b(&avcc).is_err());
}

#[test]
fn converts_annex_b_to_avcc() {
    let annex_b = [0, 0, 0, 1, 0x67, 0x42, 0, 0, 0, 1, 0x68];
    let avcc = annex_b_to_avcc(&annex_b).unwrap_or_default();
    assert_eq!(avcc, vec![0, 0, 0, 2, 0x67, 0x42, 0, 0, 0, 1, 0x68]);
}

#[test]
fn rejects_an_empty_annex_b_stream() {
    assert!(annex_b_to_avcc(&[]).is_err());
}
