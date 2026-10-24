// WHAT: a minimal base64 codec for the `<data>` element of XML plists.
// WHY: the format crates depend on nothing but the standard library, so
//      the codec is written here instead of pulling in a crate. Only the
//      standard alphabet is supported, which is what Apple's encoder
//      emits.
// EXAMPLE: `decode("aGk=")` yields the bytes `b"hi"`.

use crate::error::PlistError;

/// The standard base64 alphabet.
const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Decodes a base64 string, ignoring ASCII whitespace.
///
/// # Errors
///
/// Returns [`PlistError::Xml`] when the input contains a character that
/// is neither whitespace nor part of the standard alphabet, or when the
/// padding is malformed.
pub fn decode(input: &str) -> Result<Vec<u8>, PlistError> {
    let mut out = Vec::with_capacity(input.len() / 4 * 3);
    let mut accumulator: u32 = 0;
    let mut bits: u32 = 0;
    let mut padding = 0_usize;

    for byte in input.bytes() {
        if byte.is_ascii_whitespace() {
            continue;
        }
        if byte == b'=' {
            padding += 1;
            if padding > 2 {
                return Err(PlistError::Xml("base64 padding is too long".into()));
            }
            continue;
        }
        if padding > 0 {
            return Err(PlistError::Xml(
                "base64 data continues after the padding".into(),
            ));
        }
        let value = ALPHABET
            .iter()
            .position(|candidate| *candidate == byte)
            .ok_or_else(|| PlistError::Xml(format!("invalid base64 character {byte:#04x}")))?;
        accumulator = (accumulator << 6) | value as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((accumulator >> bits) & 0xff) as u8);
        }
    }

    if bits >= 6 {
        return Err(PlistError::Xml(
            "base64 data has a dangling group of bits".into(),
        ));
    }
    Ok(out)
}

/// Encodes bytes into a padded base64 string.
pub fn encode(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(ALPHABET[((triple >> 18) & 0x3f) as usize] as char);
        out.push(ALPHABET[((triple >> 12) & 0x3f) as usize] as char);
        if chunk.len() > 1 {
            out.push(ALPHABET[((triple >> 6) & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(ALPHABET[(triple & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    #[test]
    fn decodes_the_rfc4648_examples() {
        assert_eq!(decode("").unwrap_or_default(), Vec::<u8>::new());
        assert_eq!(decode("Zg==").unwrap_or_default(), b"f".to_vec());
        assert_eq!(decode("Zm8=").unwrap_or_default(), b"fo".to_vec());
        assert_eq!(decode("Zm9v").unwrap_or_default(), b"foo".to_vec());
        assert_eq!(decode("Zm9vYmFy").unwrap_or_default(), b"foobar".to_vec());
    }

    #[test]
    fn ignores_whitespace() {
        assert_eq!(
            decode("Zm9v\n YmFy").unwrap_or_default(),
            b"foobar".to_vec()
        );
    }

    #[test]
    fn rejects_invalid_characters() {
        assert!(decode("Zm9v!").is_err());
    }

    #[test]
    fn rejects_data_after_padding() {
        assert!(decode("Zg==Zg==").is_err());
    }

    #[test]
    fn rejects_a_dangling_group() {
        assert!(decode("Zm9vY").is_err());
    }

    #[test]
    fn round_trips_arbitrary_bytes() {
        for len in 0..32_usize {
            let data: Vec<u8> = (0..len).map(|index| (index * 7 + 3) as u8).collect();
            let text = encode(&data);
            assert_eq!(decode(&text).unwrap_or_default(), data, "length {len}");
        }
    }
}
