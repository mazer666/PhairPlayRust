// WHAT: the configuration of an ALAC stream and the parser of its
//      magic cookie.
// WHY: an AirPlay sender describes the stream in a "magic cookie"
//      before the frames start. The cookie holds the frame length, the
//      bit depth, the channel count and the parameters of the adaptive
//      Golomb coder. The cookie arrives from the network, so every
//      field is checked before the decoder allocates a buffer with it.
// EXAMPLE: the cookie of a 44100 Hz stereo stream with 16 bit samples
//      names a frame length of 4096 samples.
//
// ATTRIBUTION: the layout of the cookie follows the reference decoder by
// Apple Inc. (`ALACAudioTypes.h` and `ALACDecoder.cpp`, Copyright (c)
// 2011 Apple Inc., licensed under the Apache License, Version 2.0). The
// Rust code is an independent implementation; no C code was copied.

use crate::error::AlacError;

/// The size of the `ALACSpecificConfig` structure.
pub const ALAC_SPECIFIC_CONFIG_SIZE: usize = 24;

/// The size of the atom that wraps the configuration.
pub const ALAC_ATOM_SIZE: usize = 12;

/// The version of the format that this decoder implements.
pub const ALAC_VERSION: u8 = 0;

/// The largest number of channels that the format defines.
pub const ALAC_MAX_CHANNELS: u8 = 8;

/// The largest bit depth that the format defines.
pub const ALAC_MAX_BIT_DEPTH: u8 = 32;

/// The configuration of an ALAC stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AlacConfig {
    /// The number of samples in one frame.
    pub frame_length: u32,
    /// The version of the format that the sender uses.
    pub compatible_version: u8,
    /// The number of bits per sample.
    pub bit_depth: u8,
    /// The prediction offset of the Golomb coder.
    pub pb: u8,
    /// The mean offset of the Golomb coder.
    pub mb: u8,
    /// The prediction width of the Golomb coder.
    pub kb: u8,
    /// The number of channels.
    pub num_channels: u8,
    /// The largest run of zeros that the coder may emit.
    pub max_run: u16,
    /// The largest number of bytes a frame may use.
    pub max_frame_bytes: u32,
    /// The average bit rate of the stream.
    pub avg_bit_rate: u32,
    /// The sample rate in hertz.
    pub sample_rate: u32,
}

impl AlacConfig {
    /// Parses the magic cookie of an ALAC stream.
    ///
    /// WHY: the cookie may be wrapped in a `frma` and an `alac` atom, in
    ///      which case the configuration starts twelve bytes further in.
    ///      The reference implementation reads those bytes without
    ///      checking their length; the port refuses a short cookie
    ///      instead of reading past the buffer.
    ///
    /// # Errors
    ///
    /// Returns an error when the cookie is too short or names a
    /// configuration that the decoder cannot use.
    ///
    /// # Example
    ///
    /// ```
    /// use phairplay_alac::AlacConfig;
    ///
    /// let mut cookie = vec![0_u8; 24];
    /// cookie[0..4].copy_from_slice(&4096u32.to_be_bytes());
    /// cookie[4] = 0;      // version
    /// cookie[5] = 16;     // bit depth
    /// cookie[9] = 2;      // channels
    /// cookie[20..24].copy_from_slice(&44_100u32.to_be_bytes());
    /// let config = AlacConfig::from_magic_cookie(&cookie).unwrap_or_else(|err| panic!("{err}"));
    /// assert_eq!(config.frame_length, 4096);
    /// assert_eq!(config.num_channels, 2);
    /// ```
    pub fn from_magic_cookie(cookie: &[u8]) -> Result<AlacConfig, AlacError> {
        let mut offset = 0_usize;
        if cookie.len() >= 8 && &cookie[4..8] == b"frma" {
            offset += ALAC_ATOM_SIZE;
        }
        if cookie.len() >= offset + 8 && &cookie[offset + 4..offset + 8] == b"alac" {
            offset += ALAC_ATOM_SIZE;
        }
        let body = cookie
            .get(offset..)
            .ok_or_else(|| AlacError::InvalidConfig("the cookie is too short".into()))?;
        if body.len() < ALAC_SPECIFIC_CONFIG_SIZE {
            return Err(AlacError::InvalidConfig(format!(
                "the cookie holds {} bytes, but {ALAC_SPECIFIC_CONFIG_SIZE} are needed",
                body.len()
            )));
        }

        let config = AlacConfig {
            frame_length: read_u32(body, 0),
            compatible_version: body[4],
            bit_depth: body[5],
            pb: body[6],
            mb: body[7],
            kb: body[8],
            num_channels: body[9],
            max_run: u16::from_be_bytes([body[10], body[11]]),
            max_frame_bytes: read_u32(body, 12),
            avg_bit_rate: read_u32(body, 16),
            sample_rate: read_u32(body, 20),
        };
        config.check()?;
        Ok(config)
    }

    /// Checks that the configuration is usable.
    ///
    /// # Errors
    ///
    /// Returns an error when a field holds a value that the decoder
    /// cannot work with.
    pub fn check(&self) -> Result<(), AlacError> {
        if self.compatible_version > ALAC_VERSION {
            return Err(AlacError::InvalidConfig(format!(
                "the cookie names the version {}, but only {ALAC_VERSION} is known",
                self.compatible_version
            )));
        }
        if self.bit_depth == 0 || self.bit_depth > ALAC_MAX_BIT_DEPTH {
            return Err(AlacError::InvalidConfig(format!(
                "the bit depth {} is not usable",
                self.bit_depth
            )));
        }
        if self.num_channels == 0 || self.num_channels > ALAC_MAX_CHANNELS {
            return Err(AlacError::InvalidConfig(format!(
                "the channel count {} is not usable",
                self.num_channels
            )));
        }
        if self.frame_length == 0 {
            return Err(AlacError::InvalidConfig(
                "the frame length must not be zero".into(),
            ));
        }
        Ok(())
    }

    /// Returns the number of bytes one frame of samples uses.
    pub fn frame_bytes(&self) -> usize {
        self.frame_length as usize * self.num_channels as usize * (self.bit_depth as usize / 8)
    }
}

/// Reads a big endian `u32` at an offset.
fn read_u32(data: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    /// Builds a magic cookie of 24 bytes.
    fn cookie() -> Vec<u8> {
        let mut bytes = vec![0_u8; ALAC_SPECIFIC_CONFIG_SIZE];
        bytes[0..4].copy_from_slice(&4096u32.to_be_bytes());
        bytes[4] = 0;
        bytes[5] = 16;
        bytes[6] = 40;
        bytes[7] = 10;
        bytes[8] = 14;
        bytes[9] = 2;
        bytes[10..12].copy_from_slice(&255u16.to_be_bytes());
        bytes[12..16].copy_from_slice(&8192u32.to_be_bytes());
        bytes[16..20].copy_from_slice(&705_600u32.to_be_bytes());
        bytes[20..24].copy_from_slice(&44_100u32.to_be_bytes());
        bytes
    }

    #[test]
    fn parses_a_magic_cookie() {
        let config = AlacConfig::from_magic_cookie(&cookie()).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(config.frame_length, 4096);
        assert_eq!(config.bit_depth, 16);
        assert_eq!(config.num_channels, 2);
        assert_eq!(config.max_run, 255);
        assert_eq!(config.sample_rate, 44_100);
        assert_eq!(config.avg_bit_rate, 705_600);
    }

    #[test]
    fn skips_the_frma_and_alac_atoms() {
        let mut wrapped = vec![0_u8; ALAC_ATOM_SIZE];
        wrapped[4..8].copy_from_slice(b"frma");
        wrapped.extend_from_slice(&cookie());
        let config = AlacConfig::from_magic_cookie(&wrapped).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(config.frame_length, 4096);
    }

    #[test]
    fn skips_the_alac_atom() {
        let mut wrapped = vec![0_u8; ALAC_ATOM_SIZE];
        wrapped[4..8].copy_from_slice(b"alac");
        wrapped.extend_from_slice(&cookie());
        let config = AlacConfig::from_magic_cookie(&wrapped).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(config.frame_length, 4096);
    }

    #[test]
    fn refuses_a_short_cookie() {
        assert!(matches!(
            AlacConfig::from_magic_cookie(&[0_u8; 10]),
            Err(AlacError::InvalidConfig(_))
        ));
    }

    #[test]
    fn refuses_an_unknown_version() {
        let mut bytes = cookie();
        bytes[4] = 1;
        assert!(matches!(
            AlacConfig::from_magic_cookie(&bytes),
            Err(AlacError::InvalidConfig(_))
        ));
    }

    #[test]
    fn refuses_an_unusable_bit_depth() {
        let mut bytes = cookie();
        bytes[5] = 0;
        assert!(matches!(
            AlacConfig::from_magic_cookie(&bytes),
            Err(AlacError::InvalidConfig(_))
        ));
    }

    #[test]
    fn refuses_an_unusable_channel_count() {
        let mut bytes = cookie();
        bytes[9] = 0;
        assert!(matches!(
            AlacConfig::from_magic_cookie(&bytes),
            Err(AlacError::InvalidConfig(_))
        ));
    }

    #[test]
    fn refuses_a_zero_frame_length() {
        let mut bytes = cookie();
        bytes[0..4].copy_from_slice(&0u32.to_be_bytes());
        assert!(matches!(
            AlacConfig::from_magic_cookie(&bytes),
            Err(AlacError::InvalidConfig(_))
        ));
    }

    #[test]
    fn computes_the_size_of_a_frame() {
        let config = AlacConfig::from_magic_cookie(&cookie()).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(config.frame_bytes(), 4096 * 2 * 2);
    }
}
