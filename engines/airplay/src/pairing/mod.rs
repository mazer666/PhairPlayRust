//! The pairing handshakes of AirPlay.
//!
//! WHY: an AirPlay receiver does not accept every sender on the network. It
//! proves its identity with an Ed25519 key and proves the sender's identity
//! with an ephemeral X25519 key agreement. On top of that, the user may
//! require a PIN: a four-digit code shown on the receiver and typed on the
//! sender, protected by a lock-out on the number of failed attempts.
//!
//! | Module | Responsibility |
//! |---|---|
//! | [`identity`] | The long-lived Ed25519 identity of the receiver. |
//! | [`verify`] | The pair-setup / pair-verify exchange. |
//! | [`legacy`] | The legacy SRP-6a PIN pairing. |
//! | [`store`] | The paired controllers and the attempt counter. |
//! | [`raop_rsa`] | The legacy AirTunes RSA key unwrap. |

pub mod identity;
pub mod legacy;
pub mod raop_rsa;
pub mod store;
pub mod verify;

pub use identity::{IdentityStore, InMemoryIdentityStore, PairingIdentity, SharedIdentity};
pub use legacy::{LegacyPinPairing, PinReply};
pub use store::{InMemoryPairingStore, PairingStore};
pub use verify::PairingSession;
