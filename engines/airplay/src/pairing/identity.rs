//! The long-lived identity of the receiver.
//!
//! WHY: the AirPlay pair-setup hands the sender the receiver's Ed25519
//! public key. That key must stay the same across restarts — a sender that
//! sees a new key every time treats the receiver as a new device and starts
//! the pairing dance again. The private seed therefore lives in a store that
//! the platform layer provides; the engine only ever sees the trait.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_airplay::pairing::identity::{IdentityStore, InMemoryIdentityStore, PairingIdentity};
//!
//! let store = InMemoryIdentityStore::new();
//! let identity = PairingIdentity::load_or_create(&store).expect("identity");
//! assert_eq!(identity.public_key().len(), 32);
//! ```

use std::fmt::Debug;
use std::sync::Arc;

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

use crate::error::AirPlayError;

/// The size of an Ed25519 seed, public key and signature.
pub const ED25519_KEY_BYTES: usize = 32;

/// The size of an Ed25519 signature.
pub const ED25519_SIGNATURE_BYTES: usize = 64;

/// A store that keeps the Ed25519 seed of the receiver.
///
/// WHY a trait: the engine must not touch the file system or a keychain on
/// its own — the platform layer owns persistence. The trait keeps the engine
/// testable and lets the native shells plug in their own secure storage.
pub trait IdentityStore: Send + Sync + Debug {
    /// Loads the stored seed, if there is one.
    ///
    /// # Errors
    /// Returns [`AirPlayError::Io`] when the store cannot be read.
    fn load_seed(&self) -> Result<Option<[u8; ED25519_KEY_BYTES]>, AirPlayError>;

    /// Stores the seed.
    ///
    /// # Errors
    /// Returns [`AirPlayError::Io`] when the store cannot be written.
    fn store_seed(&self, seed: &[u8; ED25519_KEY_BYTES]) -> Result<(), AirPlayError>;
}

/// An identity store that keeps the seed in memory.
///
/// Used in tests and as the fallback when the platform provides no
/// persistence.
#[derive(Debug, Default)]
pub struct InMemoryIdentityStore {
    seed: std::sync::Mutex<Option<[u8; ED25519_KEY_BYTES]>>,
}

impl InMemoryIdentityStore {
    /// Creates an empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl IdentityStore for InMemoryIdentityStore {
    fn load_seed(&self) -> Result<Option<[u8; ED25519_KEY_BYTES]>, AirPlayError> {
        let guard = self
            .seed
            .lock()
            .map_err(|_| AirPlayError::invalid_state("identity store is poisoned"))?;
        Ok(*guard)
    }

    fn store_seed(&self, seed: &[u8; ED25519_KEY_BYTES]) -> Result<(), AirPlayError> {
        let mut guard = self
            .seed
            .lock()
            .map_err(|_| AirPlayError::invalid_state("identity store is poisoned"))?;
        *guard = Some(*seed);
        Ok(())
    }
}

/// The Ed25519 identity of the receiver.
#[derive(Debug, Clone)]
pub struct PairingIdentity {
    signing_key: SigningKey,
    public_key: [u8; ED25519_KEY_BYTES],
}

impl PairingIdentity {
    /// Builds an identity from a raw 32-byte seed.
    #[must_use]
    pub fn from_seed(seed: &[u8; ED25519_KEY_BYTES]) -> Self {
        let signing_key = SigningKey::from_bytes(seed);
        let public_key = signing_key.verifying_key().to_bytes();
        Self {
            signing_key,
            public_key,
        }
    }

    /// Loads the identity from the store, creating and storing one if needed.
    ///
    /// # Errors
    /// Returns [`AirPlayError::Io`] when the store cannot be read or written.
    pub fn load_or_create(store: &dyn IdentityStore) -> Result<Self, AirPlayError> {
        if let Some(seed) = store.load_seed()? {
            return Ok(Self::from_seed(&seed));
        }
        let mut seed = [0u8; ED25519_KEY_BYTES];
        rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut seed);
        store.store_seed(&seed)?;
        Ok(Self::from_seed(&seed))
    }

    /// The 32-byte Ed25519 public key of the receiver.
    #[must_use]
    pub fn public_key(&self) -> [u8; ED25519_KEY_BYTES] {
        self.public_key
    }

    /// Signs a message with the receiver's identity.
    #[must_use]
    pub fn sign(&self, message: &[u8]) -> [u8; ED25519_SIGNATURE_BYTES] {
        self.signing_key.sign(message).to_bytes()
    }

    /// Verifies an Ed25519 signature of a public key.
    #[must_use]
    pub fn verify(public_key: &[u8; ED25519_KEY_BYTES], message: &[u8], signature: &[u8]) -> bool {
        let Ok(key) = VerifyingKey::from_bytes(public_key) else {
            return false;
        };
        let Ok(signature) = Signature::from_slice(signature) else {
            return false;
        };
        key.verify(message, &signature).is_ok()
    }
}

/// A shared identity handle.
pub type SharedIdentity = Arc<PairingIdentity>;

#[cfg(test)]
mod tests;
