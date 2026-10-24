// WHAT: the element readers of the ALAC decoder.
// WHY: an ALAC frame holds one or more elements, and every element has
//      its own header, its own residuals and its own prediction. The
//      readers live in their own file so that the decoder itself stays
//      readable.
// EXAMPLE: `read_single_channel` reads a mono element and writes its
//          samples into the interleaved output.
//
// ATTRIBUTION: this file is a native Rust port of Apple's ALAC decoder
// (`ALACDecoder.cpp`, Copyright (c) 2011 Apple Inc., licensed under the
// Apache License, Version 2.0). The port was written for PhairPlayRust;
// no C code was copied.

use crate::bits::BitReader;
use crate::decoder::AlacDecoder;
use crate::error::AlacError;
use crate::framing::{read_uncompressed, read_uncompressed_pair};
use crate::golomb::{AgParams, dyn_decomp};
use crate::matrix::{apply_shift, unmix};
use crate::predictor::{unpc_block, unpc_block_in_place};

/// The largest number of coefficients of one channel.
const MAX_COEFFICIENTS: usize = 32;

impl AlacDecoder {
    /// Reads a single channel element and writes its samples.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn read_single_channel(
        &mut self,
        bits: &mut BitReader<'_>,
        samples: &mut [i32],
        channel_index: usize,
        num_channels: usize,
        num_samples: usize,
    ) -> Result<usize, AlacError> {
        let _instance = bits.read_small(4)?;
        let unused = bits.read(12)?;
        if unused != 0 {
            return Err(AlacError::Malformed(
                "the unused header bits of the element are not zero".into(),
            ));
        }
        let header = bits.read(4)?;
        let partial = (header >> 3) & 1;
        let bytes_shifted = (header >> 1) & 3;
        if bytes_shifted == 3 {
            return Err(AlacError::Malformed(
                "the shift flag of the element is not usable".into(),
            ));
        }
        let escape = header & 1;
        // WHY: the reference implementation resets the shift to zero in the
        // escape path — an escaped element carries the samples in full and
        // is never shifted afterwards. Without this the element would apply
        // the shift of a *previous* element.
        let mut applied_shift = bytes_shifted;
        // WHY: the shift is subtracted from the width of a sample. If the
        // width is smaller than the shift, the subtraction underflows — and
        // a subtraction that underflows is a panic in a debug build and a
        // huge number in a release build. The check has to come first.
        let shift_bits = bytes_shifted * 8;
        if u32::from(self.config.bit_depth) < shift_bits {
            return Err(AlacError::Malformed(format!(
                "a channel of {} bit samples cannot carry a shift of {shift_bits} bits",
                self.config.bit_depth
            )));
        }
        let chan_bits = self.config.bit_depth as u32 - shift_bits;
        if chan_bits > 32 {
            return Err(AlacError::Malformed(format!(
                "a channel of {} bit samples with a shift of {bytes_shifted} bytes does not fit",
                self.config.bit_depth
            )));
        }
        let num_samples = self.read_partial_length(bits, partial, num_samples)?;

        if escape == 0 {
            let _mix_bits = bits.read(8)?;
            let _mix_res = bits.read(8)? as i8;
            let mode_byte = bits.read(8)?;
            let mode_u = (mode_byte >> 4) as u8;
            let den_shift_u = mode_byte & 0x0f;
            let coefficient_byte = bits.read(8)?;
            let pb_factor_u = coefficient_byte >> 5;
            let num_u = (coefficient_byte & 0x1f) as usize;
            let mut coefficients = [0_i16; MAX_COEFFICIENTS];
            for coefficient in coefficients.iter_mut().take(num_u) {
                *coefficient = bits.read(16)? as i16;
            }

            let mut shift_reader = None;
            if bytes_shifted != 0 {
                shift_reader = Some(bits.clone());
                bits.advance((bytes_shifted * 8) * num_samples as u32)?;
            }

            let pb = self.config.pb as u32;
            let params = AgParams::new(
                self.config.mb as u32,
                (pb * pb_factor_u) / 4,
                self.config.kb as u32,
                num_samples as u32,
                num_samples as u32,
                self.config.max_run as u32,
            );
            let mut position = bits.bit_position();
            dyn_decomp(
                &params,
                bits.data(),
                &mut position,
                &mut self.predictor,
                num_samples,
                chan_bits,
            )?;
            bits.seek_to_bit(position);

            if mode_u == 0 {
                unpc_block(
                    &self.predictor,
                    &mut self.mix_u,
                    num_samples,
                    &mut coefficients,
                    num_u,
                    chan_bits,
                    den_shift_u,
                )?;
            } else {
                unpc_block_in_place(&mut self.predictor, num_samples, chan_bits)?;
                unpc_block(
                    &self.predictor,
                    &mut self.mix_u,
                    num_samples,
                    &mut coefficients,
                    num_u,
                    chan_bits,
                    den_shift_u,
                )?;
            }

            if let Some(mut reader) = shift_reader {
                for index in 0..num_samples {
                    self.shift_buffer[index] = reader.read(bytes_shifted * 8)? as u16;
                }
            }
        } else {
            read_uncompressed(bits, &mut self.mix_u, num_samples, chan_bits)?;
            // The escape path carries the samples in full: the shift of the
            // header does not apply to them.
            applied_shift = 0;
        }

        for index in 0..num_samples {
            let value = if matches!(self.config.bit_depth, 24 | 32) && applied_shift != 0 {
                apply_shift(self.mix_u[index], self.shift_buffer[index], applied_shift)
            } else {
                self.mix_u[index]
            };
            let position = channel_index + index * num_channels;
            if position < samples.len() {
                samples[position] = value;
            }
        }
        Ok(num_samples)
    }

    /// Reads a channel pair element and writes its samples.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn read_channel_pair(
        &mut self,
        bits: &mut BitReader<'_>,
        samples: &mut [i32],
        channel_index: usize,
        num_channels: usize,
        num_samples: usize,
    ) -> Result<usize, AlacError> {
        let _instance = bits.read_small(4)?;
        let unused = bits.read(12)?;
        if unused != 0 {
            return Err(AlacError::Malformed(
                "the unused header bits of the element are not zero".into(),
            ));
        }
        let header = bits.read(4)?;
        let partial = (header >> 3) & 1;
        let bytes_shifted = (header >> 1) & 3;
        if bytes_shifted == 3 {
            return Err(AlacError::Malformed(
                "the shift flag of the element is not usable".into(),
            ));
        }
        let escape = header & 1;
        // WHY: the reference implementation resets the shift to zero in the
        // escape path — an escaped element carries the samples in full and
        // is never shifted afterwards.
        let mut applied_shift = bytes_shifted;
        let num_samples = self.read_partial_length(bits, partial, num_samples)?;

        let mut mix_bits = 0_u32;
        let mut mix_res = 0_i32;
        // A stereo element codes the difference of the two channels, so
        // it needs one bit more than the width of a sample. A stream of
        // 32 bit samples would need 33 bits, which no bit field of the
        // format can hold; such a frame is refused instead of being
        // decoded into a value that does not exist.
        // WHY: see the single-channel path — the subtraction has to be
        // guarded, or a narrow sample width underflows the shift.
        let shift_bits = bytes_shifted * 8;
        if u32::from(self.config.bit_depth) < shift_bits {
            return Err(AlacError::Malformed(format!(
                "a stereo element of {} bit samples cannot carry a shift of {shift_bits} bits",
                self.config.bit_depth
            )));
        }
        let mut chan_bits = self.config.bit_depth as u32 - shift_bits + 1;
        if chan_bits > 32 {
            return Err(AlacError::Malformed(format!(
                "a stereo element of {} bit samples needs {chan_bits} bits",
                self.config.bit_depth
            )));
        }

        if escape == 0 {
            mix_bits = bits.read(8)?;
            mix_res = bits.read(8)? as i8 as i32;
            let mode_byte = bits.read(8)?;
            let mode_u = (mode_byte >> 4) as u8;
            let den_shift_u = mode_byte & 0x0f;
            let coefficient_byte = bits.read(8)?;
            let pb_factor_u = coefficient_byte >> 5;
            let num_u = (coefficient_byte & 0x1f) as usize;
            let mut coefficients_u = [0_i16; MAX_COEFFICIENTS];
            for coefficient in coefficients_u.iter_mut().take(num_u) {
                *coefficient = bits.read(16)? as i16;
            }
            let mode_byte = bits.read(8)?;
            let mode_v = (mode_byte >> 4) as u8;
            let den_shift_v = mode_byte & 0x0f;
            let coefficient_byte = bits.read(8)?;
            let pb_factor_v = coefficient_byte >> 5;
            let num_v = (coefficient_byte & 0x1f) as usize;
            let mut coefficients_v = [0_i16; MAX_COEFFICIENTS];
            for coefficient in coefficients_v.iter_mut().take(num_v) {
                *coefficient = bits.read(16)? as i16;
            }

            let mut shift_reader = None;
            if bytes_shifted != 0 {
                shift_reader = Some(bits.clone());
                bits.advance((bytes_shifted * 8) * 2 * num_samples as u32)?;
            }

            let pb = self.config.pb as u32;
            let mut position = bits.bit_position();

            let params_u = AgParams::new(
                self.config.mb as u32,
                (pb * pb_factor_u) / 4,
                self.config.kb as u32,
                num_samples as u32,
                num_samples as u32,
                self.config.max_run as u32,
            );
            dyn_decomp(
                &params_u,
                bits.data(),
                &mut position,
                &mut self.predictor,
                num_samples,
                chan_bits,
            )?;
            bits.seek_to_bit(position);
            if mode_u == 0 {
                unpc_block(
                    &self.predictor,
                    &mut self.mix_u,
                    num_samples,
                    &mut coefficients_u,
                    num_u,
                    chan_bits,
                    den_shift_u,
                )?;
            } else {
                unpc_block_in_place(&mut self.predictor, num_samples, chan_bits)?;
                unpc_block(
                    &self.predictor,
                    &mut self.mix_u,
                    num_samples,
                    &mut coefficients_u,
                    num_u,
                    chan_bits,
                    den_shift_u,
                )?;
            }

            let params_v = AgParams::new(
                self.config.mb as u32,
                (pb * pb_factor_v) / 4,
                self.config.kb as u32,
                num_samples as u32,
                num_samples as u32,
                self.config.max_run as u32,
            );
            let mut position = bits.bit_position();
            dyn_decomp(
                &params_v,
                bits.data(),
                &mut position,
                &mut self.predictor,
                num_samples,
                chan_bits,
            )?;
            bits.seek_to_bit(position);
            if mode_v == 0 {
                unpc_block(
                    &self.predictor,
                    &mut self.mix_v,
                    num_samples,
                    &mut coefficients_v,
                    num_v,
                    chan_bits,
                    den_shift_v,
                )?;
            } else {
                unpc_block_in_place(&mut self.predictor, num_samples, chan_bits)?;
                unpc_block(
                    &self.predictor,
                    &mut self.mix_v,
                    num_samples,
                    &mut coefficients_v,
                    num_v,
                    chan_bits,
                    den_shift_v,
                )?;
            }

            if let Some(mut reader) = shift_reader {
                for index in 0..num_samples * 2 {
                    self.shift_buffer[index] = reader.read(bytes_shifted * 8)? as u16;
                }
            }
        } else {
            chan_bits = self.config.bit_depth as u32;
            read_uncompressed_pair(
                bits,
                &mut self.mix_u,
                &mut self.mix_v,
                num_samples,
                chan_bits,
            )?;
            // The escape path carries the samples in full: the shift of the
            // header does not apply to them.
            applied_shift = 0;
        }

        let mixed = unmix(
            &self.mix_u[..num_samples],
            &self.mix_v[..num_samples],
            mix_bits,
            mix_res,
        );
        for index in 0..num_samples {
            let left = if matches!(self.config.bit_depth, 24 | 32) && applied_shift != 0 {
                apply_shift(
                    mixed.left[index],
                    self.shift_buffer[index * 2],
                    applied_shift,
                )
            } else {
                mixed.left[index]
            };
            let right = if matches!(self.config.bit_depth, 24 | 32) && applied_shift != 0 {
                apply_shift(
                    mixed.right[index],
                    self.shift_buffer[index * 2 + 1],
                    applied_shift,
                )
            } else {
                mixed.right[index]
            };
            let left_position = channel_index + index * num_channels;
            let right_position = left_position + 1;
            if left_position < samples.len() {
                samples[left_position] = left;
            }
            if right_position < samples.len() {
                samples[right_position] = right;
            }
        }
        Ok(num_samples)
    }

    /// Reads the length of a partial frame, if the frame announces one.
    pub(crate) fn read_partial_length(
        &self,
        bits: &mut BitReader<'_>,
        partial: u32,
        num_samples: usize,
    ) -> Result<usize, AlacError> {
        if partial == 0 {
            return Ok(num_samples);
        }
        let high = bits.read(16)?;
        let low = bits.read(16)?;
        let length = ((high << 16) | low) as usize;
        if length > self.config.frame_length as usize {
            return Err(AlacError::FrameTooLong {
                announced: length as u32,
                limit: self.config.frame_length,
            });
        }
        Ok(length)
    }
}
