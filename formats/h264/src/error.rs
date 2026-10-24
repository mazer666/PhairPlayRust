// WHAT: the error type of the H.264 codec.
// WHY: every parser in this crate works on bytes that arrived from the
//      network. A rejection has to name the reason, otherwise the
//      receiver log cannot tell a broken sender from an attack.
// EXAMPLE: a bit reader that runs past the end of the buffer returns
//          `H264Error::Truncated`.

use thiserror::Error;

/// Everything that can go wrong while parsing an H.264 bitstream.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum H264Error {
    /// The reader asked for more bits than the buffer holds.
    #[error("the bitstream ends after {available} bits, but {needed} were needed")]
    Truncated {
        /// The number of bits the parser wanted.
        needed: usize,
        /// The number of bits the buffer holds.
        available: usize,
    },

    /// An Exp-Golomb code had more leading zeros than the format allows.
    #[error("an Exp-Golomb code has {leading_zeros} leading zeros, which does not fit")]
    ExpGolombOverflow {
        /// The number of leading zero bits that were found.
        leading_zeros: u32,
    },

    /// A NAL unit is malformed.
    #[error("malformed NAL unit: {0}")]
    InvalidNalUnit(String),

    /// The bitstream uses a feature this parser does not implement.
    #[error("unsupported bitstream feature: {0}")]
    UnsupportedFeature(String),

    /// The length-prefixed (AVCC) container is malformed.
    #[error("malformed length-prefixed stream: {0}")]
    InvalidAvcc(String),

    /// The RTP payload is malformed.
    #[error("malformed RTP payload: {0}")]
    InvalidRtp(String),
}

impl H264Error {
    /// Returns `true` when the error means "the input ended too early".
    ///
    /// WHY: the reassembler treats a truncated fragment differently from
    ///      a broken one, so it needs to be able to tell them apart.
    pub fn is_truncated(&self) -> bool {
        matches!(self, H264Error::Truncated { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncated_is_recognised() {
        assert!(
            H264Error::Truncated {
                needed: 8,
                available: 2
            }
            .is_truncated()
        );
        assert!(!H264Error::InvalidNalUnit("x".into()).is_truncated());
    }

    #[test]
    fn messages_name_the_numbers() {
        let text = H264Error::Truncated {
            needed: 40,
            available: 7,
        }
        .to_string();
        assert!(text.contains("40"), "{text}");
        assert!(text.contains('7'), "{text}");
    }
}
