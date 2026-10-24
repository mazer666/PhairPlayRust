// WHAT: the adaptive Golomb decoder of the ALAC format.
// WHY: the ALAC residuals are coded with a Golomb code whose prefix
//      length depends on the running mean of the residuals. The prefix
//      is a run of one bits that ends with a zero bit, followed by a
//      value field whose width depends on the same mean. Decoding it
//      means reading a bit at a time from an arbitrary bit offset, which
//      is exactly where the reference implementation reads past the end
//      of its buffer.
// EXAMPLE: with a mean of 128 the code of a residual of `3` is the bit
//          pattern `111110`.
//
// ATTRIBUTION: this file is a native Rust port of Apple's adaptive
// Golomb decoder (`ag_dec.c` and `aglib.h`, Copyright (c) 2001-2011
// Apple Inc., licensed under the Apache License, Version 2.0). The port
// was written for PhairPlayRust; no C code was copied, the arithmetic
// uses the explicit wrapping operations of Rust, and every read is
// bounds checked.

use crate::error::AlacError;

/// The shift of the quantiser base.
pub const QB_SHIFT: u32 = 9;

/// The quantiser base.
pub const QB: u32 = 1 << QB_SHIFT;

/// The shift of the mean multiplier.
pub const MMUL_SHIFT: u32 = 2;

/// The shift of the mean denominator.
pub const MDEN_SHIFT: u32 = QB_SHIFT - MMUL_SHIFT - 1;

/// The mean offset.
pub const MOFF: u32 = 1 << (MDEN_SHIFT - 2);

/// The bit offset of the run length code.
pub const BITOFF: u32 = 24;

/// The largest prefix the 32 bit code may have.
pub const MAX_PREFIX_32: u32 = 9;

/// The largest prefix the 16 bit code may have.
pub const MAX_PREFIX_16: u32 = 9;

/// The number of bits of the 16 bit value field.
pub const MAX_DATATYPE_BITS_16: u32 = 16;

/// The largest mean that is tracked.
pub const N_MAX_MEAN_CLAMP: u32 = 0xffff;

/// The mean that is used when the clamp bites.
pub const N_MEAN_CLAMP_VAL: u32 = 0xffff;

/// The parameters of the adaptive Golomb coder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgParams {
    /// The running mean.
    pub mb: u32,
    /// The initial mean.
    pub mb0: u32,
    /// The prediction offset.
    pub pb: u32,
    /// The prediction width.
    pub kb: u32,
    /// The mask of the prediction width.
    pub wb: u32,
    /// The quantiser base minus the prediction offset.
    pub qb: u32,
    /// The width of the frame.
    pub fw: u32,
    /// The width of a sector.
    pub sw: u32,
    /// The largest run of zeros.
    pub maxrun: u32,
}

impl AgParams {
    /// Builds the parameters of the coder.
    ///
    /// WHY: the prediction width is limited to 31 bits, because the
    ///      reference implementation builds a mask with a shift that
    ///      would be undefined for a larger width.
    pub fn new(m: u32, p: u32, k: u32, f: u32, s: u32, maxrun: u32) -> AgParams {
        let kb = k.min(31);
        AgParams {
            mb: m,
            mb0: m,
            pb: p,
            kb,
            wb: (1_u32 << kb) - 1,
            qb: QB.wrapping_sub(p),
            fw: f,
            sw: s,
            maxrun,
        }
    }
}

/// Returns the number of leading zero bits of a value.
///
/// WHY: the reference implementation counts the leading zeros of a signed
///      value. A negative value behaves differently there, and the
///      decoder relies on that behaviour, so the port keeps it.
fn lead(value: u32) -> u32 {
    if value & 0x8000_0000 != 0 {
        0
    } else {
        value.leading_zeros()
    }
}

/// Returns the base-two logarithm of `value + 3`, rounded down.
fn lg3a(value: u32) -> u32 {
    31 - lead(value.wrapping_add(3))
}

/// Reads four bytes, big endian, and zero fills past the end.
fn read32bit(data: &[u8], offset: usize) -> u32 {
    let mut value = 0_u32;
    for index in 0..4 {
        let byte = data.get(offset + index).copied().unwrap_or(0);
        value = (value << 8) | byte as u32;
    }
    value
}

/// Reads `num_bits` bits at an arbitrary bit offset.
///
/// WHY: the reference implementation shifts by `32 - numBits`, which
///      underflows when more than 32 bits are asked for. The port clamps
///      the width to 32 bits, so that a hostile configuration can never
///      make the shift overflow. The caller rejects a width that does
///      not fit before it gets here.
fn get_stream_bits(data: &[u8], bit_offset: usize, num_bits: u32) -> u32 {
    let num_bits = num_bits.min(32);
    if num_bits == 0 {
        return 0;
    }
    let byte_offset = bit_offset / 8;
    let offset = (bit_offset & 7) as u32;
    let load1 = read32bit(data, byte_offset);
    let mut result = if num_bits + offset > 32 {
        let mut value = load1 << offset;
        let load2 = data.get(byte_offset + 4).copied().unwrap_or(0) as u32;
        let shift = 8 - (num_bits + offset - 32);
        let load2 = if shift >= 32 { 0 } else { load2 >> shift };
        value >>= 32 - num_bits;
        value |= load2;
        value
    } else {
        load1 >> (32 - num_bits - offset)
    };
    if num_bits != 32 {
        result &= !(u32::MAX << num_bits);
    }
    result
}

/// Reads one adaptive Golomb code with a 32 bit value field.
fn dyn_get_32bit(
    data: &[u8],
    bit_position: &mut usize,
    m: u32,
    k: u32,
    max_bits: u32,
) -> Result<u32, AlacError> {
    let mut temp_bits = *bit_position;
    let stream_long = read32bit(data, temp_bits / 8) << (temp_bits & 7);
    let mut result = lead(!stream_long);

    if result >= MAX_PREFIX_32 {
        let max_bits = max_bits.min(32);
        result = get_stream_bits(data, temp_bits + MAX_PREFIX_32 as usize, max_bits);
        temp_bits += (MAX_PREFIX_32 + max_bits) as usize;
    } else {
        temp_bits += result as usize + 1;
        if k != 1 {
            let shifted = stream_long << (result + 1);
            let value = if k == 0 { 0 } else { shifted >> (32 - k) };
            temp_bits += k as usize;
            temp_bits -= 1;
            result = result.wrapping_mul(m);
            if value >= 2 {
                result = result.wrapping_add(value - 1);
                temp_bits += 1;
            }
        }
    }

    *bit_position = temp_bits;
    Ok(result)
}

/// Reads one adaptive Golomb code with a 16 bit value field.
fn dyn_get(data: &[u8], bit_position: &mut usize, m: u32, k: u32) -> Result<u32, AlacError> {
    let mut temp_bits = *bit_position;
    let stream_long = read32bit(data, temp_bits / 8) << (temp_bits & 7);
    let mut prefix = lead(!stream_long);

    let result = if prefix >= MAX_PREFIX_16 {
        prefix = MAX_PREFIX_16;
        temp_bits += prefix as usize;
        let shifted = stream_long << prefix;
        temp_bits += MAX_DATATYPE_BITS_16 as usize;
        shifted >> (32 - MAX_DATATYPE_BITS_16)
    } else {
        temp_bits += prefix as usize + 1;
        let shifted = stream_long << (prefix + 1);
        let value = if k == 0 { 0 } else { shifted >> (32 - k) };
        temp_bits += k as usize;
        let mut result = prefix.wrapping_mul(m).wrapping_add(value).wrapping_sub(1);
        if value < 2 {
            result = result.wrapping_sub(value.wrapping_sub(1));
            temp_bits -= 1;
        }
        result
    };

    *bit_position = temp_bits;
    Ok(result)
}

/// Decodes `num_samples` residuals into `out`.
///
/// `bit_position` names the first bit of the residual data and is
/// advanced by the number of bits that were consumed.
///
/// # Errors
///
/// Returns an error when the bitstream ends inside a code or when the
/// decoder wants to write more residuals than `out` can hold.
pub fn dyn_decomp(
    params: &AgParams,
    data: &[u8],
    bit_position: &mut usize,
    out: &mut [i32],
    num_samples: usize,
    max_size: u32,
) -> Result<u32, AlacError> {
    if out.len() < num_samples {
        return Err(AlacError::Malformed(
            "the output buffer is smaller than the sample count".into(),
        ));
    }
    let start_position = *bit_position;
    let max_position = data.len() * 8;
    let pb = params.pb;
    let kb = params.kb;
    let wb = params.wb;

    let mut mb = params.mb0;
    let mut zmode: u32 = 0;
    let mut count: usize = 0;

    while count < num_samples {
        if *bit_position >= max_position {
            return Err(AlacError::Truncated {
                needed: 1,
                available: max_position.saturating_sub(*bit_position),
            });
        }
        let m = mb >> QB_SHIFT;
        let mut k = lg3a(m).min(kb);
        let m = (1_u32 << k) - 1;

        let n = dyn_get_32bit(data, bit_position, m, k, max_size)?;
        if *bit_position > max_position {
            return Err(AlacError::Truncated {
                needed: *bit_position,
                available: max_position,
            });
        }

        let ndecode = n.wrapping_add(zmode);
        let multiplier: i32 = if ndecode & 1 == 1 { -1 } else { 1 };
        let delta = ((ndecode as i64 + 1) >> 1) as i32 * multiplier;
        out[count] = delta;
        count += 1;

        mb = pb
            .wrapping_mul(n.wrapping_add(zmode))
            .wrapping_add(mb)
            .wrapping_sub((pb.wrapping_mul(mb)) >> QB_SHIFT);
        if n > N_MAX_MEAN_CLAMP {
            mb = N_MEAN_CLAMP_VAL;
        }
        zmode = 0;

        if mb.wrapping_shl(MMUL_SHIFT) < QB && count < num_samples {
            zmode = 1;
            k = lead(mb)
                .wrapping_sub(BITOFF)
                .wrapping_add((mb.wrapping_add(MOFF)) >> MDEN_SHIFT);
            let k = k.min(31);
            let mz = ((1_u32 << k) - 1) & wb;
            let run = dyn_get(data, bit_position, mz, k)?;
            if *bit_position > max_position {
                return Err(AlacError::Truncated {
                    needed: *bit_position,
                    available: max_position,
                });
            }
            if count + run as usize > num_samples {
                return Err(AlacError::FrameTooLong {
                    announced: count as u32 + run,
                    limit: num_samples as u32,
                });
            }
            for _ in 0..run {
                out[count] = 0;
                count += 1;
            }
            if run >= 65535 {
                zmode = 0;
            }
            mb = 0;
        }
    }

    Ok((*bit_position - start_position) as u32)
}

#[cfg(test)]
#[path = "golomb_tests.rs"]
mod tests;
