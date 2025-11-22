# STM32 Falcon512 Signer - Frontend

A simple web interface for signing messages using an STM32 microcontroller with Falcon512 post-quantum signatures.

## Quick Start

```bash
# Install dependencies
npm install

# Start development server
npm run dev
```

Then open http://localhost:5173 in Chrome, Edge, or Opera.

## Usage

1. **Connect your STM32 device**
   - Connect the STM32H750B-DK board via USB (CN13 connector)
   - Make sure the firmware is running

2. **Connect in browser**
   - Click "Connect to STM32"
   - The browser will show **only your STM32 device** (VID: 0x0483, PID: 0x5740)
   - If no device appears, make sure the STM32 is plugged in and firmware is running
   - You should see "✅ Connected to STM32!" with VID/PID confirmation

3. **Sign a message**
   - Enter any text message (default: "Hello, STM32!")
   - Click "Sign Message"
   - Press button B0 on the STM32 board when prompted
   - Wait for the signature to appear

## Browser Requirements

**Supported browsers:**
- Chrome 89+
- Edge 89+
- Opera 75+

**Not supported:**
- Firefox (no Web Serial API)
- Safari (no Web Serial API)

## Protocol

The frontend uses a simple text-based protocol:

**Send:** `message\n` (message with newline)

**Receive:**
```
SIGNATURE: [hex signature]
PUBLIC_KEY: [hex public key]
```

## Troubleshooting

### "Web Serial API Not Supported"
Use Chrome, Edge, or Opera browser.

### No device appears in the port selector

The frontend is configured to show **only** the STM32 device (VID: 0x0483, PID: 0x5740).

If nothing appears:
1. Make sure the STM32H750B-DK is connected via USB (CN13 connector)
2. Verify the firmware is running (check RTT logs or LED indicators)
3. Close any other programs using the serial port (like the Rust usb-client)
4. Try unplugging and replugging the USB cable
5. On Linux, you may need permissions: `sudo usermod -a -G dialout $USER` (then log out/in)

### Connection opens but no response
1. Make sure the STM32 firmware is running
2. Check that you're using the correct USB port (CN13 on STM32H750B-DK)
3. Try disconnecting and reconnecting the USB cable
4. Press the reset button on the STM32 board

### Timeout waiting for response
You need to press button B0 on the STM32 board to trigger the signing operation.

## Development

The app is built with:
- React + TypeScript
- Vite
- Web Serial API

Main file: `src/App.tsx` - Contains all the logic in a single component for simplicity.

## License

Part of the falcon512-stm32 repository.
