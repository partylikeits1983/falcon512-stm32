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

    /// Try to read a message from USB serial port
    /// Returns Some(message) if a complete message was received, None otherwise
    pub fn try_read_message<'a, B: usb_device::bus::UsbBus>(
        &mut self,
        serial: &mut SerialPort<'a, B>,
    ) -> Option<&[u8]> {
        let mut buf = [0u8; 64];
        match serial.read(&mut buf) {
            Ok(count) if count > 0 => {
                rprintln!("Received {} bytes via USB", count);

                // Append to message buffer
                for i in 0..count {
                    if self.message_buffer.push(buf[i]).is_err() {
                        rprintln!("ERROR: Message too large!");
                        self.message_buffer.clear();
                        return None;
                    }
                }

                // Check for newline (message complete)
                if buf[..count].contains(&b'\n') || buf[..count].contains(&b'\r') {
                    // Remove trailing newline/carriage return
                    while self.message_buffer.last() == Some(&b'\n')
                        || self.message_buffer.last() == Some(&b'\r')
                    {
                        self.message_buffer.pop();
                    }

                    if !self.message_buffer.is_empty() {
                        rprintln!("Message complete: {} bytes", self.message_buffer.len());
                        return Some(&self.message_buffer);
                    } else {
                        self.message_buffer.clear();
                    }
                }
            }
            _ => {}
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

    /// Send a signed response via USB
    pub fn send_signed_response<'a, B: usb_device::bus::UsbBus>(
        &self,
        serial: &mut SerialPort<'a, B>,
        message: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) {
        rprintln!("Sending response...");
        rprintln!("Signature size: {} bytes", signature.len());

        // Send response header
        let header = b"SIGNED:\n";
        let _ = serial.write(header);

        // Send original message
        let _ = serial.write(message);
        let _ = serial.write(b"\nSIGNATURE:\n");

        // Send signature (hex encoded for readability)
        for byte in signature.iter() {
            let hex = format_hex(*byte);
            let _ = serial.write(&hex);
        }
        let _ = serial.write(b"\nPUBLIC_KEY:\n");

        // Send public key (hex encoded for readability)
        for byte in public_key.iter() {
            let hex = format_hex(*byte);
            let _ = serial.write(&hex);
        }
        let _ = serial.write(b"\n");

        rprintln!("Response sent via USB");
    }
}
