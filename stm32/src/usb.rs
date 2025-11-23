//! USB module - handles USB communication and message protocol

use heapless::Vec as HVec;
use rtt_target::rprintln;
use usbd_serial::SerialPort;

use crate::signing::{format_hex, MAX_MESSAGE_SIZE};

/// USB message handler for receiving and sending messages
pub struct UsbMessageHandler {
    message_buffer: HVec<u8, MAX_MESSAGE_SIZE>,
}

impl UsbMessageHandler {
    /// Create a new USB message handler
    pub fn new() -> Self {
        Self {
            message_buffer: HVec::new(),
        }
    }

    /// Try to read a message from USB serial port with improved error handling
    /// Returns Some(message) if a complete message was received, None otherwise
    pub fn try_read_message<'a, B: usb_device::bus::UsbBus>(
        &mut self,
        serial: &mut SerialPort<'a, B>,
    ) -> Option<&[u8]> {
        let mut buf = [0u8; 64];
        match serial.read(&mut buf) {
            Ok(count) if count > 0 => {
                rprintln!("Received {} bytes via USB", count);

                // Append to message buffer with bounds checking
                for i in 0..count {
                    if self.message_buffer.push(buf[i]).is_err() {
                        rprintln!("ERROR: Message too large! Clearing buffer.");
                        self.message_buffer.clear();
                        return None;
                    }
                }

                // Check for newline (message complete)
                if buf[..count].contains(&b'\n') || buf[..count].contains(&b'\r') {
                    // Remove trailing newline/carriage return
                    while self.message_buffer.last() == Some(&b'\n')
                        || self.message_buffer.last() == Some(&b'\r')
                        || self.message_buffer.last() == Some(&b' ')
                        || self.message_buffer.last() == Some(&b'\t')
                    {
                        self.message_buffer.pop();
                    }

                    if !self.message_buffer.is_empty() {
                        rprintln!("Message complete: {} bytes", self.message_buffer.len());
                        // Log first few bytes for debugging
                        if self.message_buffer.len() >= 8 {
                            rprintln!(
                                "Message starts with: {:02x} {:02x} {:02x} {:02x}...",
                                self.message_buffer[0],
                                self.message_buffer[1],
                                self.message_buffer[2],
                                self.message_buffer[3]
                            );
                        }
                        return Some(&self.message_buffer);
                    } else {
                        rprintln!("Empty message received, ignoring");
                        self.message_buffer.clear();
                    }
                }
            }
            Ok(0) => {
                // No data available - this is normal
            }
            Ok(_) => {
                // This handles any other Ok(count) values that might occur
                // Should not happen in practice but satisfies the compiler
            }
            Err(usb_device::UsbError::WouldBlock) => {
                // No data available - this is normal
            }
            Err(e) => {
                rprintln!("USB read error: {:?}", e);
                // Clear buffer on error to prevent corruption
                self.message_buffer.clear();
            }
        }
        None
    }

    /// Clear the message buffer
    pub fn clear_buffer(&mut self) {
        self.message_buffer.clear();
    }

    /// Get a reference to the current message buffer
    pub fn get_message(&self) -> &[u8] {
        &self.message_buffer
    }

    /// Send a signed response via USB with improved error handling and chunking
    pub fn send_signed_response<'a, B: usb_device::bus::UsbBus>(
        &self,
        serial: &mut SerialPort<'a, B>,
        message: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) {
        rprintln!("Sending response...");
        rprintln!(
            "Message: {} bytes, Signature: {} bytes, PubKey: {} bytes",
            message.len(),
            signature.len(),
            public_key.len()
        );

        // Helper function to send data in chunks with error handling
        let send_chunked = |serial: &mut SerialPort<'a, B>, data: &[u8]| -> bool {
            const CHUNK_SIZE: usize = 32; // Send in smaller chunks for reliability
            let mut offset = 0;

            while offset < data.len() {
                let end = core::cmp::min(offset + CHUNK_SIZE, data.len());
                let chunk = &data[offset..end];

                match serial.write(chunk) {
                    Ok(written) => {
                        if written == 0 {
                            rprintln!("USB write returned 0 bytes, retrying...");
                            return false;
                        }
                        offset += written;
                    }
                    Err(usb_device::UsbError::WouldBlock) => {
                        // Buffer full, wait a bit
                        for _ in 0..1000 {
                            cortex_m::asm::nop();
                        }
                        continue;
                    }
                    Err(e) => {
                        rprintln!("USB write error: {:?}", e);
                        return false;
                    }
                }

                // Small delay between chunks
                for _ in 0..100 {
                    cortex_m::asm::nop();
                }
            }
            true
        };

        // Send response header
        let header = b"SIGNED:\n";
        if !send_chunked(serial, header) {
            rprintln!("Failed to send header");
            return;
        }

        // Send original message
        if !send_chunked(serial, message) {
            rprintln!("Failed to send message");
            return;
        }

        if !send_chunked(serial, b"\nSIGNATURE:\n") {
            rprintln!("Failed to send signature header");
            return;
        }

        // Send signature (hex encoded for readability) in chunks
        let mut hex_buffer = [0u8; 64]; // Buffer for hex chunks
        let mut hex_pos = 0;

        for byte in signature.iter() {
            let hex = format_hex(*byte);
            if hex_pos + 2 >= hex_buffer.len() {
                // Send current buffer
                if !send_chunked(serial, &hex_buffer[..hex_pos]) {
                    rprintln!("Failed to send signature chunk");
                    return;
                }
                hex_pos = 0;
            }
            hex_buffer[hex_pos] = hex[0];
            hex_buffer[hex_pos + 1] = hex[1];
            hex_pos += 2;
        }

        // Send remaining hex data
        if hex_pos > 0 {
            if !send_chunked(serial, &hex_buffer[..hex_pos]) {
                rprintln!("Failed to send final signature chunk");
                return;
            }
        }

        if !send_chunked(serial, b"\nPUBLIC_KEY:\n") {
            rprintln!("Failed to send public key header");
            return;
        }

        // Send public key (hex encoded for readability) in chunks
        hex_pos = 0;
        for byte in public_key.iter() {
            let hex = format_hex(*byte);
            if hex_pos + 2 >= hex_buffer.len() {
                // Send current buffer
                if !send_chunked(serial, &hex_buffer[..hex_pos]) {
                    rprintln!("Failed to send public key chunk");
                    return;
                }
                hex_pos = 0;
            }
            hex_buffer[hex_pos] = hex[0];
            hex_buffer[hex_pos + 1] = hex[1];
            hex_pos += 2;
        }

        // Send remaining hex data
        if hex_pos > 0 {
            if !send_chunked(serial, &hex_buffer[..hex_pos]) {
                rprintln!("Failed to send final public key chunk");
                return;
            }
        }

        // Send final newline
        if !send_chunked(serial, b"\n") {
            rprintln!("Failed to send final newline");
            return;
        }

        rprintln!("Response sent successfully via USB");
    }
}
