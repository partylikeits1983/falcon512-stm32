#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m_rt::entry;
use falcon_rust::falcon512;
use rand_chacha::ChaCha20Rng;
use rand_core::{RngCore, SeedableRng};

// Set up the global allocator for heap allocations
use embedded_alloc::Heap;

#[global_allocator]
static HEAP: Heap = Heap::empty();

// ====== KEY STORAGE IN FLASH ======
// Keys are stored in a reserved flash section (see memory.x)
// Flash address: 0x080FE000 (last 8KB of 1MB flash)
// Layout:
//   - Bytes 0-1280: Secret key (1281 bytes)
//   - Bytes 1281-2177: Public key (897 bytes)
//   - Remaining: unused/padding
const KEYS_FLASH_ADDR: usize = 0x080FE000;
const SK_SIZE: usize = 1281;
const PK_SIZE: usize = 897;

/// Simple example demonstrating Falcon512 signing on STM32 with pre-generated keys
///
/// This example shows:
/// 1. Loading pre-generated keys from flash memory
/// 2. Signing a message
/// 3. Verifying the signature
///
/// Key generation workflow:
/// 1. Run keygen tool on laptop: `cd keygen && cargo run --release`
/// 2. Flash keys to reserved section using flash_keys tool
/// 3. Flash and run this firmware
///
/// In a real application, you would:
/// - Use hardware RNG for signing randomness (not key generation!)
/// - Handle errors appropriately
/// - Use serial/UART for communication
#[entry]
fn main() -> ! {
    // Initialize the heap allocator
    // Allocate 64KB for heap (adjust based on available RAM)
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 64 * 1024;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
    }

    // Initialize RNG for signing randomness ONLY
    // NOTE: In production, use hardware RNG from your STM32 chip!
    // Example: let mut rng = stm32h7xx_hal::rng::Rng::new(device.RNG, &clocks);
    let seed = [0x42u8; 32]; // DO NOT use fixed seed in production!
    let mut rng = ChaCha20Rng::from_seed(seed);

    // Load pre-generated keys from flash memory
    // SAFETY: Reading from flash memory at a fixed address
    // The keys must be flashed to this address using the flash_keys tool
    let sk_bytes: &[u8] = unsafe {
        core::slice::from_raw_parts(KEYS_FLASH_ADDR as *const u8, SK_SIZE)
    };
    let pk_bytes: &[u8] = unsafe {
        core::slice::from_raw_parts((KEYS_FLASH_ADDR + SK_SIZE) as *const u8, PK_SIZE)
    };

    // Reconstruct keys from bytes
    // Note: If keys haven't been flashed yet (all zeros), this will fail
    let secret_key = falcon512::SecretKey::from_bytes(sk_bytes)
        .expect("Failed to load secret key - have you flashed keys using flash_keys tool?");
    let public_key = falcon512::PublicKey::from_bytes(pk_bytes)
        .expect("Failed to load public key - have you flashed keys using flash_keys tool?");

    // Message to sign
    let message = b"Hello from STM32 with Falcon512!";

    // Sign the message
    let signature = falcon512::sign_with_rng(message, &secret_key, &mut rng);

    // Verify the signature
    let is_valid = falcon512::verify(message, &signature, &public_key);

    // In a real application, you would:
    // - Output results via UART/serial
    // - Toggle LEDs based on verification result
    // - Store signature for transmission

    if is_valid {
        // Signature is valid - success!
        // Example: toggle_led_green();
    } else {
        // Signature verification failed
        // Example: toggle_led_red();
    }

    // Serialize signature for transmission
    let _sig_bytes = signature.to_bytes();

    // Signature size for Falcon512: variable, typically ~666 bytes

    loop {
        // Infinite loop - embedded systems don't exit
        cortex_m::asm::nop();
    }
}
