# Quick Start Guide

## Prerequisites

Before you begin, ensure you have:
- Rust toolchain installed
- ARM target: `rustup target add thumbv7em-none-eabihf`
- probe-rs installed: `cargo install probe-rs-tools`
- STM32H7 board with ST-LINK debugger connected

## Complete Workflow

### 1. Generate Keys (One-time, on your laptop)

```bash
cd keygen
cargo run --release
```

This creates:
- `secret_key.bin` (1281 bytes) - **Keep this secret!**
- `public_key.bin` (897 bytes)
- Console output with Rust arrays (for reference)

### 2. Prepare Keys for Flashing

```bash
cd ../flash_keys
cargo run --release -- \
  --sk-file ../keygen/secret_key.bin \
  --pk-file ../keygen/public_key.bin
```

This creates `keys.bin` (8 KB) ready for flashing.

### 3. Flash Keys to STM32

**Connect your STM32 board**, then:

```bash
probe-rs download \
  --chip STM32H743ZITx \
  --binary-format Bin \
  --base-address 0x081FE000 \
  keys.bin
```

**Note**: The base address `0x081FE000` is for STM32H743ZI (2MB flash). For other chips:
- STM32F407 (1MB flash): use `0x080FE000`
- Adjust `--chip` for your specific STM32 model

### 4. Build and Flash Firmware

```bash
cd ../stm32
cargo build --release
probe-rs run --chip STM32H743ZITx target/thumbv7em-none-eabihf/release/falcon512-stm32
```

## Troubleshooting

### "Probe not found"

**Cause**: ST-LINK debugger not connected or not detected.

**Solutions**:
1. Connect your STM32 board via USB
2. Check USB cable (must support data, not just power)
3. Install ST-LINK drivers (Windows) or check permissions (Linux/macOS)
4. Try: `probe-rs list` to see if probe is detected
5. On macOS, you may need to allow USB access in System Settings

### "Chip not found" or wrong chip

**Cause**: Wrong chip specified in `--chip` parameter.

**Solutions**:
1. Find your exact chip model (printed on the MCU)
2. List available chips: `probe-rs chip list | grep STM32`
3. Update commands with correct chip name

### "Failed to load secret key"

**Cause**: Keys haven't been flashed yet, or flash was erased.

**Solution**: Complete steps 1-3 to flash keys.

### Build errors

```bash
# Clean and rebuild
cargo clean
cargo build --release

# Update dependencies
cargo update
```

## Security Reminders

- ✅ **DO**: Keep `secret_key.bin` secure and backed up
- ✅ **DO**: Generate unique keys per device for production
- ✅ **DO**: Use `.gitignore` to prevent committing keys
- ❌ **DON'T**: Commit `secret_key.bin` to version control
- ❌ **DON'T**: Share secret keys publicly
- ❌ **DON'T**: Reuse the same keys across multiple devices in production

## Next Steps

- Read [`KEYGEN_WORKFLOW.md`](KEYGEN_WORKFLOW.md) for detailed documentation
- See [`README.md`](README.md) for project overview
- Check `stm32/README.md` for firmware details

## Hardware Compatibility

This project is configured for STM32H7 but can be adapted for:
- **STM32F4** series (F407, F429, etc.) - Adjust `memory.x` and chip name
- **STM32H7** series (H743, H750, etc.) - Current configuration
- **STM32L4** series - Adjust memory configuration
- Other Cortex-M4F/M7 chips with sufficient RAM (128KB+)

To adapt for your chip:
1. Update `stm32/memory.x` with correct flash/RAM sizes
2. Update `--chip` parameter in commands
3. Adjust reserved key section address if needed