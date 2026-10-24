// WHAT: the bit reader of the ALAC decoder.
// WHY: the reference decoder reads three bytes ahead of the current
//      position and trusts the caller that the buffer is long enough.
//      A receiver that decodes a frame from the network cannot make that
//      assumption, so this reader checks every access and reports the
//      shortfall instead of reading past the buffer.
// EXAMPLE: `BitReader::read` returns the next sixteen bits, most
//          significant bit first.
//
// ATTRIBUTION: this file is a native Rust port of Apple's ALAC bit
// utilities (`ALACBitUtilities.c` and `ALACBitUtilities.h`,
// Copyright (c) 2011 Apple Inc., licensed under the Apache License,
// Version 2.0). The port was written for PhairPlayRust; no C code was
// copied, and the bounds checks are new.

use crate::error::AlacError;

/// A reader over the bits of an ALAC frame.
#[derive(Debug, Clone)]
pub struct BitReader<'a> {
    data: &'a [u8],
    byte_position: usize,
    bit_index: u32,
}

impl<'a> BitReader<'a> {
    /// Creates a reader over `data`.
    ///
    /// # Example
    ///
    /// ```
    /// use phairplay_alac::BitReader;
    ///
    /// let mut reader = BitReader::new(&[0b1010_1100]);
    /// assert_eq!(reader.read(4).unwrap_or(0), 0b1010);
    /// ```
    pub fn new(data: &'a [u8]) -> BitReader<'a> {
        BitReader {
            data,
            byte_position: 0,
            bit_index: 0,
        }
    }

    /// Returns the number of bits that have not been read yet.
    pub fn bits_remaining(&self) -> usize {
        self.data.len() * 8 - self.byte_position * 8 - self.bit_index as usize
    }

    /// Returns the current position, counted in bits from the start.
    pub fn bit_position(&self) -> usize {
        self.byte_position * 8 + self.bit_index as usize
    }

    /// Returns `true` when at least one bit is left.
    pub fn has_more_data(&self) -> bool {
        self.bits_remaining() > 0
    }

    /// Reads up to sixteen bits, most significant bit first.
    ///
    /// # Errors
    ///
    /// Returns [`AlacError::Truncated`] when the buffer does not hold the
    /// requested bits.
    pub fn read(&mut self, num_bits: u32) -> Result<u32, AlacError> {
        if num_bits > 16 {
            return Err(AlacError::Unsupported(format!(
                "cannot read {num_bits} bits at once"
            )));
        }
        let needed = (self.bit_index + num_bits).div_ceil(8);
        if self.byte_position + needed as usize > self.data.len() {
            return Err(AlacError::Truncated {
                needed: self.bit_position() + num_bits as usize,
                available: self.bits_remaining(),
            });
        }
        let mut value = 0_u32;
        for _ in 0..num_bits {
            let byte = self.data[self.byte_position];
            let bit = 7 - self.bit_index;
            value = (value << 1) | ((byte >> bit) & 1) as u32;
            self.bit_index += 1;
            if self.bit_index == 8 {
                self.bit_index = 0;
                self.byte_position += 1;
            }
        }
        Ok(value)
    }

    /// Reads up to eight bits, most significant bit first.
    ///
    /// # Errors
    ///
    /// Returns [`AlacError::Truncated`] when the buffer does not hold the
    /// requested bits.
    pub fn read_small(&mut self, num_bits: u32) -> Result<u8, AlacError> {
        if num_bits > 8 {
            return Err(AlacError::Unsupported(format!(
                "cannot read {num_bits} bits at once"
            )));
        }
        Ok(self.read(num_bits)? as u8)
    }

    /// Reads a single bit and tells whether it is set.
    ///
    /// # Errors
    ///
    /// Returns [`AlacError::Truncated`] when the buffer is exhausted.
    pub fn read_one(&mut self) -> Result<u8, AlacError> {
        Ok(self.read(1)? as u8)
    }

    /// Skips `num_bits` bits.
    ///
    /// # Errors
    ///
    /// Returns [`AlacError::Truncated`] when the buffer does not hold the
    /// requested bits.
    pub fn advance(&mut self, num_bits: u32) -> Result<(), AlacError> {
        let needed = (self.bit_index + num_bits).div_ceil(8);
        if self.byte_position + needed as usize > self.data.len() {
            return Err(AlacError::Truncated {
                needed: self.bit_position() + num_bits as usize,
                available: self.bits_remaining(),
            });
        }
        self.bit_index += num_bits;
        self.byte_position += (self.bit_index >> 3) as usize;
        self.bit_index &= 7;
        Ok(())
    }

    /// Skips to the next byte boundary.
    pub fn byte_align(&mut self) {
        if self.bit_index != 0 {
            self.bit_index = 0;
            self.byte_position += 1;
        }
    }

    /// Returns the number of bytes that are left, rounded up.
    pub fn bytes_remaining(&self) -> usize {
        self.bits_remaining().div_ceil(8)
    }

    /// Returns the buffer that the reader works on.
    pub fn data(&self) -> &'a [u8] {
        self.data
    }

    /// Moves the reader to an absolute bit position.
    ///
    /// WHY: the adaptive Golomb decoder works on the raw buffer and on
    ///      an absolute bit position, because it reads four bytes at an
    ///      arbitrary offset. The reader has to follow it afterwards.
    pub fn seek_to_bit(&mut self, position: usize) {
        self.byte_position = position / 8;
        self.bit_index = (position % 8) as u32;
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    #[test]
    fn reads_bits_most_significant_first() {
        let mut reader = BitReader::new(&[0b1010_1100, 0b0101_0011]);
        assert_eq!(reader.read(4).unwrap_or(0), 0b1010);
        assert_eq!(reader.read(4).unwrap_or(0), 0b1100);
        assert_eq!(reader.read(8).unwrap_or(0), 0b0101_0011);
        assert_eq!(reader.bits_remaining(), 0);
    }

    #[test]
    fn reads_across_a_byte_boundary() {
        let mut reader = BitReader::new(&[0b1111_0000, 0b1010_1010]);
        assert_eq!(reader.read(6).unwrap_or(0), 0b111100);
        assert_eq!(reader.read(6).unwrap_or(0), 0b001010);
    }

    #[test]
    fn reports_a_truncated_read() {
        let mut reader = BitReader::new(&[0xff]);
        let error = match reader.read(12) {
            Ok(_) => panic!("expected an error"),
            Err(err) => err,
        };
        assert!(matches!(error, AlacError::Truncated { .. }), "{error:?}");
    }

    #[test]
    fn refuses_to_read_more_than_sixteen_bits() {
        let mut reader = BitReader::new(&[0xff; 4]);
        assert!(matches!(reader.read(17), Err(AlacError::Unsupported(_))));
    }

    #[test]
    fn advances_over_bits() {
        let mut reader = BitReader::new(&[0xff; 4]);
        reader.advance(12).unwrap_or(());
        assert_eq!(reader.bit_position(), 12);
        assert_eq!(reader.read(4).unwrap_or(0), 0b1111);
    }

    #[test]
    fn aligns_to_the_next_byte() {
        let mut reader = BitReader::new(&[0b1010_1100, 0b0101_0011]);
        reader.read(3).unwrap_or(0);
        reader.byte_align();
        assert_eq!(reader.bit_position(), 8);
        assert_eq!(reader.read(8).unwrap_or(0), 0b0101_0011);
    }

    #[test]
    fn counts_the_remaining_bits() {
        let reader = BitReader::new(&[0xff; 4]);
        assert_eq!(reader.bits_remaining(), 32);
        assert_eq!(reader.bytes_remaining(), 4);
    }
}
