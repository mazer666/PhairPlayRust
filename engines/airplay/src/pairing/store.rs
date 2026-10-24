//! The store of paired controllers.
//!
//! WHY: the PIN flow is protected by a *lock-out*, not by the entropy of the
//! PIN. A four-digit code has ten thousand possibilities, and a neighbour on
//! the WLAN can try them; the only defence that actually works is to bound
//! the number of failed attempts. The counter therefore lives in a store the
//! platform layer persists, so that restarting the receiver does not hand the
//! attacker a fresh budget.
//!
//! EXAMPLE:
//!
//! ```
//! use phairplay_airplay::pairing::store::{InMemoryPairingStore, PairingStore};
//!
//! let store = InMemoryPairingStore::new();
//! store.add_controller("mac-1", &[1u8; 32]).expect("store");
//! assert!(store.has_any_controller().expect("query"));
//! assert_eq!(store.failed_attempts().expect("query"), 0);
//! ```

use std::fmt::Debug;
use std::sync::Mutex;

use crate::error::AirPlayError;

/// The store of paired controllers and of the failed attempt counter.
pub trait PairingStore: Send + Sync + Debug {
    /// Stores the long-term public key of a controller.
    ///
    /// # Errors
    /// Returns [`AirPlayError::Io`] when the store cannot be written.
    fn add_controller(&self, controller_id: &str, long_term_key: &[u8]) -> Result<(), AirPlayError>;

    /// Loads the long-term public key of a controller.
    ///
    /// # Errors
    /// Returns [`AirPlayError::Io`] when the store cannot be read.
    fn controller(&self, controller_id: &str) -> Result<Option<Vec<u8>>, AirPlayError>;

    /// Removes a controller from the store.
    ///
    /// # Errors
    /// Returns [`AirPlayError::Io`] when the store cannot be written.
    fn remove_controller(&self, controller_id: &str) -> Result<(), AirPlayError>;

    /// Whether any controller is paired.
    ///
    /// # Errors
    /// Returns [`AirPlayError::Io`] when the store cannot be read.
    fn has_any_controller(&self) -> Result<bool, AirPlayError>;

    /// The number of failed pairing attempts since the last success.
    ///
    /// # Errors
    /// Returns [`AirPlayError::Io`] when the store cannot be read.
    fn failed_attempts(&self) -> Result<u32, AirPlayError>;

    /// Records a failed attempt and returns the new count.
    ///
    /// # Errors
    /// Returns [`AirPlayError::Io`] when the store cannot be written.
    fn record_failed_attempt(&self) -> Result<u32, AirPlayError>;

    /// Resets the failed attempt counter.
    ///
    /// # Errors
    /// Returns [`AirPlayError::Io`] when the store cannot be written.
    fn reset_failed_attempts(&self) -> Result<(), AirPlayError>;
}

/// A pairing store that keeps everything in memory.
#[derive(Debug, Default)]
pub struct InMemoryPairingStore {
    controllers: Mutex<std::collections::BTreeMap<String, Vec<u8>>>,
    failed_attempts: Mutex<u32>,
}

impl InMemoryPairingStore {
    /// Creates an empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl PairingStore for InMemoryPairingStore {
    fn add_controller(
        &self,
        controller_id: &str,
        long_term_key: &[u8],
    ) -> Result<(), AirPlayError> {
        let mut guard = self
            .controllers
            .lock()
            .map_err(|_| AirPlayError::invalid_state("pairing store is poisoned"))?;
        guard.insert(controller_id.to_string(), long_term_key.to_vec());
        Ok(())
    }

    fn controller(&self, controller_id: &str) -> Result<Option<Vec<u8>>, AirPlayError> {
        let guard = self
            .controllers
            .lock()
            .map_err(|_| AirPlayError::invalid_state("pairing store is poisoned"))?;
        Ok(guard.get(controller_id).cloned())
    }

    fn remove_controller(&self, controller_id: &str) -> Result<(), AirPlayError> {
        let mut guard = self
            .controllers
            .lock()
            .map_err(|_| AirPlayError::invalid_state("pairing store is poisoned"))?;
        guard.remove(controller_id);
        Ok(())
    }

    fn has_any_controller(&self) -> Result<bool, AirPlayError> {
        let guard = self
            .controllers
            .lock()
            .map_err(|_| AirPlayError::invalid_state("pairing store is poisoned"))?;
        Ok(!guard.is_empty())
    }

    fn failed_attempts(&self) -> Result<u32, AirPlayError> {
        let guard = self
            .failed_attempts
            .lock()
            .map_err(|_| AirPlayError::invalid_state("pairing store is poisoned"))?;
        Ok(*guard)
    }

    fn record_failed_attempt(&self) -> Result<u32, AirPlayError> {
        let mut guard = self
            .failed_attempts
            .lock()
            .map_err(|_| AirPlayError::invalid_state("pairing store is poisoned"))?;
        *guard = guard.saturating_add(1);
        Ok(*guard)
    }

    fn reset_failed_attempts(&self) -> Result<(), AirPlayError> {
        let mut guard = self
            .failed_attempts
            .lock()
            .map_err(|_| AirPlayError::invalid_state("pairing store is poisoned"))?;
        *guard = 0;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
