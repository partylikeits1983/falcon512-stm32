use anyhow::{Context, Result};
use clap::Parser;
use falcon_rust::falcon512;
use serialport::SerialPort;
use std::io::{Read, Write};
use std::time::Duration;

/// USB client for communicating with STM32 Falcon512 signer
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Serial port path (e.g., /dev/ttyACM0 on Linux, COM3 on Windows)
    #[arg(short, long)]
    port: Option<String>,

    /// Message to sign
    #[arg(short, long)]
    message: String,

    /// List available serial ports
    #[arg(short, long)]
    list: bool,

    /// Timeout in seconds
    #[arg(short, long, default_value = "30")]
    timeout: u64,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // List ports if requested
    if args.list {
        list_ports()?;
        return Ok(());
    }

    // Get port path
    let port_path = if let Some(path) = args.port {
        path
    } else {
        // Try to auto-detect STM32 device
        find_stm32_port()?
    };

    println!("🔌 Connecting to STM32 on {}...", port_path);

    // Open serial port
    let mut port = serialport::new(&port_path, 115200)
        .timeout(Duration::from_secs(args.timeout))
        .open()
        .context("Failed to open serial port")?;

    println!("✅ Connected!");
    println!("📤 Sending message: \"{}\"", args.message);

    // Send message with newline
    let message_with_newline = format!("{}\n", args.message);
    port.write_all(message_with_newline.as_bytes())
        .context("Failed to write to serial port")?;
    port.flush().context("Failed to flush serial port")?;

    println!("⏳ Waiting for STM32 to receive message...");
    println!("👆 Press button B0 on the STM32 board to sign the message");

    // Read response
    let response = read_response(&mut port, args.timeout)?;

    println!("\n📥 Received response from STM32:");
    println!("{}", response);

    // Parse signature and public key
    let sig_start = response
        .find("SIGNATURE:")
        .context("Missing SIGNATURE in response - response may be incomplete")?;
    let pk_start = response
        .find("PUBLIC_KEY:")
        .context("Missing PUBLIC_KEY in response - response may be incomplete")?;

    let signature_hex = response[sig_start + 10..pk_start].trim();
    let public_key_hex = response[pk_start + 11..].trim();

    println!("\n🔐 Signature (hex):");
    println!("{}", signature_hex);

    println!("\n🔑 Public Key (hex):");
    println!("{}", public_key_hex);

    // Decode hex signature
    let sig_bytes = hex_decode(signature_hex).with_context(|| {
        format!(
            "Failed to decode signature hex (length: {})",
            signature_hex.len()
        )
    })?;
    let pk_bytes = hex_decode(public_key_hex).with_context(|| {
        format!(
            "Failed to decode public key hex (length: {})",
            public_key_hex.len()
        )
    })?;

    println!(
        "\n📊 Decoded {} signature bytes and {} public key bytes",
        sig_bytes.len(),
        pk_bytes.len()
    );

    // Parse signature and public key
    let signature = falcon512::Signature::from_bytes(&sig_bytes).map_err(|_| {
        anyhow::anyhow!(
            "Failed to parse signature - expected {} bytes, got {}",
            666,
            sig_bytes.len()
        )
    })?;
    let public_key = falcon512::PublicKey::from_bytes(&pk_bytes).map_err(|_| {
        anyhow::anyhow!(
            "Failed to parse public key - expected {} bytes, got {}",
            897,
            pk_bytes.len()
        )
    })?;

    // Verify signature
    println!("\n🔍 Verifying signature...");
    let is_valid = falcon512::verify(args.message.as_bytes(), &signature, &public_key);

    if is_valid {
        println!("✅ Signature verification PASSED!");
        println!("✅ Message successfully signed and verified!");
    } else {
        println!("❌ Signature verification FAILED!");
        anyhow::bail!("Signature verification failed");
    }

    Ok(())
}

fn list_ports() -> Result<()> {
    println!("📋 Available serial ports:");
    let ports = serialport::available_ports().context("Failed to list serial ports")?;

    if ports.is_empty() {
        println!("  No serial ports found");
        return Ok(());
    }

    for port in ports {
        println!(
            "  • {} - {}",
            port.port_name,
            port_type_name(&port.port_type)
        );
    }

    Ok(())
}

fn port_type_name(port_type: &serialport::SerialPortType) -> String {
    match port_type {
        serialport::SerialPortType::UsbPort(info) => {
            format!("USB (VID: {:04x}, PID: {:04x})", info.vid, info.pid)
        }
        serialport::SerialPortType::PciPort => "PCI".to_string(),
        serialport::SerialPortType::BluetoothPort => "Bluetooth".to_string(),
        serialport::SerialPortType::Unknown => "Unknown".to_string(),
    }
}

fn find_stm32_port() -> Result<String> {
    let ports = serialport::available_ports().context("Failed to list serial ports")?;

    // Look for STM32 VID (0x0483) - this matches our firmware's VID/PID
    for port in &ports {
        if let serialport::SerialPortType::UsbPort(info) = &port.port_type {
            // STM32 VID (0x0483) - our firmware uses PID 0x5740 for CDC
            if info.vid == 0x0483 {
                println!(
                    "🔍 Auto-detected STM32 device: {} (VID: {:04x}, PID: {:04x})",
                    port.port_name, info.vid, info.pid
                );
                return Ok(port.port_name.clone());
            }
        }
    }

    // If no STM32 found, list available ports and fail
    println!("⚠️  No STM32 device found (VID: 0x0483)");
    println!("📋 Available ports:");
    for port in &ports {
        if let serialport::SerialPortType::UsbPort(info) = &port.port_type {
            println!(
                "  • {} - VID: {:04x}, PID: {:04x}",
                port.port_name, info.vid, info.pid
            );
        } else {
            println!(
                "  • {} - {}",
                port.port_name,
                port_type_name(&port.port_type)
            );
        }
    }

    anyhow::bail!(
        "No STM32 device found. Please:\n\
                   1. Connect USB cable to CN13 (USB OTG FS) connector on STM32H750B-DK\n\
                   2. Ensure firmware is running (check RTT logs)\n\
                   3. Or specify port manually with --port option"
    );
}

fn read_response(port: &mut Box<dyn SerialPort>, timeout_secs: u64) -> Result<String> {
    let mut response = String::new();
    let mut buffer = [0u8; 1024];
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(timeout_secs);

    loop {
        // Check timeout
        if start.elapsed() > timeout {
            anyhow::bail!("Timeout waiting for response from STM32");
        }

        // Try to read
        match port.read(&mut buffer) {
            Ok(n) if n > 0 => {
                let chunk = String::from_utf8_lossy(&buffer[..n]);
                response.push_str(&chunk);

                // Check if we have a complete response (ends with newline after public key)
                if response.contains("PUBLIC_KEY:") && response.ends_with('\n') {
                    break;
                }
            }
            Ok(_) => {
                // No data available, sleep briefly
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                // Timeout on read, continue waiting
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                return Err(e).context("Error reading from serial port");
            }
        }
    }

    Ok(response)
}

fn hex_decode(hex_str: &str) -> Result<Vec<u8>> {
    let hex_str = hex_str.replace('\n', "").replace('\r', "").replace(' ', "");

    if hex_str.len() % 2 != 0 {
        anyhow::bail!("Hex string has odd length");
    }

    let mut bytes = Vec::with_capacity(hex_str.len() / 2);
    for i in (0..hex_str.len()).step_by(2) {
        let byte_str = &hex_str[i..i + 2];
        let byte = u8::from_str_radix(byte_str, 16)
            .with_context(|| format!("Invalid hex byte: {}", byte_str))?;
        bytes.push(byte);
    }

    Ok(bytes)
}
