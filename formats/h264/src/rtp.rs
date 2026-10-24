// WHAT: the reassembly of fragmented H.264 NAL units from RTP.
// WHY: an AirPlay or Miracast sender sends a large picture as a series
//      of RTP packets that carry a *fragmentation unit* instead of a
//      complete NAL unit. The receiver has to glue the fragments back
//      together. A peer that never sends the start or the end fragment
//      would make the receiver buffer forever, so the reassembler holds
//      a bounded amount of data and refuses the rest.
// EXAMPLE: `FuAReassembler::push` returns the complete NAL unit once
//          the fragment with the end bit arrives.

use crate::error::H264Error;
use crate::nal::NalUnit;

/// The RTP payload type of a fragmentation unit.
pub const FU_A_PAYLOAD_TYPE: u8 = 28;

/// The RTP payload type of a single NAL unit packet.
pub const SINGLE_NAL_PAYLOAD_TYPE: u8 = 1;

/// The RTP payload type of a STAP-A aggregation packet.
pub const STAP_A_PAYLOAD_TYPE: u8 = 24;

/// The largest number of bytes a single reassembled NAL unit may have.
pub const MAX_REASSEMBLED_NAL_SIZE: usize = 4 * 1024 * 1024;

/// The state of an unfinished fragmentation unit.
#[derive(Debug, Clone)]
struct Fragment {
    header: u8,
    data: Vec<u8>,
}

/// The reassembler of fragmented NAL units.
///
/// WHY: the reassembler is a state machine. A start fragment opens a
///      unit, the middle fragments extend it and the end fragment closes
///      it. A fragment that arrives without a start is dropped, because
///      the receiver cannot know where the unit begins. A payload is
///      never held back: every payload that arrives is processed, and
///      the units it carries are handed to the caller right away.
#[derive(Debug, Clone, Default)]
pub struct FuAReassembler {
    fragment: Option<Fragment>,
}

impl FuAReassembler {
    /// Creates an empty reassembler.
    ///
    /// # Example
    ///
    /// ```
    /// use phairplay_h264::FuAReassembler;
    ///
    /// let reassembler = FuAReassembler::new();
    /// assert!(reassembler.is_idle());
    /// ```
    pub fn new() -> FuAReassembler {
        FuAReassembler { fragment: None }
    }

    /// Returns `true` when no fragment is being collected.
    pub fn is_idle(&self) -> bool {
        self.fragment.is_none()
    }

    /// Drops the fragment that is being collected, if any.
    pub fn reset(&mut self) {
        self.fragment = None;
    }

    /// Feeds one RTP payload into the reassembler.
    ///
    /// Returns every NAL unit that the payload carries. A fragmented
    /// unit is returned when its last fragment arrives, a payload with a
    /// single NAL unit is returned right away, and an aggregation
    /// packet returns all the units it holds.
    ///
    /// WHY: the return value is a list and not a single unit, because
    ///      one RTP packet can carry several NAL units. A reassembler
    ///      that kept the extra units in a queue would have to decide
    ///      which packet to drop, and every choice loses data.
    ///
    /// # Errors
    ///
    /// Returns an error when the payload is too short, when a fragment
    /// would make the NAL unit grow beyond the limit, or when a fragment
    /// arrives that does not fit the state machine.
    ///
    /// # Example
    ///
    /// ```
    /// use phairplay_h264::FuAReassembler;
    ///
    /// let mut reassembler = FuAReassembler::new();
    /// let units = reassembler.push(&[0x67, 0x42]).unwrap_or_default();
    /// assert_eq!(units.len(), 1);
    /// ```
    pub fn push(&mut self, payload: &[u8]) -> Result<Vec<NalUnit>, H264Error> {
        let Some(header) = payload.first().copied() else {
            return Err(H264Error::InvalidRtp("the payload is empty".into()));
        };
        let nal_type = header & 0x1f;
        match nal_type {
            FU_A_PAYLOAD_TYPE => self.push_fragment(payload),
            STAP_A_PAYLOAD_TYPE => Self::push_aggregation(payload),
            _ if (1..=23).contains(&nal_type) => {
                let unit = NalUnit::from_header(header, payload[1..].to_vec())?;
                Ok(vec![unit])
            }
            other => Err(H264Error::InvalidRtp(format!(
                "the payload type {other} cannot be reassembled"
            ))),
        }
    }

    /// Handles one fragmentation unit.
    fn push_fragment(&mut self, payload: &[u8]) -> Result<Vec<NalUnit>, H264Error> {
        if payload.len() < 3 {
            return Err(H264Error::InvalidRtp(
                "a fragmentation unit needs a header and a fragment header".into(),
            ));
        }
        let fu_indicator = payload[0];
        let fu_header = payload[1];
        let start = (fu_header & 0x80) != 0;
        let end = (fu_header & 0x40) != 0;
        let reserved = (fu_header & 0x20) != 0;
        let nal_type = fu_header & 0x1f;
        if reserved {
            return Err(H264Error::InvalidRtp(
                "the reserved bit of the fragment header is set".into(),
            ));
        }
        let fragment = &payload[2..];

        if start {
            if self.fragment.is_some() {
                // The peer restarted a unit without ending the previous
                // one. The old fragment is dropped.
                self.fragment = None;
            }
            let header = (fu_indicator & 0xe0) | nal_type;
            if fragment.len() > MAX_REASSEMBLED_NAL_SIZE {
                return Err(H264Error::InvalidRtp(
                    "the fragment is larger than the reassembly limit".into(),
                ));
            }
            self.fragment = Some(Fragment {
                header,
                data: fragment.to_vec(),
            });
            if !end {
                return Ok(Vec::new());
            }
        } else {
            let Some(current) = self.fragment.as_mut() else {
                // A continuation without a start: the unit is lost.
                return Ok(Vec::new());
            };
            if current.data.len() + fragment.len() > MAX_REASSEMBLED_NAL_SIZE {
                self.fragment = None;
                return Err(H264Error::InvalidRtp(
                    "the reassembled NAL unit is larger than the limit".into(),
                ));
            }
            current.data.extend_from_slice(fragment);
        }

        if !end {
            return Ok(Vec::new());
        }

        let Some(finished) = self.fragment.take() else {
            return Ok(Vec::new());
        };
        let unit = NalUnit::from_header(finished.header, finished.data)?;
        Ok(vec![unit])
    }

    /// Handles one STAP-A aggregation packet.
    ///
    /// WHY: a sender may pack several small NAL units into one packet.
    ///      The reassembler returns all of them, because a queue would
    ///      have to drop one of the two packets that meet.
    fn push_aggregation(payload: &[u8]) -> Result<Vec<NalUnit>, H264Error> {
        let mut rest = &payload[1..];
        let mut units: Vec<NalUnit> = Vec::new();
        while !rest.is_empty() {
            if rest.len() < 2 {
                return Err(H264Error::InvalidRtp(
                    "an aggregation packet ends inside a length field".into(),
                ));
            }
            let length = u16::from_be_bytes([rest[0], rest[1]]) as usize;
            rest = &rest[2..];
            if length == 0 || length > rest.len() {
                return Err(H264Error::InvalidRtp(
                    "an aggregation packet announces a length that does not fit".into(),
                ));
            }
            let raw = &rest[..length];
            rest = &rest[length..];
            let Some(header) = raw.first().copied() else {
                return Err(H264Error::InvalidRtp("an aggregated unit is empty".into()));
            };
            let unit = NalUnit::from_header(header, raw[1..].to_vec())?;
            units.push(unit);
        }
        Ok(units)
    }
}

#[cfg(test)]
#[path = "rtp_tests.rs"]
mod tests;
