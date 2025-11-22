#![no_std]
#![no_main]

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use cortex_m_rt::entry;
use falcon_rust::falcon512;
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;
use stm32h7xx_hal::{pac, prelude::*, rcc::rec::UsbClkSel};

// USB imports
use heapless::Vec as HVec;
use stm32h7xx_hal::usb_hs::{UsbBus, USB1};
use usb_device::device::UsbDeviceBuilder;
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

// Set up the global allocator for heap allocations
use embedded_alloc::Heap;

#[global_allocator]
static HEAP: Heap = Heap::empty();

// Import keys from separate module
mod keys;
use keys::SK_BYTES;

// Maximum message size (adjust as needed)
const MAX_MESSAGE_SIZE: usize = 512;

// Simple delay function
fn delay_ms(ms: u32) {
    for _ in 0..(ms * 10000) {
        cortex_m::asm::nop();
    }
}

// State machine for USB message handling
enum SigningState {
    WaitingForMessage,
    MessageReceived,
    Signing,
    SignatureReady,
}

/// USB-based Falcon512 signing with button confirmation
///
/// Workflow:
/// 1. Listen for message from USB
/// 2. On USB message received, flash LED rapidly until button click
/// 3. On button click, sign the message
/// 4. Send signed message back via USB
#[entry]
fn main() -> ! {
    // Initialize RTT for debug output
    rtt_init_print!();
    rprintln!("=== STM32H743 Falcon512 USB Signing System ===");
    rprintln!("Initializing...");

    // Get device peripherals
    let dp = pac::Peripherals::take().unwrap();
    let _cp = cortex_m::Peripherals::take().unwrap();

    let pwr = dp.PWR.constrain();
    let pwrcfg = pwr.freeze();
    let rcc = dp.RCC.constrain();
    let mut ccdr = rcc.sys_ck(200.MHz()).freeze(pwrcfg, &dp.SYSCFG);

    // Configure USB clock - use HSI48 (internal 48MHz oscillator)
    rprintln!("Configuring USB clock...");
    let _ = ccdr.clocks.hsi48_ck().expect("HSI48 must run");
    ccdr.peripheral.kernel_usb_clk_mux(UsbClkSel::Hsi48);
    rprintln!("USB clock configured (HSI48)");

    // Enable USB voltage regulator (required for some H7 variants)
    rprintln!("Configuring USB power...");
    unsafe {
        let pwr = &*pac::PWR::ptr();
        // Try to enable USB regulator if available
        pwr.cr3.modify(|_, w| w.usbregen().set_bit());
        // Small delay to let it stabilize
        delay_ms(10);
    }
    rprintln!("USB power configured");

    // Setup LED on PE3
    let gpioe = dp.GPIOE.split(ccdr.peripheral.GPIOE);
    let mut led = gpioe.pe3.into_push_pull_output();

    // Setup button B0 (User button)
    // Common pins: PC13 on most STM32H7 Nucleo/Discovery boards
    // If B0 is on a different pin on your board, change pc13 to the correct pin
    // The button is typically active-low (pressed = low voltage)
    let gpioc = dp.GPIOC.split(ccdr.peripheral.GPIOC);
    let button = gpioc.pc13.into_pull_up_input();

    rprintln!("LED initialized on PE3");
    rprintln!("Button B0 (User) initialized on PC13");

    // Startup blink
    for _ in 0..2 {
        led.set_high();
        delay_ms(100);
        led.set_low();
        delay_ms(100);
    }

    // Initialize heap allocator (384KB for Falcon512)
    rprintln!("Setting up heap allocator (384KB)...");
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 384 * 1024;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe {
            let heap_ptr = core::ptr::addr_of_mut!(HEAP_MEM);
            HEAP.init((*heap_ptr).as_mut_ptr() as usize, HEAP_SIZE)
        }
    }
    rprintln!("Heap initialized");

    // Initialize RNG
    rprintln!("Initializing RNG...");
    let seed = [0x42u8; 32]; // TODO: Use hardware RNG in production
    let mut rng = ChaCha20Rng::from_seed(seed);

    // Load keys
    rprintln!("Loading keys...");
    let secret_key = match falcon512::SecretKey::from_bytes(&SK_BYTES) {
        Ok(sk) => {
            rprintln!("Secret key loaded");
            sk
        }
        Err(_) => {
            rprintln!("ERROR: Failed to load secret key");
            loop {
                cortex_m::asm::nop();
            }
        }
    };

    // Setup USB - USB1 OTG FS on PA11/PA12 (CN13 connector on STM32H750B-DK)
    rprintln!("Initializing USB...");
    rprintln!("Step 1: Splitting GPIOA");
    let gpioa = dp.GPIOA.split(ccdr.peripheral.GPIOA);

    rprintln!("Step 2: Configuring USB pins (PA11=D-, PA12=D+)");
    let usb_dm = gpioa.pa11.into_alternate::<10>();
    let usb_dp = gpioa.pa12.into_alternate::<10>();

    rprintln!("Step 3: Creating USB peripheral (USB1 OTG FS)");
    // USB endpoint memory
    static mut EP_MEMORY: [u32; 1024] = [0; 1024];

    // Use USB1 (OTG FS) peripheral - this is connected to CN13 on the board
    let usb = USB1::new(
        dp.OTG1_HS_GLOBAL,
        dp.OTG1_HS_DEVICE,
        dp.OTG1_HS_PWRCLK,
        usb_dm,
        usb_dp,
        ccdr.peripheral.USB1OTG,
        &ccdr.clocks,
    );

    rprintln!("Step 4: Creating USB bus");
    let usb_bus = UsbBus::new(usb, unsafe { &mut EP_MEMORY });

    rprintln!("Step 5: Creating serial port");
    let mut serial = SerialPort::new(&usb_bus);

    rprintln!("Step 6: Building USB device");
    rprintln!("Step 6a: Creating device builder");
    // Use STM32 VID/PID for CDC device (0x0483:0x5740)
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x0483, 0x5740))
        .device_class(USB_CLASS_CDC)
        .build();

    rprintln!("Step 7: USB device built successfully!");

    rprintln!("USB initialized successfully!");

    // Give USB time to enumerate
    rprintln!("Waiting for USB enumeration...");
    delay_ms(2000);

    rprintln!("\n=== Ready to receive messages via USB ===");
    rprintln!("Send a message to sign it with button confirmation\n");

    // Message buffer
    let mut message_buffer: HVec<u8, MAX_MESSAGE_SIZE> = HVec::new();
    let mut state = SigningState::WaitingForMessage;
    let mut blink_counter = 0u32;

    loop {
        // Poll USB
        if !usb_dev.poll(&mut [&mut serial]) {
            continue;
        }

        match state {
            SigningState::WaitingForMessage => {
                // Try to read from USB
                let mut buf = [0u8; 64];
                match serial.read(&mut buf) {
                    Ok(count) if count > 0 => {
                        rprintln!("Received {} bytes via USB", count);

                        // Append to message buffer
                        for i in 0..count {
                            if message_buffer.push(buf[i]).is_err() {
                                rprintln!("ERROR: Message too large!");
                                message_buffer.clear();
                                break;
                            }
                        }

                        // Check for newline (message complete)
                        if buf[..count].contains(&b'\n') || buf[..count].contains(&b'\r') {
                            // Remove trailing newline/carriage return
                            while message_buffer.last() == Some(&b'\n')
                                || message_buffer.last() == Some(&b'\r')
                            {
                                message_buffer.pop();
                            }

                            if !message_buffer.is_empty() {
                                rprintln!("Message complete: {} bytes", message_buffer.len());
                                rprintln!("Waiting for button press to sign...");
                                state = SigningState::MessageReceived;
                                blink_counter = 0;
                            } else {
                                message_buffer.clear();
                            }
                        }
                    }
                    _ => {}
                }

                // Slow blink when waiting
                led.set_high();
                delay_ms(500);
                led.set_low();
                delay_ms(500);
            }

            SigningState::MessageReceived => {
                // Flash LED rapidly until button press
                if blink_counter % 10 == 0 {
                    led.toggle();
                }
                blink_counter += 1;

                // Check button (active low on most Nucleo boards)
                if button.is_low() {
                    rprintln!("Button pressed! Signing message...");
                    state = SigningState::Signing;
                    led.set_high(); // LED on during signing
                    delay_ms(200); // Debounce
                }

                delay_ms(50); // Rapid blink delay
            }

            SigningState::Signing => {
                // Sign the message
                rprintln!("Signing {} byte message...", message_buffer.len());
                let signature = falcon512::sign_with_rng(&message_buffer, &secret_key, &mut rng);
                led.set_low();

                rprintln!("Signature generated!");

                // Prepare response: original message + signature
                let sig_bytes = signature.to_bytes();
                rprintln!("Signature size: {} bytes", sig_bytes.len());

                // Send response header
                let header = b"SIGNED:\n";
                let _ = serial.write(header);

                // Send original message
                let _ = serial.write(&message_buffer);
                let _ = serial.write(b"\nSIGNATURE:\n");

                // Send signature (hex encoded for readability)
                for byte in sig_bytes.iter() {
                    let hex = format_hex(*byte);
                    let _ = serial.write(&hex);
                }
                let _ = serial.write(b"\n");

                rprintln!("Response sent via USB");

                // Clear buffer and return to waiting
                message_buffer.clear();
                state = SigningState::WaitingForMessage;

                // Success blink
                for _ in 0..3 {
                    led.set_high();
                    delay_ms(200);
                    led.set_low();
                    delay_ms(200);
                }

                rprintln!("Ready for next message\n");
            }

            SigningState::SignatureReady => {
                // This state is not used in current implementation
                state = SigningState::WaitingForMessage;
            }
        }
    }
}

// Helper function to format byte as hex
fn format_hex(byte: u8) -> [u8; 2] {
    const HEX_CHARS: &[u8] = b"0123456789ABCDEF";
    [
        HEX_CHARS[(byte >> 4) as usize],
        HEX_CHARS[(byte & 0x0F) as usize],
    ]
}
