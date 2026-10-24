// WHAT: a bit reader for the H.264 RBSP syntax.
// WHY: the Exp-Golomb codes of the specification are the classic place
//      where a parser overflows. The code number of a `ue(v)` element is
//      `2^leadingZeroBits - 1 + suffix`, and with 31 leading zero bits
//      the term `2^31` does not fit into a signed 32-bit integer. This
//      reader therefore counts in `u64`, clamps the result, and refuses a
//      code with more leading zeros than the format allows.
// EXAMPLE: a buffer of 31 zero bits, a one bit and 31 one bits decodes
//          to the code number 4 294 967 294 without overflowing.

use crate::error::H264Error;

/// The largest number of leading zero bits an Exp-Golomb code may have.
pub const MAX_LEADING_ZERO_BITS: u32 = 32;

/// A reader over the bits of an RBSP.
#[derive(Debug, Clone)]
pub struct BitReader<'a> {
    data: &'a [u8],
    bit_position: usize,
}

impl<'a> BitReader<'a> {
    /// Creates a reader over `data`.
    ///
    /// # Example
    ///
    /// ```
    /// use phairplay_h264::BitReader;
    ///
    /// let mut reader = BitReader::new(&[0b1010_0000]);
    /// assert_eq!(reader.read_bits(4).unwrap_or(0), 0b1010);
    /// ```
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            bit_position: 0,
        }
    }

    /// Returns the number of bits that have not been read yet.
    pub fn bits_remaining(&self) -> usize {
        self.data.len() * 8 - self.bit_position
    }

    /// Returns the current position, counted in bits from the start.
    pub fn bit_position(&self) -> usize {
        self.bit_position
    }

    /// Returns `true` when at least one bit is left.
    pub fn has_more_data(&self) -> bool {
        self.bits_remaining() > 0
    }

    /// Reads a single bit.
    ///
    /// # Errors
    ///
    /// Returns [`H264Error::Truncated`] when the buffer is exhausted.
    pub fn read_bit(&mut self) -> Result<u32, H264Error> {
        if self.bits_remaining() == 0 {
            return Err(H264Error::Truncated {
                needed: 1,
                available: 0,
            });
        }
        let byte = self.data[self.bit_position / 8];
        let shift = 7 - (self.bit_position % 8);
        self.bit_position += 1;
        Ok(((byte >> shift) & 1) as u32)
    }

    /// Reads `count` bits, most significant bit first.
    ///
    /// # Errors
    ///
    /// Returns [`H264Error::Truncated`] when the buffer is exhausted.
    pub fn read_bits(&mut self, count: u32) -> Result<u32, H264Error> {
        if count > 32 {
            return Err(H264Error::UnsupportedFeature(format!(
                "cannot read {count} bits at once"
            )));
        }
        if count as usize > self.bits_remaining() {
            return Err(H264Error::Truncated {
                needed: count as usize,
                available: self.bits_remaining(),
            });
        }
        let mut value = 0_u32;
        for _ in 0..count {
            value = (value << 1) | self.read_bit()?;
        }
        Ok(value)
    }

    /// Reads one bit and tells whether it is set.
    ///
    /// # Errors
    ///
    /// Returns [`H264Error::Truncated`] when the buffer is exhausted.
    pub fn read_flag(&mut self) -> Result<bool, H264Error> {
        Ok(self.read_bit()? == 1)
    }

    /// Reads an unsigned Exp-Golomb code, `ue(v)`.
    ///
    /// WHY: the code number is computed in `u64`. With 31 leading zero
    ///      bits the term `2^31` overflows a signed 32-bit integer, and
    ///      with 32 leading zero bits the result does not fit into a
    ///      `u32` at all. The reader computes in `u64` and clamps.
    ///
    /// # Errors
    ///
    /// Returns [`H264Error::ExpGolombOverflow`] when the code has more
    /// leading zeros than the format allows, and
    /// [`H264Error::Truncated`] when the buffer ends inside the code.
    pub fn read_ue(&mut self) -> Result<u32, H264Error> {
        let mut leading_zeros: u32 = 0;
        loop {
            if self.read_bit()? == 1 {
                break;
            }
            leading_zeros += 1;
            if leading_zeros > MAX_LEADING_ZERO_BITS {
                return Err(H264Error::ExpGolombOverflow { leading_zeros });
            }
        }
        if leading_zeros == 0 {
            return Ok(0);
        }
        let suffix = self.read_bits(leading_zeros)? as u64;
        // `1u64 << 31` is the first term that does not fit into an `i32`.
        let code_number = ((1_u64 << leading_zeros) - 1) + suffix;
        Ok(code_number.min(u32::MAX as u64) as u32)
    }

    /// Reads a signed Exp-Golomb code, `se(v)`.
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`BitReader::read_ue`].
    pub fn read_se(&mut self) -> Result<i32, H264Error> {
        let code_number = self.read_ue()?;
        let value = if code_number % 2 == 0 {
            -((code_number / 2) as i64)
        } else {
            code_number.div_ceil(2) as i64
        };
        Ok(value.clamp(i32::MIN as i64, i32::MAX as i64) as i32)
    }

    /// Skips to the next byte boundary.
    pub fn byte_align(&mut self) {
        let remainder = self.bit_position % 8;
        if remainder != 0 {
            self.bit_position += 8 - remainder;
        }
    }

    /// Returns `true` when the RBSP holds more data after the current
    /// position.
    ///
    /// WHY: the picture parameter set ends with fields that are only
    ///      present in a later revision of the specification. The RBSP
    ///      ends with a one bit followed by zero bits, so the last set
    ///      bit marks the end of the payload.
    pub fn more_rbsp_data(&self) -> bool {
        let total = self.data.len() * 8;
        for index in (0..total).rev() {
            let byte = self.data[index / 8];
            let bit = 7 - (index % 8);
            if (byte >> bit) & 1 == 1 {
                return self.bit_position < index;
            }
        }
        self.bit_position < total
    }

    /// Returns the bits that follow the current position, byte aligned.
    ///
    /// # Errors
    ///
    /// Returns [`H264Error::Truncated`] when the reader is not at a byte
    /// boundary or when no byte is left.
    pub fn read_remaining_bytes(&mut self) -> Result<&'a [u8], H264Error> {
        if self.bit_position % 8 != 0 {
            return Err(H264Error::Truncated {
                needed: 8,
                available: self.bits_remaining(),
            });
        }
        let start = self.bit_position / 8;
        if start > self.data.len() {
            return Err(H264Error::Truncated {
                needed: 1,
                available: 0,
            });
        }
        self.bit_position = self.data.len() * 8;
        Ok(&self.data[start..])
    }
}

/// A bit writer that the tests use to build bit streams.
#[cfg(test)]
pub(crate) struct TestBitWriter {
    bytes: Vec<u8>,
    partial: u8,
    filled: u8,
}

#[cfg(test)]
impl TestBitWriter {
    /// Creates an empty writer.
    pub(crate) fn new() -> TestBitWriter {
        TestBitWriter {
            bytes: Vec::new(),
            partial: 0,
            filled: 0,
        }
    }

    /// Appends one bit.
    pub(crate) fn bit(&mut self, value: bool) {
        self.partial = (self.partial << 1) | u8::from(value);
        self.filled += 1;
        if self.filled == 8 {
            self.bytes.push(self.partial);
            self.partial = 0;
            self.filled = 0;
        }
    }

    /// Appends `count` bits, most significant bit first.
    pub(crate) fn bits(&mut self, value: u32, count: u32) {
        for shift in (0..count).rev() {
            self.bit((value >> shift) & 1 == 1);
        }
    }

    /// Appends an unsigned Exp-Golomb code.
    pub(crate) fn ue(&mut self, value: u32) {
        let code = value + 1;
        let length = 32 - code.leading_zeros();
        for _ in 0..(length - 1) {
            self.bit(false);
        }
        self.bits(code, length);
    }

    /// Returns the bytes written so far, padded with zero bits.
    pub(crate) fn finish(&mut self) -> Vec<u8> {
        if self.filled > 0 {
            self.partial <<= 8 - self.filled;
            self.bytes.push(self.partial);
            self.partial = 0;
            self.filled = 0;
        }
        self.bytes.clone()
    }
}

/// Removes the emulation prevention bytes from an RBSP.
///
/// WHY: the byte sequence `00 00 03` inside a NAL unit payload is an
///      escape for the start code `00 00 01`. A parser that works on the
///      raw payload would read the `03` as data.
///
/// # Example
///
/// ```
/// use phairplay_h264::unescape_rbsp;
///
/// assert_eq!(unescape_rbsp(&[0x00, 0x00, 0x03, 0x01]), vec![0x00, 0x00, 0x01]);
/// ```
pub fn unescape_rbsp(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len());
    let mut zeros = 0_usize;
    for byte in data {
        if zeros >= 2 && *byte == 0x03 {
            zeros = 0;
            continue;
        }
        if *byte == 0x00 {
            zeros += 1;
        } else {
            zeros = 0;
        }
        out.push(*byte);
    }
    out
}

#[cfg(test)]
#[path = "bits_tests.rs"]
mod tests;
