// WHAT: the VUI (video usability information) part of a sequence
//      parameter set.
// WHY: the VUI carries the resolution hints, the colour information and
//      the timing information that a receiver needs to configure its
//      sink. It is optional, so every field of the parser is optional
//      too.
// EXAMPLE: a VUI with `aspect_ratio_idc` 1 describes a square pixel
//          aspect ratio.

use crate::bits::BitReader;
use crate::error::H264Error;

/// The HRD (hypothetical reference decoder) parameters.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HrdParameters {
    /// The number of coded picture buffers, minus one.
    pub cpb_cnt_minus1: u32,
    /// The scale of the bit rate values.
    pub bit_rate_scale: u8,
    /// The scale of the buffer size values.
    pub cpb_size_scale: u8,
    /// The bit rate of every buffer, minus one.
    pub bit_rate_value_minus1: Vec<u32>,
    /// The size of every buffer, minus one.
    pub cpb_size_value_minus1: Vec<u32>,
    /// Whether every buffer uses constant bit rate.
    pub cbr_flag: Vec<bool>,
    /// The length of the initial delay, minus one.
    pub initial_cpb_removal_delay_length_minus1: u8,
    /// The length of the removal delay, minus one.
    pub cpb_removal_delay_length_minus1: u8,
    /// The length of the output delay, minus one.
    pub dpb_output_delay_length_minus1: u8,
    /// The length of the time offset.
    pub time_offset_length: u8,
}

/// The video usability information of a sequence parameter set.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VuiParameters {
    /// Whether the aspect ratio is present.
    pub aspect_ratio_info_present_flag: bool,
    /// The aspect ratio identifier.
    pub aspect_ratio_idc: u8,
    /// The width of the sample aspect ratio, when it is extended.
    pub sar_width: u16,
    /// The height of the sample aspect ratio, when it is extended.
    pub sar_height: u16,
    /// Whether the overscan information is present.
    pub overscan_info_present_flag: bool,
    /// Whether the stream is meant for overscan display.
    pub overscan_appropriate_flag: bool,
    /// Whether the video signal type is present.
    pub video_signal_type_present_flag: bool,
    /// The video format.
    pub video_format: u8,
    /// Whether the signal uses the full range.
    pub video_full_range_flag: bool,
    /// Whether the colour description is present.
    pub colour_description_present_flag: bool,
    /// The colour primaries.
    pub colour_primaries: u8,
    /// The transfer characteristics.
    pub transfer_characteristics: u8,
    /// The matrix coefficients.
    pub matrix_coefficients: u8,
    /// Whether the chroma sample location is present.
    pub chroma_loc_info_present_flag: bool,
    /// The chroma sample location of the top field.
    pub chroma_sample_loc_type_top_field: u32,
    /// The chroma sample location of the bottom field.
    pub chroma_sample_loc_type_bottom_field: u32,
    /// Whether the timing information is present.
    pub timing_info_present_flag: bool,
    /// The number of units in a tick.
    pub num_units_in_tick: u32,
    /// The time scale.
    pub time_scale: u32,
    /// Whether the frame rate is fixed.
    pub fixed_frame_rate_flag: bool,
    /// Whether the NAL HRD parameters are present.
    pub nal_hrd_parameters_present_flag: bool,
    /// The NAL HRD parameters.
    pub nal_hrd_parameters: Option<HrdParameters>,
    /// Whether the VCL HRD parameters are present.
    pub vcl_hrd_parameters_present_flag: bool,
    /// The VCL HRD parameters.
    pub vcl_hrd_parameters: Option<HrdParameters>,
    /// Whether the decoder has a low delay.
    pub low_delay_hrd_flag: bool,
    /// Whether the picture structure is signalled in the slice header.
    pub pic_struct_present_flag: bool,
    /// Whether the bitstream restriction is present.
    pub bitstream_restriction_flag: bool,
    /// Whether motion vectors may cross picture boundaries.
    pub motion_vectors_over_pic_boundaries_flag: bool,
    /// The maximum number of bytes per picture, as a denominator.
    pub max_bytes_per_pic_denom: u32,
    /// The maximum number of bits per macroblock, as a denominator.
    pub max_bits_per_mb_denom: u32,
    /// The base-two logarithm of the maximum motion vector length.
    pub log2_max_mv_length_horizontal: u32,
    /// The base-two logarithm of the maximum vertical motion vector length.
    pub log2_max_mv_length_vertical: u32,
    /// The maximum number of frames that may be reordered.
    pub max_num_reorder_frames: u32,
    /// The maximum number of frames the decoder may buffer.
    pub max_dec_frame_buffering: u32,
}

impl VuiParameters {
    /// Parses the VUI parameters.
    ///
    /// # Errors
    ///
    /// Returns an error when the bitstream ends inside the parameters.
    pub(crate) fn parse(reader: &mut BitReader<'_>) -> Result<VuiParameters, H264Error> {
        let mut vui = VuiParameters {
            aspect_ratio_info_present_flag: reader.read_flag()?,
            ..VuiParameters::default()
        };

        if vui.aspect_ratio_info_present_flag {
            vui.aspect_ratio_idc = reader.read_bits(8)? as u8;
            if vui.aspect_ratio_idc == 255 {
                vui.sar_width = reader.read_bits(16)? as u16;
                vui.sar_height = reader.read_bits(16)? as u16;
            }
        }

        vui.overscan_info_present_flag = reader.read_flag()?;
        if vui.overscan_info_present_flag {
            vui.overscan_appropriate_flag = reader.read_flag()?;
        }

        vui.video_signal_type_present_flag = reader.read_flag()?;
        if vui.video_signal_type_present_flag {
            vui.video_format = reader.read_bits(3)? as u8;
            vui.video_full_range_flag = reader.read_flag()?;
            vui.colour_description_present_flag = reader.read_flag()?;
            if vui.colour_description_present_flag {
                vui.colour_primaries = reader.read_bits(8)? as u8;
                vui.transfer_characteristics = reader.read_bits(8)? as u8;
                vui.matrix_coefficients = reader.read_bits(8)? as u8;
            }
        }

        vui.chroma_loc_info_present_flag = reader.read_flag()?;
        if vui.chroma_loc_info_present_flag {
            vui.chroma_sample_loc_type_top_field = reader.read_ue()?;
            vui.chroma_sample_loc_type_bottom_field = reader.read_ue()?;
        }

        vui.timing_info_present_flag = reader.read_flag()?;
        if vui.timing_info_present_flag {
            vui.num_units_in_tick = reader.read_bits(32)?;
            vui.time_scale = reader.read_bits(32)?;
            vui.fixed_frame_rate_flag = reader.read_flag()?;
        }

        vui.nal_hrd_parameters_present_flag = reader.read_flag()?;
        if vui.nal_hrd_parameters_present_flag {
            vui.nal_hrd_parameters = Some(HrdParameters::parse(reader)?);
        }
        vui.vcl_hrd_parameters_present_flag = reader.read_flag()?;
        if vui.vcl_hrd_parameters_present_flag {
            vui.vcl_hrd_parameters = Some(HrdParameters::parse(reader)?);
        }
        if vui.nal_hrd_parameters_present_flag || vui.vcl_hrd_parameters_present_flag {
            vui.low_delay_hrd_flag = reader.read_flag()?;
        }

        vui.pic_struct_present_flag = reader.read_flag()?;
        vui.bitstream_restriction_flag = reader.read_flag()?;
        if vui.bitstream_restriction_flag {
            vui.motion_vectors_over_pic_boundaries_flag = reader.read_flag()?;
            vui.max_bytes_per_pic_denom = reader.read_ue()?;
            vui.max_bits_per_mb_denom = reader.read_ue()?;
            vui.log2_max_mv_length_horizontal = reader.read_ue()?;
            vui.log2_max_mv_length_vertical = reader.read_ue()?;
            vui.max_num_reorder_frames = reader.read_ue()?;
            vui.max_dec_frame_buffering = reader.read_ue()?;
        }

        Ok(vui)
    }
}

impl HrdParameters {
    /// Parses the HRD parameters.
    ///
    /// # Errors
    ///
    /// Returns an error when the bitstream ends inside the parameters.
    pub(crate) fn parse(reader: &mut BitReader<'_>) -> Result<HrdParameters, H264Error> {
        let mut hrd = HrdParameters {
            cpb_cnt_minus1: reader.read_ue()?,
            ..HrdParameters::default()
        };
        if hrd.cpb_cnt_minus1 > 31 {
            return Err(H264Error::UnsupportedFeature(format!(
                "{} coded picture buffers are not supported",
                hrd.cpb_cnt_minus1 + 1
            )));
        }
        hrd.bit_rate_scale = reader.read_bits(4)? as u8;
        hrd.cpb_size_scale = reader.read_bits(4)? as u8;
        for _ in 0..=hrd.cpb_cnt_minus1 {
            hrd.bit_rate_value_minus1.push(reader.read_ue()?);
            hrd.cpb_size_value_minus1.push(reader.read_ue()?);
            hrd.cbr_flag.push(reader.read_flag()?);
        }
        hrd.initial_cpb_removal_delay_length_minus1 = reader.read_bits(5)? as u8;
        hrd.cpb_removal_delay_length_minus1 = reader.read_bits(5)? as u8;
        hrd.dpb_output_delay_length_minus1 = reader.read_bits(5)? as u8;
        hrd.time_offset_length = reader.read_bits(5)? as u8;
        Ok(hrd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bits::TestBitWriter;

    /// Parses a VUI, failing the test when the bitstream is broken.
    fn parse_vui(bytes: &[u8]) -> VuiParameters {
        let mut reader = BitReader::new(bytes);
        match VuiParameters::parse(&mut reader) {
            Ok(vui) => vui,
            Err(err) => panic!("{err}"),
        }
    }

    #[test]
    fn parses_a_minimal_vui() {
        // Every flag is zero: 8 bits of zeroes.
        let vui = parse_vui(&[0x00, 0x00]);
        assert!(!vui.aspect_ratio_info_present_flag);
        assert!(!vui.video_signal_type_present_flag);
        assert!(!vui.timing_info_present_flag);
        assert!(!vui.bitstream_restriction_flag);
    }

    #[test]
    fn parses_the_timing_information() {
        // Four flags that are all zero, then the timing information.
        let mut writer = TestBitWriter::new();
        writer.bit(false); // aspect_ratio_info_present_flag
        writer.bit(false); // overscan_info_present_flag
        writer.bit(false); // video_signal_type_present_flag
        writer.bit(false); // chroma_loc_info_present_flag
        writer.bit(true); // timing_info_present_flag
        writer.bits(1_000_000_000, 32); // num_units_in_tick
        writer.bits(60_000_000, 32); // time_scale
        writer.bit(true); // fixed_frame_rate_flag
        writer.bit(false); // nal_hrd_parameters_present_flag
        writer.bit(false); // vcl_hrd_parameters_present_flag
        writer.bit(false); // pic_struct_present_flag
        writer.bit(false); // bitstream_restriction_flag
        let bytes = writer.finish();
        let vui = parse_vui(&bytes);
        assert!(vui.timing_info_present_flag);
        assert_eq!(vui.num_units_in_tick, 1_000_000_000);
        assert_eq!(vui.time_scale, 60_000_000);
        assert!(vui.fixed_frame_rate_flag);
    }

    #[test]
    fn reports_a_truncated_vui() {
        let mut reader = BitReader::new(&[0b0000_1000]);
        assert!(VuiParameters::parse(&mut reader).is_err());
    }
}
