//! The key derivation of the mirrored media streams.
//!
//! WHY: the mirroring path encrypts its media with a key that both sides
//! derive from three inputs: the stream key the sender hands over, the shared
//! secret of the pairing exchange, and the identifier of the stream. The
//! derivation is a chain of SHA-512 hashes, and it must match the sender
//! byte for byte — a single byte off produces a stream that decrypts into
//! noise.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_airplay::mirror::stream_key_material;
//!
//! let (key, iv) = stream_key_material(&[1u8; 16], &[2u8; 32], 42);
//! assert_eq!(key.len(), 16);
//! assert_eq!(iv.len(), 16);
//! assert_ne!(key, iv);
//! ```

use sha2::{Digest, Sha512};

/// The length of a derived key.
pub const KEY_BYTES: usize = 16;

/// Derives the key and the initialisation vector of a mirrored stream.
///
/// * `eaeskey` = `SHA-512(aesKey ‖ ecdhSecret)[:16]`
/// * `key` = `SHA-512("AirPlayStreamKey" ‖ id ‖ eaeskey)[:16]`
/// * `iv` = `SHA-512("AirPlayStreamIV" ‖ id ‖ eaeskey)[:16]`
#[must_use]
pub fn stream_key_material(
    aes_key: &[u8],
    ecdh_secret: &[u8],
    stream_connection_id: u64,
) -> ([u8; KEY_BYTES], [u8; KEY_BYTES]) {
    let eaeskey = audio_key(aes_key, ecdh_secret);
    let id = stream_connection_id.to_string();
    let key = derive(b"AirPlayStreamKey", id.as_bytes(), &eaeskey);
    let iv = derive(b"AirPlayStreamIV", id.as_bytes(), &eaeskey);
    (key, iv)
}

/// Derives the key of the mirrored audio stream.
///
/// `SHA-512(aesKey ‖ ecdhSecret)[:16]`. The initialisation vector of the
/// audio stream is the raw `eiv` of the `SETUP` request.
#[must_use]
pub fn audio_key(aes_key: &[u8], ecdh_secret: &[u8]) -> [u8; KEY_BYTES] {
    let mut hasher = Sha512::new();
    hasher.update(aes_key);
    hasher.update(ecdh_secret);
    let digest = hasher.finalize();
    let mut key = [0u8; KEY_BYTES];
    key.copy_from_slice(&digest[..KEY_BYTES]);
    key
}

/// Hashes a label, the stream identifier and the stream key.
fn derive(label: &[u8], id: &[u8], stream_key: &[u8]) -> [u8; KEY_BYTES] {
    let mut hasher = Sha512::new();
    hasher.update(label);
    hasher.update(id);
    hasher.update(stream_key);
    let digest = hasher.finalize();
    let mut key = [0u8; KEY_BYTES];
    key.copy_from_slice(&digest[..KEY_BYTES]);
    key
}

/// Converts AVCC NAL units into the Annex-B form a decoder expects.
///
/// The decrypted mirror payload carries the NAL units with a four-byte
/// big-endian length prefix; the decoders want them separated by start
/// codes.
#[must_use]
pub fn avcc_to_annex_b(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() + 16);
    let mut position = 0usize;
    while position + 4 <= data.len() {
        let length = u32::from_be_bytes([
            data[position],
            data[position + 1],
            data[position + 2],
            data[position + 3],
        ]) as usize;
        position += 4;
        if length == 0 || position + length > data.len() {
            break;
        }
        out.extend_from_slice(&[0, 0, 0, 1]);
        out.extend_from_slice(&data[position..position + length]);
        position += length;
    }
    out
}

#[cfg(test)]
mod tests;
