//! The legacy AirTunes/RAOP audio key unwrap.
//!
//! WHY: before FairPlay, an AirPlay-1 sender shipped the 16-byte AES-128
//! stream key RSA-encrypted with the public AirPort Express key (OAEP with
//! SHA-1). Every open-source AirPlay receiver recovers it with the matching
//! *private* key — the long-public "AirPort Express" key that shairport-sync,
//! RPiPlay and UxPlay all ship. Modern senders use FairPlay instead, so this
//! path only serves legacy senders; it complements the FairPlay module and
//! covers all three key mechanisms of the protocol.
//!
//! The key is embedded as PKCS#8 DER in hexadecimal form. It is public
//! knowledge, not a secret of this project: it is the same constant that
//! shairport-sync calls `g_airport_key`.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_airplay::pairing::raop_rsa;
//!
//! // A blob that is not a valid ciphertext is rejected, never guessed.
//! assert!(raop_rsa::decrypt_aes_key(&[0u8; 256]).is_none());
//! ```

use std::sync::OnceLock;

use rsa::pkcs8::DecodePrivateKey;
use rsa::{Oaep, RsaPrivateKey};

/// The size of the unwrapped AES-128 key.
const AES_KEY_BYTES: usize = 16;

/// The AirPort Express RSA-2048 private key, PKCS#8 DER in hexadecimal.
const AIRPORT_PRIVATE_KEY_HEX: &str = "308204bf020100300d06092a864886f70d0101010500048204a9308204a50201\
    000282010100e7d744f2a2e2788b6c1f55a08eb70544a8fa7945aa8be6c62ce5\
    f51cbdd4dc6842fe3d1083dd2edec1bfd4252dc02e6f398bdf0e6148ea84855e\
    2e442da6d62664f674a1f304929ade4f6893ef2df6e711a8c77a0d91c9d98082\
    2e50d12922afea40ea9f0e14c0f76938c5f3882fc0323dd9fe55155f51bb5921\
    c201629fd73352d5e2efaabf9ba048d7b813a2b6767f6c3ccf1eb4ce673d037b\
    0d2ea30c5fffeb06f8d08adde409571a9c689fef10728855dd8cfb9a8bef5c89\
    43ef3b5faa15dde698beddf3599603eb3e6f61372bb628f6559f599a78bf5006\
    87aa7f4976c0562d412956f8989e18a6355bd81597825e0fc875343ec7821176\
    25cdbf98447b02030100010282010100e5f00c72f577d604b9a4ce4122aa84b0\
    1743ec995acfcc7f4ab27c0b187f90665be359df1259818deeed79d3b1ef845e\
    4ddddac9a155373b5e270d8e1315001a2e527d54cdf9000a5768bc98d4446b37\
    bbbd00b29dd8b53062133b2a6e77f4ee32505622904da720fb1c12c03996da71\
    3a0506098edbedecf936d0fa9cbd5929abb0eda35799502f9894dcb8fc569a89\
    2d17780324a2b6c3166e346709134b854041b867706b58fef2a0db922b77628b\
    68e69693c7af43bf2a73d0b732377a0ba17b44f051e9bf79849dcb3332571fd8\
    a70933c2d60bdec479934a3daca40bb6f2f37c0a9d07106eadc8b369a03f2f41\
    c880098e8add46240dac68cc5354f36102818100f7e0bf5a1e6718319a8b6209\
    c31714440459f973856613b17ae1508bb3e6316e6b7f462d2f7d64412b84b76b\
    c23f2b0c3562455279b243a9f7316f958007b34c61f768e2d44ed5ff2b272817\
    ec32b3e493929228fae78e774ca0f75ebd69d59202798f116e360c6438b32e1b\
    d8b9dc1e3232f0d30918883cc43ef8dda22c369102818100ef6ffff994f1e564\
    41aa0035fd19a0c8d6f02378c70580d9c48420791df407c591fb6ebfca322c30\
    86dd901fd2fae1aebb64adf6bb79ff8051bebd0cd820ab8987400601a7b2fe93\
    90cacc9acab8ed2bf91d186d8f69643d7efe0f5d56df7577a2d035ea5413fc98\
    d8f3f908da059a379da4b1cc38f15d560a83cc317153c84b02818100d0ebafbc\
    4025ba818c75702334384e8f696f804d7aa0e7764e507bb7d3dfefc7d678c668\
    2d3fad713441beeae724a09ec09bdc3bc0709c9133d489ece2a51add05312749\
    0f9286d173c8a4054dc20a575c7e4c0c9834f4a1de874917a3e400eaf885062d\
    b5cb7e343689e711f75fe783d7e19192fd769cd542bea4b90107ecd10281807f\
    4018dc7dea292da53042386f3105a0778adc6f3de690da2b74c5055983edf574\
    661a2fd7b7de8053ccc0e208f0c8ac626f597d3d99d2ce51a37b39ae4b7e9ef2\
    c075f0bf3d83cacd32da969192c2899235825c07d1cd3259a1906cdcd499cb61\
    3e22c94cb1ea971906609df1b0f48b063f17372034369499b5fdf970ef440d02\
    818100904ee920f944ef5aaf7c9420a00f5e9b48082c0b84e0fbb5dda2a22677\
    dfb7b8488db2bee64c9bdd3cac66fa320e76f71ce2af2272bbbd76cab94e084a\
    0c41d9b0771dc63340c1accf5a89da01b437986f269cf0c216e15ea14a038cda\
    692af0eb6db00e78802b9325204d2d20028a3f8cb13468e80f64188e1046ba1b\
    e458a6";

/// The parsed private key.
fn private_key() -> Option<&'static RsaPrivateKey> {
    static KEY: OnceLock<Option<RsaPrivateKey>> = OnceLock::new();
    KEY.get_or_init(|| {
        let der = decode_hex(AIRPORT_PRIVATE_KEY_HEX)?;
        RsaPrivateKey::from_pkcs8_der(&der).ok()
    })
    .as_ref()
}

/// Decodes a hexadecimal string.
fn decode_hex(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 {
        return None;
    }
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(text.len() / 2);
    for pair in bytes.chunks_exact(2) {
        let high = (pair[0] as char).to_digit(16)?;
        let low = (pair[1] as char).to_digit(16)?;
        out.push(((high << 4) | low) as u8);
    }
    Some(out)
}

/// Unwraps a legacy `rsaaeskey` blob into the 16-byte AES-128 key.
///
/// Returns `None` for a blob that cannot be decrypted or that does not
/// decrypt to a key of the expected length.
#[must_use]
pub fn decrypt_aes_key(blob: &[u8]) -> Option<Vec<u8>> {
    let key = private_key()?;
    let padding = Oaep::new::<sha1::Sha1>();
    let decrypted = key.decrypt(padding, blob).ok()?;
    if decrypted.len() != AES_KEY_BYTES {
        return None;
    }
    Some(decrypted)
}

/// The public half of the embedded key, for the round-trip test.
#[must_use]
pub fn public_key() -> Option<rsa::RsaPublicKey> {
    private_key().map(rsa::RsaPublicKey::from)
}

#[cfg(test)]
mod tests;
