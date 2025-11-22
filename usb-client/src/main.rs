use anyhow::{Context, Result};
use clap::Parser;
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

    // Parse and display signature
    if let Some(sig_start) = response.find("SIGNATURE:") {
        let signature = &response[sig_start + 10..].trim();
        println!("\n🔐 Signature (hex):");
        println!("{}", signature);
        println!("\n✅ Message successfully signed!");
    } else {
        println!("⚠️  Warning: Could not parse signature from response");
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

    // Look for STM32 VID (0x0483) or common USB-serial adapters
    for port in &ports {
        if let serialport::SerialPortType::UsbPort(info) = &port.port_type {
            // STM32 VID
            if info.vid == 0x0483 {
                println!("🔍 Auto-detected STM32 device: {}", port.port_name);
                return Ok(port.port_name.clone());
            }
        }
    }

    // If no STM32 found, try to use the first available port
    if let Some(port) = ports.first() {
        println!(
            "⚠️  No STM32 device found, using first available port: {}",
            port.port_name
        );
        return Ok(port.port_name.clone());
    }

    anyhow::bail!("No serial ports found. Please specify port with --port option");
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

                // Check if we have a complete response (ends with newline after signature)
                if response.contains("SIGNATURE:") && response.ends_with('\n') {
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
