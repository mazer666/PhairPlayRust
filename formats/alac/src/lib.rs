//! Apple Lossless (ALAC) decoder for PhairPlayRust.
//!
//! WHAT: this crate decodes the Apple Lossless codec: it reads the
//!      magic cookie of a stream, walks the elements of a frame, decodes
//!      the adaptive Golomb coded residuals, undoes the dynamic
//!      prediction and the channel mixing, and hands the samples back.
//!
//! WHY: an AirPlay sender may deliver lossless audio, and the receiver
//!      has to turn those frames into PCM. The decoder is a port of the
//!      reference decoder by Apple, which is published under the Apache
//!      License 2.0. The port is a native Rust implementation: it uses
//!      slices instead of raw pointers, it returns errors instead of
//!      reading past the end of a buffer, and it needs no `unsafe`.
//!
//! EXAMPLE:
//! ```
//! use phairplay_alac::{AlacConfig, AlacDecoder};
//!
//! let config = AlacConfig {
//!     frame_length: 4,
//!     bit_depth: 16,
//!     num_channels: 1,
//!     ..AlacConfig::default()
//! };
//! let mut decoder = AlacDecoder::new(config).unwrap_or_else(|err| panic!("{err}"));
//! assert_eq!(decoder.config().bit_depth, 16);
//! ```
//!
//! # Attribution
//!
//! The decoder is derived from the reference implementation by Apple
//! Inc. (`ALACDecoder.cpp`, `ALACBitUtilities.c`, `ag_dec.c`,
//! `dp_dec.c` and `matrix_dec.c`), Copyright (c) 2001-2011 Apple Inc.,
//! licensed under the Apache License, Version 2.0. The port was written
//! for PhairPlayRust; no C code was copied. The reference implementation
//! is listed in `THIRD_PARTY.md`.

#![warn(missing_docs)]

mod bits;
mod config;
mod decoder;
mod elements;
mod error;
mod framing;
mod golomb;
mod matrix;
mod predictor;

pub use bits::BitReader;
pub use config::{
    ALAC_ATOM_SIZE, ALAC_MAX_BIT_DEPTH, ALAC_MAX_CHANNELS, ALAC_SPECIFIC_CONFIG_SIZE, ALAC_VERSION,
    AlacConfig,
};
pub use decoder::{AlacDecoder, DecodedAudio};
pub use error::AlacError;
pub use golomb::AgParams;
pub use matrix::{MixedChannels, pack_samples, unmix};
pub use predictor::{NUM_COEF_PAIRS, unpc_block, unpc_block_in_place};
