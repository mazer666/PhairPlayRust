//! The AirPlay pair-setup and pair-verify exchange.
//!
//! WHY: an AirPlay sender proves that it may stream in two steps. In
//! *pair-setup* it learns the receiver's Ed25519 public key; in *pair-verify*
//! both sides perform an ephemeral X25519 key agreement, sign the transcript
//! with their long-term Ed25519 keys and exchange the signatures encrypted
//! with the derived AES-128-CTR keystream. The result is a shared secret the
//! receiver uses to derive the per-stream encryption keys.
//!
//! The exchange is a *raw binary* one — the AirPlay variant of the HomeKit
//! pairing — and it is not the HomeKit TLV8 encoding.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_airplay::pairing::identity::PairingIdentity;
//! use phairplay_airplay::pairing::verify::PairingSession;
//!
//! let identity = PairingIdentity::from_seed(&[3u8; 32]);
//! let mut session = PairingSession::new(std::sync::Arc::new(identity));
//! let public_key = session.pair_setup(&[0u8; 32]).expect("pair setup");
//! assert_eq!(public_key.len(), 32);
//! ```

use std::sync::Arc;

use aes::Aes128;
use ctr::cipher::{KeyIvInit, StreamCipher};
use ctr::Ctr128BE;
use sha2::{Digest, Sha512};
use x25519_dalek::{PublicKey, StaticSecret};

use crate::error::AirPlayError;
use crate::pairing::identity::PairingIdentity;

/// The salt of the AES key of the pair-verify cipher.
const SALT_KEY: &[u8] = b"Pair-Verify-AES-Key";

/// The salt of the AES initialisation vector of the pair-verify cipher.
const SALT_IV: &[u8] = b"Pair-Verify-AES-IV";

/// The size of the X25519 public key.
pub const X25519_KEY_BYTES: usize = 32;

/// The size of the Ed25519 public key.
pub const ED25519_KEY_BYTES: usize = 32;

/// The size of an Ed25519 signature.
pub const SIGNATURE_BYTES: usize = 64;

/// The state of a running pair-verify exchange.
struct VerifyState {
    our_public: [u8; X25519_KEY_BYTES],
    their_public: [u8; X25519_KEY_BYTES],
    their_ed_public: [u8; ED25519_KEY_BYTES],
    cipher: Ctr128BE<Aes128>,
    shared_secret: [u8; 32],
}

/// The pairing state of one control connection.
pub struct PairingSession {
    identity: Arc<PairingIdentity>,
    state: Option<VerifyState>,
}

impl std::fmt::Debug for PairingSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PairingSession")
            .field("verified", &self.is_verified())
            .finish()
    }
}

impl PairingSession {
    /// Creates a session for the given receiver identity.
    #[must_use]
    pub fn new(identity: Arc<PairingIdentity>) -> Self {
        Self {
            identity,
            state: None,
        }
    }

    /// Handles the `POST /pair-setup` request.
    ///
    /// The request body is the sender's 32-byte Ed25519 public key; the
    /// answer is the receiver's own public key.
    ///
    /// # Errors
    /// Returns [`AirPlayError::Parse`] when the body has the wrong size.
    pub fn pair_setup(&self, body: &[u8]) -> Result<Vec<u8>, AirPlayError> {
        if body.len() != ED25519_KEY_BYTES {
            return Err(AirPlayError::parse(format!(
                "pair-setup expects {ED25519_KEY_BYTES} bytes, got {}",
                body.len()
            )));
        }
        Ok(self.identity.public_key().to_vec())
    }

    /// Handles the `POST /pair-verify` request.
    ///
    /// The leading state byte selects the step: `1` is the first message,
    /// `0` the second.
    ///
    /// # Errors
    /// Returns [`AirPlayError::Parse`] when the body is malformed and
    /// [`AirPlayError::Crypto`] when the sender's signature does not verify.
    pub fn pair_verify(&mut self, body: &[u8]) -> Result<Vec<u8>, AirPlayError> {
        if body.len() < 4 {
            return Err(AirPlayError::parse(format!(
                "pair-verify body is too short: {} bytes",
                body.len()
            )));
        }
        match body[0] {
            1 => self.verify_first_message(body),
            0 => self.verify_second_message(body),
            other => Err(AirPlayError::parse(format!(
                "unknown pair-verify state {other}"
            ))),
        }
    }

    /// The Ed25519 public key of the receiver.
    #[must_use]
    pub fn public_key(&self) -> [u8; ED25519_KEY_BYTES] {
        self.identity.public_key()
    }

    /// The shared secret, once the exchange has completed.
    #[must_use]
    pub fn shared_secret(&self) -> Option<[u8; 32]> {
        self.state.as_ref().map(|state| state.shared_secret)
    }

    /// Whether the exchange completed and the sender proved its identity.
    #[must_use]
    pub fn is_verified(&self) -> bool {
        self.state.is_some() && self.shared_secret().is_some()
    }

    fn verify_first_message(&mut self, body: &[u8]) -> Result<Vec<u8>, AirPlayError> {
        if body.len() != 4 + X25519_KEY_BYTES + ED25519_KEY_BYTES {
            return Err(AirPlayError::parse(format!(
                "pair-verify message 1 has the wrong size: {} bytes",
                body.len()
            )));
        }
        let mut their_public = [0u8; X25519_KEY_BYTES];
        their_public.copy_from_slice(&body[4..4 + X25519_KEY_BYTES]);
        let mut their_ed_public = [0u8; ED25519_KEY_BYTES];
        their_ed_public.copy_from_slice(&body[4 + X25519_KEY_BYTES..]);

        let secret = StaticSecret::random_from_rng(rand::rngs::OsRng);
        let our_public = PublicKey::from(&secret);
        let shared = secret.diffie_hellman(&PublicKey::from(their_public));
        let mut shared_secret = [0u8; 32];
        shared_secret.copy_from_slice(shared.as_bytes());

        let mut transcript = [0u8; 64];
        transcript[..32].copy_from_slice(our_public.as_bytes());
        transcript[32..].copy_from_slice(&their_public);
        let signature = self.identity.sign(&transcript);

        let (key, iv) = derive_cipher_material(&shared_secret);
        let mut cipher = Ctr128BE::<Aes128>::new(&key.into(), &iv.into());
        let mut encrypted = signature.to_vec();
        cipher.apply_keystream(&mut encrypted);

        let mut response = Vec::with_capacity(96);
        response.extend_from_slice(our_public.as_bytes());
        response.extend_from_slice(&encrypted);

        self.state = Some(VerifyState {
            our_public: our_public.to_bytes(),
            their_public,
            their_ed_public,
            cipher,
            shared_secret,
        });
        Ok(response)
    }

    fn verify_second_message(&mut self, body: &[u8]) -> Result<Vec<u8>, AirPlayError> {
        if body.len() != 4 + SIGNATURE_BYTES {
            return Err(AirPlayError::parse(format!(
                "pair-verify message 2 has the wrong size: {} bytes",
                body.len()
            )));
        }
        let Some(state) = self.state.as_mut() else {
            return Err(AirPlayError::invalid_state(
                "pair-verify message 2 arrived before message 1",
            ));
        };
        let mut decrypted = body[4..].to_vec();
        state.cipher.apply_keystream(&mut decrypted);

        let mut transcript = [0u8; 64];
        transcript[..32].copy_from_slice(&state.their_public);
        transcript[32..].copy_from_slice(&state.our_public);
        if !PairingIdentity::verify(&state.their_ed_public, &transcript, &decrypted) {
            return Err(AirPlayError::crypto(
                "pair-verify signature does not verify",
            ));
        }
        Ok(Vec::new())
    }
}

/// Derives the AES key and the initialisation vector of the pair-verify cipher.
fn derive_cipher_material(shared_secret: &[u8; 32]) -> ([u8; 16], [u8; 16]) {
    let mut key = [0u8; 16];
    let mut iv = [0u8; 16];
    let mut hasher = Sha512::new();
    hasher.update(SALT_KEY);
    hasher.update(shared_secret);
    key.copy_from_slice(&hasher.finalize()[..16]);
    let mut hasher = Sha512::new();
    hasher.update(SALT_IV);
    hasher.update(shared_secret);
    iv.copy_from_slice(&hasher.finalize()[..16]);
    (key, iv)
}

#[cfg(test)]
mod tests;
