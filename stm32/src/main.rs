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
use stm32h7xx_hal::usb_hs::{UsbBus, USB2};
use usb_device::device::{UsbDeviceBuilder, UsbDeviceState};
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

// Set up the global allocator for heap allocations
use embedded_alloc::Heap;

#[global_allocator]
static HEAP: Heap = Heap::empty();

// Import keys from separate module
mod keys;
use keys::{PK_BYTES, SK_BYTES};

// Import signing and USB modules
mod signing;
mod usb;

use signing::Signer;
use usb::UsbMessageHandler;

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
}

// USB connection state tracking
#[derive(PartialEq)]
enum UsbConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Suspended,
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

    // Enhanced USB power and hardware configuration for USB-C only operation
    rprintln!("Configuring USB power and hardware for USB-C standalone...");
    unsafe {
        let pwr = &*pac::PWR::ptr();
        let rcc = &*pac::RCC::ptr();

        // Enable USB regulator - CRITICAL for USB-C operation
        pwr.cr3.modify(|_, w| w.usbregen().set_bit());

        // Wait for USB regulator to be ready with timeout
        let mut timeout_counter = 0u32;
        let max_timeout = 100000;
        while !pwr.cr3.read().usb33rdy().bit_is_set() && timeout_counter < max_timeout {
            cortex_m::asm::nop();
            timeout_counter += 1;
        }

        if timeout_counter >= max_timeout {
            rprintln!("WARNING: USB regulator timeout - this may prevent USB-C operation");
        } else {
            rprintln!("✅ USB regulator ready for USB-C operation");
        }

        // Ensure HSI48 is stable - required for USB clock
        let mut hsi48_timeout = 0u32;
        while !rcc.cr.read().hsi48rdy().bit_is_set() && hsi48_timeout < 50000 {
            cortex_m::asm::nop();
            hsi48_timeout += 1;
        }

        if hsi48_timeout >= 50000 {
            rprintln!("WARNING: HSI48 not stable - USB may not work");
        } else {
            rprintln!("✅ HSI48 clock stable for USB");
        }

        // Enable USB OTG FS clock (USB2 on H7)
        rcc.ahb1enr.modify(|_, w| w.usb2otgen().set_bit());

        // Small delay for hardware stabilization
        delay_ms(10);
    }
    rprintln!("✅ USB power and hardware configured for USB-C standalone operation");

    // Setup LED on PE3
    let gpioe = dp.GPIOE.split(ccdr.peripheral.GPIOE);
    let mut led = gpioe.pe3.into_push_pull_output();

    // Setup buttons for WeAct STM32H7 Core Board
    // K1 user button on PC13 (active HIGH - pressed = HIGH)
    let gpioc = dp.GPIOC.split(ccdr.peripheral.GPIOC);
    let button_key = gpioc.pc13.into_pull_down_input();

    // BOOT0 button on PB2 (active HIGH - pressed = HIGH)
    let gpiob = dp.GPIOB.split(ccdr.peripheral.GPIOB);
    let button_boot = gpiob.pb2.into_pull_down_input();

    rprintln!("LED initialized on PE3");
    rprintln!("Buttons initialized: K1(PC13) and BOOT0(PB2) - both active-HIGH");

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
    let rng = ChaCha20Rng::from_seed(seed);

    // Load keys and create signer
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

    // Create signer
    let mut signer = Signer::new(secret_key, rng);

    // Setup USB - USB2 OTG FS on PA11/PA12 (CN13 connector on STM32H750B-DK)
    rprintln!("Initializing USB...");
    rprintln!("Step 1: Splitting GPIOA");
    let gpioa = dp.GPIOA.split(ccdr.peripheral.GPIOA);

    rprintln!("Step 2: Configuring USB pins (PA11=D-, PA12=D+)");
    let usb_dm = gpioa.pa11.into_alternate::<10>();
    let usb_dp = gpioa.pa12.into_alternate::<10>();

    rprintln!("Step 3: Pre-configuring USB hardware for USB-C standalone operation");

    // CRITICAL: Configure USB hardware registers for USB-C only operation
    unsafe {
        let usb_otg_global = &*pac::OTG2_HS_GLOBAL::ptr();

        rprintln!("Step 3a: Forcing device mode for USB-C standalone");
        // Force device mode - MANDATORY for USB-C only operation
        usb_otg_global.gusbcfg.modify(|_, w| w.fdmod().set_bit());
        delay_ms(10);

        rprintln!("Step 3b: Setting USB turnaround time for full-speed");
        // Configure USB turnaround time for full speed (5 AHB clocks)
        usb_otg_global.gusbcfg.modify(|_, w| w.trdt().bits(0x9));

        rprintln!("Step 3c: DISABLING VBUS sensing - KEY for USB-C standalone!");
        // CRITICAL: Disable VBUS sensing for USB-C only operation!
        // This is THE most important setting for USB-C standalone operation
        // Without this, the device will never enumerate when only USB-C is connected
        usb_otg_global.gccfg.modify(|_, w| {
            w.vbden()
                .clear_bit() // DISABLE VBUS detection - critical for USB-C only
                .pwrdwn()
                .clear_bit() // Power up the USB transceiver
        });

        delay_ms(20);
        rprintln!("✅ USB hardware configured for USB-C standalone - VBUS sensing DISABLED");
    }

    rprintln!("Step 4: Creating USB peripheral (USB2 OTG FS)");
    // USB endpoint memory - increased size for better buffering
    static mut EP_MEMORY: [u32; 2048] = [0; 2048];

    // Use USB2 (OTG_FS on PA11/PA12) - this is connected to CN13 on the board
    let usb = USB2::new(
        dp.OTG2_HS_GLOBAL,
        dp.OTG2_HS_DEVICE,
        dp.OTG2_HS_PWRCLK,
        usb_dm,
        usb_dp,
        ccdr.peripheral.USB2OTG,
        &ccdr.clocks,
    );

    rprintln!("Step 5: Creating USB bus");
    #[allow(static_mut_refs)]
    let usb_bus = UsbBus::new(usb, unsafe { &mut EP_MEMORY });

    rprintln!("Step 6: Creating serial port");
    let mut serial = SerialPort::new(&usb_bus);

    rprintln!("Step 7: Building USB device");
    rprintln!("Step 7a: Creating device builder");
    // Use STM32 VID/PID for CDC device (0x0483:0x5740)
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x0483, 0x5740))
        .device_class(USB_CLASS_CDC)
        .build();

    rprintln!("Step 8: USB device built successfully!");

    rprintln!("✅ USB device initialized for USB-C standalone operation!");

    // Optimized USB enumeration for USB-C standalone
    rprintln!("🔄 Starting USB enumeration for USB-C standalone...");

    // Initial delay to allow USB hardware to stabilize
    delay_ms(500);

    // Aggressive USB polling to ensure reliable enumeration with USB-C only
    rprintln!("🔄 Performing intensive USB polling for reliable USB-C enumeration...");
    for i in 0..200 {
        // Increased cycles for better USB-C compatibility
        usb_dev.poll(&mut [&mut serial]);

        // Check if we've enumerated successfully
        if usb_dev.state() == UsbDeviceState::Configured {
            rprintln!("✅ USB enumeration successful after {} cycles!", i + 1);
            break;
        }

        // Very fast polling for USB-C compatibility
        delay_ms(2);

        if i % 50 == 0 {
            rprintln!(
                "USB enumeration cycle {}/200 (state: {:?})",
                i + 1,
                usb_dev.state()
            );
        }
    }

    // Final state check
    match usb_dev.state() {
        UsbDeviceState::Configured => {
            rprintln!("✅ USB-C enumeration SUCCESSFUL - device ready!");
        }
        state => {
            rprintln!(
                "⚠️  USB enumeration incomplete (state: {:?}) - will continue polling in main loop",
                state
            );
        }
    }

    rprintln!("\n🔌 === USB-C STANDALONE MODE ACTIVE ===");
    rprintln!("✅ Device powered from USB-C only (no ST-LINK needed)");
    rprintln!("✅ USB CDC device ready for communication");
    rprintln!("✅ VBUS sensing disabled for USB-C compatibility");
    rprintln!("📱 Send a message via USB to sign with Falcon512");
    rprintln!("🔘 Press K1(PC13) or BOOT0(PB2) button to confirm signing\n");

    // Create USB message handler
    let mut usb_handler = UsbMessageHandler::new();
    let mut state = SigningState::WaitingForMessage;
    let mut usb_state = UsbConnectionState::Disconnected;
    let mut blink_counter = 0u32;
    let mut led_counter = 0u32;
    let mut usb_poll_counter = 0u32;
    let mut last_usb_state = UsbDeviceState::Default;

    loop {
        // Poll USB continuously and frequently - CRITICAL for stable enumeration
        usb_poll_counter += 1;
        usb_dev.poll(&mut [&mut serial]);

        // Check USB device state and handle state changes
        let current_usb_state = usb_dev.state();
        if current_usb_state != last_usb_state {
            rprintln!(
                "USB state changed: {:?} -> {:?}",
                last_usb_state,
                current_usb_state
            );
            last_usb_state = current_usb_state;

            // Update our connection state tracking
            match current_usb_state {
                UsbDeviceState::Default => {
                    if usb_state != UsbConnectionState::Connecting {
                        rprintln!("USB: Connecting...");
                        usb_state = UsbConnectionState::Connecting;
                    }
                }
                UsbDeviceState::Configured => {
                    if usb_state != UsbConnectionState::Connected {
                        rprintln!("USB: Connected and configured!");
                        usb_state = UsbConnectionState::Connected;
                        // Reset message handler on new connection
                        usb_handler.clear_buffer();
                    }
                }
                UsbDeviceState::Suspend => {
                    rprintln!("USB: Suspended");
                    usb_state = UsbConnectionState::Suspended;
                }
                _ => {
                    if usb_state != UsbConnectionState::Disconnected {
                        rprintln!("USB: Disconnected");
                        usb_state = UsbConnectionState::Disconnected;
                        // Clear any pending messages on disconnect
                        usb_handler.clear_buffer();
                        state = SigningState::WaitingForMessage;
                    }
                }
            }
        }

        // Handle USB suspend/resume for better power management
        if usb_state == UsbConnectionState::Suspended {
            // In suspend state, poll less frequently to save power
            if usb_poll_counter % 1000 == 0 {
                // Check if we've resumed
                continue;
            }
        }

        // USB-C standalone status indication via LED
        if usb_poll_counter % 25000 == 0 {
            match usb_state {
                UsbConnectionState::Disconnected => {
                    // Slow blink: Device powered from USB-C but not enumerated yet
                    if (usb_poll_counter / 25000) % 4 < 2 {
                        led.set_high();
                    } else {
                        led.set_low();
                    }
                }
                UsbConnectionState::Connecting => {
                    // Fast blink: USB-C enumeration in progress
                    led.toggle();
                }
                UsbConnectionState::Suspended => {
                    // Very slow pulse: USB suspended (host may be sleeping)
                    if (usb_poll_counter / 25000) % 8 < 1 {
                        led.set_high();
                    } else {
                        led.set_low();
                    }
                }
                UsbConnectionState::Connected => {
                    // LED behavior handled by signing state machine
                }
            }
        }

        // Allow message processing when USB is connected OR connecting (more permissive)
        // This allows the device to work even during USB enumeration
        let usb_ready = usb_state == UsbConnectionState::Connected
            || usb_state == UsbConnectionState::Connecting;

        if !usb_ready {
            continue;
        }

        match state {
            SigningState::WaitingForMessage => {
                // Try to read from USB
                if let Some(_message) = usb_handler.try_read_message(&mut serial) {
                    rprintln!("Message received! Waiting for button press to sign...");
                    state = SigningState::MessageReceived;
                    blink_counter = 0;
                }

                // Optimized blink when waiting and connected (non-blocking)
                led_counter += 1;
                if led_counter % 30000 == 0 {
                    // Faster blink for better responsiveness
                    led.toggle();
                }
            }

            SigningState::MessageReceived => {
                // Continue USB polling during button wait
                if usb_poll_counter % 100 == 0 {
                    // Check if USB disconnected during button wait (more permissive)
                    if usb_state == UsbConnectionState::Disconnected {
                        rprintln!("USB disconnected during button wait, resetting...");
                        usb_handler.clear_buffer();
                        state = SigningState::WaitingForMessage;
                        continue;
                    }
                }

                // Flash LED rapidly until button press (optimized)
                blink_counter += 1;
                if blink_counter % 5000 == 0 {
                    // Faster flashing for better user feedback
                    led.toggle();
                }

                // Debug: Print button states periodically
                if blink_counter % 100000 == 0 {
                    rprintln!(
                        "Button states - K1(PC13): {}, BOOT0(PB2): {}",
                        if button_key.is_high() {
                            "PRESSED"
                        } else {
                            "not pressed"
                        },
                        if button_boot.is_high() {
                            "PRESSED"
                        } else {
                            "not pressed"
                        }
                    );
                }

                // Check both buttons (active HIGH - pressed = HIGH)
                if button_key.is_high() || button_boot.is_high() {
                    let btn_name = if button_key.is_high() {
                        "K1(PC13)"
                    } else {
                        "BOOT0(PB2)"
                    };
                    rprintln!("Button {} pressed! Starting signing...", btn_name);

                    // Confirmation blinks: 3 quick blinks with optimized USB polling
                    for _ in 0..3 {
                        led.set_high();
                        for _ in 0..5000 {
                            // Reduced for faster blinks
                            usb_dev.poll(&mut [&mut serial]);
                            cortex_m::asm::nop();
                        }
                        led.set_low();
                        for _ in 0..5000 {
                            // Reduced for faster blinks
                            usb_dev.poll(&mut [&mut serial]);
                            cortex_m::asm::nop();
                        }
                    }

                    state = SigningState::Signing;
                    led.set_high(); // LED stays on during signing
                }
            }

            SigningState::Signing => {
                // Verify USB is still connected before signing (more permissive)
                if usb_state == UsbConnectionState::Disconnected {
                    rprintln!("USB disconnected during signing, aborting...");
                    usb_handler.clear_buffer();
                    state = SigningState::WaitingForMessage;
                    led.set_low();
                    continue;
                }

                // Get the message from the handler's buffer
                let message = usb_handler.get_message();

                // Sign the message (LED is already on)
                rprintln!("Signing message of {} bytes...", message.len());
                let sig_bytes = signer.sign_message(message);

                // Send response via USB with retry logic
                let mut send_attempts = 0;
                let max_attempts = 3;
                while send_attempts < max_attempts {
                    // Check USB connection before sending (more permissive)
                    if usb_state == UsbConnectionState::Disconnected {
                        rprintln!("USB disconnected before sending response");
                        break;
                    }

                    usb_handler.send_signed_response(&mut serial, message, &sig_bytes, &PK_BYTES);

                    // Give time for data to be sent and poll USB
                    for _ in 0..1000 {
                        usb_dev.poll(&mut [&mut serial]);
                        cortex_m::asm::nop();
                    }

                    send_attempts += 1;
                    if send_attempts < max_attempts {
                        rprintln!("Retrying response send (attempt {})", send_attempts + 1);
                        delay_ms(10);
                    }
                    break; // For now, don't retry - just send once
                }

                // Clear buffer and return to waiting
                usb_handler.clear_buffer();
                state = SigningState::WaitingForMessage;

                // Success: LED off, then 3 optimized blinks with continuous USB polling
                led.set_low();
                for _ in 0..3 {
                    for _ in 0..100000 {
                        // Reduced for faster completion
                        usb_dev.poll(&mut [&mut serial]);
                        cortex_m::asm::nop();
                    }
                    led.set_high();
                    for _ in 0..100000 {
                        // Reduced for faster completion
                        usb_dev.poll(&mut [&mut serial]);
                        cortex_m::asm::nop();
                    }
                    led.set_low();
                }

                rprintln!("Signing complete! Ready for next message\n");
            }
        }
    }
}
