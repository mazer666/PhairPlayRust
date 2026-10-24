// WHAT: the mixing of the two channels of a stereo frame.
// WHY: an ALAC frame may store the two channels in a mixed form, so
//      that the difference between them codes better. The decoder has
//      to undo that mixing before it hands the samples to the sink.
//      The reference implementation writes the result into a byte buffer
//      in the width of the stream; the port computes the two channels
//      and lets the caller pack them.
// EXAMPLE: with a mix residue of zero the two channels are stored
//          separately and nothing has to be undone.
//
// ATTRIBUTION: this file is a native Rust port of Apple's mixing
// routines (`matrix_dec.c` and `matrixlib.h`, Copyright (c) 2004-2011
// Apple Inc., licensed under the Apache License, Version 2.0). The port
// was written for PhairPlayRust; no C code was copied.

/// The two channels of a mixed frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MixedChannels {
    /// The left channel.
    pub left: Vec<i32>,
    /// The right channel.
    pub right: Vec<i32>,
}

/// Undoes the mixing of a stereo frame.
///
/// `mix_res` is the residue of the mixing; a residue of zero means that
/// the two channels were stored separately.
pub fn unmix(u: &[i32], v: &[i32], mix_bits: u32, mix_res: i32) -> MixedChannels {
    let count = u.len().min(v.len());
    let mut left = Vec::with_capacity(count);
    let mut right = Vec::with_capacity(count);
    for index in 0..count {
        let first = u[index];
        let second = v[index];
        if mix_res != 0 {
            let l = first
                .wrapping_add(second)
                .wrapping_sub((mix_res.wrapping_mul(second)) >> mix_bits);
            let r = l.wrapping_sub(second);
            left.push(l);
            right.push(r);
        } else {
            left.push(first);
            right.push(second);
        }
    }
    MixedChannels { left, right }
}

/// Applies the shifted out low bits to a sample.
///
/// WHY: a 24 or 32 bit stream may shift the low bytes of the samples
///      out of the predictor, so that they code better. The decoder has
///      to put them back.
pub fn apply_shift(value: i32, shifted: u16, bytes_shifted: u32) -> i32 {
    let shift = bytes_shifted * 8;
    if shift == 0 {
        return value;
    }
    (value << shift) | i32::from(shifted)
}

/// Packs interleaved samples into the byte layout of the format.
///
/// WHY: the reference decoder writes the samples into a buffer of the
///      width of the stream. The port keeps the samples as numbers and
///      offers the packing as a separate step, so that the caller can
///      choose the buffer it wants to hand to the sink.
///
/// # Errors
///
/// Returns an error when the bit depth is not one of 16, 20, 24 or 32.
pub fn pack_samples(samples: &[i32], bit_depth: u8) -> Result<Vec<u8>, crate::error::AlacError> {
    match bit_depth {
        16 => {
            let mut out = Vec::with_capacity(samples.len() * 2);
            for sample in samples {
                out.extend_from_slice(&(*sample as i16).to_le_bytes());
            }
            Ok(out)
        }
        20 => {
            let mut out = Vec::with_capacity(samples.len() * 3);
            for sample in samples {
                let value = *sample << 4;
                out.push(((value >> 16) & 0xff) as u8);
                out.push(((value >> 8) & 0xff) as u8);
                out.push((value & 0xff) as u8);
            }
            Ok(out)
        }
        24 => {
            let mut out = Vec::with_capacity(samples.len() * 3);
            for sample in samples {
                let value = *sample;
                out.push(((value >> 16) & 0xff) as u8);
                out.push(((value >> 8) & 0xff) as u8);
                out.push((value & 0xff) as u8);
            }
            Ok(out)
        }
        32 => {
            let mut out = Vec::with_capacity(samples.len() * 4);
            for sample in samples {
                out.extend_from_slice(&sample.to_le_bytes());
            }
            Ok(out)
        }
        other => Err(crate::error::AlacError::Malformed(format!(
            "the bit depth {other} cannot be packed"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_separate_channels_apart() {
        let mixed = unmix(&[1, 2, 3], &[4, 5, 6], 0, 0);
        assert_eq!(mixed.left, vec![1, 2, 3]);
        assert_eq!(mixed.right, vec![4, 5, 6]);
    }

    #[test]
    fn undoes_the_mixing() {
        // The reference formula: l = u + v - ((mixres * v) >> mixbits).
        let mixed = unmix(&[10, 20], &[3, 4], 2, 1);
        // l = u + v - ((mixres * v) >> mixbits) gives 13 and 23,
        // r = l - v gives 10 and 19.
        assert_eq!(mixed.left, vec![13, 23]);
        assert_eq!(mixed.right, vec![10, 19]);
    }

    #[test]
    fn applies_the_shifted_out_bits() {
        assert_eq!(apply_shift(1, 0, 0), 1);
        assert_eq!(apply_shift(1, 0x00ff, 1), 0x1ff);
        assert_eq!(apply_shift(0, 0x0001, 2), 1);
    }

    #[test]
    fn packs_samples_into_sixteen_bits() {
        let packed = pack_samples(&[1, -1], 16).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(packed, vec![1, 0, 0xff, 0xff]);
    }

    #[test]
    fn packs_samples_into_twenty_four_bits() {
        let packed = pack_samples(&[0x0000_0abcdef], 24).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(packed, vec![0xab, 0xcd, 0xef]);
    }

    #[test]
    fn packs_samples_into_twenty_bits() {
        // The twenty bits are left justified in three bytes.
        let packed = pack_samples(&[0x0000_0abc], 20).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(packed, vec![0x00, 0xab, 0xc0]);
    }

    #[test]
    fn packs_samples_into_thirty_two_bits() {
        let packed = pack_samples(&[1], 32).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(packed, vec![1, 0, 0, 0]);
    }

    #[test]
    fn refuses_an_unknown_bit_depth() {
        assert!(pack_samples(&[1], 12).is_err());
    }
}
