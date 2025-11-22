//! Flash Keys Tool
//!
//! This tool creates a binary file containing Falcon512 keys that can be
//! flashed to the reserved flash section on your STM32.
//!
//! Usage:
//!   1. Generate keys: cd keygen && cargo run --release > keys_output.txt
//!   2. Extract the hex arrays from keys_output.txt
//!   3. Run this tool: cargo run --release -- --sk-file sk.bin --pk-file pk.bin
//!   4. Flash the output: probe-rs download --chip STM32H743ZITx --format Bin --base-address 0x080FE000 keys.bin
//!
//! Or use the helper script that does all steps.

use clap::Parser;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to secret key binary file
    #[arg(short, long)]
    sk_file: PathBuf,

    /// Path to public key binary file
    #[arg(short, long)]
    pk_file: PathBuf,

    /// Output binary file (default: keys.bin)
    #[arg(short, long, default_value = "keys.bin")]
    output: PathBuf,
}

const SK_SIZE: usize = 1281;
const PK_SIZE: usize = 897;
const TOTAL_SIZE: usize = 8192; // 8KB reserved section

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("=== Falcon512 Flash Keys Tool ===\n");

    // Read secret key
    println!("Reading secret key from: {}", args.sk_file.display());
    let sk_bytes = fs::read(&args.sk_file)?;
    if sk_bytes.len() != SK_SIZE {
        return Err(format!(
            "Secret key file has wrong size: {} bytes (expected {})",
            sk_bytes.len(),
            SK_SIZE
        )
        .into());
    }

    // Read public key
    println!("Reading public key from: {}", args.pk_file.display());
    let pk_bytes = fs::read(&args.pk_file)?;
    if pk_bytes.len() != PK_SIZE {
        return Err(format!(
            "Public key file has wrong size: {} bytes (expected {})",
            pk_bytes.len(),
            PK_SIZE
        )
        .into());
    }

    // Create output buffer (8KB, initialized to 0xFF like erased flash)
    let mut output_buffer = vec![0xFF; TOTAL_SIZE];

    // Copy keys into buffer
    output_buffer[0..SK_SIZE].copy_from_slice(&sk_bytes);
    output_buffer[SK_SIZE..SK_SIZE + PK_SIZE].copy_from_slice(&pk_bytes);

    // Write output file
    println!("Writing combined keys to: {}", args.output.display());
    let mut file = fs::File::create(&args.output)?;
    file.write_all(&output_buffer)?;

    println!("\n✓ Success! Created {}", args.output.display());
    println!("\nFlash this file to your STM32 using:");
    println!("  probe-rs download --chip STM32H743ZITx \\");
    println!("    --binary-format Bin \\");
    println!("    --base-address 0x080FE000 \\");
    println!("    {}", args.output.display());
    println!("\nOr use OpenOCD:");
    println!("  openocd -f interface/stlink.cfg -f target/stm32h7x.cfg \\");
    println!(
        "    -c \"program {} 0x080FE000 verify reset exit\"",
        args.output.display()
    );

    Ok(())
}
