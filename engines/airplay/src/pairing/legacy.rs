//! The legacy AirPlay PIN pairing (SRP-6a).
//!
//! WHY: the AirPlay 2 access control uses a four-digit code that the user
//! types on the sending device and reads on the receiver. The exchange is the
//! Apple variant of SRP-6a with the 2048-bit group of RFC 5054, generator 2
//! and SHA-1, carried inside property lists:
//!
//! | Step | Request | Answer |
//! |---|---|---|
//! | 1 | `{method: "pin", user: <id>}` | `{pk: B, salt}` |
//! | 2 | `{pk: A, proof: M1}` | `{proof: M2}` |
//! | 3 | `{epk, authTag}` | `{epk, authTag}` |
//!
//! Step 2 proves that the user knows the PIN; step 3 exchanges the Ed25519
//! public keys under an AES-128-GCM key derived from the SRP session key.
//!
//! SECURITY: a four-digit code is low entropy. The defence is the *lock-out*
//! on the number of failed attempts (see [`crate::pairing::store`]), not the
//! length of the code.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_airplay::pairing::legacy::LegacyPinPairing;
//! use phairplay_airplay::plist::PlistValue;
//!
//! let mut pairing = LegacyPinPairing::new("1234", [0u8; 32]);
//! let request = PlistValue::dictionary(vec![
//!     ("method".into(), PlistValue::String("pin".into())),
//!     ("user".into(), PlistValue::String("mac-1".into())),
//! ]);
//! let reply = pairing.handle(&request).expect("step 1");
//! assert!(reply.values.iter().any(|(key, _)| key == "pk"));
//! ```

use aes_gcm::aead::consts::U16;
use aes_gcm::aead::{AeadInPlace, KeyInit};
use aes_gcm::{AesGcm, aes::Aes128};
use rand::RngCore;
use rsa::BigUint;
use sha1::{Digest, Sha1};
use sha2::Sha512;

use crate::error::AirPlayError;
use crate::plist::PlistValue;

/// The size of the SRP group in bytes.
const GROUP_BYTES: usize = 256;

/// The size of the salt.
const SALT_BYTES: usize = 16;

/// The RFC 5054 2048-bit group prime.
const GROUP_PRIME: &str = "\
AC6BDB41324A9A9BF166DE5E1389582FAF72B6651987EE07FC3192943DB56050\
A37329CBB4A099ED8193E0757767A13DD52312AB4B03310DCD7F48A9DA04FD50\
E8083969EDB767B0CF6095179A163AB3661A05FBD5FAAAE82918A9962F0B93B8\
55F97993EC975EEAA80D740ADBF4FF747359D041D5C33EA71D281E446B14773B\
CA97B43A23FB801676BD207A436C6481F1D2B9078717461A5B9D32E688F87748\
544523B524B0D57D5EA77A2775D2ECFA032CFBDBF52FB3786160279004E57AE6\
AF874E7303CE53299CCC041C7BC308D82A5698F3A8D0C38271AE35F8E9DBFBB6\
94B5C803D89F7AE435DE236D525F54759B65E372FCD68EF20FA7111F9E4AFF73";

/// The answer of one step of the PIN exchange.
#[derive(Debug, Clone, PartialEq)]
pub struct PinReply {
    /// The values the answer carries.
    pub values: Vec<(String, PlistValue)>,
    /// Whether the pairing is complete after this step.
    pub complete: bool,
}

impl PinReply {
    /// Builds a reply from its values.
    fn new(values: Vec<(&str, PlistValue)>, complete: bool) -> Self {
        Self {
            values: values
                .into_iter()
                .map(|(key, value)| (key.to_string(), value))
                .collect(),
            complete,
        }
    }
}

/// The state of the first step of the exchange.
#[derive(Debug)]
struct StepOne {
    username: String,
    salt: Vec<u8>,
    server_private: BigUint,
    server_public: BigUint,
    server_public_bytes: Vec<u8>,
    verifier: BigUint,
}

/// The server side of the legacy PIN pairing.
pub struct LegacyPinPairing {
    pin: String,
    accessory_public: [u8; 32],
    step_one: Option<StepOne>,
    session_key: Vec<u8>,
}

impl std::fmt::Debug for LegacyPinPairing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LegacyPinPairing")
            .field("step", &self.step_one.is_some())
            .field("session_key", &self.session_key.is_empty())
            .finish()
    }
}

impl LegacyPinPairing {
    /// Creates a pairing session for the given PIN and receiver public key.
    #[must_use]
    pub fn new(pin: &str, accessory_public: [u8; 32]) -> Self {
        Self {
            pin: pin.to_string(),
            accessory_public,
            step_one: None,
            session_key: Vec::new(),
        }
    }

    /// Handles one request of the exchange.
    ///
    /// # Errors
    /// Returns [`AirPlayError::Parse`] when the request does not match any
    /// step of the exchange and [`AirPlayError::Crypto`] when the client
    /// proves the wrong PIN.
    pub fn handle(&mut self, request: &PlistValue) -> Result<PinReply, AirPlayError> {
        let has = |key: &str| request.get(key).is_some();
        if has("epk") && has("authTag") {
            self.step_three(request)
        } else if has("pk") && has("proof") {
            self.step_two(request)
        } else if has("method") || has("user") {
            self.step_one(request)
        } else {
            Err(AirPlayError::parse(
                "pair-setup-pin request matches no step of the exchange",
            ))
        }
    }

    /// Step 1: build the verifier and return the salt and the server's key.
    fn step_one(&mut self, request: &PlistValue) -> Result<PinReply, AirPlayError> {
        let username = request
            .get("user")
            .and_then(PlistValue::as_str)
            .unwrap_or_default()
            .to_string();

        let mut salt = vec![0u8; SALT_BYTES];
        rand::rngs::OsRng.fill_bytes(&mut salt);
        if salt[0] == 0 {
            salt[0] = 1;
        }

        let inner = sha1(&[format!("{}:{}", username, self.pin).as_bytes()]);
        let x = BigUint::from_bytes_be(&sha1(&[&salt, &inner]));
        let verifier = BigUint::from(2u8).modpow(&x, &group());

        let server_private = random_private();
        let k = BigUint::from_bytes_be(&sha1(&[&pad(&group()), &pad(&BigUint::from(2u8))]));
        let server_public =
            (&k * &verifier + BigUint::from(2u8).modpow(&server_private, &group())) % group();
        let server_public_bytes = to_bytes(&server_public);

        self.step_one = Some(StepOne {
            username,
            salt: salt.clone(),
            server_private,
            server_public,
            server_public_bytes: server_public_bytes.clone(),
            verifier,
        });

        Ok(PinReply::new(
            vec![
                ("pk", PlistValue::Data(server_public_bytes)),
                ("salt", PlistValue::Data(salt)),
            ],
            false,
        ))
    }

    /// Step 2: verify the proof of the client and answer with the proof.
    fn step_two(&mut self, request: &PlistValue) -> Result<PinReply, AirPlayError> {
        let state = self.step_one.as_ref().ok_or_else(|| {
            AirPlayError::invalid_state("pair-setup-pin step 2 arrived before step 1")
        })?;
        let client_public = request
            .get("pk")
            .and_then(PlistValue::as_data)
            .ok_or_else(|| AirPlayError::parse("pair-setup-pin step 2 without a client key"))?;
        let client_proof = request
            .get("proof")
            .and_then(PlistValue::as_data)
            .ok_or_else(|| AirPlayError::parse("pair-setup-pin step 2 without a proof"))?;

        let a = BigUint::from_bytes_be(client_public);
        if a == BigUint::from(0u8) || a >= group() {
            return Err(AirPlayError::crypto(
                "pair-setup-pin client key is out of range",
            ));
        }
        let a_bytes = to_bytes(&a);

        let u = BigUint::from_bytes_be(&sha1(&[&pad(&a), &pad(&state.server_public)]));
        let shared = ((&a * state.verifier.modpow(&u, &group())) % group())
            .modpow(&state.server_private, &group());
        let session_key = session_key(&to_bytes(&shared));

        let expected = sha1(&[
            &h_n_xor_h_g(),
            &sha1(&[state.username.as_bytes()]),
            &state.salt,
            &a_bytes,
            &state.server_public_bytes,
            &session_key,
        ]);
        if !constant_time_eq(&expected, client_proof) {
            return Err(AirPlayError::crypto("pair-setup-pin proof does not match"));
        }

        self.session_key = session_key.clone();
        let m2 = sha1(&[&a_bytes, client_proof, &session_key]);
        Ok(PinReply::new(vec![("proof", PlistValue::Data(m2))], false))
    }

    /// Step 3: exchange the Ed25519 public keys under the session key.
    fn step_three(&mut self, request: &PlistValue) -> Result<PinReply, AirPlayError> {
        if self.session_key.is_empty() {
            return Err(AirPlayError::invalid_state(
                "pair-setup-pin step 3 arrived before step 2",
            ));
        }
        let encrypted = request
            .get("epk")
            .and_then(PlistValue::as_data)
            .ok_or_else(|| AirPlayError::parse("pair-setup-pin step 3 without a key"))?;
        let tag = request
            .get("authTag")
            .and_then(PlistValue::as_data)
            .ok_or_else(|| AirPlayError::parse("pair-setup-pin step 3 without a tag"))?;
        if encrypted.is_empty() || tag.len() != 16 {
            return Err(AirPlayError::parse(
                "pair-setup-pin step 3 has a malformed payload",
            ));
        }

        let (key, base_iv) = derive_step_three_material(&self.session_key);
        // The client encrypts with the first counter value.
        let mut iv = base_iv;
        increment_counter(&mut iv);
        let mut buffer = encrypted.to_vec();
        let cipher = AesGcm::<Aes128, U16>::new_from_slice(&key)
            .map_err(|_| AirPlayError::crypto("pair-setup-pin key is malformed"))?;
        let nonce = aes_gcm::Nonce::<U16>::from_slice(&iv);
        let tag_bytes = aes_gcm::Tag::<U16>::from_slice(tag);
        cipher
            .decrypt_in_place_detached(nonce, &[], &mut buffer, tag_bytes)
            .map_err(|_| AirPlayError::crypto("pair-setup-pin client key is not authentic"))?;

        // The server answers with the next counter value of the same key.
        let mut reply_iv = base_iv;
        increment_counter(&mut reply_iv);
        increment_counter(&mut reply_iv);
        let mut reply = self.accessory_public.to_vec();
        let cipher = AesGcm::<Aes128, U16>::new_from_slice(&key)
            .map_err(|_| AirPlayError::crypto("pair-setup-pin key is malformed"))?;
        let nonce = aes_gcm::Nonce::<U16>::from_slice(&reply_iv);
        let tag = cipher
            .encrypt_in_place_detached(nonce, &[], &mut reply)
            .map_err(|_| AirPlayError::crypto("pair-setup-pin encryption failed"))?;
        let tag = tag.to_vec();

        Ok(PinReply::new(
            vec![
                ("epk", PlistValue::Data(reply)),
                ("authTag", PlistValue::Data(tag)),
            ],
            true,
        ))
    }
}

/// The SRP group prime.
fn group() -> BigUint {
    BigUint::parse_bytes(GROUP_PRIME.as_bytes(), 16).unwrap_or_else(|| BigUint::from(0u8))
}

/// A random private exponent.
fn random_private() -> BigUint {
    loop {
        let mut bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut bytes);
        let candidate = BigUint::from_bytes_be(&bytes);
        if candidate != BigUint::from(0u8) && candidate < group() {
            return candidate;
        }
    }
}

/// The minimal big-endian representation of a number.
fn to_bytes(value: &BigUint) -> Vec<u8> {
    value.to_bytes_be()
}

/// The left-padded representation of a number, padded to the group size.
fn pad(value: &BigUint) -> Vec<u8> {
    let raw = to_bytes(value);
    if raw.len() >= GROUP_BYTES {
        return raw;
    }
    let mut padded = vec![0u8; GROUP_BYTES];
    padded[GROUP_BYTES - raw.len()..].copy_from_slice(&raw);
    padded
}

/// The SHA-1 of the concatenation of its parts.
fn sha1(parts: &[&[u8]]) -> Vec<u8> {
    let mut hasher = Sha1::new();
    for part in parts {
        hasher.update(part);
    }
    hasher.finalize().to_vec()
}

/// `H(N) XOR H(g)`.
fn h_n_xor_h_g() -> Vec<u8> {
    let h_n = sha1(&[&to_bytes(&group())]);
    let h_g = sha1(&[&[0x02u8]]);
    h_n.iter().zip(h_g.iter()).map(|(a, b)| a ^ b).collect()
}

/// The Apple session key: `H(S | 0x00000000) ‖ H(S | 0x00000001)`.
fn session_key(shared: &[u8]) -> Vec<u8> {
    let mut key = sha1(&[shared, &[0, 0, 0, 0]]);
    key.extend_from_slice(&sha1(&[shared, &[0, 0, 0, 1]]));
    key
}

/// The AES key and the initialisation vector of the third step.
fn derive_step_three_material(session_key: &[u8]) -> ([u8; 16], [u8; 16]) {
    let mut key = [0u8; 16];
    let mut iv = [0u8; 16];
    let mut hasher = Sha512::new();
    hasher.update(b"Pair-Setup-AES-Key");
    hasher.update(session_key);
    key.copy_from_slice(&hasher.finalize()[..16]);
    let mut hasher = Sha512::new();
    hasher.update(b"Pair-Setup-AES-IV");
    hasher.update(session_key);
    iv.copy_from_slice(&hasher.finalize()[..16]);
    (key, iv)
}

/// Increments a 16-byte counter, big-endian, with carry.
fn increment_counter(counter: &mut [u8; 16]) {
    for index in (0..16).rev() {
        counter[index] = counter[index].wrapping_add(1);
        if counter[index] != 0 {
            break;
        }
    }
}

/// Compares two byte strings without an early exit.
fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut difference = 0u8;
    for (a, b) in left.iter().zip(right.iter()) {
        difference |= a ^ b;
    }
    difference == 0
}

#[cfg(test)]
mod tests;
