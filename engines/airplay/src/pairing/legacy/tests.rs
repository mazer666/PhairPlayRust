#![allow(clippy::expect_used, clippy::unwrap_used)]

use super::*;

use crate::error::AirPlayError;

/// The client side of the exchange, used to verify the server.
///
/// The client computes the session key the standard way:
/// `S = (B - k·v)^(a + u·x) mod N`, which is the same value the server
/// reaches through `S = (A · v^u)^b mod N`.
struct Client {
    username: String,
    pin: String,
    private: BigUint,
    public: BigUint,
    session_key: Vec<u8>,
}

impl Client {
    fn new(username: &str, pin: &str) -> Self {
        let private = random_private();
        let public = BigUint::from(2u8).modpow(&private, &group());
        Self {
            username: username.to_string(),
            pin: pin.to_string(),
            private,
            public,
            session_key: Vec::new(),
        }
    }

    /// Answers the server's step 1 and returns the proof.
    fn proof(&mut self, salt: &[u8], server_public: &[u8]) -> Vec<u8> {
        let server_public = BigUint::from_bytes_be(server_public);
        let inner = sha1(&[format!("{}:{}", self.username, self.pin).as_bytes()]);
        let x = BigUint::from_bytes_be(&sha1(&[salt, &inner]));
        let verifier = BigUint::from(2u8).modpow(&x, &group());
        let k = BigUint::from_bytes_be(&sha1(&[&pad(&group()), &pad(&BigUint::from(2u8))]));
        let u = BigUint::from_bytes_be(&sha1(&[&pad(&self.public), &pad(&server_public)]));
        let base = (&server_public + group() - (&k * &verifier) % group()) % group();
        let exponent = (&self.private + &u * &x) % group();
        let shared = base.modpow(&exponent, &group());
        self.session_key = session_key(&to_bytes(&shared));
        sha1(&[
            &h_n_xor_h_g(),
            &sha1(&[self.username.as_bytes()]),
            salt,
            &to_bytes(&self.public),
            &server_public.to_bytes_be(),
            &self.session_key,
        ])
    }

    /// The expected answer of the server to the proof.
    fn expected_proof(&self, client_proof: &[u8]) -> Vec<u8> {
        sha1(&[&to_bytes(&self.public), client_proof, &self.session_key])
    }

    /// Encrypts the client key for the third step.
    fn encrypt_key(&self, key: &[u8; 32]) -> (Vec<u8>, Vec<u8>) {
        // The client encrypts with the first counter value.
        let (aes_key, mut iv) = derive_step_three_material(&self.session_key);
        increment_counter(&mut iv);
        let mut buffer = key.to_vec();
        let cipher = AesGcm::<Aes128, U16>::new_from_slice(&aes_key).expect("valid key");
        let nonce = aes_gcm::Nonce::<U16>::from_slice(&iv);
        let tag = cipher
            .encrypt_in_place_detached(nonce, &[], &mut buffer)
            .expect("encryptable");
        (buffer, tag.to_vec())
    }

    /// Decrypts the server key of the third step.
    fn decrypt_key(&self, encrypted: &[u8], tag: &[u8]) -> Option<Vec<u8>> {
        // The server answers with the second counter value.
        let (aes_key, mut iv) = derive_step_three_material(&self.session_key);
        increment_counter(&mut iv);
        increment_counter(&mut iv);
        let mut buffer = encrypted.to_vec();
        let cipher = AesGcm::<Aes128, U16>::new_from_slice(&aes_key).ok()?;
        let nonce = aes_gcm::Nonce::<U16>::from_slice(&iv);
        let tag = aes_gcm::Tag::<U16>::from_slice(tag);
        cipher
            .decrypt_in_place_detached(nonce, &[], &mut buffer, tag)
            .ok()?;
        Some(buffer)
    }
}

#[test]
fn the_group_prime_has_the_expected_size() {
    assert_eq!(group().to_bytes_be().len(), GROUP_BYTES);
}

#[test]
fn step_one_answers_with_a_key_and_a_salt() {
    let mut pairing = LegacyPinPairing::new("1234", [0u8; 32]);
    let request = PlistValue::dictionary(vec![
        ("method".into(), PlistValue::String("pin".into())),
        ("user".into(), PlistValue::String("mac-1".into())),
    ]);
    let reply = pairing.handle(&request).expect("step 1");
    assert!(!reply.complete);
    let pk = reply.values.iter().find(|(key, _)| key == "pk");
    let salt = reply.values.iter().find(|(key, _)| key == "salt");
    assert!(pk.is_some());
    assert!(salt.is_some());
    let pk = pk.and_then(|(_, value)| value.as_data()).expect("pk");
    assert_eq!(pk.len(), GROUP_BYTES);
    let salt = salt.and_then(|(_, value)| value.as_data()).expect("salt");
    assert_eq!(salt.len(), SALT_BYTES);
}

#[test]
fn the_public_key_is_always_the_size_of_the_group() {
    // WHY: the sender reads the public key as a fixed number of bytes. The
    // minimal representation of a number drops the leading zeros, and a key
    // that is one byte shorter than the group is a key that the sender
    // reads as a different number — the pairing breaks. The probability of
    // that is one in 256, and it broke the pairing of the sessions that
    // were unlucky.
    //
    // WHY the loop runs the exchange many times: the key is random, and a
    // single run does not reach the case that the test is about. The loop
    // runs it often enough that the case is reached — and the test proves
    // that it is not.
    let mut lengths = std::collections::BTreeSet::new();
    for _ in 0..2_000 {
        let mut pairing = LegacyPinPairing::new("1234", [0u8; 32]);
        let request = PlistValue::dictionary(vec![
            ("method".into(), PlistValue::String("pin".into())),
            ("user".into(), PlistValue::String("mac-1".into())),
        ]);
        let reply = pairing.handle(&request).expect("step 1");
        let pk = reply
            .values
            .iter()
            .find(|(key, _)| key == "pk")
            .and_then(|(_, value)| value.as_data())
            .expect("pk");
        lengths.insert(pk.len());
    }
    assert_eq!(
        lengths,
        std::collections::BTreeSet::from([GROUP_BYTES]),
        "the public key is not always the size of the group: {lengths:?}"
    );
}

#[test]
fn a_client_that_knows_the_pin_completes_the_exchange() {
    let accessory_public = [0x5Au8; 32];
    let mut pairing = LegacyPinPairing::new("1234", accessory_public);
    let request = PlistValue::dictionary(vec![
        ("method".into(), PlistValue::String("pin".into())),
        ("user".into(), PlistValue::String("mac-1".into())),
    ]);
    let reply = pairing.handle(&request).expect("step 1");
    let server_public = reply
        .values
        .iter()
        .find(|(key, _)| key == "pk")
        .and_then(|(_, value)| value.as_data())
        .expect("pk")
        .to_vec();
    let salt = reply
        .values
        .iter()
        .find(|(key, _)| key == "salt")
        .and_then(|(_, value)| value.as_data())
        .expect("salt")
        .to_vec();

    let mut client = Client::new("mac-1", "1234");
    let proof = client.proof(&salt, &server_public);
    let request = PlistValue::dictionary(vec![
        ("pk".into(), PlistValue::Data(client.public.to_bytes_be())),
        ("proof".into(), PlistValue::Data(proof.clone())),
    ]);
    let reply = pairing.handle(&request).expect("step 2");
    let server_proof = reply
        .values
        .iter()
        .find(|(key, _)| key == "proof")
        .and_then(|(_, value)| value.as_data())
        .expect("proof")
        .to_vec();
    let expected = client.expected_proof(&proof);
    assert_eq!(server_proof, expected);
    assert!(!reply.complete);

    // Step 3: exchange the Ed25519 public keys.
    let client_public = [0x11u8; 32];
    let (epk, tag) = client.encrypt_key(&client_public);
    let request = PlistValue::dictionary(vec![
        ("epk".into(), PlistValue::Data(epk)),
        ("authTag".into(), PlistValue::Data(tag)),
    ]);
    let reply = pairing.handle(&request).expect("step 3");
    assert!(reply.complete);
    let server_key = reply
        .values
        .iter()
        .find(|(key, _)| key == "epk")
        .and_then(|(_, value)| value.as_data())
        .expect("epk");
    let server_tag = reply
        .values
        .iter()
        .find(|(key, _)| key == "authTag")
        .and_then(|(_, value)| value.as_data())
        .expect("authTag");
    assert_eq!(
        client.decrypt_key(server_key, server_tag),
        Some(accessory_public.to_vec())
    );
}

#[test]
fn an_unknown_request_is_rejected() {
    let mut pairing = LegacyPinPairing::new("1234", [0u8; 32]);
    let request = PlistValue::dictionary(vec![("nonsense".into(), PlistValue::Integer(1))]);
    let error = pairing.handle(&request).expect_err("must fail");
    assert!(matches!(error, AirPlayError::Parse { .. }));
}

#[test]
fn step_two_before_step_one_is_rejected() {
    let mut pairing = LegacyPinPairing::new("1234", [0u8; 32]);
    let request = PlistValue::dictionary(vec![
        ("pk".into(), PlistValue::Data(vec![1u8; 32])),
        ("proof".into(), PlistValue::Data(vec![0u8; 20])),
    ]);
    let error = pairing.handle(&request).expect_err("must fail");
    assert!(matches!(error, AirPlayError::InvalidState { .. }));
}

#[test]
fn step_three_before_step_two_is_rejected() {
    let mut pairing = LegacyPinPairing::new("1234", [0u8; 32]);
    let request = PlistValue::dictionary(vec![
        ("epk".into(), PlistValue::Data(vec![1u8; 32])),
        ("authTag".into(), PlistValue::Data(vec![0u8; 16])),
    ]);
    let error = pairing.handle(&request).expect_err("must fail");
    assert!(matches!(error, AirPlayError::InvalidState { .. }));
}

#[test]
fn a_wrong_pin_is_rejected() {
    let mut pairing = LegacyPinPairing::new("1234", [0u8; 32]);
    let request = PlistValue::dictionary(vec![
        ("method".into(), PlistValue::String("pin".into())),
        ("user".into(), PlistValue::String("mac-1".into())),
    ]);
    let reply = pairing.handle(&request).expect("step 1");
    let server_public = reply
        .values
        .iter()
        .find(|(key, _)| key == "pk")
        .and_then(|(_, value)| value.as_data())
        .expect("pk")
        .to_vec();
    let salt = reply
        .values
        .iter()
        .find(|(key, _)| key == "salt")
        .and_then(|(_, value)| value.as_data())
        .expect("salt")
        .to_vec();
    let mut client = Client::new("mac-1", "9999");
    let proof = client.proof(&salt, &server_public);
    let request = PlistValue::dictionary(vec![
        ("pk".into(), PlistValue::Data(client.public.to_bytes_be())),
        ("proof".into(), PlistValue::Data(proof)),
    ]);
    let error = pairing.handle(&request).expect_err("must fail");
    assert!(matches!(error, AirPlayError::Crypto { .. }));
}

#[test]
fn a_client_key_outside_the_group_is_rejected() {
    let mut pairing = LegacyPinPairing::new("1234", [0u8; 32]);
    let request = PlistValue::dictionary(vec![
        ("method".into(), PlistValue::String("pin".into())),
        ("user".into(), PlistValue::String("mac-1".into())),
    ]);
    let _ = pairing.handle(&request).expect("step 1");
    let request = PlistValue::dictionary(vec![
        ("pk".into(), PlistValue::Data(vec![0u8; 32])),
        ("proof".into(), PlistValue::Data(vec![0u8; 20])),
    ]);
    let error = pairing.handle(&request).expect_err("must fail");
    assert!(matches!(error, AirPlayError::Crypto { .. }));
}

#[test]
fn the_counter_increments_with_carry() {
    let mut counter = [0xFFu8; 16];
    increment_counter(&mut counter);
    assert_eq!(counter, [0u8; 16]);
    let mut counter = [0u8; 16];
    counter[15] = 1;
    increment_counter(&mut counter);
    assert_eq!(counter[15], 2);
}

#[test]
fn the_session_key_is_forty_bytes() {
    let key = session_key(&[1u8; 32]);
    assert_eq!(key.len(), 40);
}
