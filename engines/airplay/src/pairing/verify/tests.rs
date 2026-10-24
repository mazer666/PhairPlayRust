
use super::*;

use crate::error::AirPlayError;
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};

/// The sender side of the exchange, used to test the receiver.
struct Sender {
    identity: SigningKey,
    secret: StaticSecret,
    public: PublicKey,
    /// The Ed25519 public key of the receiver, learned in pair-setup.
    receiver_ed_public: [u8; 32],
}

impl Sender {
    fn new(seed: u8, receiver_ed_public: [u8; 32]) -> Self {
        let identity = SigningKey::from_bytes(&[seed; 32]);
        let secret = StaticSecret::random_from_rng(rand::rngs::OsRng);
        let public = PublicKey::from(&secret);
        Self {
            identity,
            secret,
            public,
            receiver_ed_public,
        }
    }

    fn ed_public(&self) -> [u8; 32] {
        self.identity.verifying_key().to_bytes()
    }

    fn first_message(&self) -> Vec<u8> {
        let mut body = vec![1u8, 0, 0, 0];
        body.extend_from_slice(self.public.as_bytes());
        body.extend_from_slice(&self.ed_public());
        body
    }

    fn second_message(&self, receiver_public: &[u8]) -> Vec<u8> {
        let mut their_public = [0u8; 32];
        their_public.copy_from_slice(receiver_public);
        let shared = self.secret.diffie_hellman(&PublicKey::from(their_public));
        let mut shared_secret = [0u8; 32];
        shared_secret.copy_from_slice(shared.as_bytes());
        let (key, iv) = derive_cipher_material(&shared_secret);

        // The client signs the transcript in the opposite order of the
        // receiver: (client public ‖ receiver public).
        let mut transcript = [0u8; 64];
        transcript[..32].copy_from_slice(self.public.as_bytes());
        transcript[32..].copy_from_slice(&their_public);
        let signature = self.identity.sign(&transcript);

        // The first 64 bytes of the keystream belong to the receiver's
        // signature, so the client has to skip them.
        let mut cipher = Ctr128BE::<Aes128>::new(&key.into(), &iv.into());
        let mut skipped = [0u8; 64];
        cipher.apply_keystream(&mut skipped);
        let mut encrypted = signature.to_vec();
        cipher.apply_keystream(&mut encrypted);

        let mut body = vec![0u8, 0, 0, 0];
        body.extend_from_slice(&encrypted);
        body
    }

    fn verify(&self, receiver_public: &[u8], encrypted: &[u8]) -> bool {
        let mut their_public = [0u8; 32];
        their_public.copy_from_slice(receiver_public);
        let shared = self.secret.diffie_hellman(&PublicKey::from(their_public));
        let mut shared_secret = [0u8; 32];
        shared_secret.copy_from_slice(shared.as_bytes());
        let (key, iv) = derive_cipher_material(&shared_secret);
        let mut cipher = Ctr128BE::<Aes128>::new(&key.into(), &iv.into());
        let mut decrypted = encrypted.to_vec();
        cipher.apply_keystream(&mut decrypted);

        // The receiver signs (receiver public ‖ client public).
        let mut transcript = [0u8; 64];
        transcript[..32].copy_from_slice(&their_public);
        transcript[32..].copy_from_slice(self.public.as_bytes());
        let Ok(signature) = ed25519_dalek::Signature::from_slice(&decrypted) else {
            return false;
        };
        VerifyingKey::from_bytes(&self.receiver_ed_public)
            .map(|key| key.verify(&transcript, &signature).is_ok())
            .unwrap_or(false)
    }
}

fn receiver() -> PairingSession {
    let identity = PairingIdentity::from_seed(&[9u8; 32]);
    PairingSession::new(Arc::new(identity))
}

/// The Ed25519 public key of the receiver the tests use.
fn receiver_ed_public() -> [u8; 32] {
    PairingIdentity::from_seed(&[9u8; 32]).public_key()
}

#[test]
fn pair_setup_returns_the_receiver_public_key() {
    let session = receiver();
    let response = session.pair_setup(&[0u8; 32]).expect("pair setup");
    assert_eq!(response.len(), 32);
}

#[test]
fn pair_setup_rejects_a_body_of_the_wrong_size() {
    let session = receiver();
    let error = session.pair_setup(&[0u8; 16]).expect_err("must fail");
    assert!(matches!(error, AirPlayError::Parse { .. }));
}

#[test]
fn a_complete_exchange_produces_a_shared_secret() {
    let mut session = receiver();
    let sender = Sender::new(4, receiver_ed_public());
    let first = session
        .pair_verify(&sender.first_message())
        .expect("message 1");
    assert_eq!(first.len(), 96);
    assert!(session.shared_secret().is_some());

    let second = sender.second_message(&first[..32]);
    let response = session.pair_verify(&second).expect("message 2");
    assert!(response.is_empty());
    assert!(session.is_verified());
}

#[test]
fn the_sender_can_verify_the_receiver_signature() {
    let mut session = receiver();
    let sender = Sender::new(5, receiver_ed_public());
    let first = session
        .pair_verify(&sender.first_message())
        .expect("message 1");
    assert!(sender.verify(&first[..32], &first[32..]));
}

#[test]
fn a_forged_signature_is_rejected() {
    let mut session = receiver();
    let sender = Sender::new(6, receiver_ed_public());
    let _ = session
        .pair_verify(&sender.first_message())
        .expect("message 1");
    let mut second = sender.second_message(&[0u8; 32]);
    // Corrupt the signature.
    second[10] ^= 0xFF;
    let error = session.pair_verify(&second).expect_err("must fail");
    assert!(matches!(error, AirPlayError::Crypto { .. }));
}

#[test]
fn the_second_message_without_the_first_is_rejected() {
    let mut session = receiver();
    let error = session.pair_verify(&[0u8; 68]).expect_err("must fail");
    assert!(matches!(error, AirPlayError::InvalidState { .. }));
}

#[test]
fn a_message_of_the_wrong_size_is_rejected() {
    let mut session = receiver();
    let error = session.pair_verify(&[1u8, 0, 0, 0, 1, 2]).expect_err("must fail");
    assert!(matches!(error, AirPlayError::Parse { .. }));
}

#[test]
fn an_unknown_state_byte_is_rejected() {
    let mut session = receiver();
    let error = session.pair_verify(&[7u8; 68]).expect_err("must fail");
    assert!(matches!(error, AirPlayError::Parse { .. }));
}

#[test]
fn the_derived_key_material_is_stable() {
    let secret = [0u8; 32];
    let (key, iv) = derive_cipher_material(&secret);
    assert_eq!(key.len(), 16);
    assert_eq!(iv.len(), 16);
    assert_ne!(key, iv);
}
