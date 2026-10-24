//! The interleaved RTP transport of AirPlay mirroring.
//!
//! WHY: after the `RECORD` acknowledgement, the sender switches the same TCP
//! connection from RTSP text to binary interleaved framing (RFC 2326 §10.12):
//!
//! ```text
//! ┌──────┬────────────┬───────────────┬──────────────────────────────┐
//! │ '$'  │ channel(1) │ length(2, BE)  │ RTP/RTCP payload (N bytes)  │
//! │  1B  │     1B     │      2B        │            N bytes            │
//! └──────┴────────────┴───────────────┴──────────────────────────────┘
//! ```
//!
//! The video stream arrives as H.264 in RTP packets (RFC 6184). A large NAL
//! unit is fragmented into *FU-A* packets that have to be reassembled before
//! the decoder sees them. Both the framing and the reassembly are untrusted
//! input: the length field is bounded, the reassembly buffer is bounded, and
//! an incomplete fragment is dropped rather than delivered half-decoded.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_airplay::rtp::FuAReassembler;
//!
//! let mut reassembler = FuAReassembler::new();
//! // FU-A start: indicator 0x7C, header 0x85 (start, type 5)
//! let start = [0x7Cu8, 0x85, 0xAA];
//! // FU-A end: indicator 0x7C, header 0x45 (end, type 5)
//! let end = [0x7Cu8, 0x45, 0xBB];
//! assert!(reassembler.push(&start, 0).is_none());
//! let nal = reassembler.push(&end, 0).expect("the NAL unit is complete");
//! assert_eq!(nal.data, vec![0x65, 0xAA, 0xBB]);
//! ```

use std::time::Duration;

use tokio::io::AsyncRead;

use crate::error::AirPlayError;
use crate::rtsp::reader::StreamBuffer;

/// The byte that marks the start of an interleaved frame.
pub const INTERLEAVED_MARKER: u8 = 0x24;

/// The channel of the video RTP stream.
pub const CHANNEL_VIDEO_RTP: u8 = 0;

/// The channel of the video RTCP stream.
pub const CHANNEL_VIDEO_RTCP: u8 = 1;

/// The size of the fixed RTP header.
pub const RTP_FIXED_HEADER_BYTES: usize = 12;

/// The NAL unit type of an FU-A packet.
pub const NAL_TYPE_FU_A: u8 = 28;

/// The clock rate of an H.264 RTP stream.
pub const RTP_VIDEO_CLOCK_HZ: u64 = 90_000;

/// The maximum size of one reassembled NAL unit.
pub const MAX_NAL_UNIT_BYTES: usize = 4 * 1024 * 1024;

/// The header of an RTP packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RtpHeader {
    /// The payload type of the packet.
    pub payload_type: u8,
    /// Whether the marker bit is set.
    pub marker: bool,
    /// The sequence number of the packet.
    pub sequence: u16,
    /// The timestamp of the packet, in the clock of the stream.
    pub timestamp: u32,
}

/// One interleaved frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterleavedFrame {
    /// The channel the frame arrived on.
    pub channel: u8,
    /// The payload of the frame.
    pub payload: Vec<u8>,
}

/// A reassembled NAL unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReassembledNal {
    /// The NAL unit, header included.
    pub data: Vec<u8>,
    /// The presentation timestamp in microseconds.
    pub pts_us: u64,
}

/// Splits an RTP packet into its header and its payload.
///
/// # Errors
/// Returns [`AirPlayError::Parse`] when the packet is too short or carries a
/// header extension that does not fit.
pub fn parse_rtp(frame: &[u8]) -> Result<(RtpHeader, &[u8]), AirPlayError> {
    if frame.len() < RTP_FIXED_HEADER_BYTES {
        return Err(AirPlayError::parse(format!(
            "RTP frame of {} bytes is shorter than the header",
            frame.len()
        )));
    }
    let csrc_count = usize::from(frame[0] & 0x0F);
    let has_extension = frame[0] & 0x10 != 0;
    let mut payload_offset = RTP_FIXED_HEADER_BYTES + csrc_count * 4;
    if has_extension {
        if frame.len() < payload_offset + 4 {
            return Err(AirPlayError::parse("RTP header extension is truncated"));
        }
        let extension_words = usize::from(u16::from_be_bytes([
            frame[payload_offset + 2],
            frame[payload_offset + 3],
        ]));
        payload_offset = payload_offset
            .checked_add(4 + extension_words * 4)
            .ok_or_else(|| AirPlayError::parse("RTP header extension overflows"))?;
    }
    if payload_offset > frame.len() {
        return Err(AirPlayError::parse("RTP payload is out of range"));
    }
    let header = RtpHeader {
        payload_type: frame[1] & 0x7F,
        marker: frame[1] & 0x80 != 0,
        sequence: u16::from_be_bytes([frame[2], frame[3]]),
        timestamp: u32::from_be_bytes([frame[4], frame[5], frame[6], frame[7]]),
    };
    Ok((header, &frame[payload_offset..]))
}

/// Converts an RTP timestamp into microseconds.
#[must_use]
pub fn timestamp_to_micros(timestamp: u32) -> u64 {
    u64::from(timestamp) * 1_000_000 / RTP_VIDEO_CLOCK_HZ
}

/// The reassembler of the FU-A fragments of one NAL unit.
#[derive(Debug, Default)]
pub struct FuAReassembler {
    buffer: Option<Vec<u8>>,
}

impl FuAReassembler {
    /// Creates an empty reassembler.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether a fragment is being accumulated.
    #[must_use]
    pub fn is_accumulating(&self) -> bool {
        self.buffer.is_some()
    }

    /// Drops an incomplete fragment.
    pub fn reset(&mut self) {
        self.buffer = None;
    }

    /// Feeds one RTP payload into the reassembler.
    ///
    /// Returns the reassembled NAL unit when the fragment completes it, and
    /// `None` otherwise. A payload that is not an FU-A packet is delivered
    /// unchanged — it is a complete NAL unit already.
    pub fn push(&mut self, payload: &[u8], pts_us: u64) -> Option<ReassembledNal> {
        let &first = payload.first()?;
        if first & 0x1F != NAL_TYPE_FU_A {
            // A complete NAL unit: an unfinished fragment is dropped.
            self.reset();
            return Some(ReassembledNal {
                data: payload.to_vec(),
                pts_us,
            });
        }
        if payload.len() < 2 {
            self.reset();
            return None;
        }
        let indicator = payload[0];
        let header = payload[1];
        let is_start = header & 0x80 != 0;
        let is_end = header & 0x40 != 0;

        if is_start {
            // A new fragment discards an incomplete predecessor.
            let mut buffer = Vec::with_capacity(payload.len());
            buffer.push((indicator & 0x60) | (header & 0x1F));
            buffer.extend_from_slice(&payload[2..]);
            self.buffer = Some(buffer);
        } else {
            let Some(buffer) = self.buffer.as_mut() else {
                // A fragment without a start bit cannot be reassembled.
                return None;
            };
            buffer.extend_from_slice(&payload[2..]);
            if buffer.len() > MAX_NAL_UNIT_BYTES {
                self.reset();
                return None;
            }
        }

        if !is_end {
            return None;
        }
        let data = self.buffer.take()?;
        Some(ReassembledNal { data, pts_us })
    }
}

/// Reads one interleaved frame.
///
/// Bytes that are not the interleaved marker are skipped — a sender may
/// interleave RTSP keep-alives with the media.
///
/// # Errors
/// Returns [`AirPlayError::LimitExceeded`] when the frame is larger than
/// `max_frame_bytes` and [`AirPlayError::Timeout`] when the peer stalls.
pub async fn read_interleaved_frame<R: AsyncRead + Unpin>(
    stream: &mut StreamBuffer<R>,
    max_frame_bytes: usize,
    timeout: Duration,
) -> Result<Option<InterleavedFrame>, AirPlayError> {
    loop {
        let marker = stream.read_byte(timeout).await?;
        let Some(marker) = marker else {
            return Ok(None);
        };
        if marker != INTERLEAVED_MARKER {
            continue;
        }
        let Some(channel) = stream.read_byte(timeout).await? else {
            return Ok(None);
        };
        let Some(high) = stream.read_byte(timeout).await? else {
            return Ok(None);
        };
        let Some(low) = stream.read_byte(timeout).await? else {
            return Ok(None);
        };
        let length = usize::from(u16::from_be_bytes([high, low]));
        if length == 0 || length > max_frame_bytes {
            return Err(AirPlayError::limit_exceeded(format!(
                "interleaved frame of {length} bytes exceeds the {max_frame_bytes} byte limit"
            )));
        }
        let payload = stream.read_exact(length, timeout).await?;
        return Ok(Some(InterleavedFrame { channel, payload }));
    }
}

#[cfg(test)]
mod tests;
