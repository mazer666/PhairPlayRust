// WHAT: the framing of an AAC low delay (ELD) frame.
// WHY: an AirPlay sender that streams AAC-ELD does not use the ADTS
//      framing. Every ELD frame starts with a 42 bit header whose sync
//      word names the number of channels. The receiver needs the header
//      to know how many channels and how many samples a frame carries.
// EXAMPLE: the sync word `0x9FFD` names a frame with one channel.

use crate::error::AacError;

/// The sync word of an ELD frame with one channel.
pub const ELD_SYNCWORD_MONO: u16 = 0x9ffd;

/// The sync word of an ELD frame with two channels.
pub const ELD_SYNCWORD_STEREO: u16 = 0x9ffe;

/// The sync word of an ELD frame with three to eight channels.
pub const ELD_SYNCWORD_MULTI: u16 = 0x9fff;

/// The number of samples one ELD frame carries.
pub const ELD_SAMPLES_PER_FRAME: u32 = 960;

/// The number of bits the ELD header occupies.
pub const ELD_HEADER_BITS: u32 = 42;

/// The number of bytes the ELD header occupies, rounded up.
pub const ELD_HEADER_BYTES: usize = 6;

/// A parsed AAC-ELD frame header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EldFrameHeader {
    /// The sync word of the frame.
    pub syncword: u16,
    /// The number of channels the frame carries.
    pub channels: u8,
    /// Whether an extension follows the header.
    pub extension_flag: bool,
    /// Whether the frame carries section data.
    pub aac_section_data_present_flag: bool,
    /// Whether the frame was downsampled.
    pub down_sample_mode: bool,
    /// Whether the downsample delay is present.
    pub down_sampledelay_present: bool,
    /// The extension bits of the header.
    pub extension_bits: u16,
    /// The header byte of the frame.
    pub header_byte: u16,
}

impl EldFrameHeader {
    /// Parses an AAC-ELD frame header.
    ///
    /// # Errors
    ///
    /// Returns an error when the buffer is too short or when the sync
    /// word is not one of the three that the format defines.
    pub fn parse(bytes: &[u8]) -> Result<EldFrameHeader, AacError> {
        if bytes.len() < ELD_HEADER_BYTES {
            return Err(AacError::Truncated {
                needed: ELD_HEADER_BYTES,
                available: bytes.len(),
            });
        }
        let syncword = u16::from_be_bytes([bytes[0], bytes[1]]);
        let channels = match syncword {
            ELD_SYNCWORD_MONO => 1,
            ELD_SYNCWORD_STEREO => 2,
            ELD_SYNCWORD_MULTI => 8,
            _ => return Err(AacError::BadSyncWord { syncword }),
        };

        let mut reader = BitReader::new(bytes);
        // The sync word has already been read; the flags follow it.
        reader.skip(16);
        let extension_flag = reader.read_bit()?;
        let aac_section_data_present_flag = reader.read_bit()?;
        let down_sample_mode = reader.read_bit()?;
        let down_sampledelay_present = reader.read_bit()?;
        let extension_bits = reader.read_bits(11)?;
        let header_byte = reader.read_bits(11)?;

        Ok(EldFrameHeader {
            syncword,
            channels,
            extension_flag,
            aac_section_data_present_flag,
            down_sample_mode,
            down_sampledelay_present,
            extension_bits,
            header_byte,
        })
    }

    /// Returns the number of samples the frame carries.
    pub fn samples_per_frame(&self) -> u32 {
        ELD_SAMPLES_PER_FRAME
    }

    /// Returns the length of the header in bits.
    pub fn header_bits(&self) -> u32 {
        ELD_HEADER_BITS
    }
}

/// A reader over the bits of an ELD header.
struct BitReader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> BitReader<'a> {
    /// Creates a reader over `bytes`.
    fn new(bytes: &'a [u8]) -> BitReader<'a> {
        BitReader { bytes, position: 0 }
    }

    /// Skips `count` bits.
    fn skip(&mut self, count: usize) {
        self.position += count;
    }

    /// Reads a single bit.
    fn read_bit(&mut self) -> Result<bool, AacError> {
        if self.position >= self.bytes.len() * 8 {
            return Err(AacError::Truncated {
                needed: 1,
                available: 0,
            });
        }
        let byte = self.bytes[self.position / 8];
        let shift = 7 - (self.position % 8);
        self.position += 1;
        Ok((byte >> shift) & 1 == 1)
    }

    /// Reads `count` bits, most significant bit first.
    fn read_bits(&mut self, count: u32) -> Result<u16, AacError> {
        let mut value = 0_u16;
        for _ in 0..count {
            value = (value << 1) | u16::from(self.read_bit()?);
        }
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    /// Builds the six header bytes of an ELD frame.
    fn header_bytes(syncword: u16, extension_flag: bool) -> [u8; 6] {
        let mut bytes = [0_u8; 6];
        bytes[0] = (syncword >> 8) as u8;
        bytes[1] = syncword as u8;
        // The four flag bits and the eleven extension bits.
        let mut value = 0_u16;
        if extension_flag {
            value |= 0x8000;
        }
        value |= 0x4000; // aac_section_data_present_flag
        bytes[2] = (value >> 8) as u8;
        bytes[3] = value as u8;
        bytes[4] = 0x00;
        bytes[5] = 0x00;
        bytes
    }

    #[test]
    fn parses_a_mono_header() {
        let bytes = header_bytes(ELD_SYNCWORD_MONO, false);
        let header = EldFrameHeader::parse(&bytes).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(header.syncword, ELD_SYNCWORD_MONO);
        assert_eq!(header.channels, 1);
        assert!(!header.extension_flag);
        assert!(header.aac_section_data_present_flag);
        assert_eq!(header.samples_per_frame(), 960);
        assert_eq!(header.header_bits(), 42);
    }

    #[test]
    fn parses_a_stereo_header() {
        let bytes = header_bytes(ELD_SYNCWORD_STEREO, true);
        let header = EldFrameHeader::parse(&bytes).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(header.channels, 2);
        assert!(header.extension_flag);
    }

    #[test]
    fn parses_a_multichannel_header() {
        let bytes = header_bytes(ELD_SYNCWORD_MULTI, false);
        let header = EldFrameHeader::parse(&bytes).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(header.channels, 8);
    }

    #[test]
    fn rejects_an_unknown_sync_word() {
        let bytes = header_bytes(0x1234, false);
        assert!(matches!(
            EldFrameHeader::parse(&bytes),
            Err(AacError::BadSyncWord { .. })
        ));
    }

    #[test]
    fn rejects_a_truncated_header() {
        assert!(matches!(
            EldFrameHeader::parse(&[0x9f, 0xfd]),
            Err(AacError::Truncated { .. })
        ));
    }
}
