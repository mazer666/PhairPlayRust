
use super::*;
use rsa::traits::PublicKeyParts;

#[test]
fn the_embedded_key_parses() {
    let key = private_key().expect("the embedded key must parse");
    assert_eq!(key.n().bits(), 2048);
}

#[test]
fn a_garbage_blob_is_rejected() {
    assert!(decrypt_aes_key(&[0u8; 256]).is_none());
    assert!(decrypt_aes_key(&[0u8; 16]).is_none());
    assert!(decrypt_aes_key(&[]).is_none());
}

#[test]
fn a_wrapped_key_is_recovered() {
    // The sender encrypts the key with the public half; the receiver
    // recovers it with the private half.
    let public = public_key().expect("public key");
    let padding = Oaep::new::<sha1::Sha1>();
    let secret = [0x42u8; AES_KEY_BYTES];
    let wrapped = public
        .encrypt(&mut rand::rngs::OsRng, padding, &secret)
        .expect("encryptable");
    assert_eq!(decrypt_aes_key(&wrapped), Some(secret.to_vec()));
}

#[test]
fn the_hex_decoder_rejects_an_odd_length() {
    assert!(decode_hex("abc").is_none());
    assert_eq!(decode_hex("4142"), Some(vec![0x41, 0x42]));
}
