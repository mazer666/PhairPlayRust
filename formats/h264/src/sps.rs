// WHAT: the parser for a sequence parameter set (SPS).
// WHY: the SPS is the first thing a receiver needs from a video stream:
//      it names the profile, the level and the picture size. It is also
//      the first thing a hostile peer can attack, because it is parsed
//      before anything else and because it contains Exp-Golomb codes.
//      The parser therefore bounds every loop by the number of elements
//      the format allows.
// EXAMPLE: `SequenceParameterSet::parse` turns the RBSP of an SPS NAL
//          unit into a structure with the picture size in pixels.

use crate::bits::{BitReader, unescape_rbsp};
use crate::error::H264Error;
use crate::vui::VuiParameters;

/// The largest number of reference frames the parser accepts.
const MAX_NUM_REF_FRAMES: u32 = 16;

/// The largest number of entries in a picture order count cycle.
const MAX_POC_CYCLE_LENGTH: u32 = 255;

/// The largest picture dimension the parser accepts, in macroblocks.
const MAX_DIMENSION_IN_MBS: u32 = 1024;

/// A parsed sequence parameter set.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SequenceParameterSet {
    /// The profile identifier.
    pub profile_idc: u8,
    /// The constraint flags, in the byte after `profile_idc`.
    pub constraint_flags: u8,
    /// The level identifier.
    pub level_idc: u8,
    /// The identifier of this parameter set.
    pub seq_parameter_set_id: u32,
    /// The chroma format, when the profile announces one.
    pub chroma_format_idc: Option<u32>,
    /// Whether the colour planes are coded separately.
    pub separate_colour_plane_flag: Option<bool>,
    /// The bit depth of the luma samples, minus eight.
    pub bit_depth_luma_minus8: Option<u32>,
    /// The bit depth of the chroma samples, minus eight.
    pub bit_depth_chroma_minus8: Option<u32>,
    /// Whether the transform bypass is allowed.
    pub qpprime_y_zero_transform_bypass_flag: bool,
    /// Whether a scaling matrix follows.
    pub seq_scaling_matrix_present_flag: bool,
    /// The base-two logarithm of the maximum frame number, minus four.
    pub log2_max_frame_num_minus4: u32,
    /// The type of the picture order count.
    pub pic_order_cnt_type: u32,
    /// The base-two logarithm of the maximum order count, minus four.
    pub log2_max_pic_order_cnt_lsb_minus4: Option<u32>,
    /// Whether the order count is always zero.
    pub delta_pic_order_always_zero_flag: Option<bool>,
    /// The offset for pictures that are not references.
    pub offset_for_non_ref_pic: Option<i32>,
    /// The offset between the top and the bottom field.
    pub offset_for_top_to_bottom_field: Option<i32>,
    /// The offsets of the reference frames in the cycle.
    pub offset_for_ref_frame: Vec<i32>,
    /// The maximum number of reference frames.
    pub max_num_ref_frames: u32,
    /// Whether gaps in the frame number are allowed.
    pub gaps_in_frame_num_value_allowed_flag: bool,
    /// The picture width in macroblocks, minus one.
    pub pic_width_in_mbs_minus1: u32,
    /// The picture height in map units, minus one.
    pub pic_height_in_map_units_minus1: u32,
    /// Whether the stream holds frames only.
    pub frame_mbs_only_flag: bool,
    /// Whether macroblocks may be coded as fields.
    pub mb_adaptive_frame_field_flag: bool,
    /// Whether eight by eight inference is used.
    pub direct_8x8_inference_flag: bool,
    /// Whether the picture is cropped.
    pub frame_cropping_flag: bool,
    /// The left crop offset.
    pub frame_crop_left_offset: u32,
    /// The right crop offset.
    pub frame_crop_right_offset: u32,
    /// The top crop offset.
    pub frame_crop_top_offset: u32,
    /// The bottom crop offset.
    pub frame_crop_bottom_offset: u32,
    /// Whether the usability information follows.
    pub vui_parameters_present_flag: bool,
    /// The usability information.
    pub vui_parameters: Option<VuiParameters>,
}

impl SequenceParameterSet {
    /// Parses a sequence parameter set from its RBSP.
    ///
    /// # Errors
    ///
    /// Returns an error when the bitstream ends inside the parameter set
    /// or when it announces an implausible value.
    pub fn parse(rbsp: &[u8]) -> Result<SequenceParameterSet, H264Error> {
        let mut reader = BitReader::new(rbsp);
        let mut sps = SequenceParameterSet {
            profile_idc: reader.read_bits(8)? as u8,
            constraint_flags: reader.read_bits(8)? as u8,
            level_idc: reader.read_bits(8)? as u8,
            seq_parameter_set_id: reader.read_ue()?,
            ..SequenceParameterSet::default()
        };

        if uses_scaling_lists(sps.profile_idc) {
            sps.chroma_format_idc = Some(reader.read_ue()?);
            if sps.chroma_format_idc == Some(3) {
                sps.separate_colour_plane_flag = Some(reader.read_flag()?);
            }
            sps.bit_depth_luma_minus8 = Some(reader.read_ue()?);
            sps.bit_depth_chroma_minus8 = Some(reader.read_ue()?);
            sps.qpprime_y_zero_transform_bypass_flag = reader.read_flag()?;
            sps.seq_scaling_matrix_present_flag = reader.read_flag()?;
            if sps.seq_scaling_matrix_present_flag {
                let count = if sps.chroma_format_idc == Some(3) {
                    12
                } else {
                    8
                };
                for _ in 0..count {
                    if reader.read_flag()? {
                        read_scaling_list(&mut reader)?;
                    }
                }
            }
        }

        sps.log2_max_frame_num_minus4 = reader.read_ue()?;
        sps.pic_order_cnt_type = reader.read_ue()?;
        match sps.pic_order_cnt_type {
            0 => {
                sps.log2_max_pic_order_cnt_lsb_minus4 = Some(reader.read_ue()?);
            }
            1 => {
                sps.delta_pic_order_always_zero_flag = Some(reader.read_flag()?);
                sps.offset_for_non_ref_pic = Some(reader.read_se()?);
                sps.offset_for_top_to_bottom_field = Some(reader.read_se()?);
                let count = reader.read_ue()?;
                if count > MAX_POC_CYCLE_LENGTH {
                    return Err(H264Error::UnsupportedFeature(format!(
                        "a picture order count cycle of {count} entries is not supported"
                    )));
                }
                for _ in 0..count {
                    sps.offset_for_ref_frame.push(reader.read_se()?);
                }
            }
            _ => {}
        }

        sps.max_num_ref_frames = reader.read_ue()?;
        if sps.max_num_ref_frames > MAX_NUM_REF_FRAMES {
            return Err(H264Error::UnsupportedFeature(format!(
                "{} reference frames are not supported",
                sps.max_num_ref_frames
            )));
        }
        sps.gaps_in_frame_num_value_allowed_flag = reader.read_flag()?;
        sps.pic_width_in_mbs_minus1 = reader.read_ue()?;
        sps.pic_height_in_map_units_minus1 = reader.read_ue()?;
        if sps.pic_width_in_mbs_minus1 > MAX_DIMENSION_IN_MBS
            || sps.pic_height_in_map_units_minus1 > MAX_DIMENSION_IN_MBS
        {
            return Err(H264Error::UnsupportedFeature(
                "the picture size is larger than the parser supports".into(),
            ));
        }
        sps.frame_mbs_only_flag = reader.read_flag()?;
        if !sps.frame_mbs_only_flag {
            sps.mb_adaptive_frame_field_flag = reader.read_flag()?;
        }
        sps.direct_8x8_inference_flag = reader.read_flag()?;
        sps.frame_cropping_flag = reader.read_flag()?;
        if sps.frame_cropping_flag {
            sps.frame_crop_left_offset = reader.read_ue()?;
            sps.frame_crop_right_offset = reader.read_ue()?;
            sps.frame_crop_top_offset = reader.read_ue()?;
            sps.frame_crop_bottom_offset = reader.read_ue()?;
        }
        sps.vui_parameters_present_flag = reader.read_flag()?;
        if sps.vui_parameters_present_flag {
            sps.vui_parameters = Some(VuiParameters::parse(&mut reader)?);
        }

        Ok(sps)
    }

    /// Parses a sequence parameter set from the payload of a NAL unit.
    ///
    /// # Errors
    ///
    /// Returns an error when the payload is malformed.
    pub fn from_nal_payload(payload: &[u8]) -> Result<SequenceParameterSet, H264Error> {
        SequenceParameterSet::parse(&unescape_rbsp(payload))
    }

    /// Returns the width of the picture in pixels.
    pub fn width_in_pixels(&self) -> u32 {
        let width = (self.pic_width_in_mbs_minus1 + 1) * 16;
        let crop_x = self.crop_unit_x() * self.frame_crop_left_offset
            + self.crop_unit_x() * self.frame_crop_right_offset;
        width.saturating_sub(crop_x)
    }

    /// Returns the height of the picture in pixels.
    pub fn height_in_pixels(&self) -> u32 {
        let frame_mbs_only = u32::from(self.frame_mbs_only_flag);
        let height = (2 - frame_mbs_only) * (self.pic_height_in_map_units_minus1 + 1) * 16;
        let crop_y = self.crop_unit_y() * self.frame_crop_top_offset
            + self.crop_unit_y() * self.frame_crop_bottom_offset;
        height.saturating_sub(crop_y)
    }

    /// Returns the chroma array type of the stream.
    pub fn chroma_array_type(&self) -> u32 {
        if self.separate_colour_plane_flag == Some(true) {
            return 0;
        }
        self.chroma_format_idc.unwrap_or(1)
    }

    /// Returns the horizontal crop unit.
    fn crop_unit_x(&self) -> u32 {
        match self.chroma_array_type() {
            0 => 1,
            1 => 2,
            2 => 2,
            _ => 1,
        }
    }

    /// Returns the vertical crop unit.
    fn crop_unit_y(&self) -> u32 {
        let frame_mbs_only = u32::from(self.frame_mbs_only_flag);
        match self.chroma_array_type() {
            0 => 2 - frame_mbs_only,
            1 => 2 * (2 - frame_mbs_only),
            2 => 2 - frame_mbs_only,
            _ => 2 - frame_mbs_only,
        }
    }
}

/// Returns `true` when the profile carries the chroma and scaling fields.
fn uses_scaling_lists(profile_idc: u8) -> bool {
    matches!(
        profile_idc,
        100 | 110 | 122 | 244 | 44 | 83 | 86 | 118 | 128 | 138 | 139 | 134 | 135
    )
}

/// Reads a scaling list, as described in clause 7.3.2.1.1.1.
fn read_scaling_list(reader: &mut BitReader<'_>) -> Result<(), H264Error> {
    let mut last_scale: i32 = 8;
    let mut next_scale: i32 = 8;
    for _ in 0..16 {
        if next_scale != 0 {
            let delta = reader.read_se()?;
            next_scale = (last_scale + delta + 256) % 256;
        }
        last_scale = if next_scale == 0 {
            last_scale
        } else {
            next_scale
        };
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    /// The payload of the SPS of a 704x480 baseline stream, as sent by
    /// many cameras. The NAL unit header byte is not part of it.
    const SAMPLE_SPS: &[u8] = &[
        0x42, 0xc0, 0x1e, 0xd9, 0x00, 0xb0, 0x3d, 0xa1, 0x00, 0x00, 0x03, 0x00, 0x01, 0x00, 0x00,
        0x03, 0x00, 0x32, 0x0f, 0x16, 0x2e, 0x48,
    ];

    #[test]
    fn parses_a_baseline_sps() {
        let sps = match SequenceParameterSet::from_nal_payload(SAMPLE_SPS) {
            Ok(sps) => sps,
            Err(err) => panic!("{err}"),
        };
        assert_eq!(sps.profile_idc, 66);
        assert_eq!(sps.level_idc, 30);
        assert_eq!(sps.seq_parameter_set_id, 0);
        assert_eq!(sps.pic_order_cnt_type, 2);
        assert_eq!(sps.max_num_ref_frames, 3);
        assert_eq!(sps.pic_width_in_mbs_minus1, 43);
        assert_eq!(sps.pic_height_in_map_units_minus1, 29);
        assert!(sps.frame_mbs_only_flag);
        assert_eq!(sps.width_in_pixels(), 704);
        assert_eq!(sps.height_in_pixels(), 480);
        assert!(sps.vui_parameters_present_flag);
        let vui = sps.vui_parameters.unwrap_or_default();
        assert!(vui.timing_info_present_flag);
        assert_eq!(vui.time_scale, 50);
    }

    #[test]
    fn reports_a_truncated_sps() {
        assert!(matches!(
            SequenceParameterSet::parse(&[0x67, 0x42]),
            Err(H264Error::Truncated { .. })
        ));
    }

    #[test]
    fn rejects_an_absurd_picture_size() {
        // A width of 2^31 macroblocks cannot be right.
        let mut bytes = vec![0x67, 0x42, 0xc0];
        bytes.extend_from_slice(&[0x80]);
        assert!(SequenceParameterSet::parse(&bytes).is_err());
    }
}
