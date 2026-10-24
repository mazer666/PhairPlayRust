// WHAT: NAL units, the Annex-B byte stream format and the
//      length-prefixed (AVCC) container.
// WHY: an H.264 receiver sees two shapes of the same bitstream: the
//      Annex-B stream with start codes, which the AirPlay and Miracast
//      transports use, and the AVCC container with four byte lengths,
//      which the MP4 world uses. Both have to be split into NAL units,
//      and both arrive from the network, so the split has to survive a
//      truncated or hostile input.
// EXAMPLE: `split_annex_b(&[0, 0, 0, 1, 0x67, 0x42])` returns one NAL
//          unit of type 7 (SPS).

use crate::error::H264Error;

/// The type of a NAL unit.
///
/// The discriminants are the values of the `nal_unit_type` field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NalUnitType {
    /// A slice that is not an IDR picture.
    SliceNonIdr,
    /// A data partition A slice.
    SliceDataPartitionA,
    /// A data partition B slice.
    SliceDataPartitionB,
    /// A data partition C slice.
    SliceDataPartitionC,
    /// An IDR picture.
    SliceIdr,
    /// A sequence information SEI.
    Sei,
    /// A sequence parameter set.
    Sps,
    /// A picture parameter set.
    Pps,
    /// An access unit delimiter.
    Aud,
    /// The end of a sequence.
    EndOfSequence,
    /// The end of a stream.
    EndOfStream,
    /// A filler data unit.
    FillerData,
    /// A reserved or unknown type.
    Reserved(u8),
}

impl NalUnitType {
    /// Returns the numeric value of the type.
    pub fn value(self) -> u8 {
        match self {
            NalUnitType::SliceNonIdr => 1,
            NalUnitType::SliceDataPartitionA => 2,
            NalUnitType::SliceDataPartitionB => 3,
            NalUnitType::SliceDataPartitionC => 4,
            NalUnitType::SliceIdr => 5,
            NalUnitType::Sei => 6,
            NalUnitType::Sps => 7,
            NalUnitType::Pps => 8,
            NalUnitType::Aud => 9,
            NalUnitType::EndOfSequence => 10,
            NalUnitType::EndOfStream => 11,
            NalUnitType::FillerData => 12,
            NalUnitType::Reserved(value) => value,
        }
    }

    /// Maps a numeric value onto a type.
    ///
    /// # Example
    ///
    /// ```
    /// use phairplay_h264::NalUnitType;
    ///
    /// assert_eq!(NalUnitType::from_value(7), NalUnitType::Sps);
    /// assert_eq!(NalUnitType::from_value(31), NalUnitType::Reserved(31));
    /// ```
    pub fn from_value(value: u8) -> NalUnitType {
        match value {
            1 => NalUnitType::SliceNonIdr,
            2 => NalUnitType::SliceDataPartitionA,
            3 => NalUnitType::SliceDataPartitionB,
            4 => NalUnitType::SliceDataPartitionC,
            5 => NalUnitType::SliceIdr,
            6 => NalUnitType::Sei,
            7 => NalUnitType::Sps,
            8 => NalUnitType::Pps,
            9 => NalUnitType::Aud,
            10 => NalUnitType::EndOfSequence,
            11 => NalUnitType::EndOfStream,
            12 => NalUnitType::FillerData,
            other => NalUnitType::Reserved(other),
        }
    }

    /// Returns `true` when the type carries coded picture data.
    pub fn is_vcl(self) -> bool {
        matches!(
            self,
            NalUnitType::SliceNonIdr
                | NalUnitType::SliceDataPartitionA
                | NalUnitType::SliceDataPartitionB
                | NalUnitType::SliceDataPartitionC
                | NalUnitType::SliceIdr
        )
    }

    /// Returns `true` when the type is a parameter set.
    pub fn is_parameter_set(self) -> bool {
        matches!(self, NalUnitType::Sps | NalUnitType::Pps)
    }
}

/// One NAL unit, without its start code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NalUnit {
    /// The type of the unit.
    pub nal_type: NalUnitType,
    /// The `nal_ref_idc` field.
    pub nal_ref_idc: u8,
    /// The payload, that is the NAL unit without the header byte.
    pub payload: Vec<u8>,
}

impl NalUnit {
    /// Builds a NAL unit from its header byte and its payload.
    ///
    /// # Errors
    ///
    /// Returns an error when the header byte announces the forbidden
    /// zero type.
    pub fn from_header(header: u8, payload: Vec<u8>) -> Result<NalUnit, H264Error> {
        let forbidden_zero = (header & 0x80) != 0;
        if forbidden_zero {
            return Err(H264Error::InvalidNalUnit(
                "the forbidden_zero_bit is set".into(),
            ));
        }
        let nal_ref_idc = (header >> 5) & 0x03;
        let nal_type = NalUnitType::from_value(header & 0x1f);
        Ok(NalUnit {
            nal_type,
            nal_ref_idc,
            payload,
        })
    }

    /// Returns the header byte of the unit.
    pub fn header(&self) -> u8 {
        (self.nal_ref_idc << 5) | (self.nal_type.value() & 0x1f)
    }

    /// Returns the whole unit, header byte included.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.payload.len() + 1);
        out.push(self.header());
        out.extend_from_slice(&self.payload);
        out
    }
}

/// Splits an Annex-B stream into its NAL units.
///
/// WHY: the splitter is tolerant by design. A stream may start with
///      leading zero bytes, may carry the three byte and the four byte
///      start code, and may end with a truncated unit. The parser
///      reports what it found and never panics.
///
/// # Example
///
/// ```
/// use phairplay_h264::split_annex_b;
///
/// let units = split_annex_b(&[0, 0, 0, 1, 0x67, 0x42, 0, 0, 1, 0x68, 0xce]);
/// assert_eq!(units.len(), 2);
/// ```
pub fn split_annex_b(data: &[u8]) -> Vec<NalUnit> {
    let mut units = Vec::new();
    let mut start: Option<usize> = None;
    let mut index = 0_usize;
    while index + 2 < data.len() {
        if data[index] == 0 && data[index + 1] == 0 && data[index + 2] == 1 {
            if let Some(begin) = start {
                // The zero bytes in front of the start code belong to the
                // start code, not to the unit that precedes it.
                let mut end = index;
                while end > begin && data[end - 1] == 0 {
                    end -= 1;
                }
                push_unit(&mut units, &data[begin..end]);
            }
            index += 3;
            start = Some(index);
            continue;
        }
        index += 1;
    }
    if let Some(begin) = start {
        push_unit(&mut units, &data[begin..]);
    }
    units
}

/// Appends the NAL unit in `raw` to `units`, if it has a valid header.
fn push_unit(units: &mut Vec<NalUnit>, raw: &[u8]) {
    let Some(header) = raw.first().copied() else {
        return;
    };
    if let Ok(unit) = NalUnit::from_header(header, raw[1..].to_vec()) {
        units.push(unit);
    }
}

/// Converts a NAL unit into an Annex-B stream with a four byte start code.
///
/// # Example
///
/// ```
/// use phairplay_h264::{split_annex_b, to_annex_b, NalUnit};
///
/// let unit = NalUnit { nal_type: phairplay_h264::NalUnitType::Sps, nal_ref_idc: 3, payload: vec![1, 2] };
/// let stream = to_annex_b(&unit);
/// assert_eq!(stream, vec![0, 0, 0, 1, 0x67, 1, 2]);
/// ```
pub fn to_annex_b(unit: &NalUnit) -> Vec<u8> {
    let mut out = Vec::with_capacity(unit.payload.len() + 5);
    out.extend_from_slice(&[0, 0, 0, 1]);
    out.extend_from_slice(&unit.to_bytes());
    out
}

/// Converts a length-prefixed (AVCC) stream into an Annex-B stream.
///
/// # Errors
///
/// Returns an error when a length field points past the end of the
/// buffer or when the buffer is empty.
///
/// # Example
///
/// ```
/// use phairplay_h264::avcc_to_annex_b;
///
/// // One NAL unit of two bytes.
/// let avcc = [0, 0, 0, 2, 0x67, 0x42];
/// let annex_b = avcc_to_annex_b(&avcc).unwrap_or_default();
/// assert_eq!(annex_b, vec![0, 0, 0, 1, 0x67, 0x42]);
/// ```
pub fn avcc_to_annex_b(data: &[u8]) -> Result<Vec<u8>, H264Error> {
    let mut out = Vec::with_capacity(data.len() + 8);
    let mut index = 0_usize;
    while index < data.len() {
        if index + 4 > data.len() {
            return Err(H264Error::InvalidAvcc("a length field is truncated".into()));
        }
        let length = u32::from_be_bytes([
            data[index],
            data[index + 1],
            data[index + 2],
            data[index + 3],
        ]) as usize;
        index += 4;
        if length == 0 {
            return Err(H264Error::InvalidAvcc("a NAL unit has length zero".into()));
        }
        let end = index.checked_add(length).ok_or_else(|| {
            H264Error::InvalidAvcc("a length field overflows the address space".into())
        })?;
        if end > data.len() {
            return Err(H264Error::InvalidAvcc(
                "a length field points past the end of the buffer".into(),
            ));
        }
        let raw = &data[index..end];
        let Some(header) = raw.first().copied() else {
            return Err(H264Error::InvalidAvcc("a NAL unit is empty".into()));
        };
        let unit = NalUnit::from_header(header, raw[1..].to_vec())?;
        out.extend_from_slice(&[0, 0, 0, 1]);
        out.extend_from_slice(&unit.to_bytes());
        index = end;
    }
    Ok(out)
}

/// Converts an Annex-B stream into a length-prefixed (AVCC) stream.
///
/// # Errors
///
/// Returns an error when the stream holds no NAL unit at all.
pub fn annex_b_to_avcc(data: &[u8]) -> Result<Vec<u8>, H264Error> {
    let units = split_annex_b(data);
    if units.is_empty() {
        return Err(H264Error::InvalidAvcc(
            "the stream does not hold a NAL unit".into(),
        ));
    }
    let mut out = Vec::with_capacity(data.len());
    for unit in units {
        let bytes = unit.to_bytes();
        out.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
        out.extend_from_slice(&bytes);
    }
    Ok(out)
}

#[cfg(test)]
#[path = "nal_tests.rs"]
mod tests;
