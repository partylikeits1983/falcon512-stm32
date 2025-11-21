# Falcon512 Key Generation and Flashing Workflow

This document describes how to generate Falcon512 keys on your laptop and flash them to your STM32 device.

## Overview

The workflow separates key generation (done once on a secure laptop) from signing operations (done on the STM32). This approach:

- **Improves security**: Keys never need to be generated on the embedded device
- **Saves resources**: No need for key generation code on the STM32
- **Enables per-device keys**: Generate unique keys for each device
- **Protects keys**: Keys stored in reserved flash, never in source code

## Architecture

### Memory Layout

The STM32 flash is divided into:
- **Program Flash**: 0x08000000 - 0x080FE000 (1016 KB) - Your firmware
- **Key Storage**: 0x080FE000 - 0x08100000 (8 KB) - Reserved for keys

Key layout in the reserved section:
```
0x080FE000: Secret Key (1281 bytes)
0x080FE501: Public Key (897 bytes)
0x080FE882: Unused (padding to 8KB)
```

### Components

1. **keygen/** - Laptop tool to generate keys
2. **flash_keys/** - Laptop tool to prepare keys for flashing
3. **stm32/** - Embedded firmware that reads keys from flash

## Step-by-Step Workflow

### Step 1: Generate Keys on Your Laptop

```bash
cd keygen
cargo run --release
```

This will:
- Generate a Falcon512 key pair
- Print the keys as Rust arrays (for reference)
- Save `secret_key.bin` (1281 bytes)
- Save `public_key.bin` (897 bytes)

**Security Note**: Keep `secret_key.bin` secure! Store it in a safe location, not in version control.

### Step 2: Prepare Keys for Flashing

```bash
cd ../flash_keys
cargo run --release -- \
  --sk-file ../keygen/secret_key.bin \
  --pk-file ../keygen/public_key.bin \
  --output keys.bin
```

This creates `keys.bin` (8 KB) containing both keys in the correct layout.

### Step 3: Flash Keys to STM32

Choose one of these methods:

#### Option A: Using probe-rs (Recommended)

```bash
probe-rs download \
  --chip STM32H743ZITx \
  --format Bin \
  --base-address 0x080FE000 \
  keys.bin
```

#### Option B: Using OpenOCD

```bash
openocd \
  -f interface/stlink.cfg \
  -f target/stm32h7x.cfg \
  -c "program keys.bin 0x080FE000 verify reset exit"
```

#### Option C: Using STM32CubeProgrammer

1. Open STM32CubeProgrammer
2. Connect to your device
3. Go to "Erasing & Programming"
4. Select `keys.bin`
5. Set start address: `0x080FE000`
6. Click "Start Programming"

### Step 4: Build and Flash Firmware

```bash
cd ../stm32
cargo build --release
probe-rs run --chip STM32H743ZITx target/thumbv7em-none-eabihf/release/stm32
```

The firmware will:
1. Read keys from flash address 0x080FE000
2. Reconstruct `SecretKey` and `PublicKey` objects
3. Use them for signing operations

## Per-Device Keys

To generate unique keys for each device:

1. Generate keys for device 1:
   ```bash
   cd keygen
   cargo run --release
   mv secret_key.bin secret_key_device1.bin
   mv public_key.bin public_key_device1.bin
   ```

2. Generate keys for device 2:
   ```bash
   cargo run --release
   mv secret_key.bin secret_key_device2.bin
   mv public_key.bin public_key_device2.bin
   ```

3. Flash each device with its own keys:
   ```bash
   cd ../flash_keys
   
   # Device 1
   cargo run --release -- \
     --sk-file ../keygen/secret_key_device1.bin \
     --pk-file ../keygen/public_key_device1.bin \
     --output keys_device1.bin
   probe-rs download --chip STM32H743ZITx --format Bin --base-address 0x080FE000 keys_device1.bin
   
   # Device 2
   cargo run --release -- \
     --sk-file ../keygen/secret_key_device2.bin \
     --pk-file ../keygen/public_key_device2.bin \
     --output keys_device2.bin
   probe-rs download --chip STM32H743ZITx --format Bin --base-address 0x080FE000 keys_device2.bin
   ```

## Security Considerations

### ✅ Good Practices

- **Generate keys on a secure laptop**, not on the embedded device
- **Store secret keys securely** (encrypted storage, hardware security module)
- **Use unique keys per device** for production deployments
- **Never commit secret keys** to version control (add `*.bin` to `.gitignore`)
- **Use hardware RNG** on STM32 for signing randomness (not key generation)
- **Protect flash memory** using read-out protection (RDP) on STM32

### ⚠️ Important Notes

- The reserved flash section (0x080FE000) must not be overwritten by firmware updates
- Keys are stored in plain flash - consider using STM32 security features:
  - Read-Out Protection (RDP) Level 1 or 2
  - Proprietary Code Read-Out Protection (PCROP)
  - Secure memory areas (on STM32H7 with TrustZone)
- For maximum security, use an external secure element (e.g., ATECC608)

## Troubleshooting

### Error: "Failed to load secret key"

**Cause**: Keys haven't been flashed to the device yet, or flash is erased.

**Solution**: Follow Steps 1-3 to generate and flash keys.

### Error: "invalid secret key bytes"

**Cause**: Corrupted keys or wrong flash address.

**Solution**: 
1. Verify keys were flashed correctly
2. Check that `memory.x` matches your STM32 chip's flash size
3. Re-flash keys using Step 3

### Keys are all zeros

**Cause**: Flash section was erased or never programmed.

**Solution**: Flash keys using Step 3.

## Advanced: Using OS RNG for Key Generation

For production, modify `keygen/src/main.rs` to use the OS random number generator:

```rust
use rand::rngs::OsRng;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = OsRng;
    
    let mut keygen_seed = [0u8; 32];
    rng.fill_bytes(&mut keygen_seed);
    
    let (secret_key, public_key) = falcon512::keygen(keygen_seed);
    // ... rest of code
}
```

Add to `keygen/Cargo.toml`:
```toml
[dependencies]
rand = "0.8"
```

## File Structure

```
falcon512-stm32/
├── keygen/              # Laptop tool: generate keys
│   ├── Cargo.toml
│   └── src/main.rs
├── flash_keys/          # Laptop tool: prepare keys for flashing
│   ├── Cargo.toml
│   └── src/main.rs
├── stm32/               # Embedded firmware
│   ├── Cargo.toml
│   ├── memory.x         # Memory layout with reserved key section
│   └── src/main.rs      # Reads keys from flash
└── KEYGEN_WORKFLOW.md   # This file
```

## Quick Reference

```bash
# Generate keys
cd keygen && cargo run --release

# Prepare for flashing
cd ../flash_keys && cargo run --release -- \
  --sk-file ../keygen/secret_key.bin \
  --pk-file ../keygen/public_key.bin

# Flash keys
probe-rs download --chip STM32H743ZITx --format Bin --base-address 0x080FE000 keys.bin

# Build and run firmware
cd ../stm32 && cargo build --release
probe-rs run --chip STM32H743ZITx target/thumbv7em-none-eabihf/release/stm32