// WHAT: the ALAC decoder, the port of Apple's reference decoder.
// WHY: an AirPlay sender that streams lossless audio sends ALAC frames
//      that hold a mono element, a stereo element or both, followed by
//      the coded residuals. The decoder has to walk the elements, undo
//      the prediction and the mixing, and hand the samples to the sink.
//      Every field of a frame comes from the network, so every read is
//      checked and every length is compared with the configuration.
// EXAMPLE: `AlacDecoder::decode` turns one frame into the interleaved
//          samples of the stream.
//
// ATTRIBUTION: this file is a native Rust port of Apple's ALAC decoder
// (`ALACDecoder.cpp` and `ALACDecoder.h`, Copyright (c) 2011 Apple
// Inc., licensed under the Apache License, Version 2.0). The port was
// written for PhairPlayRust; no C code was copied, the raw pointers
// became slices with explicit lengths, and the unchecked reads of the
// reference implementation became errors.

use crate::bits::BitReader;
use crate::config::AlacConfig;
use crate::error::AlacError;
use crate::framing::{skip_data_stream_element, skip_fill_element};
use crate::matrix::pack_samples;

/// The element tag of a single channel element.
const ID_SCE: u8 = 0;

/// The element tag of a channel pair element.
const ID_CPE: u8 = 1;

/// The element tag of a coupling channel element.
const ID_CCE: u8 = 2;

/// The element tag of a low frequency effects element.
const ID_LFE: u8 = 3;

/// The element tag of a data stream element.
const ID_DSE: u8 = 4;

/// The element tag of a programme configuration element.
const ID_PCE: u8 = 5;

/// The element tag of a fill element.
const ID_FIL: u8 = 6;

/// The element tag that ends a frame.
const ID_END: u8 = 7;

/// The decoded samples of one frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedAudio {
    /// The interleaved samples of the frame.
    pub samples: Vec<i32>,
    /// The number of frames that the samples hold.
    pub frames: u32,
    /// The number of channels of the stream.
    pub channels: u8,
}

impl DecodedAudio {
    /// Returns the sample of a frame and a channel.
    ///
    /// # Example
    ///
    /// ```
    /// use phairplay_alac::DecodedAudio;
    ///
    /// let audio = DecodedAudio { samples: vec![1, 2, 3, 4], frames: 2, channels: 2 };
    /// assert_eq!(audio.sample(1, 0), Some(3));
    /// assert_eq!(audio.sample(9, 0), None);
    /// ```
    pub fn sample(&self, frame: u32, channel: u8) -> Option<i32> {
        let index = frame as usize * self.channels as usize + channel as usize;
        self.samples.get(index).copied()
    }

    /// Packs the samples into the byte layout of the format.
    ///
    /// # Errors
    ///
    /// Returns an error when the bit depth is not one of 16, 20, 24 or
    /// 32.
    pub fn to_pcm_bytes(&self, bit_depth: u8) -> Result<Vec<u8>, AlacError> {
        pack_samples(&self.samples, bit_depth)
    }
}

/// The decoder of an ALAC stream.
#[derive(Debug, Clone)]
pub struct AlacDecoder {
    pub(crate) config: AlacConfig,
    pub(crate) mix_u: Vec<i32>,
    pub(crate) mix_v: Vec<i32>,
    pub(crate) predictor: Vec<i32>,
    pub(crate) shift_buffer: Vec<u16>,
}

impl AlacDecoder {
    /// Creates a decoder for a stream.
    ///
    /// # Errors
    ///
    /// Returns an error when the configuration is not usable.
    ///
    /// # Example
    ///
    /// ```
    /// use phairplay_alac::{AlacConfig, AlacDecoder};
    ///
    /// let config = AlacConfig { frame_length: 4096, bit_depth: 16, num_channels: 2, ..AlacConfig::default() };
    /// let decoder = AlacDecoder::new(config).unwrap_or_else(|err| panic!("{err}"));
    /// assert_eq!(decoder.config().frame_length, 4096);
    /// ```
    pub fn new(config: AlacConfig) -> Result<AlacDecoder, AlacError> {
        config.check()?;
        let frames = config.frame_length as usize;
        Ok(AlacDecoder {
            config,
            mix_u: vec![0; frames],
            mix_v: vec![0; frames],
            predictor: vec![0; frames],
            shift_buffer: vec![0; frames * 2],
        })
    }

    /// Returns the configuration of the stream.
    pub fn config(&self) -> &AlacConfig {
        &self.config
    }

    /// Decodes one frame of the stream.
    ///
    /// # Errors
    ///
    /// Returns an error when the frame is malformed or when it announces
    /// more samples than the configuration allows.
    pub fn decode(&mut self, frame: &[u8]) -> Result<DecodedAudio, AlacError> {
        self.decode_with_samples(frame, self.config.frame_length)
    }

    /// Decodes one frame of the stream with an explicit sample count.
    ///
    /// # Errors
    ///
    /// Returns an error when the frame is malformed or when it announces
    /// more samples than the configuration allows.
    pub fn decode_with_samples(
        &mut self,
        frame: &[u8],
        num_samples: u32,
    ) -> Result<DecodedAudio, AlacError> {
        let num_channels = self.config.num_channels as usize;
        let capacity = (self.config.frame_length as usize).max(num_samples as usize);
        self.mix_u.resize(capacity, 0);
        self.mix_v.resize(capacity, 0);
        self.predictor.resize(capacity, 0);
        self.shift_buffer.resize(capacity * 2, 0);

        let mut bits = BitReader::new(frame);
        let mut samples: Vec<i32> = vec![0; num_samples as usize * num_channels];
        let mut channel_index = 0_usize;
        let mut total_samples = num_samples as usize;

        loop {
            if !bits.has_more_data() {
                return Err(AlacError::Truncated {
                    needed: 3,
                    available: bits.bits_remaining(),
                });
            }
            let tag = bits.read_small(3)?;
            match tag {
                ID_SCE | ID_LFE => {
                    total_samples = self.read_single_channel(
                        &mut bits,
                        &mut samples,
                        channel_index,
                        num_channels,
                        total_samples,
                    )?;
                    channel_index += 1;
                }
                ID_CPE => {
                    if channel_index + 2 > num_channels {
                        break;
                    }
                    total_samples = self.read_channel_pair(
                        &mut bits,
                        &mut samples,
                        channel_index,
                        num_channels,
                        total_samples,
                    )?;
                    channel_index += 2;
                }
                ID_CCE | ID_PCE => {
                    return Err(AlacError::Unsupported(
                        "the coupling and the programme configuration element are not supported"
                            .into(),
                    ));
                }
                ID_DSE => {
                    skip_data_stream_element(&mut bits)?;
                }
                ID_FIL => {
                    skip_fill_element(&mut bits)?;
                }
                ID_END => {
                    bits.byte_align();
                    break;
                }
                other => {
                    return Err(AlacError::Malformed(format!(
                        "the element tag {other} is not known"
                    )));
                }
            }
            if channel_index >= num_channels {
                break;
            }
        }

        Ok(DecodedAudio {
            samples,
            frames: total_samples as u32,
            channels: self.config.num_channels,
        })
    }
}

#[cfg(test)]
#[path = "decoder_tests.rs"]
mod tests;
