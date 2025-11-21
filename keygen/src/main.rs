//! Falcon512 Key Generation Tool
//!
//! This tool generates Falcon512 key pairs on your laptop and outputs them
//! as Rust byte arrays that can be embedded in your STM32 firmware.
//!
//! Usage:
//!   cargo run --release
//!
//! The output will be two const arrays (SK_BYTES and PK_BYTES) that you can
//! copy-paste into your embedded firmware.

use falcon_rust::falcon512;
use rand_chacha::ChaCha20Rng;
use rand_core::{RngCore, SeedableRng};
use std::fs;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Falcon512 Key Generation Tool ===\n");
    
    // IMPORTANT: In production, use OS RNG or hardware RNG, not a fixed seed!
    // For example: use rand::rngs::OsRng;
    // For now, using ChaCha20Rng with a fixed seed for reproducibility
    let seed = [0x42u8; 32];
    let mut rng = ChaCha20Rng::from_seed(seed);

    println!("Generating Falcon512 key pair...");
    
    // Generate a seed for key generation
    let mut keygen_seed = [0u8; 32];
    rng.fill_bytes(&mut keygen_seed);

    // Generate the key pair
    let (secret_key, public_key) = falcon512::keygen(keygen_seed);

    // Convert to bytes
    let sk_bytes = secret_key.to_bytes();
    let pk_bytes = public_key.to_bytes();

    println!("Key generation complete!\n");
    println!("Secret key size: {} bytes", sk_bytes.len());
    println!("Public key size: {} bytes\n", pk_bytes.len());

    // Print secret key as a Rust array
    println!("// Copy this into your STM32 firmware:");
    println!("// File: stm32/src/main.rs\n");
    
    print!("const SK_BYTES: [u8; {}] = [", sk_bytes.len());
    for (i, b) in sk_bytes.iter().enumerate() {
        if i % 16 == 0 {
            println!();
            print!("    ");
        }
        print!("0x{:02X}", b);
        if i < sk_bytes.len() - 1 {
            print!(", ");
        }
    }
    println!("\n];\n");

    // Print public key as a Rust array
    print!("const PK_BYTES: [u8; {}] = [", pk_bytes.len());
    for (i, b) in pk_bytes.iter().enumerate() {
        if i % 16 == 0 {
            println!();
            print!("    ");
        }
        print!("0x{:02X}", b);
        if i < pk_bytes.len() - 1 {
            print!(", ");
        }
    }
    println!("\n];");

    // Also save as binary files for easy flashing
    println!("\n=== Saving Binary Files ===");
    
    fs::write("secret_key.bin", &sk_bytes)?;
    println!("✓ Saved secret_key.bin ({} bytes)", sk_bytes.len());
    
    fs::write("public_key.bin", &pk_bytes)?;
    println!("✓ Saved public_key.bin ({} bytes)", pk_bytes.len());

    println!("\n=== Next Steps ===");
    println!("Option 1: Flash keys directly to STM32 reserved section");
    println!("  cd ../flash_keys");
    println!("  cargo run --release -- --sk-file ../keygen/secret_key.bin --pk-file ../keygen/public_key.bin");
    println!("  # Then flash keys.bin to 0x080FE000 using probe-rs or OpenOCD");
    println!();
    println!("Option 2: Copy arrays into firmware (less secure)");
    println!("  1. Copy the SK_BYTES and PK_BYTES arrays above");
    println!("  2. Paste them into your stm32/src/main.rs file");
    println!();
    println!("Security Notes:");
    println!("- Keep secret_key.bin and SK_BYTES secret! Do not commit to public repositories.");
    println!("- For production: use OS RNG (rand::rngs::OsRng) instead of fixed seed");
    println!("- For per-device keys: generate separately for each device");
    println!("- Option 1 (flash to reserved section) is more secure than Option 2");

    Ok(())
}