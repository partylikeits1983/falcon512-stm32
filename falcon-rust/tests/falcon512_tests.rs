use falcon_rust::falcon512::{keygen, sign_with_rng, verify, PublicKey, SecretKey, Signature};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

#[test]
fn test_basic_sign_and_verify() {
    // 1. Generate keys
    let seed = [42u8; 32]; // Use a fixed seed for reproducibility in tests
    let (secret_key, public_key) = keygen(seed);

    // 2. Sign a message
    let message = b"Hello, Falcon!";
    let mut rng = ChaCha20Rng::from_seed([1u8; 32]);
    let signature = sign_with_rng(message, &secret_key, &mut rng);

    // 3. Verify the signature
    let is_valid = verify(message, &signature, &public_key);
    assert!(is_valid, "Signature should be valid");

    // Verify that a different message fails verification
    let wrong_message = b"Wrong message";
    let is_invalid = verify(wrong_message, &signature, &public_key);
    assert!(
        !is_invalid,
        "Signature should be invalid for different message"
    );
}

#[test]
fn test_multiple_signatures() {
    // Generate keys once
    let seed = [123u8; 32];
    let (secret_key, public_key) = keygen(seed);

    // Sign multiple different messages
    let messages = [
        b"First message" as &[u8],
        b"Second message",
        b"Third message with more content",
    ];

    let mut rng = ChaCha20Rng::from_seed([2u8; 32]);

    for msg in messages.iter() {
        let signature = sign_with_rng(msg, &secret_key, &mut rng);
        assert!(
            verify(msg, &signature, &public_key),
            "Signature should be valid for message: {:?}",
            std::str::from_utf8(msg)
        );
    }
}

#[test]
fn test_serialization_roundtrip() {
    // Generate keys
    let seed = [99u8; 32];
    let (secret_key, public_key) = keygen(seed);

    // Sign a message
    let message = b"Test serialization";
    let mut rng = ChaCha20Rng::from_seed([3u8; 32]);
    let signature = sign_with_rng(message, &secret_key, &mut rng);

    // Serialize keys and signature
    let sk_bytes = secret_key.to_bytes();
    let pk_bytes = public_key.to_bytes();
    let sig_bytes = signature.to_bytes();

    // Deserialize
    let sk_restored = SecretKey::from_bytes(&sk_bytes).expect("Failed to deserialize secret key");
    let pk_restored = PublicKey::from_bytes(&pk_bytes).expect("Failed to deserialize public key");
    let sig_restored = Signature::from_bytes(&sig_bytes).expect("Failed to deserialize signature");

    // Verify with restored keys and signature
    assert!(
        verify(message, &sig_restored, &pk_restored),
        "Signature should be valid after serialization roundtrip"
    );

    // Verify we can sign with restored secret key
    let new_signature = sign_with_rng(message, &sk_restored, &mut rng);
    assert!(
        verify(message, &new_signature, &pk_restored),
        "New signature with restored key should be valid"
    );
}
