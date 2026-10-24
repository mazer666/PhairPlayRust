//! AAC and AAC-ELD framing for PhairPlayRust.
//!
//! WHAT: this crate reads the framing of an AAC stream: the ADTS header
//!      that describes a frame, the 42 bit header of an AAC-ELD frame,
//!      the number of samples a packet carries and the bit rate the
//!      stream implies.
//!
//! WHY: an AirPlay sender may deliver AAC in the ADTS framing or as raw
//!      AAC-ELD frames, and the receiver has to know the sample rate,
//!      the channel count and the sample count before it can hand a
//!      frame to the audio sink. The frame length is checked against the
//!      bytes that are present, so a hostile length field cannot make
//!      the receiver copy more than the buffer holds.
//!
//! EXAMPLE:
//! ```
//! use phairplay_aac::{split_adts, total_samples, AdtsHeader};
//!
//! let stream = [0xff, 0xf1, 0x50, 0x80, 0x01, 0x1f, 0xfc, 0x00];
//! let frames = split_adts(&stream).unwrap_or_else(|err| panic!("{err}"));
//! assert_eq!(frames[0].sample_rate, 44_100);
//! assert_eq!(total_samples(&frames), 1024);
//! ```

#![warn(missing_docs)]

mod adts;
mod config;
mod eld;
mod error;

pub use adts::{
    ADTS_HEADER_LENGTH, ADTS_HEADER_LENGTH_WITH_CRC, AacProfile, AacVersion, AdtsHeader,
    CHANNEL_CONFIGURATIONS, SAMPLE_RATES, split_adts, total_samples,
};
pub use config::AacConfig;
pub use eld::{
    ELD_HEADER_BITS, ELD_HEADER_BYTES, ELD_SAMPLES_PER_FRAME, ELD_SYNCWORD_MONO,
    ELD_SYNCWORD_MULTI, ELD_SYNCWORD_STEREO, EldFrameHeader,
};
pub use error::AacError;
