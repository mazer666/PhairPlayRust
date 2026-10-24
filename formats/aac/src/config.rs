// WHAT: the configuration of an AAC stream, with the sample counting
//      and the bitrate computation that go with it.
// WHY: a receiver has to know how many samples a packet carries and how
//      many bits per second the stream uses, because both are needed to
//      schedule the audio sink and to fill the statistics. The number of
//      samples per frame is not a constant: the low delay profile codes
//      960 samples, every other profile codes 1024.
// EXAMPLE: a 44100 Hz stereo stream with 1024 samples per frame holds
//          43.07 frames per second.

use crate::adts::AacProfile;
use crate::eld::ELD_SAMPLES_PER_FRAME;

/// The configuration of an AAC stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AacConfig {
    /// The object type of the stream.
    pub profile: AacProfile,
    /// The sample rate in hertz.
    pub sample_rate: u32,
    /// The number of channels.
    pub channels: u8,
    /// The number of AAC frames one packet carries.
    pub frames_per_packet: u32,
}

impl AacConfig {
    /// Creates a configuration for a stream that carries one frame per
    /// packet.
    ///
    /// # Example
    ///
    /// ```
    /// use phairplay_aac::{AacConfig, AacProfile};
    ///
    /// let config = AacConfig::new(AacProfile::LowComplexity, 44_100, 2);
    /// assert_eq!(config.samples_per_packet(), 1024);
    /// ```
    pub fn new(profile: AacProfile, sample_rate: u32, channels: u8) -> AacConfig {
        AacConfig {
            profile,
            sample_rate,
            channels,
            frames_per_packet: 1,
        }
    }

    /// Returns the number of samples one frame carries.
    pub fn samples_per_frame(&self) -> u32 {
        match self.profile {
            AacProfile::LowDelay => ELD_SAMPLES_PER_FRAME,
            _ => self.profile.samples_per_frame(),
        }
    }

    /// Returns the number of samples one packet carries.
    pub fn samples_per_packet(&self) -> u32 {
        self.samples_per_frame() * self.frames_per_packet.max(1)
    }

    /// Returns the length of one packet in seconds.
    pub fn packet_duration(&self) -> f64 {
        if self.sample_rate == 0 {
            return 0.0;
        }
        self.samples_per_packet() as f64 / self.sample_rate as f64
    }

    /// Returns the number of packets per second.
    pub fn packets_per_second(&self) -> f64 {
        if self.packet_duration() == 0.0 {
            return 0.0;
        }
        1.0 / self.packet_duration()
    }

    /// Returns the bit rate of the stream, in bits per second.
    ///
    /// WHY: the sender states the average bit rate in the ALAC and the
    ///      AAC configuration. This method computes the same number from
    ///      the format of the stream, so that the two can be compared.
    pub fn bitrate_bps(&self, bits_per_sample: u32) -> u64 {
        self.sample_rate as u64 * self.channels as u64 * bits_per_sample as u64
    }

    /// Returns the bit rate that a measured packet size implies.
    ///
    /// # Errors
    ///
    /// Returns `None` when the sample rate is zero, because then the
    /// duration of a packet is not defined.
    ///
    /// # Example
    ///
    /// ```
    /// use phairplay_aac::{AacConfig, AacProfile};
    ///
    /// let config = AacConfig::new(AacProfile::LowComplexity, 44_100, 2);
    /// // 44100 samples per second, 1024 samples per packet, 400 bytes.
    /// let bitrate = config.bitrate_from_bytes(400).unwrap_or(0);
    /// assert_eq!(bitrate, 137_812);
    /// ```
    pub fn bitrate_from_bytes(&self, bytes_per_packet: u64) -> Option<u64> {
        let samples = self.samples_per_packet();
        if self.sample_rate == 0 || samples == 0 {
            return None;
        }
        Some(bytes_per_packet * 8 * self.sample_rate as u64 / samples as u64)
    }

    /// Returns the number of bytes one packet should have for a given
    /// bit rate.
    ///
    /// # Errors
    ///
    /// Returns `None` when the sample rate is zero.
    pub fn bytes_per_packet_for_bitrate(&self, bitrate_bps: u64) -> Option<u64> {
        let samples = self.samples_per_packet();
        if self.sample_rate == 0 || samples == 0 {
            return None;
        }
        Some(bitrate_bps * samples as u64 / (8 * self.sample_rate as u64))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    #[test]
    fn counts_the_samples_of_a_low_complexity_packet() {
        let config = AacConfig::new(AacProfile::LowComplexity, 44_100, 2);
        assert_eq!(config.samples_per_frame(), 1024);
        assert_eq!(config.samples_per_packet(), 1024);
    }

    #[test]
    fn counts_the_samples_of_a_low_delay_packet() {
        let config = AacConfig::new(AacProfile::LowDelay, 44_100, 2);
        assert_eq!(config.samples_per_frame(), 960);
        assert_eq!(config.samples_per_packet(), 960);
    }

    #[test]
    fn counts_the_samples_of_a_multi_frame_packet() {
        let mut config = AacConfig::new(AacProfile::LowComplexity, 44_100, 2);
        config.frames_per_packet = 4;
        assert_eq!(config.samples_per_packet(), 4096);
    }

    #[test]
    fn computes_the_duration_of_a_packet() {
        let config = AacConfig::new(AacProfile::LowComplexity, 44_100, 2);
        let duration = config.packet_duration();
        assert!((duration - 1024.0 / 44_100.0).abs() < 1e-12, "{duration}");
        let rate = config.packets_per_second();
        assert!((rate - 44_100.0 / 1024.0).abs() < 1e-9, "{rate}");
    }

    #[test]
    fn computes_the_bitrate_of_the_format() {
        let config = AacConfig::new(AacProfile::LowComplexity, 44_100, 2);
        assert_eq!(config.bitrate_bps(16), 1_411_200);
    }

    #[test]
    fn computes_the_bitrate_from_a_measured_packet() {
        let config = AacConfig::new(AacProfile::LowComplexity, 44_100, 2);
        let bitrate = config.bitrate_from_bytes(400).unwrap_or(0);
        assert_eq!(bitrate, 137_812);
    }

    #[test]
    fn computes_the_packet_size_from_a_bitrate() {
        let config = AacConfig::new(AacProfile::LowComplexity, 44_100, 2);
        let bytes = config.bytes_per_packet_for_bitrate(141_120).unwrap_or(0);
        assert_eq!(bytes, 409);
    }

    #[test]
    fn refuses_a_stream_without_a_sample_rate() {
        let config = AacConfig::new(AacProfile::LowComplexity, 0, 2);
        assert_eq!(config.packet_duration(), 0.0);
        assert_eq!(config.packets_per_second(), 0.0);
        assert_eq!(config.bitrate_from_bytes(100), None);
        assert_eq!(config.bytes_per_packet_for_bitrate(1000), None);
    }
}
