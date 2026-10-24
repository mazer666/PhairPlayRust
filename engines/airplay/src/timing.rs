//! The Apple NTP timing service.
//!
//! WHY: an AirPlay sender probes the receiver with a 32-byte UDP packet to
//! measure the round-trip delay and to align its RTP timestamps with the
//! receiver's wall clock. Without an answer the sender either aborts the
//! session or drifts audibly out of sync. The service is deliberately tiny
//! and stateless: it answers a probe with the sender's own transmit time and
//! the receiver's receive and transmit times, and it keeps the estimated
//! clock offset for the audio path.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_airplay::timing::TimingResponder;
//!
//! let mut request = [0u8; 32];
//! request[0] = 0x80;
//! request[1] = 0xD2; // timing request
//! let response = TimingResponder::build_response(&request, 0x1234_5678_9ABC_DEF0)
//!     .expect("valid probe");
//! assert_eq!(response[1], 0xD3);
//! assert_eq!(response[2], request[2]);
//! assert_eq!(response[3], request[3]);
//! ```

use std::sync::atomic::{AtomicI64, Ordering};

use crate::error::AirPlayError;

/// The UDP port the sender probes for clock synchronisation.
pub const TIMING_PORT: u16 = 6002;

/// The size of a timing packet.
pub const PACKET_SIZE: usize = 32;

/// The type byte of a timing request.
pub const REQUEST_TYPE: u8 = 0xD2;

/// The type byte of a timing response.
pub const RESPONSE_TYPE: u8 = 0xD3;

/// Seconds between the NTP epoch (1900-01-01) and the Unix epoch (1970-01-01).
pub const NTP_EPOCH_OFFSET: u64 = 2_208_988_800;

/// The timing service of the AirPlay engine.
#[derive(Debug, Default)]
pub struct TimingResponder {
    offset_us: AtomicI64,
}

impl TimingResponder {
    /// Creates a timing responder without a clock offset estimate.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The estimated offset between the local clock and the sender's clock.
    ///
    /// A positive value means the local clock is ahead of the sender's.
    #[must_use]
    pub fn rtp_clock_offset_us(&self) -> i64 {
        self.offset_us.load(Ordering::Relaxed)
    }

    /// Answers a timing probe.
    ///
    /// Returns `Ok(None)` when the datagram is not a timing request — a short
    /// packet or an unknown type byte is ignored, not answered.
    ///
    /// # Errors
    /// Returns [`AirPlayError::Parse`] when the probe cannot be answered.
    pub fn handle_probe(&self, request: &[u8], receive_ntp: u64) -> Result<Option<[u8; 32]>, AirPlayError> {
        if request.len() < PACKET_SIZE {
            return Ok(None);
        }
        if request[1] != REQUEST_TYPE {
            return Ok(None);
        }
        let response = Self::build_response(request, receive_ntp)?;

        let reference_seconds = read_u32(&request[24..28]) as u64;
        let reference_fraction = read_u32(&request[28..32]) as u64;
        let sender_send_us = Self::ntp_to_us(reference_seconds, reference_fraction);
        let our_receive_us = Self::ntp_to_us(receive_ntp >> 32, receive_ntp & 0xFFFF_FFFF);
        self.offset_us
            .store(our_receive_us - sender_send_us, Ordering::Relaxed);
        Ok(Some(response))
    }

    /// Builds the 32-byte answer to a timing probe.
    ///
    /// # Errors
    /// Returns [`AirPlayError::Parse`] when the probe is too short.
    pub fn build_response(request: &[u8], receive_ntp: u64) -> Result<[u8; 32], AirPlayError> {
        if request.len() < PACKET_SIZE {
            return Err(AirPlayError::parse(format!(
                "timing probe of {} bytes is shorter than {PACKET_SIZE}",
                request.len()
            )));
        }
        let mut response = [0u8; PACKET_SIZE];
        response[0] = 0x80;
        response[1] = RESPONSE_TYPE;
        response[2] = request[2];
        response[3] = request[3];

        // The reference timestamp is the sender's own transmit time.
        write_u32(&mut response[8..12], read_u32(&request[24..28]));
        write_u32(&mut response[12..16], read_u32(&request[28..32]));

        // The received timestamp is the moment the probe arrived.
        write_u32(&mut response[16..20], (receive_ntp >> 32) as u32);
        write_u32(&mut response[20..24], (receive_ntp & 0xFFFF_FFFF) as u32);

        // The transmit timestamp is the moment the answer leaves.
        let transmit = Self::current_ntp_timestamp();
        write_u32(&mut response[24..28], (transmit >> 32) as u32);
        write_u32(&mut response[28..32], (transmit & 0xFFFF_FFFF) as u32);
        Ok(response)
    }

    /// The current wall clock as a packed 64-bit NTP timestamp.
    #[must_use]
    pub fn current_ntp_timestamp() -> u64 {
        let millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_millis() as u64)
            .unwrap_or(0);
        let seconds = millis / 1000 + NTP_EPOCH_OFFSET;
        let fraction = (millis % 1000) * 0x1_0000_0000 / 1000;
        (seconds << 32) | fraction
    }

    /// Converts a packed NTP timestamp into microseconds since the Unix epoch.
    #[must_use]
    pub fn ntp_to_us(seconds: u64, fraction: u64) -> i64 {
        let unix_seconds = seconds as i64 - NTP_EPOCH_OFFSET as i64;
        let sub_us = (fraction as i128 * 1_000_000 / 0x1_0000_0000) as i64;
        unix_seconds * 1_000_000 + sub_us
    }
}

/// Reads a big-endian `u32` from a slice.
fn read_u32(bytes: &[u8]) -> u32 {
    u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

/// Writes a big-endian `u32` into a slice.
fn write_u32(bytes: &mut [u8], value: u32) {
    bytes[0] = (value >> 24) as u8;
    bytes[1] = (value >> 16) as u8;
    bytes[2] = (value >> 8) as u8;
    bytes[3] = value as u8;
}

#[cfg(test)]
mod tests;
