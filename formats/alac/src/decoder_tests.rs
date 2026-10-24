#![allow(clippy::expect_used, clippy::unwrap_used)]
use super::*;
use crate::config::AlacConfig;

/// A writer that builds a bit stream.
struct BitWriter {
    bytes: Vec<u8>,
    filled: u8,
}

impl BitWriter {
    fn new() -> BitWriter {
        BitWriter {
            bytes: Vec::new(),
            filled: 0,
        }
    }

    fn bit(&mut self, value: bool) {
        if self.filled == 0 {
            self.bytes.push(0);
        }
        let last = self.bytes.len() - 1;
        if value {
            self.bytes[last] |= 1 << (7 - self.filled);
        }
        self.filled = (self.filled + 1) % 8;
    }

    fn bits(&mut self, value: u32, count: u32) {
        for shift in (0..count).rev() {
            self.bit((value >> shift) & 1 == 1);
        }
    }

    fn finish(mut self) -> Vec<u8> {
        if self.filled != 0 {
            let last = self.bytes.len() - 1;
            self.bytes[last] &= !(0xff >> self.filled);
        }
        self.bytes
    }
}

/// Builds a stereo configuration for the tests.
fn stereo_config() -> AlacConfig {
    AlacConfig {
        num_channels: 2,
        ..config()
    }
}

/// Builds a configuration for the tests.
fn config() -> AlacConfig {
    AlacConfig {
        frame_length: 4,
        compatible_version: 0,
        bit_depth: 16,
        pb: 0,
        mb: 128,
        kb: 1,
        num_channels: 1,
        max_run: 255,
        max_frame_bytes: 1024,
        avg_bit_rate: 0,
        sample_rate: 44_100,
    }
}

/// Builds an uncompressed mono frame that holds the given samples.
fn escape_frame(samples: &[i16]) -> Vec<u8> {
    let mut writer = BitWriter::new();
    writer.bits(ID_SCE as u32, 3);
    writer.bits(0, 4); // element instance tag
    writer.bits(0, 12); // unused header
    writer.bits(0b0001, 4); // partial frame, no shift, escape
    for sample in samples {
        writer.bits(*sample as u16 as u32, 16);
    }
    writer.finish()
}

/// Builds a compressed mono frame that holds the given residuals.
fn escape_stereo_frame(left: &[i16], right: &[i16]) -> Vec<u8> {
    // The two channels are interleaved: the first sample of the left
    // channel, then the first of the right channel, and so on.
    let mut writer = BitWriter::new();
    writer.bits(ID_CPE as u32, 3);
    writer.bits(0, 4); // element instance tag
    writer.bits(0, 12); // unused header
    writer.bits(0b0001, 4); // partial frame, no shift, escape
    for index in 0..left.len() {
        writer.bits(left[index] as u16 as u32, 16);
        writer.bits(right[index] as u16 as u32, 16);
    }
    writer.finish()
}

fn compressed_frame(residuals: &[i32]) -> Vec<u8> {
    let mut writer = BitWriter::new();
    writer.bits(ID_SCE as u32, 3);
    writer.bits(0, 4);
    writer.bits(0, 12);
    writer.bits(0b0000, 4); // no partial frame, no shift, no escape
    writer.bits(0, 8); // mix bits
    writer.bits(0, 8); // mix residue
    writer.bits(0, 8); // mode and denominator shift
    writer.bits(0, 8); // prediction factor and number of coefficients
    for residual in residuals {
        // The coder writes the residual as a run of one bits that
        // ends with a zero bit.
        let ones = match *residual {
            0 => 0,
            value if value > 0 => (value * 2) as u32,
            value => ((-value * 2) - 1) as u32,
        };
        for _ in 0..ones {
            writer.bit(true);
        }
        writer.bit(false);
    }
    writer.finish()
}

#[test]
fn decodes_an_uncompressed_frame() {
    let mut decoder = AlacDecoder::new(config()).unwrap_or_else(|err| panic!("{err}"));
    let frame = escape_frame(&[1, -2, 3, -4]);
    let audio = decoder.decode(&frame).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(audio.samples, vec![1, -2, 3, -4]);
    assert_eq!(audio.frames, 4);
    assert_eq!(audio.channels, 1);
}

#[test]
fn decodes_a_compressed_frame() {
    let mut decoder = AlacDecoder::new(config()).unwrap_or_else(|err| panic!("{err}"));
    let frame = compressed_frame(&[1, -2, 0, 3]);
    let audio = decoder.decode(&frame).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(audio.samples, vec![1, -2, 0, 3]);
}

#[test]
fn decodes_a_stereo_frame() {
    let mut decoder = AlacDecoder::new(stereo_config()).unwrap_or_else(|err| panic!("{err}"));
    // The configuration asks for four frames, so the frame holds four
    // samples per channel.
    let frame = escape_stereo_frame(&[10, 20, 30, 40], &[11, 21, 31, 41]);
    let audio = decoder.decode(&frame).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(audio.channels, 2);
    assert_eq!(audio.frames, 4);
    // The samples are interleaved: left, right, left, right.
    assert_eq!(audio.samples, vec![10, 11, 20, 21, 30, 31, 40, 41]);
    assert_eq!(audio.sample(0, 0), Some(10));
    assert_eq!(audio.sample(0, 1), Some(11));
    assert_eq!(audio.sample(3, 0), Some(40));
    assert_eq!(audio.sample(3, 1), Some(41));
}

#[test]
fn refuses_a_stereo_frame_of_thirty_two_bit_samples() {
    // WHY: a stereo element codes the difference of the two channels,
    //      so it needs one bit more than the width of a sample.
    //      Thirty-two bit samples would need thirty-three bits, which
    //      no field of the format can hold. The frame must be refused
    //      instead of overflowing a shift.
    let config = AlacConfig {
        bit_depth: 32,
        num_channels: 2,
        ..config()
    };
    let mut decoder = AlacDecoder::new(config).unwrap_or_else(|err| panic!("{err}"));
    let frame = escape_stereo_frame(&[1, 2, 3, 4], &[5, 6, 7, 8]);
    let error = match decoder.decode(&frame) {
        Ok(_) => panic!("expected the width check to bite"),
        Err(err) => err,
    };
    assert!(matches!(error, AlacError::Malformed(_)), "{error:?}");
}

#[test]
fn decodes_a_mono_frame_of_thirty_two_bit_samples() {
    // The single channel element needs no extra bit, so a stream of
    // thirty-two bit samples is fine.
    let config = AlacConfig {
        bit_depth: 32,
        ..config()
    };
    let mut decoder = AlacDecoder::new(config).unwrap_or_else(|err| panic!("{err}"));
    let mut writer = BitWriter::new();
    writer.bits(ID_SCE as u32, 3);
    writer.bits(0, 4);
    writer.bits(0, 12);
    writer.bits(0b0001, 4);
    for value in 1..=4_u32 {
        writer.bits(value, 32);
    }
    let frame = writer.finish();
    let audio = decoder.decode(&frame).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(audio.samples, vec![1, 2, 3, 4]);
}

#[test]
fn reports_a_truncated_frame() {
    let mut decoder = AlacDecoder::new(config()).unwrap_or_else(|err| panic!("{err}"));
    assert!(matches!(
        decoder.decode(&[0x00, 0x00]),
        Err(AlacError::Truncated { .. })
    ));
}

#[test]
fn refuses_a_frame_with_a_broken_header() {
    let mut decoder = AlacDecoder::new(config()).unwrap_or_else(|err| panic!("{err}"));
    // The twelve unused header bits are not zero.
    let mut writer = BitWriter::new();
    writer.bits(ID_SCE as u32, 3);
    writer.bits(0, 4);
    writer.bits(1, 12);
    let frame = writer.finish();
    assert!(matches!(
        decoder.decode(&frame),
        Err(AlacError::Malformed(_))
    ));
}

#[test]
fn refuses_an_unknown_element() {
    let mut decoder = AlacDecoder::new(config()).unwrap_or_else(|err| panic!("{err}"));
    let mut writer = BitWriter::new();
    writer.bits(7, 3);
    // The end tag ends the frame without an error.
    let frame = writer.finish();
    let audio = decoder.decode(&frame);
    assert!(audio.is_ok());
}

#[test]
fn refuses_a_frame_that_announces_too_many_samples() {
    let mut decoder = AlacDecoder::new(config()).unwrap_or_else(|err| panic!("{err}"));
    // The partial frame flag makes the frame announce its own length:
    // 0x00010000 samples, far more than the four the configuration
    // allows.
    let mut writer = BitWriter::new();
    writer.bits(ID_SCE as u32, 3);
    writer.bits(0, 4);
    writer.bits(0, 12);
    writer.bits(0b1000, 4);
    writer.bits(0x0001, 16);
    writer.bits(0x0000, 16);
    let frame = writer.finish();
    let error = match decoder.decode(&frame) {
        Ok(_) => panic!("expected the length check to bite"),
        Err(err) => err,
    };
    assert!(matches!(error, AlacError::FrameTooLong { .. }), "{error:?}");
}

#[test]
fn refuses_a_frame_with_a_broken_shift_flag() {
    let mut decoder = AlacDecoder::new(config()).unwrap_or_else(|err| panic!("{err}"));
    // The shift flag of the element is three, which the format does
    // not define.
    let mut writer = BitWriter::new();
    writer.bits(ID_SCE as u32, 3);
    writer.bits(0, 4);
    writer.bits(0, 12);
    writer.bits(0b0111, 4);
    let frame = writer.finish();
    assert!(matches!(
        decoder.decode(&frame),
        Err(AlacError::Malformed(_))
    ));
}

#[test]
fn refuses_a_coupling_element() {
    let mut decoder = AlacDecoder::new(config()).unwrap_or_else(|err| panic!("{err}"));
    let mut writer = BitWriter::new();
    writer.bits(ID_CCE as u32, 3);
    let frame = writer.finish();
    assert!(matches!(
        decoder.decode(&frame),
        Err(AlacError::Unsupported(_))
    ));
}

#[test]
fn packs_the_samples_of_a_frame() {
    let mut decoder = AlacDecoder::new(config()).unwrap_or_else(|err| panic!("{err}"));
    let frame = escape_frame(&[1, -2, 3, -4]);
    let audio = decoder.decode(&frame).unwrap_or_else(|err| panic!("{err}"));
    let bytes = audio.to_pcm_bytes(16).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(bytes.len(), 8);
    assert_eq!(&bytes[0..2], &[1, 0]);
}

// ─── The three findings of the second verification round ───────────────────

#[test]
fn refuses_a_shift_that_is_wider_than_the_sample() {
    // WHY: the shift of the element is subtracted from the width of a
    // sample. A sample of one bit with a shift of one byte would underflow
    // that subtraction — a panic in a debug build and a huge channel width
    // in a release build. The frame has to be refused instead.
    let narrow = AlacConfig {
        bit_depth: 1,
        ..config()
    };
    let mut decoder = AlacDecoder::new(narrow).unwrap_or_else(|err| panic!("{err}"));
    let mut writer = BitWriter::new();
    writer.bits(ID_SCE as u32, 3);
    writer.bits(0, 4);
    writer.bits(0, 12);
    // Partial frame, one byte of shift, no escape.
    writer.bits(0b0010, 4);
    let frame = writer.finish();
    assert!(
        matches!(decoder.decode(&frame), Err(AlacError::Malformed(_))),
        "a shift that does not fit into the sample must be refused"
    );
}

#[test]
fn refuses_a_stereo_shift_that_is_wider_than_the_sample() {
    let narrow = AlacConfig {
        bit_depth: 1,
        ..stereo_config()
    };
    let mut decoder = AlacDecoder::new(narrow).unwrap_or_else(|err| panic!("{err}"));
    let mut writer = BitWriter::new();
    writer.bits(ID_CPE as u32, 3);
    writer.bits(0, 4);
    writer.bits(0, 12);
    // Partial frame, two bytes of shift, no escape.
    writer.bits(0b0100, 4);
    let frame = writer.finish();
    assert!(
        matches!(decoder.decode(&frame), Err(AlacError::Malformed(_))),
        "a stereo shift that does not fit into the sample must be refused"
    );
}

#[test]
fn an_escaped_element_is_not_shifted_with_a_stale_buffer() {
    // WHY: the reference implementation resets the shift in the escape
    // path. Without that, an escaped element would apply the shift of the
    // element before it — the samples would come out wrong, and wrong
    // without any error.
    let mut decoder = AlacDecoder::new(config()).unwrap_or_else(|err| panic!("{err}"));
    // The first element is a 24 bit element with a shift of two bytes; it
    // fills the shift buffer.
    let mut first = BitWriter::new();
    first.bits(ID_SCE as u32, 3);
    first.bits(0, 4);
    first.bits(0, 12);
    // Partial frame, two bytes of shift, no escape.
    first.bits(0b0100, 4);
    first.bits(0, 8); // mix bits
    first.bits(0, 8); // mix residue
    first.bits(0, 8); // mode and denominator shift
    first.bits(0, 8); // prediction factor and number of coefficients
    for _ in 0..4 {
        first.bit(false);
    }
    let first_frame = first.finish();
    let _ = decoder.decode(&first_frame);

    // The second element escapes and carries 16 bit samples. It must come
    // out unchanged.
    let second = escape_frame(&[1, -1, 2, -2]);
    let audio = decoder
        .decode(&second)
        .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(audio.samples, vec![1, -1, 2, -2]);
}

#[test]
fn refuses_a_stereo_element_that_does_not_fit_into_the_mix_buffer() {
    // WHY: the single-channel path walks its buffer; the stereo path used
    // to index it. A buffer that is too short has to be an error, not an
    // index-out-of-bounds panic.
    let mut mix_u = [0_i32; 2];
    let mut mix_v = [0_i32; 2];
    let mut bits = crate::bits::BitReader::new(&[0_u8; 64]);
    let error = crate::framing::read_uncompressed_pair(&mut bits, &mut mix_u, &mut mix_v, 8, 16)
        .expect_err("must fail");
    assert!(matches!(error, AlacError::Malformed(_)), "{error:?}");
}
