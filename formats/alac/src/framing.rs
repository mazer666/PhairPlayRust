// WHAT: the framing helpers of the ALAC decoder.
// WHY: an ALAC frame may carry elements that hold no audio, and it may
//      carry samples that were not compressed. Both cases have their own
//      rules, and both live next to the element readers.
// EXAMPLE: `skip_fill_element` steps over the filler that a sender puts
//          between two frames.
//
// ATTRIBUTION: this file is a native Rust port of Apple's ALAC decoder
// (`ALACDecoder.cpp`, Copyright (c) 2011 Apple Inc., licensed under the
// Apache License, Version 2.0). The port was written for PhairPlayRust;
// no C code was copied.

use crate::bits::BitReader;
use crate::error::AlacError;

/// Reads uncompressed samples into a mix buffer.
///
/// WHY: an uncompressed frame stores the samples of one channel as raw
///      values. The reference implementation reads them in two parts
///      when the channel is wider than sixteen bits, so that the bit
///      reader never has to read more than sixteen bits at a time.
pub(crate) fn read_uncompressed(
    bits: &mut BitReader<'_>,
    mix: &mut [i32],
    num_samples: usize,
    chan_bits: u32,
) -> Result<(), AlacError> {
    if chan_bits == 0 || chan_bits > 32 {
        return Err(AlacError::Malformed(format!(
            "the channel width {chan_bits} is not usable"
        )));
    }
    let shift = 32 - chan_bits;
    if chan_bits <= 16 {
        for slot in mix.iter_mut().take(num_samples) {
            let value = bits.read(chan_bits)? as i32;
            *slot = (value << shift) >> shift;
        }
    } else {
        let extra_bits = chan_bits - 16;
        for slot in mix.iter_mut().take(num_samples) {
            let high = bits.read(16)? as u16 as i16 as i32;
            let value = (high << 16) >> shift;
            *slot = value | bits.read(extra_bits)? as i32;
        }
    }
    Ok(())
}

/// Reads the uncompressed samples of a stereo element.
///
/// WHY: the reference implementation stores the two channels of an
///      uncompressed stereo frame interleaved: the first sample of the
///      left channel, then the first sample of the right channel, then
///      the second of the left channel, and so on.
pub(crate) fn read_uncompressed_pair(
    bits: &mut BitReader<'_>,
    mix_u: &mut [i32],
    mix_v: &mut [i32],
    num_samples: usize,
    chan_bits: u32,
) -> Result<(), AlacError> {
    if chan_bits == 0 || chan_bits > 32 {
        return Err(AlacError::Malformed(format!(
            "the channel width {chan_bits} is not usable"
        )));
    }
    let shift = 32 - chan_bits;
    // WHY: the single-channel twin walks the buffer instead of indexing it.
    // A buffer that is shorter than the number of samples must not turn
    // into an index-out-of-bounds panic.
    if mix_u.len() < num_samples || mix_v.len() < num_samples {
        return Err(AlacError::Malformed(format!(
            "a stereo element of {num_samples} samples does not fit into the mix buffer"
        )));
    }
    if chan_bits <= 16 {
        for index in 0..num_samples {
            let first = bits.read(chan_bits)? as i32;
            mix_u[index] = (first << shift) >> shift;
            let second = bits.read(chan_bits)? as i32;
            mix_v[index] = (second << shift) >> shift;
        }
    } else {
        let extra_bits = chan_bits - 16;
        for index in 0..num_samples {
            let first = bits.read(16)? as u16 as i16 as i32;
            mix_u[index] = ((first << 16) >> shift) | bits.read(extra_bits)? as i32;
            let second = bits.read(16)? as u16 as i16 as i32;
            mix_v[index] = ((second << 16) >> shift) | bits.read(extra_bits)? as i32;
        }
    }
    Ok(())
}

/// Skips a data stream element.
pub(crate) fn skip_data_stream_element(bits: &mut BitReader<'_>) -> Result<(), AlacError> {
    let _instance = bits.read_small(4)?;
    let align = bits.read_one()?;
    let mut count = bits.read_small(8)? as u32;
    if count == 255 {
        count += bits.read_small(8)? as u32;
    }
    if align == 1 {
        bits.byte_align();
    }
    bits.advance(count * 8)
}

/// Skips a fill element.
pub(crate) fn skip_fill_element(bits: &mut BitReader<'_>) -> Result<(), AlacError> {
    let mut count = bits.read_small(4)? as i32;
    if count == 15 {
        count += bits.read_small(8)? as i32 - 1;
    }
    let count = count.max(0) as u32;
    bits.advance(count * 8)
}
