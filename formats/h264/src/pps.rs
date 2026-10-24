// WHAT: the parser for a picture parameter set (PPS).
// WHY: the PPS completes the description of a video stream: it names the
//      slice groups, the reference picture indices and the quantiser
//      offsets. It is optional in a stream, so the parser treats every
//      field as optional too.
// EXAMPLE: `PictureParameterSet::from_nal_payload` turns the RBSP of a
//          PPS NAL unit into a structure.

use crate::bits::{BitReader, unescape_rbsp};
use crate::error::H264Error;

/// The largest number of slice groups the parser accepts.
const MAX_SLICE_GROUPS: u32 = 8;

/// A parsed picture parameter set.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PictureParameterSet {
    /// The identifier of this parameter set.
    pub pic_parameter_set_id: u32,
    /// The identifier of the sequence parameter set it belongs to.
    pub seq_parameter_set_id: u32,
    /// Whether the entropy coding is not the CAVLC one.
    pub entropy_coding_mode_flag: bool,
    /// Whether the picture order count of a field is present.
    pub bottom_field_pic_order_in_frame_present_flag: bool,
    /// The number of slice groups, minus one.
    pub num_slice_groups_minus1: u32,
    /// The type of the slice group map.
    pub slice_group_map_type: u32,
    /// The run lengths of the slice group map of type zero.
    pub run_length_minus1: Vec<u32>,
    /// The top left corners of the slice group map of type two.
    pub top_left: Vec<u32>,
    /// The bottom right corners of the slice group map of type two.
    pub bottom_right: Vec<u32>,
    /// Whether the slice group map changes over time.
    pub slice_group_change_direction_flag: bool,
    /// The rate of the slice group change, minus one.
    pub slice_group_change_rate_minus1: u32,
    /// The number of active reference pictures of list zero, minus one.
    pub num_ref_idx_l0_default_active_minus1: u32,
    /// The number of active reference pictures of list one, minus one.
    pub num_ref_idx_l1_default_active_minus1: u32,
    /// Whether the prediction is weighted.
    pub weighted_pred_flag: bool,
    /// The mode of the weighted bi-prediction.
    pub weighted_bipred_idc: u32,
    /// The initial quantiser of the pictures, minus 26.
    pub pic_init_qp_minus26: i32,
    /// The initial quantiser of the field pictures, minus 26.
    pub pic_init_qs_minus26: i32,
    /// The offset of the chroma quantiser.
    pub chroma_qp_index_offset: i32,
    /// Whether the deblocking filter is signalled.
    pub deblocking_filter_control_present_flag: bool,
    /// Whether the prediction is restricted to intra blocks.
    pub constrained_intra_pred_flag: bool,
    /// Whether the redundant picture count is present.
    pub redundant_pic_cnt_present_flag: bool,
    /// Whether the eight by eight transform is used.
    pub transform_8x8_mode_flag: bool,
    /// Whether a picture scaling matrix follows.
    pub pic_scaling_matrix_present_flag: bool,
    /// The second offset of the chroma quantiser.
    pub second_chroma_qp_index_offset: Option<i32>,
}

impl PictureParameterSet {
    /// Parses a picture parameter set from its RBSP.
    ///
    /// # Errors
    ///
    /// Returns an error when the bitstream ends inside the parameter set
    /// or when it announces an implausible value.
    pub fn parse(rbsp: &[u8]) -> Result<PictureParameterSet, H264Error> {
        let mut reader = BitReader::new(rbsp);
        let mut pps = PictureParameterSet {
            pic_parameter_set_id: reader.read_ue()?,
            seq_parameter_set_id: reader.read_ue()?,
            entropy_coding_mode_flag: reader.read_flag()?,
            bottom_field_pic_order_in_frame_present_flag: reader.read_flag()?,
            num_slice_groups_minus1: reader.read_ue()?,
            ..PictureParameterSet::default()
        };
        if pps.num_slice_groups_minus1 > MAX_SLICE_GROUPS {
            return Err(H264Error::UnsupportedFeature(format!(
                "{} slice groups are not supported",
                pps.num_slice_groups_minus1 + 1
            )));
        }
        if pps.num_slice_groups_minus1 > 0 {
            pps.slice_group_map_type = reader.read_ue()?;
            match pps.slice_group_map_type {
                0 => {
                    for _ in 0..=pps.num_slice_groups_minus1 {
                        pps.run_length_minus1.push(reader.read_ue()?);
                    }
                }
                2 => {
                    for _ in 0..pps.num_slice_groups_minus1 {
                        pps.top_left.push(reader.read_ue()?);
                        pps.bottom_right.push(reader.read_ue()?);
                    }
                }
                3..=6 => {
                    pps.slice_group_change_direction_flag = reader.read_flag()?;
                    pps.slice_group_change_rate_minus1 = reader.read_ue()?;
                }
                _ => {
                    return Err(H264Error::UnsupportedFeature(format!(
                        "slice group map type {} is not supported",
                        pps.slice_group_map_type
                    )));
                }
            }
        }
        pps.num_ref_idx_l0_default_active_minus1 = reader.read_ue()?;
        pps.num_ref_idx_l1_default_active_minus1 = reader.read_ue()?;
        pps.weighted_pred_flag = reader.read_flag()?;
        pps.weighted_bipred_idc = reader.read_bits(2)?;
        pps.pic_init_qp_minus26 = reader.read_se()?;
        pps.pic_init_qs_minus26 = reader.read_se()?;
        pps.chroma_qp_index_offset = reader.read_se()?;
        pps.deblocking_filter_control_present_flag = reader.read_flag()?;
        pps.constrained_intra_pred_flag = reader.read_flag()?;
        pps.redundant_pic_cnt_present_flag = reader.read_flag()?;

        if reader.more_rbsp_data() {
            pps.transform_8x8_mode_flag = reader.read_flag()?;
            pps.pic_scaling_matrix_present_flag = reader.read_flag()?;
            if pps.pic_scaling_matrix_present_flag {
                let count = if pps.transform_8x8_mode_flag { 12 } else { 6 };
                for _ in 0..count {
                    if reader.read_flag()? {
                        read_scaling_list(&mut reader)?;
                    }
                }
            }
            pps.second_chroma_qp_index_offset = Some(reader.read_se()?);
        }

        Ok(pps)
    }

    /// Parses a picture parameter set from the payload of a NAL unit.
    ///
    /// # Errors
    ///
    /// Returns an error when the payload is malformed.
    pub fn from_nal_payload(payload: &[u8]) -> Result<PictureParameterSet, H264Error> {
        PictureParameterSet::parse(&unescape_rbsp(payload))
    }
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

    /// The payload of a baseline PPS, as sent by many cameras. The NAL
    /// unit header byte is not part of it.
    const SAMPLE_PPS: &[u8] = &[0xce, 0x38, 0x80];

    #[test]
    fn parses_a_baseline_pps() {
        let pps = match PictureParameterSet::from_nal_payload(SAMPLE_PPS) {
            Ok(pps) => pps,
            Err(err) => panic!("{err}"),
        };
        assert_eq!(pps.pic_parameter_set_id, 0);
        assert_eq!(pps.seq_parameter_set_id, 0);
        assert!(!pps.entropy_coding_mode_flag);
        assert_eq!(pps.num_slice_groups_minus1, 0);
        assert_eq!(pps.weighted_bipred_idc, 0);
        assert_eq!(pps.pic_init_qp_minus26, 0);
        assert_eq!(pps.chroma_qp_index_offset, 0);
        assert!(!pps.deblocking_filter_control_present_flag);
    }

    #[test]
    fn reports_a_truncated_pps() {
        assert!(matches!(
            PictureParameterSet::parse(&[0x68]),
            Err(H264Error::Truncated { .. })
        ));
    }

    #[test]
    fn rejects_too_many_slice_groups() {
        // A slice group count of 2^32 is not plausible.
        let mut bytes = vec![0x68];
        bytes.extend_from_slice(&[0x80]);
        assert!(PictureParameterSet::parse(&bytes).is_err());
    }
}
