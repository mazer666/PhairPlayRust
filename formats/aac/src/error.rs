// WHAT: the error type of the AAC codec.
// WHY: the AAC framing comes from the network, and a broken frame has to
//      be told apart from a broken configuration. One enum with a
//      `reason` string keeps the error surface small.
// EXAMPLE: a frame whose length field announces more bytes than the
//      buffer holds produces `AacError::Truncated`.

use thiserror::Error;

/// Everything that can go wrong while parsing AAC framing.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AacError {
    /// The buffer ends before the header is complete.
    #[error("the AAC frame is truncated after {available} bytes, but {needed} were needed")]
    Truncated {
        /// The number of bytes the parser needed.
        needed: usize,
        /// The number of bytes the buffer holds.
        available: usize,
    },

    /// The frame does not start with the expected sync word.
    #[error("the AAC frame does not start with the sync word 0x{syncword:04x}")]
    BadSyncWord {
        /// The sync word that was found.
        syncword: u16,
    },

    /// The frame announces a sample rate that the format does not define.
    #[error("the sampling frequency index {index} is not defined")]
    UnknownSampleRate {
        /// The index that was found.
        index: u8,
    },

    /// The frame announces a channel configuration that the format does
    /// not define.
    #[error("the channel configuration {configuration} is not defined")]
    UnknownChannelConfiguration {
        /// The configuration that was found.
        configuration: u8,
    },

    /// The frame announces a length that does not fit the buffer.
    #[error("the frame announces {announced} bytes, but only {available} are left")]
    LengthMismatch {
        /// The announced length.
        announced: usize,
        /// The number of bytes that are left.
        available: usize,
    },

    /// The frame is otherwise malformed.
    #[error("malformed AAC frame: {0}")]
    Malformed(String),
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    #[test]
    fn names_the_numbers() {
        let text = AacError::LengthMismatch {
            announced: 900,
            available: 12,
        }
        .to_string();
        assert!(text.contains("900"), "{text}");
        assert!(text.contains("12"), "{text}");
    }

    #[test]
    fn names_the_sync_word() {
        let text = AacError::BadSyncWord { syncword: 0x1234 }.to_string();
        assert!(text.contains("1234"), "{text}");
    }
}
