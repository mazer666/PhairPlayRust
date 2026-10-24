use super::*;

#[test]
fn reads_bits_most_significant_first() {
    let mut reader = BitReader::new(&[0b1010_1100]);
    assert_eq!(reader.read_bits(4).unwrap_or(0), 0b1010);
    assert_eq!(reader.read_bits(4).unwrap_or(0), 0b1100);
    assert_eq!(reader.bits_remaining(), 0);
}

#[test]
fn reports_a_truncated_buffer() {
    let mut reader = BitReader::new(&[0xff]);
    let error = match reader.read_bits(12) {
        Ok(_) => panic!("expected an error"),
        Err(err) => err,
    };
    assert!(matches!(error, H264Error::Truncated { .. }));
}

#[test]
fn reads_the_simple_exp_golomb_codes() {
    // 1 -> 0, 010 -> 1, 011 -> 2, 00100 -> 3
    let mut reader = BitReader::new(&[0b1010_0110, 0b0100_0000]);
    assert_eq!(reader.read_ue().unwrap_or(0), 0);
    assert_eq!(reader.read_ue().unwrap_or(0), 1);
    assert_eq!(reader.read_ue().unwrap_or(0), 2);
    assert_eq!(reader.read_ue().unwrap_or(0), 3);
}

#[test]
fn reads_the_signed_exp_golomb_codes() {
    // 1 -> 0, 010 -> 1, 011 -> -1, 00100 -> 2, 00101 -> -2
    let mut reader = BitReader::new(&[0b1010_0110, 0b0100_0010, 0b1000_0000]);
    assert_eq!(reader.read_se().unwrap_or(0), 0);
    assert_eq!(reader.read_se().unwrap_or(0), 1);
    assert_eq!(reader.read_se().unwrap_or(0), -1);
    assert_eq!(reader.read_se().unwrap_or(0), 2);
    assert_eq!(reader.read_se().unwrap_or(0), -2);
}

#[test]
fn thirty_one_leading_zeros_do_not_overflow() {
    // 31 zero bits, a one bit, then 31 one bits. The code number is
    // 2^31 - 1 + (2^31 - 1) = 2^32 - 2.
    let mut buffer = [0_u8; 8];
    for index in 0..31 {
        let byte = index / 8;
        let bit = 7 - (index % 8);
        buffer[byte] &= !(1 << bit);
    }
    buffer[3] |= 1 << (7 - (31 % 8));
    for index in 32..63 {
        let byte = index / 8;
        let bit = 7 - (index % 8);
        buffer[byte] |= 1 << bit;
    }
    let mut reader = BitReader::new(&buffer);
    let value = reader.read_ue().unwrap_or(0);
    assert_eq!(value, 4_294_967_294);
}

#[test]
fn thirty_two_leading_zeros_are_clamped() {
    // 32 zero bits, a one bit, then 32 one bits: the code number
    // does not fit into a u32 and is clamped.
    let mut buffer = [0_u8; 9];
    buffer[4] |= 1 << 7;
    for index in 33..65 {
        let byte = index / 8;
        let bit = 7 - (index % 8);
        buffer[byte] |= 1 << bit;
    }
    let mut reader = BitReader::new(&buffer);
    assert_eq!(reader.read_ue().unwrap_or(0), u32::MAX);
}

#[test]
fn too_many_leading_zeros_are_refused() {
    // 40 zero bits and no one bit at all.
    let buffer = [0_u8; 8];
    let mut reader = BitReader::new(&buffer);
    let error = match reader.read_ue() {
        Ok(_) => panic!("expected an error"),
        Err(err) => err,
    };
    assert!(
        matches!(error, H264Error::ExpGolombOverflow { .. }),
        "{error:?}"
    );
}

#[test]
fn a_truncated_code_is_reported() {
    // A single zero bit and then the end of the buffer.
    let mut reader = BitReader::new(&[0x00]);
    let error = match reader.read_ue() {
        Ok(_) => panic!("expected an error"),
        Err(err) => err,
    };
    assert!(matches!(error, H264Error::Truncated { .. }), "{error:?}");
}

#[test]
fn the_test_writer_produces_readable_codes() {
    let mut writer = TestBitWriter::new();
    writer.ue(0);
    writer.ue(1);
    writer.ue(2);
    writer.ue(300);
    let bytes = writer.finish();
    let mut reader = BitReader::new(&bytes);
    assert_eq!(reader.read_ue().unwrap_or(0), 0);
    assert_eq!(reader.read_ue().unwrap_or(0), 1);
    assert_eq!(reader.read_ue().unwrap_or(0), 2);
    assert_eq!(reader.read_ue().unwrap_or(0), 300);
}

#[test]
fn byte_alignment_moves_to_the_next_boundary() {
    let mut reader = BitReader::new(&[0xff, 0xff, 0xff]);
    let _ = reader.read_bit();
    reader.byte_align();
    assert_eq!(reader.bit_position(), 8);
    assert_eq!(reader.read_remaining_bytes().unwrap_or_default().len(), 2);
}

#[test]
fn removing_escape_bytes_works() {
    assert_eq!(unescape_rbsp(&[0x00, 0x00, 0x03, 0x01]), vec![0, 0, 1]);
    assert_eq!(unescape_rbsp(&[0x00, 0x00, 0x00, 0x03]), vec![0, 0, 0]);
    assert_eq!(unescape_rbsp(&[0x01, 0x02]), vec![1, 2]);
}
