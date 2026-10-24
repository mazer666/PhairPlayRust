//! H.264 bitstream parsing and RTP FU-A reassembly for PhairPlayRust.
//!
//! WHAT: this crate reads the parts of an H.264 bitstream that a
//!      receiver needs before it can decode a picture: the NAL units of
//!      an Annex-B stream, the sequence and picture parameter sets, and
//!      the fragmentation units that an RTP sender splits a large NAL
//!      unit into. It also converts between the Annex-B stream and the
//!      length-prefixed (AVCC) container.
//!
//! WHY: the bitstream arrives from the network, and the Exp-Golomb codes
//!      of the parameter sets are a classic overflow trap. The reader in
//!      `bits` counts in `u64` and clamps, so a sequence parameter set
//!      with 31 leading zero bits is decoded instead of overflowing the
//!      shift. Every loop in the parsers is bounded by the number of
//!      elements the specification allows.
//!
//! EXAMPLE:
//! ```
//! use phairplay_h264::{split_annex_b, SequenceParameterSet};
//!
//! let stream = [0, 0, 0, 1, 0x67, 0x42, 0xc0, 0x1e];
//! let units = split_annex_b(&stream);
//! assert_eq!(units.len(), 1);
//! assert_eq!(units[0].nal_type, phairplay_h264::NalUnitType::Sps);
//! ```

#![warn(missing_docs)]

mod bits;
mod error;
mod nal;
mod pps;
mod rtp;
mod sps;
mod vui;

pub use bits::{BitReader, MAX_LEADING_ZERO_BITS, unescape_rbsp};
pub use error::H264Error;
pub use nal::{NalUnit, NalUnitType, annex_b_to_avcc, avcc_to_annex_b, split_annex_b, to_annex_b};
pub use pps::PictureParameterSet;
pub use rtp::{
    FU_A_PAYLOAD_TYPE, FuAReassembler, MAX_REASSEMBLED_NAL_SIZE, SINGLE_NAL_PAYLOAD_TYPE,
    STAP_A_PAYLOAD_TYPE,
};
pub use sps::SequenceParameterSet;
pub use vui::{HrdParameters, VuiParameters};
