//! The H.264 sequence parameter set parser.
//!
//! WHY: an AirPlay `ANNOUNCE` never states the picture size — the resolution
//! only exists inside the sequence parameter set. A receiver that guesses the
//! size configures its decoder wrong and the picture comes out green or
//! scaled. The parser therefore reads the real dimensions out of the
//! bitstream, with the same safety rules as the reference implementation: the
//! Exp-Golomb reader works in 64-bit arithmetic and rejects codes that do
//! not fit, so a crafted SPS cannot overflow into a bogus picture size.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_airplay::h264::parse_sps;
//!
//! // A 1280x720 High-profile sequence parameter set.
//! let sps = [
//!     0x67, 0x64, 0x00, 0x28, 0xac, 0xd9, 0x40, 0x50, 0x05, 0xbb, 0x01, 0x6a,
//!     0x02, 0x02, 0x02, 0x80, 0x00, 0x00, 0x30, 0x08, 0x00, 0x00, 0x1e, 0x47,
//!     0x81, 0x8c, 0xb,
//! ];
//! let info = parse_sps(&sps).expect("valid sps");
//! assert_eq!((info.width, info.height), (1280, 720));
//! ```

/// The maximum value an Exp-Golomb code may carry.
///
/// Sixteen bits cover every field of a real sequence parameter set by a wide
/// margin — the largest is `pic_width_in_mbs_minus1`, which is 511 for 8K.
pub const MAX_UE_VALUE: u32 = 0xFFFF;

/// What a sequence parameter set says about the stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpsInfo {
    /// The `profile_idc` of the stream.
    pub profile_idc: u8,
    /// The `level_idc` of the stream.
    pub level_idc: u8,
    /// The picture width in pixels.
    pub width: u32,
    /// The picture height in pixels.
    pub height: u32,
}

/// The NAL unit type of a sequence parameter set.
pub const NAL_TYPE_SPS: u8 = 7;

/// The NAL unit type of a picture parameter set.
pub const NAL_TYPE_PPS: u8 = 8;

/// Strips an Annex-B start code from a NAL unit, if there is one.
#[must_use]
pub fn strip_start_code(data: &[u8]) -> &[u8] {
    if data.len() >= 4 && data[..4] == [0, 0, 0, 1] {
        &data[4..]
    } else if data.len() >= 3 && data[..3] == [0, 0, 1] {
        &data[3..]
    } else {
        data
    }
}

/// The NAL unit type of a NAL unit, if the buffer starts with one.
#[must_use]
pub fn nal_type(data: &[u8]) -> Option<u8> {
    let nal = strip_start_code(data);
    nal.first().map(|byte| byte & 0x1F)
}

/// Parses the resolution out of a sequence parameter set.
///
/// Returns `None` when the buffer is too short, is not an SPS, or uses a
/// bitstream feature the parser does not understand. A malformed SPS is
/// rejected, never repaired.
#[must_use]
pub fn parse_sps(sps: &[u8]) -> Option<SpsInfo> {
    let nal = strip_start_code(sps);
    if nal.len() < 4 || nal[0] & 0x1F != NAL_TYPE_SPS {
        return None;
    }
    let mut reader = BitReader::new(&nal[1..]);

    let profile_idc = reader.read_bits(8)? as u8;
    reader.read_bits(8)?; // constraint flags and two reserved zero bits
    let level_idc = reader.read_bits(8)? as u8;
    reader.read_ue()?; // seq_parameter_set_id

    let mut chroma_format_idc = 1u32;
    let mut separate_colour_plane = 0u32;

    if is_high_profile(profile_idc) {
        chroma_format_idc = reader.read_ue()?;
        if chroma_format_idc == 3 {
            separate_colour_plane = reader.read_bits(1)?;
        }
        reader.read_ue()?; // bit_depth_luma_minus8
        reader.read_ue()?; // bit_depth_chroma_minus8
        reader.read_bits(1)?; // qpprime_y_zero_transform_bypass_flag
        if reader.read_bits(1)? == 1 {
            let count = if chroma_format_idc != 3 { 8 } else { 12 };
            for index in 0..count {
                if reader.read_bits(1)? == 1 {
                    reader.skip_scaling_list(if index < 6 { 16 } else { 64 })?;
                }
            }
        }
    }

    reader.read_ue()?; // log2_max_frame_num_minus4
    let pic_order_count_type = reader.read_ue()?;
    match pic_order_count_type {
        0 => {
            reader.read_ue()?; // log2_max_pic_order_cnt_lsb_minus4
        }
        1 => {
            reader.read_bits(1)?; // delta_pic_order_always_zero_flag
            reader.read_se()?; // offset_for_non_ref_pic
            reader.read_se()?; // offset_for_top_to_bottom_field
            let count = reader.read_ue()?;
            for _ in 0..count {
                reader.read_se()?;
            }
        }
        _ => {}
    }

    reader.read_ue()?; // max_num_ref_frames
    reader.read_bits(1)?; // gaps_in_frame_num_value_allowed_flag

    let width_in_mbs = reader.read_ue()?;
    let height_in_map_units = reader.read_ue()?;
    let frame_mbs_only = reader.read_bits(1)?;
    if frame_mbs_only == 0 {
        reader.read_bits(1)?; // mb_adaptive_frame_field_flag
    }
    reader.read_bits(1)?; // direct_8x8_inference_flag

    let mut crop = [0u32; 4];
    if reader.read_bits(1)? == 1 {
        crop[0] = reader.read_ue()?;
        crop[1] = reader.read_ue()?;
        crop[2] = reader.read_ue()?;
        crop[3] = reader.read_ue()?;
    }

    let coded_width = width_in_mbs.saturating_add(1).saturating_mul(16);
    let coded_height = height_in_map_units
        .saturating_add(1)
        .saturating_mul(16)
        .saturating_mul(2 - frame_mbs_only);

    let chroma_array_type = if separate_colour_plane == 1 {
        0
    } else {
        chroma_format_idc
    };
    let sub_width_c: u32 = if chroma_array_type == 0 { 1 } else { 2 };
    let sub_height_c: u32 = if chroma_array_type == 1 { 2 } else { 1 };
    let crop_unit_x: u32 = if chroma_array_type == 0 { 1 } else { sub_width_c };
    let crop_unit_y: u32 = if chroma_array_type == 0 {
        2 - frame_mbs_only
    } else {
        sub_height_c.saturating_mul(2 - frame_mbs_only)
    };

    let width = coded_width.saturating_sub(crop[0].saturating_add(crop[1]).saturating_mul(crop_unit_x));
    let height =
        coded_height.saturating_sub(crop[2].saturating_add(crop[3]).saturating_mul(crop_unit_y));
    if width == 0 || height == 0 {
        return None;
    }

    Some(SpsInfo {
        profile_idc,
        level_idc,
        width,
        height,
    })
}

/// Whether the profile carries the extra high-profile fields.
fn is_high_profile(profile_idc: u8) -> bool {
    matches!(
        profile_idc,
        100 | 110 | 122 | 244 | 44 | 83 | 86 | 118 | 128 | 138 | 139 | 134 | 135
    )
}

/// A most-significant-bit-first bit reader.
///
/// Every read returns `None` on underflow, so a truncated or hostile bitstream
/// can never panic the receiver.
struct BitReader<'a> {
    data: &'a [u8],
    byte_position: usize,
    bit_position: u8,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            byte_position: 0,
            bit_position: 7,
        }
    }

    fn read_bit(&mut self) -> Option<u32> {
        let byte = *self.data.get(self.byte_position)?;
        let bit = (byte >> self.bit_position) & 1;
        if self.bit_position == 0 {
            self.bit_position = 7;
            self.byte_position += 1;
        } else {
            self.bit_position -= 1;
        }
        Some(u32::from(bit))
    }

    fn read_bits(&mut self, count: u32) -> Option<u32> {
        let mut value = 0u32;
        for _ in 0..count {
            value = (value << 1) | self.read_bit()?;
        }
        Some(value)
    }

    /// Reads an unsigned Exp-Golomb code.
    ///
    /// WHY the 64-bit arithmetic: with 31 leading zeros the plain 32-bit
    /// formula overflows and can return a wrapped value. The sequence
    /// parameter set is attacker-controlled input — any device on the local
    /// network can send one — and a wrapped value would turn into a bogus
    /// picture size.
    fn read_ue(&mut self) -> Option<u32> {
        let mut leading_zeros = 0u32;
        while self.read_bit()? == 0 {
            leading_zeros += 1;
            if leading_zeros > 31 {
                return None;
            }
        }
        if leading_zeros == 0 {
            return Some(0);
        }
        let value = (1u64 << leading_zeros) - 1 + u64::from(self.read_bits(leading_zeros)?);
        if value > u64::from(MAX_UE_VALUE) {
            return None;
        }
        Some(value as u32)
    }

    /// Reads a signed Exp-Golomb code.
    fn read_se(&mut self) -> Option<i32> {
        let code = self.read_ue()?;
        let value = if code % 2 == 1 {
            ((code as i64) + 1) / 2
        } else {
            -((code as i64) / 2)
        };
        i32::try_from(value).ok()
    }

    /// Skips a scaling list of `size` entries.
    fn skip_scaling_list(&mut self, size: u32) -> Option<()> {
        let mut last_scale = 8i32;
        let mut next_scale = 8i32;
        for _ in 0..size {
            if next_scale != 0 {
                let delta = self.read_se()?;
                next_scale = (last_scale + delta + 256) % 256;
            }
            if next_scale != 0 {
                last_scale = next_scale;
            }
        }
        Some(())
    }
}

#[cfg(test)]
mod tests;
