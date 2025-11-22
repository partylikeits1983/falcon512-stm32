//! Signing module - handles Falcon512 signature operations

extern crate alloc;

use alloc::vec::Vec;
use falcon_rust::falcon512;
use rand_chacha::ChaCha20Rng;
use rtt_target::rprintln;

/// Maximum message size (adjust as needed)
pub const MAX_MESSAGE_SIZE: usize = 512;

/// Signer handles Falcon512 signing operations
pub struct Signer {
    secret_key: falcon512::SecretKey,
    rng: ChaCha20Rng,
}

impl Signer {
    /// Create a new Signer with the given secret key and RNG
    pub fn new(secret_key: falcon512::SecretKey, rng: ChaCha20Rng) -> Self {
        Self { secret_key, rng }
    }

    /// Sign a message and return the signature bytes as a Vec
    pub fn sign_message(&mut self, message: &[u8]) -> Vec<u8> {
        rprintln!("Signing {} byte message...", message.len());
        let signature = falcon512::sign_with_rng(message, &self.secret_key, &mut self.rng);
        rprintln!("Signature generated!");

        signature.to_bytes()
    }
}

/// Helper function to format byte as hex
pub fn format_hex(byte: u8) -> [u8; 2] {
    const HEX_CHARS: &[u8] = b"0123456789ABCDEF";
    [
        HEX_CHARS[(byte >> 4) as usize],
        HEX_CHARS[(byte & 0x0F) as usize],
    ]
}
