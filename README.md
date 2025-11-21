# Falcon512 STM32 Project

A multi-crate Rust workspace for signing data with Falcon512 post-quantum signatures on STM32 microcontrollers.

## Project Structure

This is a Cargo workspace containing two crates:

```
falcon512-stm32/
├── Cargo.toml              # Workspace configuration
├── falcon-rust/            # Falcon512 signature library (no-std)
│   ├── Cargo.toml
│   ├── src/
│   └── tests/
└── stm32/                  # STM32 firmware implementation
    ├── Cargo.toml
    ├── build.rs
    ├── memory.x
    ├── .cargo/config.toml
    ├── src/
    │   └── main.rs
    └── README.md
```

## Crates

### falcon-rust

A fully-featured, no-std implementation of the Falcon post-quantum digital signature scheme. This crate provides:

- Falcon512 (108-bit quantum security)
- Falcon1024 (252-bit quantum security)
- Key generation, signing, and verification
- Serialization/deserialization
- No heap allocations required

**Status**: ✅ Fully implemented

### stm32

STM32 firmware that demonstrates using Falcon512 for signing operations. Includes:

- Basic signing example
- Memory configuration for STM32F4
- Build configuration for ARM Cortex-M4F
- Comprehensive documentation

**Status**: ✅ Basic implementation complete

## Quick Start

### Build Everything

```bash
# Build all workspace members
cargo build --release

# Build only falcon-rust
cargo build -p falcon-rust --release

# Build only STM32 firmware
cargo build -p falcon512-stm32 --release
```

### Build STM32 Firmware

```bash
# Install ARM target
rustup target add thumbv7em-none-eabihf

# Build and flash (requires probe-rs)
cd stm32
cargo run --release
```

### Run Tests

```bash
# Test the falcon-rust library
cargo test -p falcon-rust

# Run with standard library features
cd falcon-rust
cargo test
```

## Hardware Requirements

- **STM32 MCU**: F4 series or higher recommended
- **Flash**: 200-500 KB
- **RAM**: 128 KB+ (64 KB minimum)
- **Debug Probe**: ST-LINK, J-Link, or compatible

## Development Setup

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add ARM target
rustup target add thumbv7em-none-eabihf

# Install probe-rs for flashing/debugging
cargo install probe-rs --features cli

# Install cargo-binutils for binary inspection
cargo install cargo-binutils
rustup component add llvm-tools-preview
```

### Build Tools

```bash
# Build workspace
cargo build

# Check code
cargo check --workspace

# Format code
cargo fmt --all

# Run clippy
cargo clippy --workspace
```

## Documentation

- **falcon-rust**: See `falcon-rust/` directory and [crates.io](https://crates.io/crates/falcon-rust)
- **STM32 firmware**: See `stm32/README.md`

## Memory Considerations

Falcon512 operations require significant resources:

- **Stack**: 20-32 KB during signing
- **Keys**: ~2.2 KB total (secret + public)
- **Signature**: ~666 bytes
- **Code size**: 200-500 KB depending on optimization

Choose STM32 chips with adequate memory (F4 series recommended).

## Performance Notes

- Build with `--release` for production use
- Enable FPU support (`thumbv7em-none-eabihf` target)
- LTO is enabled for size optimization
- Signing takes several seconds on typical STM32F4 @ 168 MHz

## Use Cases

- Secure firmware updates
- Cryptographic authentication
- Post-quantum security for IoT devices
- Secure boot implementations
- Digital signatures for sensor data

## Contributing

When contributing:

1. Maintain no-std compatibility in `falcon-rust`
2. Test on actual hardware when modifying `stm32`
3. Update memory requirements if they change
4. Document any breaking changes

## License

- **falcon-rust**: MIT License
- **stm32**: Same as falcon-rust (MIT)

## Resources

- [Falcon Signature Scheme](https://falcon-sign.info/)
- [NIST PQC Project](https://csrc.nist.gov/projects/post-quantum-cryptography)
- [Embedded Rust Book](https://rust-embedded.github.io/book/)
- [cortex-m Quickstart](https://github.com/rust-embedded/cortex-m-quickstart)

## Troubleshooting

### Stack Overflow

If you encounter stack overflow during signing:
- Increase `_stack_size` in `stm32/memory.x`
- Use a chip with more RAM
- Reduce stack usage elsewhere in your application

### Out of Memory

- Use `opt-level = "z"` for smaller code size
- Enable LTO in release profile
- Remove unused features/dependencies

### Build Errors

```bash
# Clean and rebuild
cargo clean
cargo build --release

# Update dependencies
cargo update
```

## Status

- ✅ Workspace structure configured
- ✅ falcon-rust library complete
- ✅ Basic STM32 signing example
- 🔄 Hardware RNG integration (example provided)
- 🔄 UART/serial output (example provided)
- 🔄 Real-world application examples (TBD)

## Future Work

- Add examples for different STM32 families
- Implement persistent key storage
- Add UART communication examples
- Performance benchmarking on real hardware
- Power consumption measurements
- Integration with bootloader
