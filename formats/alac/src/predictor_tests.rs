use super::*;

#[test]
fn computes_the_sign_of_a_value() {
    assert_eq!(sign_of_int(0), 0);
    assert_eq!(sign_of_int(5), 1);
    assert_eq!(sign_of_int(-5), -1);
    assert_eq!(sign_of_int(i32::MIN), -1);
    assert_eq!(sign_of_int(i32::MAX), 1);
}

#[test]
fn copies_the_residuals_without_coefficients() {
    let pc = [1_i32, 2, 3, 4];
    let mut out = [0_i32; 4];
    let mut coefs = [0_i16; 4];
    unpc_block(&pc, &mut out, 4, &mut coefs, 0, 16, 9).unwrap_or(());
    assert_eq!(out, [1, 2, 3, 4]);
}

#[test]
fn sums_the_residuals_in_place() {
    let mut buffer = [1_i32, 2, 3, 4];
    unpc_block_in_place(&mut buffer, 4, 16).unwrap_or(());
    assert_eq!(buffer, [1, 3, 6, 10]);
}

#[test]
fn applies_the_first_coefficients() {
    // Two active coefficients: the prediction is the sum of the two
    // previous samples.
    let pc = [1_i32, 1, 1, 1];
    let mut out = [0_i32; 4];
    let mut coefs = [0_i16; 4];
    unpc_block(&pc, &mut out, 4, &mut coefs, 2, 16, 9).unwrap_or(());
    // out[0] = 1, out[1] = 1 + 1 = 2, out[2] = 1 + 2 = 3, out[3] = 1 + 3 = 4
    assert_eq!(out[0], 1);
    assert_eq!(out[1], 2);
}

#[test]
fn refuses_an_unusable_channel_width() {
    let pc = [0_i32; 4];
    let mut out = [0_i32; 4];
    let mut coefs = [0_i16; 4];
    assert!(matches!(
        unpc_block(&pc, &mut out, 4, &mut coefs, 0, 0, 9),
        Err(AlacError::Malformed(_))
    ));
    assert!(matches!(
        unpc_block(&pc, &mut out, 4, &mut coefs, 0, 33, 9),
        Err(AlacError::Malformed(_))
    ));
}

#[test]
fn refuses_a_buffer_that_is_too_small() {
    let pc = [0_i32; 2];
    let mut out = [0_i32; 2];
    let mut coefs = [0_i16; 4];
    assert!(matches!(
        unpc_block(&pc, &mut out, 4, &mut coefs, 0, 16, 9),
        Err(AlacError::Malformed(_))
    ));
}

#[test]
fn refuses_a_residual_buffer_that_is_too_small() {
    // No active coefficient, but fewer residuals than samples: the
    // predictor must report the error instead of reading past the
    // buffer.
    let pc: [i32; 0] = [];
    let mut out = [0_i32; 1];
    let mut coefs = [0_i16; 4];
    assert!(matches!(
        unpc_block(&pc, &mut out, 1, &mut coefs, 0, 16, 9),
        Err(AlacError::Malformed(_))
    ));
}

#[test]
fn refuses_more_coefficients_than_the_buffer_holds() {
    let pc = [0_i32; 2];
    let mut out = [0_i32; 4];
    let mut coefs = [0_i16; 4];
    assert!(matches!(
        unpc_block(&pc, &mut out, 4, &mut coefs, 4, 16, 9),
        Err(AlacError::Malformed(_))
    ));
}

#[test]
fn the_general_path_predicts_the_previous_sample() {
    // Two active coefficients that are both zero: the prediction is
    // the sum of the two previous samples.
    let pc = [1_i32, 1, 1, 1];
    let mut out = [0_i32; 4];
    let mut coefs = [0_i16; 4];
    unpc_block(&pc, &mut out, 4, &mut coefs, 2, 16, 9).unwrap_or(());
    assert_eq!(out, [1, 2, 3, 2]);
}

#[test]
fn the_four_coefficient_path_predicts_the_previous_sample() {
    // Four active coefficients that are all zero.
    let pc = [1_i32, 1, 1, 1, 1, 1];
    let mut out = [0_i32; 6];
    let mut coefs = [0_i16; 4];
    unpc_block(&pc, &mut out, 6, &mut coefs, 4, 16, 9).unwrap_or(());
    assert_eq!(out, [1, 2, 3, 4, 5, 2]);
}

#[test]
fn the_optimised_paths_differ_from_the_general_one() {
    // WHY: the reference implementation updates the first
    //      coefficient of the four and the eight coefficient path
    //      without the correction of the residual that the general
    //      path applies. A decoder has to follow the reference
    //      exactly, so the difference is part of the format.
    let pc: Vec<i32> = (0..16).map(|index| index - 8).collect();
    let mut general = [0_i32; 16];
    let mut four = [0_i32; 16];
    let mut general_coefs = [0_i16; 16];
    let mut four_coefs = [0_i16; 16];
    unpc_block(&pc, &mut general, 16, &mut general_coefs, 5, 16, 9).unwrap_or(());
    unpc_block(&pc, &mut four, 16, &mut four_coefs, 4, 16, 9).unwrap_or(());
    assert_ne!(general, four);
}
