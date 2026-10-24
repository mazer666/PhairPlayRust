// WHAT: the error type of the ALAC decoder.
// WHY: an ALAC frame is a bitstream that arrives from the network. The
//      decoder has to reject a broken frame with a reason, and it has to
//      reject it *before* it writes into an output buffer.
// EXAMPLE: a frame whose element announces more samples than the
//      configuration allows produces `AlacError::FrameTooLong`.
//
// ATTRIBUTION: the error taxonomy follows the reference decoder by
// Apple Inc. (`ALACDecoder.cpp`, Copyright (c) 2011 Apple Inc., licensed
// under the Apache License, Version 2.0). The Rust code is an
// independent implementation; no C code was copied.

use thiserror::Error;

/// Everything that can go wrong while decoding an ALAC frame.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AlacError {
    /// The bitstream ends inside a field.
    #[error("the ALAC bitstream ends after {available} bits, but {needed} were needed")]
    Truncated {
        /// The number of bits the parser wanted.
        needed: usize,
        /// The number of bits the buffer holds.
        available: usize,
    },

    /// The bitstream holds a field that the decoder does not implement.
    #[error("unsupported ALAC feature: {0}")]
    Unsupported(String),

    /// The magic cookie does not describe a usable configuration.
    #[error("invalid ALAC magic cookie: {0}")]
    InvalidConfig(String),

    /// The frame announces more samples than the configuration allows.
    #[error("the frame announces {announced} samples, but the limit is {limit}")]
    FrameTooLong {
        /// The announced number of samples.
        announced: u32,
        /// The configured limit.
        limit: u32,
    },

    /// The frame holds a field whose value is not allowed.
    #[error("malformed ALAC frame: {0}")]
    Malformed(String),
}

impl AlacError {
    /// Returns `true` when the error means "the input ended too early".
    pub fn is_truncated(&self) -> bool {
        matches!(self, AlacError::Truncated { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_a_truncated_frame() {
        assert!(
            AlacError::Truncated {
                needed: 8,
                available: 0
            }
            .is_truncated()
        );
        assert!(!AlacError::Malformed("x".into()).is_truncated());
    }

    #[test]
    fn names_the_numbers() {
        let text = AlacError::FrameTooLong {
            announced: 99_999,
            limit: 4096,
        }
        .to_string();
        assert!(text.contains("99999"), "{text}");
        assert!(text.contains("4096"), "{text}");
    }
}
