use falcon_rust::falcon512::{keygen, sign_with_rng, verify};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use std::time::Instant;

#[test]
fn test_keygen_performance() {
    let iterations = 10;
    let mut total_duration = std::time::Duration::ZERO;

    println!("\n=== Falcon-512 Key Generation Performance ===");

    for i in 0..iterations {
        let seed = [(i as u8).wrapping_mul(17); 32];
        let start = Instant::now();
        let (_secret_key, _public_key) = keygen(seed);
        let duration = start.elapsed();
        total_duration += duration;
        println!("  Iteration {}: {:?}", i + 1, duration);
    }

    let avg_duration = total_duration / iterations;
    println!("  Average: {:?}", avg_duration);
    println!("  Total: {:?}", total_duration);
}

#[test]
fn test_sign_performance() {
    let iterations = 100;
    let seed = [42u8; 32];
    let (secret_key, _public_key) = keygen(seed);
    let message = b"Performance test message for Falcon-512 signing";
    let mut rng = ChaCha20Rng::from_seed([1u8; 32]);

    let mut total_duration = std::time::Duration::ZERO;

    println!("\n=== Falcon-512 Signing Performance ===");

    for i in 0..iterations {
        let start = Instant::now();
        let _signature = sign_with_rng(message, &secret_key, &mut rng);
        let duration = start.elapsed();
        total_duration += duration;

        if i < 5 || i >= iterations - 5 {
            println!("  Iteration {}: {:?}", i + 1, duration);
        } else if i == 5 {
            println!("  ...");
        }
    }

    let avg_duration = total_duration / iterations;
    println!("  Average: {:?}", avg_duration);
    println!("  Total: {:?}", total_duration);
}

#[test]
fn test_verify_performance() {
    let iterations = 100;
    let seed = [42u8; 32];
    let (secret_key, public_key) = keygen(seed);
    let message = b"Performance test message for Falcon-512 verification";
    let mut rng = ChaCha20Rng::from_seed([1u8; 32]);
    let signature = sign_with_rng(message, &secret_key, &mut rng);

    let mut total_duration = std::time::Duration::ZERO;

    println!("\n=== Falcon-512 Verification Performance ===");

    for i in 0..iterations {
        let start = Instant::now();
        let is_valid = verify(message, &signature, &public_key);
        let duration = start.elapsed();
        total_duration += duration;
        assert!(is_valid, "Signature should be valid");

        if i < 5 || i >= iterations - 5 {
            println!("  Iteration {}: {:?}", i + 1, duration);
        } else if i == 5 {
            println!("  ...");
        }
    }

    let avg_duration = total_duration / iterations;
    println!("  Average: {:?}", avg_duration);
    println!("  Total: {:?}", total_duration);
}

#[test]
fn test_full_cycle_performance() {
    let iterations = 10;

    println!("\n=== Falcon-512 Full Cycle Performance ===");
    println!("(KeyGen + Sign + Verify)");

    let mut keygen_total = std::time::Duration::ZERO;
    let mut sign_total = std::time::Duration::ZERO;
    let mut verify_total = std::time::Duration::ZERO;

    for i in 0..iterations {
        let seed = [(i as u8).wrapping_mul(23); 32];
        let message = b"Full cycle performance test";

        // KeyGen
        let start = Instant::now();
        let (secret_key, public_key) = keygen(seed);
        let keygen_duration = start.elapsed();
        keygen_total += keygen_duration;

        // Sign
        let mut rng = ChaCha20Rng::from_seed([i as u8; 32]);
        let start = Instant::now();
        let signature = sign_with_rng(message, &secret_key, &mut rng);
        let sign_duration = start.elapsed();
        sign_total += sign_duration;

        // Verify
        let start = Instant::now();
        let is_valid = verify(message, &signature, &public_key);
        let verify_duration = start.elapsed();
        verify_total += verify_duration;
        assert!(is_valid, "Signature should be valid");

        let total = keygen_duration + sign_duration + verify_duration;
        println!(
            "  Iteration {}: KeyGen={:?}, Sign={:?}, Verify={:?}, Total={:?}",
            i + 1,
            keygen_duration,
            sign_duration,
            verify_duration,
            total
        );
    }

    println!("\n  Averages over {} iterations:", iterations);
    println!("    KeyGen:  {:?}", keygen_total / iterations);
    println!("    Sign:    {:?}", sign_total / iterations);
    println!("    Verify:  {:?}", verify_total / iterations);
    println!(
        "    Total:   {:?}",
        (keygen_total + sign_total + verify_total) / iterations
    );
}
