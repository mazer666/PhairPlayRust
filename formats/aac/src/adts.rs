// WHAT: the parser for an ADTS header.
// WHY: the ADTS header is the framing that a sender uses when it does
//      not put the AAC configuration into a separate structure. It
//      carries the sample rate, the channel configuration and the frame
//      length, so a receiver needs it to set up its sink. The length
//      field is checked against the bytes that are present before
//      anything is copied.
// EXAMPLE: the seven bytes `FF F1 50 80 00 1F FC` describe a
//          low-complexity frame of 44100 Hz with two channels.

use crate::error::AacError;

/// The length of an ADTS header without a CRC.
pub const ADTS_HEADER_LENGTH: usize = 7;

/// The length of an ADTS header with a CRC.
pub const ADTS_HEADER_LENGTH_WITH_CRC: usize = 9;

/// The object type of an AAC stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AacProfile {
    /// The main profile.
    Main,
    /// The low complexity profile, the one that is used almost always.
    LowComplexity,
    /// The scalable sample rate profile.
    ScalableSampleRate,
    /// The long term prediction profile.
    LongTermPrediction,
    /// The low delay profile, which carries 960 instead of 1024 samples.
    LowDelay,
}

impl AacProfile {
    /// Maps the two bit profile field onto a profile.
    ///
    /// # Example
    ///
    /// ```
    /// use phairplay_aac::AacProfile;
    ///
    /// assert_eq!(AacProfile::from_index(1), AacProfile::LowComplexity);
    /// ```
    pub fn from_index(index: u8) -> AacProfile {
        match index {
            0 => AacProfile::Main,
            1 => AacProfile::LowComplexity,
            2 => AacProfile::ScalableSampleRate,
            _ => AacProfile::LongTermPrediction,
        }
    }

    /// Returns the number of samples one frame of this profile carries.
    ///
    /// WHY: the low delay profile codes 960 samples per frame, every
    ///      other profile codes 1024. The receiver needs the number to
    ///      compute the duration of a packet.
    pub fn samples_per_frame(self) -> u32 {
        match self {
            AacProfile::LowDelay => 960,
            _ => 1024,
        }
    }
}

/// The MPEG version an ADTS frame belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AacVersion {
    /// MPEG-4.
    Mpeg4,
    /// MPEG-2.
    Mpeg2,
}

/// The sample rates that the format defines, in hertz.
pub const SAMPLE_RATES: [u32; 12] = [
    96_000, 88_200, 64_000, 48_000, 44_100, 32_000, 24_000, 22_050, 16_000, 12_000, 11_025, 8_000,
];

/// The number of channels that the channel configuration names.
pub const CHANNEL_CONFIGURATIONS: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 8];

/// A parsed ADTS header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdtsHeader {
    /// The MPEG version of the frame.
    pub version: AacVersion,
    /// The layer, which is always zero for AAC.
    pub layer: u8,
    /// Whether the frame carries no CRC.
    pub protection_absent: bool,
    /// The object type of the frame.
    pub profile: AacProfile,
    /// The index of the sample rate.
    pub sampling_frequency_index: u8,
    /// The sample rate in hertz.
    pub sample_rate: u32,
    /// The private bit of the sender.
    pub private_bit: bool,
    /// The channel configuration.
    pub channel_configuration: u8,
    /// The number of channels.
    pub channels: u8,
    /// Whether the frame is the original of the sender.
    pub original_copy: bool,
    /// Whether the frame belongs to a home stream.
    pub home: bool,
    /// The copyright identification bit.
    pub copyright_identification_bit: bool,
    /// The start of the copyright identification.
    pub copyright_identification_start: bool,
    /// The length of the whole frame, header included.
    pub frame_length: u16,
    /// The number of bytes that the decoder buffer holds.
    pub buffer_fullness: u16,
    /// The number of raw data blocks, minus one.
    pub num_raw_data_blocks: u8,
    /// The length of the header in bytes.
    pub header_length: usize,
}

impl AdtsHeader {
    /// Parses an ADTS header.
    ///
    /// # Errors
    ///
    /// Returns an error when the buffer is too short, when the sync word
    /// is missing, or when the frame announces a sample rate or a
    /// channel configuration that the format does not define.
    pub fn parse(bytes: &[u8]) -> Result<AdtsHeader, AacError> {
        if bytes.len() < ADTS_HEADER_LENGTH {
            return Err(AacError::Truncated {
                needed: ADTS_HEADER_LENGTH,
                available: bytes.len(),
            });
        }
        let syncword = ((bytes[0] as u16) << 4) | ((bytes[1] as u16) >> 4);
        if syncword != 0x0fff {
            return Err(AacError::BadSyncWord { syncword });
        }
        let id = (bytes[1] >> 3) & 0x01;
        let layer = (bytes[1] >> 1) & 0x03;
        let protection_absent = (bytes[1] & 0x01) == 1;
        let profile_index = (bytes[2] >> 6) & 0x03;
        let sampling_frequency_index = (bytes[2] >> 2) & 0x0f;
        let private_bit = (bytes[2] >> 1) & 0x01 == 1;
        let channel_configuration = ((bytes[2] & 0x01) << 2) | ((bytes[3] >> 6) & 0x03);
        let original_copy = (bytes[3] >> 5) & 0x01 == 1;
        let home = (bytes[3] >> 4) & 0x01 == 1;
        let copyright_identification_bit = (bytes[3] >> 3) & 0x01 == 1;
        let copyright_identification_start = (bytes[3] >> 2) & 0x01 == 1;
        let frame_length = (((bytes[3] & 0x03) as u16) << 11)
            | ((bytes[4] as u16) << 3)
            | ((bytes[5] as u16) >> 5);
        let buffer_fullness = (((bytes[5] & 0x1f) as u16) << 6) | ((bytes[6] as u16) >> 2);
        let num_raw_data_blocks = bytes[6] & 0x03;

        let Some(sample_rate) = SAMPLE_RATES.get(sampling_frequency_index as usize).copied() else {
            return Err(AacError::UnknownSampleRate {
                index: sampling_frequency_index,
            });
        };
        let Some(channels) = CHANNEL_CONFIGURATIONS
            .get(channel_configuration as usize)
            .copied()
        else {
            return Err(AacError::UnknownChannelConfiguration {
                configuration: channel_configuration,
            });
        };

        let header_length = if protection_absent {
            ADTS_HEADER_LENGTH
        } else {
            ADTS_HEADER_LENGTH_WITH_CRC
        };

        Ok(AdtsHeader {
            version: if id == 0 {
                AacVersion::Mpeg4
            } else {
                AacVersion::Mpeg2
            },
            layer,
            protection_absent,
            profile: AacProfile::from_index(profile_index),
            sampling_frequency_index,
            sample_rate,
            private_bit,
            channel_configuration,
            channels,
            original_copy,
            home,
            copyright_identification_bit,
            copyright_identification_start,
            frame_length,
            buffer_fullness,
            num_raw_data_blocks,
            header_length,
        })
    }

    /// Returns the number of samples one frame carries.
    pub fn samples_per_frame(&self) -> u32 {
        self.profile.samples_per_frame()
    }

    /// Returns the number of samples the whole frame carries.
    ///
    /// WHY: a frame may hold more than one raw data block, and every
    ///      block codes its own samples.
    pub fn samples_per_frame_total(&self) -> u32 {
        self.samples_per_frame() * (self.num_raw_data_blocks as u32 + 1)
    }

    /// Returns the number of payload bytes, that is the frame without the
    /// header.
    pub fn payload_length(&self) -> usize {
        (self.frame_length as usize).saturating_sub(self.header_length)
    }

    /// Returns the length of the frame in bytes.
    pub fn total_length(&self) -> usize {
        self.frame_length as usize
    }
}

/// Splits an ADTS stream into its frames.
///
/// # Errors
///
/// Returns an error when a frame header is malformed or when a frame
/// announces more bytes than the buffer holds.
///
/// # Example
///
/// ```
/// use phairplay_aac::split_adts;
///
/// let stream = [0xff, 0xf1, 0x50, 0x80, 0x01, 0x1f, 0xfc, 0x00];
/// let frames = split_adts(&stream).unwrap_or_else(|err| panic!("{err}"));
/// assert_eq!(frames.len(), 1);
/// ```
pub fn split_adts(data: &[u8]) -> Result<Vec<AdtsHeader>, AacError> {
    let mut frames = Vec::new();
    let mut offset = 0_usize;
    while offset < data.len() {
        let header = AdtsHeader::parse(&data[offset..])?;
        let length = header.total_length();
        if length < header.header_length {
            return Err(AacError::Malformed(
                "the frame is shorter than its header".into(),
            ));
        }
        let end = offset.checked_add(length).ok_or(AacError::LengthMismatch {
            announced: length,
            available: data.len().saturating_sub(offset),
        })?;
        if end > data.len() {
            return Err(AacError::LengthMismatch {
                announced: length,
                available: data.len() - offset,
            });
        }
        frames.push(header);
        offset = end;
    }
    Ok(frames)
}

/// Returns the number of samples that a list of frames carries.
///
/// # Example
///
/// ```
/// use phairplay_aac::{split_adts, total_samples};
///
/// let stream = [0xff, 0xf1, 0x50, 0x80, 0x01, 0x1f, 0xfc, 0x00];
/// let frames = split_adts(&stream).unwrap_or_else(|err| panic!("{err}"));
/// assert_eq!(total_samples(&frames), 1024);
/// ```
pub fn total_samples(frames: &[AdtsHeader]) -> u32 {
    frames.iter().map(AdtsHeader::samples_per_frame_total).sum()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    /// A low complexity frame of 44100 Hz with two channels.
    const SAMPLE: [u8; 7] = [0xff, 0xf1, 0x50, 0x80, 0x01, 0x1f, 0xfc];

    #[test]
    fn parses_a_low_complexity_frame() {
        let header = AdtsHeader::parse(&SAMPLE).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(header.version, AacVersion::Mpeg4);
        assert_eq!(header.layer, 0);
        assert!(header.protection_absent);
        assert_eq!(header.profile, AacProfile::LowComplexity);
        assert_eq!(header.sampling_frequency_index, 4);
        assert_eq!(header.sample_rate, 44_100);
        assert_eq!(header.channel_configuration, 2);
        assert_eq!(header.channels, 2);
        assert_eq!(header.frame_length, 8);
        assert_eq!(header.num_raw_data_blocks, 0);
        assert_eq!(header.header_length, 7);
    }

    #[test]
    fn rejects_a_missing_sync_word() {
        let mut bytes = SAMPLE;
        bytes[0] = 0x00;
        assert!(matches!(
            AdtsHeader::parse(&bytes),
            Err(AacError::BadSyncWord { .. })
        ));
    }

    #[test]
    fn rejects_a_truncated_header() {
        assert!(matches!(
            AdtsHeader::parse(&SAMPLE[..4]),
            Err(AacError::Truncated { .. })
        ));
    }

    #[test]
    fn rejects_a_reserved_sample_rate() {
        let mut bytes = SAMPLE;
        bytes[2] = 0x78;
        assert!(matches!(
            AdtsHeader::parse(&bytes),
            Err(AacError::UnknownSampleRate { .. })
        ));
    }

    #[test]
    fn reports_the_length_of_the_frame() {
        let header = AdtsHeader::parse(&SAMPLE).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(header.total_length(), 8);
        assert_eq!(header.payload_length(), 1);
    }

    #[test]
    fn counts_the_samples_of_a_frame() {
        let header = AdtsHeader::parse(&SAMPLE).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(header.samples_per_frame(), 1024);
        assert_eq!(header.samples_per_frame_total(), 1024);
    }

    #[test]
    fn splits_a_stream_into_frames() {
        // Two frames of eight bytes each: a header and one payload byte.
        let mut two = SAMPLE.to_vec();
        two.push(0x00);
        two.extend_from_slice(&SAMPLE);
        two.push(0x00);
        let frames = split_adts(&two).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(frames.len(), 2);
    }

    #[test]
    fn rejects_a_frame_that_does_not_fit() {
        let mut bytes = SAMPLE.to_vec();
        // Announce a frame of 4000 bytes.
        bytes[3] = 0x80 | 0x01;
        bytes[4] = 0xf4;
        bytes[5] = 0x1f;
        bytes[6] = 0xfc;
        assert!(matches!(
            split_adts(&bytes),
            Err(AacError::LengthMismatch { .. })
        ));
    }

    #[test]
    fn counts_the_samples_of_a_stream() {
        let mut two = SAMPLE.to_vec();
        two.push(0x00);
        two.extend_from_slice(&SAMPLE);
        two.push(0x00);
        let frames = split_adts(&two).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(total_samples(&frames), 2048);
    }
}
