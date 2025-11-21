#![no_std]

extern crate alloc;

use falcon_rust::falcon512::{keygen, sign_with_rng, verify};
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;

#[test]
fn test_nostd_basic_sign_and_verify() {
    // 1. Generate keys
    let seed = [42u8; 32];
    let (secret_key, public_key) = keygen(seed);

    // 2. Sign a message
    let message = b"Hello, Falcon no-std!";
    let mut rng = ChaCha20Rng::from_seed([1u8; 32]);
    let signature = sign_with_rng(message, &secret_key, &mut rng);

    // 3. Verify the signature
    let is_valid = verify(message, &signature, &public_key);
    assert!(is_valid, "Signature should be valid in no-std environment");

    // Verify that a different message fails verification
    let wrong_message = b"Wrong message";
    let is_invalid = verify(wrong_message, &signature, &public_key);
    assert!(
        !is_invalid,
        "Signature should be invalid for different message in no-std"
    );
}

#[test]
fn test_nostd_serialization() {
    // Generate keys
    let seed = [99u8; 32];
    let (secret_key, public_key) = keygen(seed);

    // Sign a message
    let message = b"Test no-std serialization";
    let mut rng = ChaCha20Rng::from_seed([3u8; 32]);
    let signature = sign_with_rng(message, &secret_key, &mut rng);

    // Serialize
    let sk_bytes = secret_key.to_bytes();
    let pk_bytes = public_key.to_bytes();
    let sig_bytes = signature.to_bytes();

    // Deserialize
    let sk_restored = falcon_rust::falcon512::SecretKey::from_bytes(&sk_bytes)
        .expect("Failed to deserialize secret key");
    let pk_restored = falcon_rust::falcon512::PublicKey::from_bytes(&pk_bytes)
        .expect("Failed to deserialize public key");
    let sig_restored = falcon_rust::falcon512::Signature::from_bytes(&sig_bytes)
        .expect("Failed to deserialize signature");

    // Verify with restored keys
    assert!(
        verify(message, &sig_restored, &pk_restored),
        "Signature should be valid after no-std serialization roundtrip"
    );

    // Verify we can sign with restored secret key
    let new_signature = sign_with_rng(message, &sk_restored, &mut rng);
    assert!(
        verify(message, &new_signature, &pk_restored),
        "New signature with restored key should be valid in no-std"
    );
}
