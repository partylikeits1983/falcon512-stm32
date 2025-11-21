use falcon_rust::falcon512::{keygen, sign_with_rng, verify};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

// Memory tracking allocator
struct TrackingAllocator;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static PEAK_ALLOCATED: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = System.alloc(layout);
        if !ret.is_null() {
            let size = layout.size();
            // Use saturating_add to prevent overflow
            let prev = ALLOCATED.load(Ordering::SeqCst);
            let current = prev.saturating_add(size);
            ALLOCATED.store(current, Ordering::SeqCst);

            // Update peak if necessary
            let mut peak = PEAK_ALLOCATED.load(Ordering::SeqCst);
            while current > peak {
                match PEAK_ALLOCATED.compare_exchange_weak(
                    peak,
                    current,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                ) {
                    Ok(_) => break,
                    Err(x) => peak = x,
                }
            }
        }
        ret
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        let size = layout.size();
        let current = ALLOCATED.load(Ordering::SeqCst);
        // Prevent underflow - only subtract if we have enough allocated
        if current >= size {
            ALLOCATED.fetch_sub(size, Ordering::SeqCst);
        }
    }
}

#[global_allocator]
static GLOBAL: TrackingAllocator = TrackingAllocator;

fn reset_memory_tracking() {
    ALLOCATED.store(0, Ordering::SeqCst);
    PEAK_ALLOCATED.store(0, Ordering::SeqCst);
}

fn get_current_allocated() -> usize {
    ALLOCATED.load(Ordering::SeqCst)
}

fn get_peak_allocated() -> usize {
    PEAK_ALLOCATED.load(Ordering::SeqCst)
}

#[test]
fn test_keygen_memory_usage() {
    println!("\n=== Falcon-512 Key Generation Memory Usage ===");

    // Warm up to ensure any lazy statics are initialized
    let seed = [0u8; 32];
    let _ = keygen(seed);

    // Reset and measure
    reset_memory_tracking();
    let baseline = get_current_allocated();
    println!("Baseline memory: {} bytes", baseline);

    let seed = [42u8; 32];
    let (secret_key, public_key) = keygen(seed);

    let after_keygen = get_current_allocated();
    let peak = get_peak_allocated();

    println!("After keygen: {} bytes", after_keygen);
    println!("Peak during keygen: {} bytes", peak);
    println!(
        "Net allocation: {} bytes",
        after_keygen.saturating_sub(baseline)
    );
    println!(
        "Peak temporary usage: {} bytes",
        peak.saturating_sub(baseline)
    );

    // Keep keys alive to measure their size
    let sk_size = std::mem::size_of_val(&secret_key);
    let pk_size = std::mem::size_of_val(&public_key);
    println!("\nStack sizes:");
    println!("  SecretKey struct: {} bytes", sk_size);
    println!("  PublicKey struct: {} bytes", pk_size);

    // Serialize to see actual data size
    let sk_bytes = secret_key.to_bytes();
    let pk_bytes = public_key.to_bytes();
    println!("\nSerialized sizes:");
    println!("  SecretKey: {} bytes", sk_bytes.len());
    println!("  PublicKey: {} bytes", pk_bytes.len());
}

#[test]
fn test_sign_memory_usage() {
    println!("\n=== Falcon-512 Signing Memory Usage ===");

    // Generate keys first
    let seed = [42u8; 32];
    let (secret_key, _public_key) = keygen(seed);
    let message = b"Test message for memory profiling";

    // Warm up
    let mut rng = ChaCha20Rng::from_seed([1u8; 32]);
    let _ = sign_with_rng(message, &secret_key, &mut rng);

    // Reset and measure
    reset_memory_tracking();
    let baseline = get_current_allocated();
    println!("Baseline memory: {} bytes", baseline);

    let mut rng = ChaCha20Rng::from_seed([2u8; 32]);
    let signature = sign_with_rng(message, &secret_key, &mut rng);

    let after_sign = get_current_allocated();
    let peak = get_peak_allocated();

    println!("After signing: {} bytes", after_sign);
    println!("Peak during signing: {} bytes", peak);
    println!(
        "Net allocation: {} bytes",
        after_sign.saturating_sub(baseline)
    );
    println!(
        "Peak temporary usage: {} bytes",
        peak.saturating_sub(baseline)
    );

    let sig_size = std::mem::size_of_val(&signature);
    println!("\nStack size:");
    println!("  Signature struct: {} bytes", sig_size);

    let sig_bytes = signature.to_bytes();
    println!("\nSerialized size:");
    println!("  Signature: {} bytes", sig_bytes.len());
}

#[test]
fn test_verify_memory_usage() {
    println!("\n=== Falcon-512 Verification Memory Usage ===");

    // Generate keys and signature first
    let seed = [42u8; 32];
    let (secret_key, public_key) = keygen(seed);
    let message = b"Test message for memory profiling";
    let mut rng = ChaCha20Rng::from_seed([1u8; 32]);
    let signature = sign_with_rng(message, &secret_key, &mut rng);

    // Warm up
    let _ = verify(message, &signature, &public_key);

    // Reset and measure
    reset_memory_tracking();
    let baseline = get_current_allocated();
    println!("Baseline memory: {} bytes", baseline);

    let is_valid = verify(message, &signature, &public_key);

    let after_verify = get_current_allocated();
    let peak = get_peak_allocated();

    println!("After verification: {} bytes", after_verify);
    println!("Peak during verification: {} bytes", peak);
    println!(
        "Net allocation: {} bytes",
        after_verify.saturating_sub(baseline)
    );
    println!(
        "Peak temporary usage: {} bytes",
        peak.saturating_sub(baseline)
    );
    println!("Signature valid: {}", is_valid);
}

#[test]
fn test_multiple_operations_memory() {
    println!("\n=== Falcon-512 Multiple Operations Memory Usage ===");

    reset_memory_tracking();
    let baseline = get_current_allocated();
    println!("Initial baseline: {} bytes", baseline);

    // Key generation
    let seed = [42u8; 32];
    let (secret_key, public_key) = keygen(seed);
    let after_keygen = get_current_allocated();
    let peak_keygen = get_peak_allocated();
    println!("\nAfter keygen:");
    println!("  Current: {} bytes", after_keygen);
    println!("  Peak: {} bytes", peak_keygen);

    // Multiple signatures
    let message = b"Test message";
    let mut rng = ChaCha20Rng::from_seed([1u8; 32]);

    for i in 0..5 {
        let before = get_current_allocated();
        let signature = sign_with_rng(message, &secret_key, &mut rng);
        let after = get_current_allocated();
        let peak = get_peak_allocated();

        println!("\nSignature {}:", i + 1);
        println!("  Before: {} bytes", before);
        println!("  After: {} bytes", after);
        println!("  Peak: {} bytes", peak);
        println!("  Delta: {} bytes", after.saturating_sub(before));

        // Verify
        let before_verify = get_current_allocated();
        let is_valid = verify(message, &signature, &public_key);
        let after_verify = get_current_allocated();

        println!(
            "  Verify delta: {} bytes",
            after_verify.saturating_sub(before_verify)
        );
        println!("  Valid: {}", is_valid);

        // Drop signature to free memory
        drop(signature);
        let after_drop = get_current_allocated();
        println!("  After drop: {} bytes", after_drop);
    }

    let final_allocated = get_current_allocated();
    let final_peak = get_peak_allocated();
    println!("\nFinal state:");
    println!("  Current: {} bytes", final_allocated);
    println!("  Peak: {} bytes", final_peak);
    println!(
        "  Net from baseline: {} bytes",
        final_allocated.saturating_sub(baseline)
    );
}
