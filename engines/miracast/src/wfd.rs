//! The Wi-Fi Display information element.
//!
//! WHY: a Miracast sender and a Miracast sink introduce themselves with a
//! *WFD information element* — a sequence of typed blocks that describe the
//! device, the codecs it speaks and the transports it can use. The sink
//! parses the element of its peer to learn what the peer can do.
//!
//! HOW: the element is a list of blocks. Every block is a one-byte type, a
//! one-byte length and then exactly that many bytes of payload. A block whose
//! length runs past the end of the buffer is a broken element, and a broken
//! element is **rejected** — never repaired, never partially read. The
//! attacker here is a device on the local network, so every length is
//! checked before a single byte is copied.
//!
//! ## Example
//!
//! ```
//! use phairplay_miracast::wfd::WfdInformationElement;
//!
//! // A protocol version block: type 0x10, length 2, version 1.0.
//! let element = WfdInformationElement::parse(&[0x10, 0x02, 0x01, 0x00])
//!     .expect("a well-formed element");
//! assert_eq!(element.blocks().len(), 1);
//! ```

use crate::error::MiracastError;

/// The largest information element the parser accepts, in bytes.
///
/// WHY: the element travels in a RTSP message that is already length-limited,
/// but the element itself is parsed on its own. A sender must not be able to
/// make the receiver allocate an arbitrary amount of memory.
pub const MAX_ELEMENT_BYTES: usize = 64 * 1024;

/// The largest number of blocks one element may contain.
pub const MAX_BLOCKS: usize = 64;

/// One block of an information element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WfdBlock<'a> {
    /// The type of the block.
    pub kind: WfdBlockKind,
    /// The payload of the block.
    pub payload: &'a [u8],
}

/// The type of a block of an information element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WfdBlockKind {
    /// The device type: source, sink or both.
    DeviceType,
    /// The device information block.
    DeviceInformation,
    /// The WFD protocol version the peer speaks.
    ProtocolVersion,
    /// The capabilities the peer offers.
    Capability,
    /// The sink information: the audio and video formats.
    SinkInfo,
    /// The transport capabilities the peer offers.
    TransportCapability,
    /// Any block type this parser does not know.
    Unknown(u8),
}

impl WfdBlockKind {
    /// Maps a raw type byte onto a block kind.
    #[must_use]
    pub const fn from_byte(value: u8) -> Self {
        match value {
            0x00 => WfdBlockKind::DeviceType,
            0x01 => WfdBlockKind::DeviceInformation,
            0x10 => WfdBlockKind::ProtocolVersion,
            0x12 => WfdBlockKind::Capability,
            0x13 => WfdBlockKind::SinkInfo,
            0x15 => WfdBlockKind::TransportCapability,
            other => WfdBlockKind::Unknown(other),
        }
    }
}

/// A parsed WFD information element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WfdInformationElement<'a> {
    blocks: Vec<WfdBlock<'a>>,
}

impl<'a> WfdInformationElement<'a> {
    /// Parses an information element.
    ///
    /// # Errors
    /// Returns [`MiracastError::ElementTooLarge`] when the element is longer
    /// than [`MAX_ELEMENT_BYTES`], [`MiracastError::TooManyBlocks`] when it
    /// holds more than [`MAX_BLOCKS`] blocks, and
    /// [`MiracastError::TruncatedBlock`] when a block announces a length that
    /// runs past the end of the buffer.
    pub fn parse(input: &'a [u8]) -> Result<Self, MiracastError> {
        if input.len() > MAX_ELEMENT_BYTES {
            return Err(MiracastError::ElementTooLarge {
                length: input.len(),
                limit: MAX_ELEMENT_BYTES,
            });
        }

        let mut blocks = Vec::new();
        let mut offset = 0usize;

        while offset < input.len() {
            if blocks.len() >= MAX_BLOCKS {
                return Err(MiracastError::TooManyBlocks { limit: MAX_BLOCKS });
            }
            if input.len() - offset < 2 {
                return Err(MiracastError::TruncatedBlock {
                    offset,
                    reason: "a block needs at least a type and a length byte".to_string(),
                });
            }

            let kind = WfdBlockKind::from_byte(input[offset]);
            let length = usize::from(input[offset + 1]);
            let start = offset + 2;
            let end = start
                .checked_add(length)
                .ok_or(MiracastError::TruncatedBlock {
                    offset,
                    reason: "the length of a block overflows".to_string(),
                })?;

            if end > input.len() {
                return Err(MiracastError::TruncatedBlock {
                    offset,
                    reason: format!(
                        "the block announces {length} bytes but only {} are left",
                        input.len() - start
                    ),
                });
            }

            blocks.push(WfdBlock {
                kind,
                payload: &input[start..end],
            });
            offset = end;
        }

        Ok(Self { blocks })
    }

    /// The blocks of the element.
    #[must_use]
    pub fn blocks(&self) -> &[WfdBlock<'a>] {
        &self.blocks
    }

    /// The first block of a given kind.
    #[must_use]
    pub fn block(&self, kind: WfdBlockKind) -> Option<&WfdBlock<'a>> {
        self.blocks.iter().find(|block| block.kind == kind)
    }

    /// The WFD protocol version the peer speaks, if it announced one.
    ///
    /// # Errors
    /// Returns [`MiracastError::MalformedBlock`] when the version block does
    /// not carry exactly two bytes.
    pub fn protocol_version(&self) -> Result<Option<(u8, u8)>, MiracastError> {
        let Some(block) = self.block(WfdBlockKind::ProtocolVersion) else {
            return Ok(None);
        };
        if block.payload.len() != 2 {
            return Err(MiracastError::MalformedBlock {
                kind: "protocol version",
                reason: format!("expected 2 bytes, found {}", block.payload.len()),
            });
        }
        Ok(Some((block.payload[0], block.payload[1])))
    }

    /// Whether the element announces that the peer is a sink.
    ///
    /// # Errors
    /// Returns [`MiracastError::MalformedBlock`] when the device type block
    /// does not carry exactly one byte.
    pub fn is_sink(&self) -> Result<bool, MiracastError> {
        let Some(block) = self.block(WfdBlockKind::DeviceType) else {
            return Ok(false);
        };
        if block.payload.len() != 1 {
            return Err(MiracastError::MalformedBlock {
                kind: "device type",
                reason: format!("expected 1 byte, found {}", block.payload.len()),
            });
        }
        // WHY: the device type is a bit field; bit 1 is the sink role.
        Ok(block.payload[0] & 0x02 != 0)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    #[test]
    fn an_element_with_one_block_is_parsed() {
        let element = WfdInformationElement::parse(&[0x10, 0x02, 0x01, 0x00]).expect("parse");
        assert_eq!(element.blocks().len(), 1);
        assert!(element.block(WfdBlockKind::ProtocolVersion).is_some());
    }

    #[test]
    fn several_blocks_are_parsed_in_order() {
        let input = [0x00, 0x01, 0x03, 0x10, 0x02, 0x01, 0x00];
        let element = WfdInformationElement::parse(&input).expect("parse");
        assert_eq!(element.blocks().len(), 2);
        assert_eq!(element.blocks()[0].kind, WfdBlockKind::DeviceType);
        assert_eq!(element.blocks()[1].kind, WfdBlockKind::ProtocolVersion);
    }

    #[test]
    fn an_empty_element_is_valid() {
        let element = WfdInformationElement::parse(&[]).expect("parse");
        assert!(element.blocks().is_empty());
    }

    #[test]
    fn the_protocol_version_is_read() {
        let element = WfdInformationElement::parse(&[0x10, 0x02, 0x01, 0x00]).expect("parse");
        assert_eq!(element.protocol_version().expect("version"), Some((1, 0)));
    }

    #[test]
    fn a_block_that_runs_past_the_end_is_rejected() {
        // The block announces 10 bytes but only 2 are present.
        let error = WfdInformationElement::parse(&[0x10, 0x0a, 0x01, 0x00]).expect_err("must fail");
        assert!(
            matches!(error, MiracastError::TruncatedBlock { .. }),
            "{error}"
        );
    }

    #[test]
    fn a_trailing_type_byte_without_a_length_is_rejected() {
        let error = WfdInformationElement::parse(&[0x10]).expect_err("must fail");
        assert!(
            matches!(error, MiracastError::TruncatedBlock { .. }),
            "{error}"
        );
    }

    #[test]
    fn an_oversized_element_is_rejected() {
        let input = vec![0_u8; MAX_ELEMENT_BYTES + 1];
        let error = WfdInformationElement::parse(&input).expect_err("must fail");
        assert!(
            matches!(error, MiracastError::ElementTooLarge { .. }),
            "{error}"
        );
    }

    #[test]
    fn too_many_blocks_are_rejected() {
        // Every block is two bytes long and carries no payload.
        let input = vec![0x00_u8; MAX_BLOCKS * 2 + 2];
        let error = WfdInformationElement::parse(&input).expect_err("must fail");
        assert!(
            matches!(error, MiracastError::TooManyBlocks { .. }),
            "{error}"
        );
    }

    #[test]
    fn a_version_block_of_the_wrong_length_is_rejected() {
        let element = WfdInformationElement::parse(&[0x10, 0x01, 0x01]).expect("parse");
        let error = element.protocol_version().expect_err("must fail");
        assert!(
            matches!(error, MiracastError::MalformedBlock { .. }),
            "{error}"
        );
    }

    #[test]
    fn the_sink_role_is_read_from_the_device_type() {
        let element = WfdInformationElement::parse(&[0x00, 0x01, 0x02]).expect("parse");
        assert!(element.is_sink().expect("sink"));
    }

    #[test]
    fn a_source_only_device_is_not_a_sink() {
        let element = WfdInformationElement::parse(&[0x00, 0x01, 0x01]).expect("parse");
        assert!(!element.is_sink().expect("sink"));
    }
}
