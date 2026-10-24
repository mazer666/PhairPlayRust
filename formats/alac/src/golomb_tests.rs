use super::*;

/// Builds a bit string of one bits followed by a zero bit.
fn unary(value: u32) -> Vec<u8> {
    let mut bits: Vec<bool> = vec![true; value as usize];
    bits.push(false);
    let mut out = vec![0_u8; bits.len().div_ceil(8)];
    for (index, bit) in bits.iter().enumerate() {
        if *bit {
            out[index / 8] |= 1 << (7 - (index % 8));
        }
    }
    out
}

/// Concatenates two bit buffers.
fn concat(left: &[u8], left_bits: usize, right: &[u8]) -> Vec<u8> {
    let mut out = left.to_vec();
    let total = left_bits + right.len() * 8;
    out.resize(total.div_ceil(8), 0);
    for index in 0..right.len() * 8 {
        if (right[index / 8] >> (7 - (index % 8))) & 1 == 1 {
            out[(left_bits + index) / 8] |= 1 << (7 - ((left_bits + index) % 8));
        }
    }
    out
}

#[test]
fn counts_the_leading_zeroes() {
    assert_eq!(lead(0), 32);
    assert_eq!(lead(1), 31);
    assert_eq!(lead(0x8000_0000), 0);
    assert_eq!(lead(0x4000_0000), 1);
}

#[test]
fn computes_the_logarithm_of_the_mean() {
    assert_eq!(lg3a(0), 1);
    assert_eq!(lg3a(1), 2);
    assert_eq!(lg3a(5), 3);
}

#[test]
fn decodes_a_simple_run_of_residuals() {
    // A mean of 128 keeps the coder in its simplest mode: every
    // residual is a run of one bits that ends with a zero bit.
    let params = AgParams::new(128, 0, 1, 0, 0, 255);
    // The residuals are 1, -2, 0 and 3.
    let mut bits = unary(2);
    bits = concat(&bits, 3, &unary(3));
    bits = concat(&bits, 7, &unary(0));
    bits = concat(&bits, 8, &unary(6));

    let mut position = 0_usize;
    let mut out = [0_i32; 4];
    let consumed = dyn_decomp(&params, &bits, &mut position, &mut out, 4, 16).unwrap_or(0);
    assert_eq!(out, [1, -2, 0, 3]);
    assert_eq!(consumed as usize, position);
}

#[test]
fn decodes_a_zero_residual() {
    let params = AgParams::new(128, 0, 1, 0, 0, 255);
    let bits = unary(0);
    let mut position = 0_usize;
    let mut out = [0_i32; 1];
    dyn_decomp(&params, &bits, &mut position, &mut out, 1, 16).unwrap_or(0);
    assert_eq!(out, [0]);
}

#[test]
fn refuses_a_truncated_bitstream() {
    // A buffer of one bits: the prefix of the first code is longer
    // than the format allows, so the decoder has to read the value
    // field of sixteen bits that the buffer does not hold.
    let params = AgParams::new(128, 0, 1, 0, 0, 255);
    let bits = [0xff_u8];
    let mut position = 0_usize;
    let mut out = [0_i32; 4];
    let error = match dyn_decomp(&params, &bits, &mut position, &mut out, 4, 16) {
        Ok(_) => panic!("expected an error"),
        Err(err) => err,
    };
    assert!(matches!(error, AlacError::Truncated { .. }), "{error:?}");
}

#[test]
fn refuses_an_output_buffer_that_is_too_small() {
    let params = AgParams::new(128, 0, 1, 0, 0, 255);
    let mut position = 0_usize;
    let mut out = [0_i32; 1];
    assert!(matches!(
        dyn_decomp(&params, &[0x00], &mut position, &mut out, 4, 16),
        Err(AlacError::Malformed(_))
    ));
}

#[test]
fn builds_the_parameters_of_the_coder() {
    let params = AgParams::new(10, 40, 14, 100, 100, 255);
    assert_eq!(params.mb, 10);
    assert_eq!(params.mb0, 10);
    assert_eq!(params.pb, 40);
    assert_eq!(params.kb, 14);
    assert_eq!(params.wb, 16_383);
    assert_eq!(params.qb, QB - 40);
}

#[test]
fn limits_the_prediction_width() {
    let params = AgParams::new(10, 40, 200, 0, 0, 255);
    assert_eq!(params.kb, 31);
    assert_eq!(params.wb, 0x7fff_ffff);
}
