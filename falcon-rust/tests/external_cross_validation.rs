//! Cross-validation test using external falcon-rust crate
//!
//! This test signs messages with our implementation and verifies them
//! with the external falcon-rust v0.1.2 crate from crates.io.
//! This provides strong assurance of interoperability.

use falcon_rust::falcon512::{keygen, sign_with_rng};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

#[test]
fn test_our_signatures_verified_by_external_crate() {
    println!("\n=== Testing Our Signatures with External Verifier ===\n");

    // Generate keypair with our implementation
    let keygen_seed = [42u8; 32];
    let (our_secret_key, our_public_key) = keygen(keygen_seed);
    println!("✓ Generated keypair with our implementation");

    // Serialize public key for external use
    let pk_bytes = our_public_key.to_bytes();
    let external_pk = falcon_rust_external::falcon512::PublicKey::from_bytes(&pk_bytes)
        .expect("External crate should deserialize our public key");
    println!("✓ External crate successfully deserialized our public key");

    // Test messages
    let test_messages = vec![
        b"Hello, Falcon!".to_vec(),
        b"Test message 1".to_vec(),
        b"Test message 2 with more content".to_vec(),
        b"".to_vec(),    // Empty message
        vec![0xAB; 100], // Binary data
        b"The quick brown fox jumps over the lazy dog".to_vec(),
        vec![0xFF; 500], // Large binary message
        b"1234567890".to_vec(),
        b"Special chars: !@#$%^&*()".to_vec(),
        b"Unicode test: \xE2\x9C\x93".to_vec(),
    ];

    let mut rng = ChaCha20Rng::from_seed([1u8; 32]);
    let mut success_count = 0;

    for (i, message) in test_messages.iter().enumerate() {
        // Sign with our implementation
        let our_signature = sign_with_rng(message, &our_secret_key, &mut rng);

        // First verify with our own implementation (sanity check)
        assert!(
            falcon_rust::falcon512::verify(message, &our_signature, &our_public_key),
            "Our implementation failed to verify its own signature for message {}",
            i
        );

        // Serialize signature for external verification
        let sig_bytes = our_signature.to_bytes();
        let external_sig = falcon_rust_external::falcon512::Signature::from_bytes(&sig_bytes)
            .expect(&format!(
                "External crate should deserialize our signature {}",
                i
            ));

        // Verify with external implementation
        let external_verified =
            falcon_rust_external::falcon512::verify(message, &external_sig, &external_pk);

        assert!(
            external_verified,
            "External crate failed to verify our signature for message {} (length: {} bytes)",
            i,
            message.len()
        );

        success_count += 1;
        println!(
            "  ✓ Message {} verified by external crate (length: {} bytes)",
            i + 1,
            message.len()
        );
    }

    println!(
        "\n=== All {} signatures successfully verified by external crate! ===\n",
        success_count
    );
}

#[test]
fn test_multiple_keypairs_external_verification() {
    println!("\n=== Testing Multiple Keypairs with External Verifier ===\n");

    let num_keypairs = 5;
    let messages_per_key = 3;

    for key_idx in 0..num_keypairs {
        let keygen_seed = [key_idx as u8; 32];
        let (our_secret_key, our_public_key) = keygen(keygen_seed);

        let pk_bytes = our_public_key.to_bytes();
        let external_pk = falcon_rust_external::falcon512::PublicKey::from_bytes(&pk_bytes)
            .expect("Failed to deserialize public key");

        let mut rng = ChaCha20Rng::from_seed([key_idx as u8; 32]);

        for msg_idx in 0..messages_per_key {
            let message = format!("Message {} for keypair {}", msg_idx, key_idx);
            let message_bytes = message.as_bytes();

            let our_signature = sign_with_rng(message_bytes, &our_secret_key, &mut rng);
            let sig_bytes = our_signature.to_bytes();

            let external_sig = falcon_rust_external::falcon512::Signature::from_bytes(&sig_bytes)
                .expect("Failed to deserialize signature");

            let verified =
                falcon_rust_external::falcon512::verify(message_bytes, &external_sig, &external_pk);

            assert!(
                verified,
                "External verification failed for keypair {} message {}",
                key_idx, msg_idx
            );
        }

        println!(
            "  ✓ Keypair {} - all {} messages verified by external crate",
            key_idx + 1,
            messages_per_key
        );
    }

    println!(
        "\n=== All {} keypairs with {} messages each verified! ===\n",
        num_keypairs, messages_per_key
    );
}
