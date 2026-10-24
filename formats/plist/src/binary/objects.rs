// WHAT: the object decoder of the binary property-list format.
// WHY: the trailer, the offset table and the object markers are three
//      different trust boundaries. The trailer is parsed in `decode.rs`;
//      this file decodes the objects themselves, always checking a length
//      against the bytes that are present before anything is allocated.
// EXAMPLE: `BinaryDecoder::decode_object` turns an object index into a
//          [`Value`](crate::Value) and refuses a cyclic reference.

use crate::error::PlistError;
use crate::limits::PlistLimits;
use crate::value::Value;

/// The state of one binary property-list decoding run.
pub(super) struct BinaryDecoder<'a> {
    pub(super) input: &'a [u8],
    pub(super) offsets: Vec<usize>,
    pub(super) ref_size: usize,
    pub(super) limits: &'a PlistLimits,
    pub(super) region_end: usize,
    pub(super) cache: Vec<Option<Value>>,
    pub(super) in_progress: Vec<bool>,
    pub(super) nodes_used: usize,
}

impl BinaryDecoder<'_> {
    /// Decodes the object with the given index.
    pub(super) fn decode_object(&mut self, index: u64, depth: usize) -> Result<Value, PlistError> {
        let slot = usize::try_from(index).map_err(|_| PlistError::ObjectIndexOutOfRange {
            index,
            count: self.offsets.len(),
        })?;
        if slot >= self.offsets.len() {
            return Err(PlistError::ObjectIndexOutOfRange {
                index,
                count: self.offsets.len(),
            });
        }
        if depth > self.limits.max_depth {
            return Err(PlistError::DepthLimitExceeded {
                limit: self.limits.max_depth,
            });
        }
        if self.in_progress[slot] {
            return Err(PlistError::CyclicReference { index });
        }
        if let Some(cached) = &self.cache[slot] {
            let nodes = cached.node_count();
            let value = cached.clone();
            self.charge_nodes(nodes)?;
            return Ok(value);
        }

        self.in_progress[slot] = true;
        let result = self.decode_object_at(self.offsets[slot], depth);
        self.in_progress[slot] = false;
        let value = result?;
        self.charge_nodes(value.node_count())?;
        self.cache[slot] = Some(value.clone());
        Ok(value)
    }

    /// Charges the expansion budget and rejects a graph that expands too far.
    fn charge_nodes(&mut self, nodes: usize) -> Result<(), PlistError> {
        self.nodes_used = self.nodes_used.saturating_add(nodes);
        if self.nodes_used > self.limits.max_expanded_nodes {
            return Err(PlistError::NodeBudgetExceeded {
                limit: self.limits.max_expanded_nodes,
            });
        }
        Ok(())
    }

    /// Decodes the object that starts at `offset`.
    fn decode_object_at(&mut self, offset: usize, depth: usize) -> Result<Value, PlistError> {
        let marker = *self
            .input
            .get(offset)
            .ok_or(PlistError::Truncated { offset })?;
        let kind = marker >> 4;
        let low = marker & 0x0f;

        match kind {
            0x0 => self.decode_singleton(marker, low),
            0x1 => self.decode_integer(offset, low),
            0x2 => self.decode_real(offset, low),
            0x3 => self.decode_date(offset),
            0x4 => self.decode_data(offset, low),
            0x5 => self.decode_ascii(offset, low),
            0x6 => self.decode_utf16(offset, low),
            0x8 => Err(PlistError::UnsupportedObject {
                description: "unique identifier",
            }),
            0xa => self.decode_array(offset, low, depth),
            0xd => self.decode_dictionary(offset, low, depth),
            _ => Err(PlistError::UnknownObjectType { marker }),
        }
    }

    /// Decodes the singletons `null`, `false`, `true` and `fill`.
    fn decode_singleton(&self, marker: u8, low: u8) -> Result<Value, PlistError> {
        match low {
            0x08 => Ok(Value::Boolean(false)),
            0x09 => Ok(Value::Boolean(true)),
            0x00 => Err(PlistError::UnsupportedObject {
                description: "null",
            }),
            0x0f => Err(PlistError::UnsupportedObject {
                description: "fill",
            }),
            _ => Err(PlistError::UnknownObjectType { marker }),
        }
    }

    /// Decodes an integer of `1 << exponent` bytes.
    fn decode_integer(&self, offset: usize, exponent: u8) -> Result<Value, PlistError> {
        let size = match exponent {
            0 => 1_usize,
            1 => 2,
            2 => 4,
            3 => 8,
            4 => 16,
            _ => {
                return Err(PlistError::UnknownObjectType {
                    marker: 0x10 | exponent,
                });
            }
        };
        let raw = self.slice(offset + 1, size)?;
        match size {
            1 => Ok(Value::Integer(raw[0] as i64)),
            2 => Ok(Value::Integer(u16::from_be_bytes([raw[0], raw[1]]) as i64)),
            4 => Ok(Value::Integer(
                u32::from_be_bytes([raw[0], raw[1], raw[2], raw[3]]) as i64,
            )),
            8 => Ok(Value::Integer(i64::from_be_bytes([
                raw[0], raw[1], raw[2], raw[3], raw[4], raw[5], raw[6], raw[7],
            ]))),
            _ => {
                // A 128-bit integer only fits when its high half is empty
                // and its sign bit is clear.
                if raw[..8].iter().any(|byte| *byte != 0) || raw[8] & 0x80 != 0 {
                    return Err(PlistError::IntegerOutOfRange);
                }
                let mut bytes = [0_u8; 8];
                bytes.copy_from_slice(&raw[8..16]);
                Ok(Value::Integer(i64::from_be_bytes(bytes)))
            }
        }
    }

    /// Decodes a floating point number.
    ///
    /// The low nibble of the marker holds the base-two logarithm of the
    /// byte count, so `0x22` is a four byte `f32` and `0x23` an eight
    /// byte `f64`.
    fn decode_real(&self, offset: usize, exponent: u8) -> Result<Value, PlistError> {
        match exponent {
            2 => {
                let raw = self.slice(offset + 1, 4)?;
                let bits = u32::from_be_bytes([raw[0], raw[1], raw[2], raw[3]]);
                Ok(Value::Real(f32::from_bits(bits) as f64))
            }
            3 => {
                let raw = self.slice(offset + 1, 8)?;
                let mut bytes = [0_u8; 8];
                bytes.copy_from_slice(raw);
                Ok(Value::Real(f64::from_be_bytes(bytes)))
            }
            _ => Err(PlistError::UnsupportedObject {
                description: "real of an unsupported width",
            }),
        }
    }

    /// Decodes a date, in seconds since the Apple epoch.
    fn decode_date(&self, offset: usize) -> Result<Value, PlistError> {
        let raw = self.slice(offset + 1, 8)?;
        let mut bytes = [0_u8; 8];
        bytes.copy_from_slice(raw);
        Ok(Value::Date(f64::from_be_bytes(bytes)))
    }

    /// Decodes a data blob.
    fn decode_data(&self, offset: usize, low: u8) -> Result<Value, PlistError> {
        let length = self.read_count(offset, low)?;
        if length > self.limits.max_data_len as u64 {
            return Err(PlistError::DataTooLarge {
                length,
                limit: self.limits.max_data_len,
            });
        }
        let start = offset + 1 + self.count_bytes(offset, low);
        let bytes = self.slice(start, length as usize)?;
        Ok(Value::Data(bytes.to_vec()))
    }

    /// Decodes an ASCII string.
    fn decode_ascii(&self, offset: usize, low: u8) -> Result<Value, PlistError> {
        let length = self.read_count(offset, low)?;
        if length > self.limits.max_string_len as u64 {
            return Err(PlistError::StringTooLong {
                length,
                limit: self.limits.max_string_len,
            });
        }
        let start = offset + 1 + self.count_bytes(offset, low);
        let raw = self.slice(start, length as usize)?;
        let text = std::str::from_utf8(raw)
            .map_err(|_| PlistError::InvalidTextEncoding { encoding: "UTF-8" })?;
        Ok(Value::String(text.to_string()))
    }

    /// Decodes a UTF-16 big endian string.
    fn decode_utf16(&self, offset: usize, low: u8) -> Result<Value, PlistError> {
        let length = self.read_count(offset, low)?;
        if length > self.limits.max_string_len as u64 {
            return Err(PlistError::StringTooLong {
                length,
                limit: self.limits.max_string_len,
            });
        }
        let start = offset + 1 + self.count_bytes(offset, low);
        let raw = self.slice(start, length as usize * 2)?;
        let units: Vec<u16> = raw
            .chunks_exact(2)
            .map(|pair| u16::from_be_bytes([pair[0], pair[1]]))
            .collect();
        let text = String::from_utf16(&units)
            .map_err(|_| PlistError::InvalidTextEncoding { encoding: "UTF-16" })?;
        Ok(Value::String(text))
    }

    /// Decodes an array.
    fn decode_array(&mut self, offset: usize, low: u8, depth: usize) -> Result<Value, PlistError> {
        let count = self.read_count(offset, low)?;
        if count > self.limits.max_container_len as u64 {
            return Err(PlistError::ContainerTooLarge {
                length: count,
                limit: self.limits.max_container_len,
            });
        }
        let start = offset + 1 + self.count_bytes(offset, low);
        let mut items = Vec::new();
        for index in 0..count {
            let reference = self.read_ref(start + index as usize * self.ref_size)?;
            items.push(self.decode_object(reference, depth + 1)?);
        }
        Ok(Value::Array(items))
    }

    /// Decodes a dictionary.
    fn decode_dictionary(
        &mut self,
        offset: usize,
        low: u8,
        depth: usize,
    ) -> Result<Value, PlistError> {
        let count = self.read_count(offset, low)?;
        if count > self.limits.max_container_len as u64 {
            return Err(PlistError::ContainerTooLarge {
                length: count,
                limit: self.limits.max_container_len,
            });
        }
        let start = offset + 1 + self.count_bytes(offset, low);
        let key_start = start;
        let value_start = key_start + count as usize * self.ref_size;
        let mut entries = Vec::new();
        for index in 0..count {
            let key_ref = self.read_ref(key_start + index as usize * self.ref_size)?;
            let value_ref = self.read_ref(value_start + index as usize * self.ref_size)?;
            let key = self.decode_object(key_ref, depth + 1)?;
            let key = match key {
                Value::String(text) => text,
                other => {
                    return Err(PlistError::Encode(format!(
                        "a dictionary key must be a string, found {other:?}"
                    )));
                }
            };
            let value = self.decode_object(value_ref, depth + 1)?;
            entries.push((key, value));
        }
        Ok(Value::Dictionary(entries))
    }

    /// Returns the number of bytes the count field of an object occupies.
    ///
    /// The count is stored inline, right behind the marker byte, when it
    /// does not fit into the low nibble. It is *not* a separate entry of
    /// the offset table.
    fn count_bytes(&self, offset: usize, low: u8) -> usize {
        if low == 0x0f {
            match self.input.get(offset + 1).map(|marker| marker & 0x0f) {
                Some(exponent) if exponent <= 3 => 1 + (1_usize << exponent),
                _ => 0,
            }
        } else {
            0
        }
    }

    /// Reads the element count of a container, a string or a data blob.
    ///
    /// The low nibble of the marker holds the count literally. Only when
    /// it is `0xF` an integer object follows, which is read inline.
    fn read_count(&self, offset: usize, low: u8) -> Result<u64, PlistError> {
        if low != 0x0f {
            return Ok(low as u64);
        }
        let size = match self.input.get(offset + 1).map(|marker| marker & 0x0f) {
            Some(exponent) if exponent <= 3 => 1_usize << exponent,
            _ => {
                return Err(PlistError::BadTrailer {
                    reason: "a count field is not an integer object",
                });
            }
        };
        let raw = self.slice(offset + 2, size)?;
        let mut value = 0_u64;
        for byte in raw {
            value = (value << 8) | *byte as u64;
        }
        Ok(value)
    }

    /// Reads one object reference.
    fn read_ref(&self, offset: usize) -> Result<u64, PlistError> {
        let raw = self.slice(offset, self.ref_size)?;
        let mut value = 0_u64;
        for byte in raw {
            value = (value << 8) | *byte as u64;
        }
        Ok(value)
    }

    /// Returns `size` bytes at `offset`, checking that they are present.
    fn slice(&self, offset: usize, size: usize) -> Result<&[u8], PlistError> {
        let end = offset
            .checked_add(size)
            .ok_or(PlistError::LengthExceedsInput {
                length: size as u64,
                remaining: 0,
            })?;
        if end > self.region_end {
            return Err(PlistError::LengthExceedsInput {
                length: size as u64,
                remaining: self.region_end.saturating_sub(offset),
            });
        }
        Ok(&self.input[offset..end])
    }
}
