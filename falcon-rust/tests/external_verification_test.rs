//! External verification test
//!
//! This test uses our implementation to sign messages, then uses the
//! external falcon-rust crate (v0.1.2) to verify those signatures.
//! This provides strong assurance of interoperability and correctness.
//!
//! Note: This test is NOT no_std compatible as it uses the external crate.

use falcon_rust::falcon512::{keygen, sign_with_rng};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

// Import the external falcon-rust crate for verification
// This assumes the external crate is added as a dev-dependency
#[cfg(test)]
mod external_falcon {
    // Re-export the external crate's types
    pub use falcon_rust_external::falcon512;
}

#[test]
fn test_our_signature_external_verification() {
    // Generate keypair with our implementation
    let seed = [42u8; 32];
    let (our_secret_key, our_public_key) = keygen(seed);

    // Create a test message
    let message = b"Test message for cross-implementation verification";

    // Sign with our implementation
    let mut rng = ChaCha20Rng::from_seed([1u8; 32]);
    let our_signature = sign_with_rng(message, &our_secret_key, &mut rng);

    // First verify with our own implementation (sanity check)
    assert!(
        falcon_rust::falcon512::verify(message, &our_signature, &our_public_key),
        "Our implementation should verify its own signature"
    );

    // Serialize our keys and signature for external verification
    let pk_bytes = our_public_key.to_bytes();
    let sig_bytes = our_signature.to_bytes();

    // Deserialize with external implementation
    let external_pk = external_falcon::falcon512::PublicKey::from_bytes(&pk_bytes)
        .expect("External implementation should deserialize our public key");
    let external_sig = external_falcon::falcon512::Signature::from_bytes(&sig_bytes)
        .expect("External implementation should deserialize our signature");

    // Verify with external implementation
    let external_verification =
        external_falcon::falcon512::verify(message, &external_sig, &external_pk);

    assert!(
        external_verification,
        "External implementation should verify our signature"
    );
}

#[test]
fn test_multiple_messages_external_verification() {
    let seed = [99u8; 32];
    let (our_secret_key, our_public_key) = keygen(seed);
    let pk_bytes = our_public_key.to_bytes();

    let external_pk = external_falcon::falcon512::PublicKey::from_bytes(&pk_bytes)
        .expect("Failed to deserialize public key");

    let large_message = vec![0xAB; 1000];
    let test_messages: Vec<&[u8]> = vec![
        b"Message 1",
        b"Message 2 with more content",
        b"",            // Empty message
        &large_message, // Large message
    ];

    let mut rng = ChaCha20Rng::from_seed([2u8; 32]);

    for (i, msg) in test_messages.iter().enumerate() {
        // Sign with our implementation
        let our_signature = sign_with_rng(msg, &our_secret_key, &mut rng);
        let sig_bytes = our_signature.to_bytes();

        // Verify with our implementation
        assert!(
            falcon_rust::falcon512::verify(msg, &our_signature, &our_public_key),
            "Our verification failed for message {}",
            i
        );

        // Verify with external implementation
        let external_sig = external_falcon::falcon512::Signature::from_bytes(&sig_bytes)
            .expect(&format!("Failed to deserialize signature {}", i));

        assert!(
            external_falcon::falcon512::verify(msg, &external_sig, &external_pk),
            "External verification failed for message {}",
            i
        );
    }
}

#[test]
fn test_external_rejects_invalid_signature() {
    let seed = [123u8; 32];
    let (our_secret_key, our_public_key) = keygen(seed);

    let message = b"Valid message";
    let mut rng = ChaCha20Rng::from_seed([3u8; 32]);
    let our_signature = sign_with_rng(message, &our_secret_key, &mut rng);

    // Serialize
    let pk_bytes = our_public_key.to_bytes();
    let sig_bytes = our_signature.to_bytes();

    // Deserialize with external implementation
    let external_pk = external_falcon::falcon512::PublicKey::from_bytes(&pk_bytes)
        .expect("Failed to deserialize public key");
    let external_sig = external_falcon::falcon512::Signature::from_bytes(&sig_bytes)
        .expect("Failed to deserialize signature");

    // Test with wrong message
    let wrong_message = b"Wrong message";
    assert!(
        !external_falcon::falcon512::verify(wrong_message, &external_sig, &external_pk),
        "External implementation should reject signature for wrong message"
    );

    // Test with corrupted signature
    let mut corrupted_sig_bytes = sig_bytes.clone();
    corrupted_sig_bytes[50] ^= 0xFF;

    if let Ok(corrupted_sig) =
        external_falcon::falcon512::Signature::from_bytes(&corrupted_sig_bytes)
    {
        assert!(
            !external_falcon::falcon512::verify(message, &corrupted_sig, &external_pk),
            "External implementation should reject corrupted signature"
        );
    }
}

#[test]
fn test_key_serialization_compatibility() {
    // Test that keys serialized by our implementation can be used by external implementation
    let seed = [77u8; 32];
    let (our_secret_key, our_public_key) = keygen(seed);

    // Serialize with our implementation
    let pk_bytes = our_public_key.to_bytes();
    let sk_bytes = our_secret_key.to_bytes();

    // Deserialize with external implementation
    let external_pk = external_falcon::falcon512::PublicKey::from_bytes(&pk_bytes)
        .expect("External implementation should deserialize our public key");
    let external_sk = external_falcon::falcon512::SecretKey::from_bytes(&sk_bytes)
        .expect("External implementation should deserialize our secret key");

    // Sign with external implementation
    let message = b"Cross-implementation key test";
    let external_signature = external_falcon::falcon512::sign(message, &external_sk);

    // Verify with external implementation
    assert!(
        external_falcon::falcon512::verify(message, &external_signature, &external_pk),
        "External implementation should verify its own signature with our keys"
    );

    // Verify with our implementation
    let sig_bytes = external_signature.to_bytes();
    let our_signature = falcon_rust::falcon512::Signature::from_bytes(&sig_bytes)
        .expect("Our implementation should deserialize external signature");

    assert!(
        falcon_rust::falcon512::verify(message, &our_signature, &our_public_key),
        "Our implementation should verify external signature with our keys"
    );
}

#[test]
fn test_deterministic_signing_compatibility() {
    // Test that deterministic signing produces compatible results
    let keygen_seed = [55u8; 32];
    let (our_secret_key, our_public_key) = keygen(keygen_seed);

    let message = b"Deterministic test message";
    let signing_seed = [88u8; 32];

    // Sign with our implementation
    let mut our_rng = ChaCha20Rng::from_seed(signing_seed);
    let our_signature = sign_with_rng(message, &our_secret_key, &mut our_rng);

    // Serialize and verify with external implementation
    let pk_bytes = our_public_key.to_bytes();
    let sig_bytes = our_signature.to_bytes();

    let external_pk = external_falcon::falcon512::PublicKey::from_bytes(&pk_bytes)
        .expect("Failed to deserialize public key");
    let external_sig = external_falcon::falcon512::Signature::from_bytes(&sig_bytes)
        .expect("Failed to deserialize signature");

    assert!(
        external_falcon::falcon512::verify(message, &external_sig, &external_pk),
        "External implementation should verify deterministic signature"
    );
}
