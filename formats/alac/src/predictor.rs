// WHAT: the dynamic predictor of the ALAC format.
// WHY: after the residuals are decoded, the decoder has to run them
//      through a linear predictor whose coefficients the sender chooses
//      per frame. The predictor adapts its coefficients while it runs,
//      and the adaptation is what makes the format lossless. The
//      reference implementation relies on the wrapping behaviour of a
//      32-bit integer, so this port spells the wrapping operations out.
// EXAMPLE: with no active coefficient the predictor simply copies the
//          residuals, and with 31 active coefficients it sums them up.
//
// ATTRIBUTION: this file is a native Rust port of Apple's dynamic
// predictor (`dp_dec.c` and `dplib.h`, Copyright (c) 2001-2011 Apple
// Inc., licensed under the Apache License, Version 2.0). The port was
// written for PhairPlayRust; no C code was copied.

use crate::error::AlacError;

/// The largest number of active coefficients.
pub const NUM_COEF_PAIRS: usize = 16;

/// Returns the sign of a value: `1`, `-1` or `0`.
///
/// WHY: the reference implementation computes the sign with a bit trick
///      that treats the value as an unsigned one. The port keeps the
///      same result, because the adaptation of the coefficients depends
///      on it.
fn sign_of_int(value: i32) -> i32 {
    let negative_shift = (value.wrapping_neg() as u32) >> 31;
    (negative_shift as i32) | (value >> 31)
}

/// Runs the predictor over `num` residuals.
///
/// `pc` holds the decoded residuals, `out` receives the predicted
/// samples and `coefs` holds the coefficients, which the predictor
/// adapts in place.
///
/// # Errors
///
/// Returns an error when the channel width is not usable or when the
/// buffers are too small for the number of active coefficients.
pub fn unpc_block(
    pc: &[i32],
    out: &mut [i32],
    num: usize,
    coefs: &mut [i16],
    num_active: usize,
    chan_bits: u32,
    den_shift: u32,
) -> Result<(), AlacError> {
    if !(1..=32).contains(&chan_bits) {
        return Err(AlacError::Malformed(format!(
            "the channel width {chan_bits} is not usable"
        )));
    }
    if num == 0 {
        return Ok(());
    }
    if out.len() < num {
        return Err(AlacError::Malformed(
            "the output buffer is smaller than the sample count".into(),
        ));
    }
    if num_active > 0 && out.len() < num_active + 1 {
        return Err(AlacError::Malformed(
            "the output buffer is smaller than the number of coefficients".into(),
        ));
    }
    // The predictor reads one residual per sample, and the first
    // `num_active` residuals of the history as well.
    if pc.len() < num {
        return Err(AlacError::Malformed(
            "the residual buffer is smaller than the sample count".into(),
        ));
    }
    if num_active > 0 && pc.len() < num_active + 1 {
        return Err(AlacError::Malformed(
            "the residual buffer is smaller than the number of coefficients".into(),
        ));
    }
    if num_active > 0 && coefs.len() < num_active {
        return Err(AlacError::Malformed(
            "the coefficient buffer is smaller than the number of coefficients".into(),
        ));
    }

    let chan_shift = 32 - chan_bits;
    let den_half: i32 = if den_shift == 0 {
        0
    } else {
        1_i32 << (den_shift - 1)
    };

    out[0] = pc[0];
    if num_active == 0 {
        let count = num.min(pc.len());
        out[1..count].copy_from_slice(&pc[1..count]);
        return Ok(());
    }

    for index in 1..=num_active {
        let delta = pc[index].wrapping_add(out[index - 1]);
        out[index] = (delta << chan_shift) >> chan_shift;
    }
    let limit = num_active + 1;

    match num_active {
        4 => predict_four(pc, out, num, coefs, chan_shift, den_shift, den_half, limit),
        8 => predict_eight(pc, out, num, coefs, chan_shift, den_shift, den_half, limit),
        _ => predict_general(
            pc, out, num, coefs, num_active, chan_shift, den_shift, den_half, limit,
        ),
    }
    Ok(())
}

/// Runs the predictor in place, with 31 active coefficients.
///
/// WHY: the reference decoder uses this short cut when the sender asks
///      for the "in place" mode. The sum of the residuals is the
///      prediction, and the result is written back into the same buffer.
pub fn unpc_block_in_place(
    buffer: &mut [i32],
    num: usize,
    chan_bits: u32,
) -> Result<(), AlacError> {
    if !(1..=32).contains(&chan_bits) {
        return Err(AlacError::Malformed(format!(
            "the channel width {chan_bits} is not usable"
        )));
    }
    if num == 0 {
        return Ok(());
    }
    if buffer.len() < num {
        return Err(AlacError::Malformed(
            "the buffer is smaller than the sample count".into(),
        ));
    }
    let chan_shift = 32 - chan_bits;
    let mut previous = buffer[0];
    for slot in buffer.iter_mut().take(num).skip(1) {
        let delta = *slot + previous;
        previous = (delta << chan_shift) >> chan_shift;
        *slot = previous;
    }
    Ok(())
}

/// The predictor with four active coefficients.
#[allow(clippy::too_many_arguments)]
fn predict_four(
    pc: &[i32],
    out: &mut [i32],
    num: usize,
    coefs: &mut [i16],
    chan_shift: u32,
    den_shift: u32,
    den_half: i32,
    limit: usize,
) {
    let mut a0 = coefs[0];
    let mut a1 = coefs[1];
    let mut a2 = coefs[2];
    let mut a3 = coefs[3];

    for index in limit..num {
        let top = out[index - limit];
        let b0 = top.wrapping_sub(out[index - 1]);
        let b1 = top.wrapping_sub(out[index - 2]);
        let b2 = top.wrapping_sub(out[index - 3]);
        let b3 = top.wrapping_sub(out[index - 4]);

        let sum = den_half
            .wrapping_sub(a0 as i32 * b0)
            .wrapping_sub(a1 as i32 * b1)
            .wrapping_sub(a2 as i32 * b2)
            .wrapping_sub(a3 as i32 * b3);
        let sum = sum >> den_shift;

        let delta = pc[index];
        let mut delta_zero = delta;
        let sign = sign_of_int(delta);
        let delta = delta.wrapping_add(top).wrapping_add(sum);
        out[index] = (delta << chan_shift) >> chan_shift;

        if sign > 0 {
            let sgn = sign_of_int(b3);
            a3 = a3.wrapping_sub(sgn as i16);
            delta_zero = delta_zero.wrapping_sub((sgn * b3) >> den_shift);
            if delta_zero <= 0 {
                continue;
            }
            let sgn = sign_of_int(b2);
            a2 = a2.wrapping_sub(sgn as i16);
            delta_zero = delta_zero.wrapping_sub(2 * ((sgn * b2) >> den_shift));
            if delta_zero <= 0 {
                continue;
            }
            let sgn = sign_of_int(b1);
            a1 = a1.wrapping_sub(sgn as i16);
            delta_zero = delta_zero.wrapping_sub(3 * ((sgn * b1) >> den_shift));
            if delta_zero <= 0 {
                continue;
            }
            a0 = a0.wrapping_sub(sign_of_int(b0) as i16);
        } else if sign < 0 {
            let sgn = -sign_of_int(b3);
            a3 = a3.wrapping_sub(sgn as i16);
            delta_zero = delta_zero.wrapping_sub((sgn * b3) >> den_shift);
            if delta_zero >= 0 {
                continue;
            }
            let sgn = -sign_of_int(b2);
            a2 = a2.wrapping_sub(sgn as i16);
            delta_zero = delta_zero.wrapping_sub(2 * ((sgn * b2) >> den_shift));
            if delta_zero >= 0 {
                continue;
            }
            let sgn = -sign_of_int(b1);
            a1 = a1.wrapping_sub(sgn as i16);
            delta_zero = delta_zero.wrapping_sub(3 * ((sgn * b1) >> den_shift));
            if delta_zero >= 0 {
                continue;
            }
            a0 = a0.wrapping_add(sign_of_int(b0) as i16);
        }
    }

    coefs[0] = a0;
    coefs[1] = a1;
    coefs[2] = a2;
    coefs[3] = a3;
}

/// The predictor with eight active coefficients.
#[allow(clippy::too_many_arguments)]
fn predict_eight(
    pc: &[i32],
    out: &mut [i32],
    num: usize,
    coefs: &mut [i16],
    chan_shift: u32,
    den_shift: u32,
    den_half: i32,
    limit: usize,
) {
    let mut coefficients = [0_i16; 8];
    coefficients.copy_from_slice(&coefs[..8]);

    for index in limit..num {
        let top = out[index - limit];
        let b0 = top.wrapping_sub(out[index - 1]);
        let b1 = top.wrapping_sub(out[index - 2]);
        let b2 = top.wrapping_sub(out[index - 3]);
        let b3 = top.wrapping_sub(out[index - 4]);
        let b4 = top.wrapping_sub(out[index - 5]);
        let b5 = top.wrapping_sub(out[index - 6]);
        let b6 = top.wrapping_sub(out[index - 7]);
        let b7 = top.wrapping_sub(out[index - 8]);

        let mut sum = den_half;
        for (coefficient, difference) in coefficients.iter().zip([b0, b1, b2, b3, b4, b5, b6, b7]) {
            sum = sum.wrapping_sub(*coefficient as i32 * difference);
        }
        let sum = sum >> den_shift;

        let delta = pc[index];
        let mut delta_zero = delta;
        let sign = sign_of_int(delta);
        let delta = delta.wrapping_add(top).wrapping_add(sum);
        out[index] = (delta << chan_shift) >> chan_shift;

        if sign > 0 {
            for step in 0..8_usize {
                let difference = [b0, b1, b2, b3, b4, b5, b6, b7][7 - step];
                let sgn = sign_of_int(difference);
                coefficients[7 - step] = coefficients[7 - step].wrapping_sub(sgn as i16);
                delta_zero =
                    delta_zero.wrapping_sub((step + 1) as i32 * ((sgn * difference) >> den_shift));
                if delta_zero <= 0 {
                    break;
                }
            }
        } else if sign < 0 {
            for step in 0..8_usize {
                let difference = [b0, b1, b2, b3, b4, b5, b6, b7][7 - step];
                let sgn = -sign_of_int(difference);
                coefficients[7 - step] = coefficients[7 - step].wrapping_sub(sgn as i16);
                delta_zero =
                    delta_zero.wrapping_sub((step + 1) as i32 * ((sgn * difference) >> den_shift));
                if delta_zero >= 0 {
                    break;
                }
            }
        }
    }

    coefs[..8].copy_from_slice(&coefficients);
}

/// The predictor with any number of active coefficients.
#[allow(clippy::too_many_arguments)]
fn predict_general(
    pc: &[i32],
    out: &mut [i32],
    num: usize,
    coefs: &mut [i16],
    num_active: usize,
    chan_shift: u32,
    den_shift: u32,
    den_half: i32,
    limit: usize,
) {
    for index in limit..num {
        let top = out[index - limit];
        let mut sum = 0_i32;
        for step in 0..num_active {
            let difference = out[index - 1 - step].wrapping_sub(top);
            sum = sum.wrapping_add(coefs[step] as i32 * difference);
        }

        let delta = pc[index];
        let mut delta_zero = delta;
        let sign = sign_of_int(delta);
        let delta = delta
            .wrapping_add(top)
            .wrapping_add((sum.wrapping_add(den_half)) >> den_shift);
        out[index] = (delta << chan_shift) >> chan_shift;

        if sign > 0 {
            for step in (0..num_active).rev() {
                let difference = top.wrapping_sub(out[index - 1 - step]);
                let sgn = sign_of_int(difference);
                coefs[step] = coefs[step].wrapping_sub(sgn as i16);
                delta_zero = delta_zero
                    .wrapping_sub((num_active - step) as i32 * ((sgn * difference) >> den_shift));
                if delta_zero <= 0 {
                    break;
                }
            }
        } else if sign < 0 {
            for step in (0..num_active).rev() {
                let difference = top.wrapping_sub(out[index - 1 - step]);
                let sgn = sign_of_int(difference);
                coefs[step] = coefs[step].wrapping_add(sgn as i16);
                delta_zero = delta_zero
                    .wrapping_sub((num_active - step) as i32 * ((-sgn * difference) >> den_shift));
                if delta_zero >= 0 {
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "predictor_tests.rs"]
mod tests;
