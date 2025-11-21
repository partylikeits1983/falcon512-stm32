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

/// Simple example demonstrating Falcon512 signing on STM32
///
/// This example shows:
/// 1. Key generation from a seed
/// 2. Signing a message
/// 3. Verifying the signature
///
/// In a real application, you would:
/// - Use hardware RNG for seed generation
/// - Store keys in secure storage
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

    // Initialize RNG with a seed
    // NOTE: In production, use hardware RNG from your STM32 chip!
    // Example: let mut rng = stm32f4xx_hal::rng::Rng::new(device.RNG, &clocks);
    let seed = [0x42u8; 32]; // DO NOT use fixed seed in production!
    let mut rng = ChaCha20Rng::from_seed(seed);

    // Generate a seed for key generation
    let mut keygen_seed = [0u8; 32];
    rng.fill_bytes(&mut keygen_seed);

    // Generate Falcon512 key pair
    let (secret_key, public_key) = falcon512::keygen(keygen_seed);

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

    // Example: Serialize keys and signature for storage/transmission
    let _sk_bytes = secret_key.to_bytes();
    let _pk_bytes = public_key.to_bytes();
    let _sig_bytes = signature.to_bytes();

    // Example sizes for Falcon512:
    // - Secret key: ~1281 bytes
    // - Public key: ~897 bytes
    // - Signature: variable, typically ~666 bytes

    loop {
        // Infinite loop - embedded systems don't exit
        cortex_m::asm::nop();
    }
}
