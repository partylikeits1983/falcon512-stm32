# falcon512-stm32

STM32 firmware for signing data using the Falcon512 post-quantum signature scheme.

## Overview

This crate provides a basic implementation of Falcon512 signing for STM32 microcontrollers. It demonstrates:

- Key generation using Falcon512
- Message signing with the generated private key
- Signature verification with the public key
- Proper no-std embedded environment setup

## Hardware Requirements

This firmware is configured for STM32F4 series microcontrollers (specifically STM32F407), but can be adapted for other STM32 families.

**Memory Requirements:**
- **Flash**: ~200-500 KB (depending on optimization)
- **RAM**: Minimum 64 KB, 128 KB+ recommended
- **Stack**: 32 KB+ (Falcon operations are stack-intensive)

## Configuration

### For Different STM32 Chips

1. **Update `Cargo.toml`** - Uncomment and modify the HAL dependency:
   ```toml
   stm32f4xx-hal = { version = "0.21", features = ["stm32f407"] }
   ```

2. **Update `.cargo/config.toml`** - Set the correct target:
   - STM32F0/L0: `thumbv6m-none-eabi`
   - STM32F1/F2/L1: `thumbv7m-none-eabi`
   - STM32F3/F4/L4: `thumbv7em-none-eabihf`
   - STM32F7/H7: `thumbv7em-none-eabihf`

3. **Update `memory.x`** - Set correct flash and RAM sizes for your chip

## Building

### Prerequisites

Install the ARM toolchain:

```bash
# Add ARM target
rustup target add thumbv7em-none-eabihf

# Install cargo-binutils for inspecting binaries
cargo install cargo-binutils
rustup component add llvm-tools-preview

# Optional: Install probe-rs for flashing/debugging
cargo install probe-rs --features cli
```

### Build Commands

```bash
# Build the firmware
cargo build --release

# Check binary size
cargo size --release -- -A

# Build with verbose output
cargo build --release -v
```

## Flashing

### Using probe-rs (Recommended)

```bash
cargo run --release
```

### Using OpenOCD

```bash
openocd -f interface/stlink.cfg -f target/stm32f4x.cfg -c "program target/thumbv7em-none-eabihf/release/falcon512-stm32 verify reset exit"
```

### Using st-flash

```bash
st-flash write target/thumbv7em-none-eabihf/release/falcon512-stm32 0x8000000
```

## Using Hardware RNG

The example uses a deterministic RNG (`ChaCha20Rng`) for demonstration. In production, use your STM32's hardware RNG:

```rust
// Example for STM32F4
use stm32f4xx_hal::{prelude::*, rng::Rng};

let dp = stm32f4xx_hal::pac::Peripherals::take().unwrap();
let rcc = dp.RCC.constrain();
let clocks = rcc.cfgr.freeze();
let mut hw_rng = Rng::new(dp.RNG, &clocks);

// Use hw_rng with Falcon
let signature = falcon512::sign_with_rng(message, &secret_key, &mut hw_rng);
```

## Memory Usage

Falcon512 has significant memory requirements:

- **Secret Key**: ~1,281 bytes
- **Public Key**: ~897 bytes
- **Signature**: ~666 bytes (variable)
- **Stack Usage**: 20-32 KB during signing operations

Ensure your STM32 has sufficient RAM and configure an adequate stack size in `memory.x`.

## Performance Considerations

1. **Optimization**: Always build with `--release` for reasonable performance
2. **FPU**: Use the `eabihf` target for chips with FPU support
3. **LTO**: Enabled in release profile for better optimization
4. **Code Size**: Use `opt-level = "z"` if code size is critical

## Debugging

### Using probe-rs

```bash
probe-rs run --chip STM32F407VGTx target/thumbv7em-none-eabihf/release/falcon512-stm32
```

### Using GDB with OpenOCD

```bash
# Terminal 1: Start OpenOCD
openocd -f interface/stlink.cfg -f target/stm32f4x.cfg

# Terminal 2: Connect with GDB
arm-none-eabi-gdb target/thumbv7em-none-eabihf/release/falcon512-stm32
(gdb) target remote :3333
(gdb) monitor reset halt
(gdb) load
(gdb) continue
```

## Integration Example

For a complete integration with UART output:

```rust
use stm32f4xx_hal::{prelude::*, serial::{config::Config, Serial}};

let tx_pin = gpioa.pa2.into_alternate();
let rx_pin = gpioa.pa3.into_alternate();

let serial = Serial::new(
    dp.USART2,
    (tx_pin, rx_pin),
    Config::default().baudrate(115200.bps()),
    &clocks,
).unwrap();

let (mut tx, _rx) = serial.split();

// Output signing results
writeln!(tx, "Signature valid: {}", is_valid).unwrap();
```

## License

This crate follows the same license as the falcon-rust library (MIT).

## Resources

- [Falcon Specification](https://falcon-sign.info/falcon.pdf)
- [cortex-m-rt Documentation](https://docs.rs/cortex-m-rt/)
- [Embedded Rust Book](https://rust-embedded.github.io/book/)
- [STM32F4 HAL Documentation](https://docs.rs/stm32f4xx-hal/)
