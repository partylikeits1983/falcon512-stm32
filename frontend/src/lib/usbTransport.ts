/**
 * USB Transport Layer using Web Serial API
 * Handles communication with STM32 microcontroller over USB CDC
 */

// Protocol constants
const START_BYTE = 0xaa;
const CMD_SIGN_HASH = 0x01;
const CMD_SIGNATURE = 0x02;
const CMD_ERROR = 0xff;

export interface UsbTransportConfig {
  baudRate?: number;
  dataBits?: 7 | 8;
  stopBits?: 1 | 2;
  parity?: 'none' | 'even' | 'odd';
  bufferSize?: number;
  flowControl?: 'none' | 'hardware';
}

const DEFAULT_CONFIG: Required<UsbTransportConfig> = {
  baudRate: 115200,
  dataBits: 8,
  stopBits: 1,
  parity: 'none',
  bufferSize: 255,
  flowControl: 'none',
};

export class UsbTransport {
  private port: SerialPort | null = null;
  private reader: ReadableStreamDefaultReader<Uint8Array> | null = null;
  private writer: WritableStreamDefaultWriter<Uint8Array> | null = null;
  private config: Required<UsbTransportConfig>;
  private readBuffer: Uint8Array = new Uint8Array(0);
  private messageHandlers: ((msg: Uint8Array) => void)[] = [];
  private isReading = false;

  constructor(config: UsbTransportConfig = {}) {
    this.config = { ...DEFAULT_CONFIG, ...config };
  }

  /**
   * Check if Web Serial API is available
   */
  static isSupported(): boolean {
    return 'serial' in navigator;
  }

  /**
   * Connect to STM32 device
   */
  async connect(): Promise<void> {
    if (!UsbTransport.isSupported()) {
      throw new Error('Web Serial API is not supported in this browser. Please use Chrome, Edge, or Opera.');
    }

    try {
      // Request port from user
      this.port = await navigator.serial.requestPort();

      // Open the port with configuration
      await this.port.open({
        baudRate: this.config.baudRate,
        dataBits: this.config.dataBits,
        stopBits: this.config.stopBits,
        parity: this.config.parity,
        bufferSize: this.config.bufferSize,
        flowControl: this.config.flowControl,
      });

      // Get reader and writer
      if (this.port.readable) {
        this.reader = this.port.readable.getReader();
      }
      if (this.port.writable) {
        this.writer = this.port.writable.getWriter();
      }

      // Start reading loop
      this.startReadLoop();
    } catch (error) {
      throw new Error(`Failed to connect: ${error instanceof Error ? error.message : String(error)}`);
    }
  }

  /**
   * Disconnect from device
   */
  async disconnect(): Promise<void> {
    this.isReading = false;

    if (this.reader) {
      try {
        await this.reader.cancel();
        this.reader.releaseLock();
      } catch (e) {
        console.warn('Error releasing reader:', e);
      }
      this.reader = null;
    }

    if (this.writer) {
      try {
        await this.writer.close();
      } catch (e) {
        console.warn('Error closing writer:', e);
      }
      this.writer = null;
    }

    if (this.port) {
      try {
        await this.port.close();
      } catch (e) {
        console.warn('Error closing port:', e);
      }
      this.port = null;
    }

    this.readBuffer = new Uint8Array(0);
    this.messageHandlers = [];
  }

  /**
   * Check if connected
   */
  isConnected(): boolean {
    return this.port !== null && this.reader !== null && this.writer !== null;
  }

  /**
   * Send raw data
   */
  private async send(data: Uint8Array): Promise<void> {
    if (!this.writer) {
      throw new Error('Not connected');
    }

    try {
      await this.writer.write(data);
    } catch (error) {
      throw new Error(`Failed to send data: ${error instanceof Error ? error.message : String(error)}`);
    }
  }

  /**
   * Encode a packet with protocol framing
   * Format: START_BYTE | CMD | LEN_HIGH | LEN_LOW | PAYLOAD | CRC8
   */
  private encodePacket(cmd: number, payload: Uint8Array): Uint8Array {
    const len = payload.length;
    const packet = new Uint8Array(5 + len);
    
    packet[0] = START_BYTE;
    packet[1] = cmd;
    packet[2] = (len >> 8) & 0xff; // Length high byte
    packet[3] = len & 0xff;        // Length low byte
    packet.set(payload, 4);
    
    // Simple CRC8 checksum
    packet[4 + len] = this.calculateCrc8(packet.slice(0, 4 + len));
    
    return packet;
  }

  /**
   * Calculate CRC8 checksum
   */
  private calculateCrc8(data: Uint8Array): number {
    let crc = 0;
    for (let i = 0; i < data.length; i++) {
      crc ^= data[i];
      for (let j = 0; j < 8; j++) {
        if (crc & 0x80) {
          crc = (crc << 1) ^ 0x07;
        } else {
          crc <<= 1;
        }
      }
    }
    return crc & 0xff;
  }

  /**
   * Start continuous read loop
   */
  private async startReadLoop(): Promise<void> {
    if (!this.reader || this.isReading) {
      return;
    }

    this.isReading = true;

    try {
      while (this.isReading && this.reader) {
        const { value, done } = await this.reader.read();
        
        if (done) {
          break;
        }

        if (value) {
          this.processIncomingData(value);
        }
      }
    } catch (error) {
      if (this.isReading) {
        console.error('Read loop error:', error);
        // Notify handlers of error
        this.messageHandlers.forEach(handler => {
          try {
            handler(new Uint8Array([CMD_ERROR]));
          } catch (e) {
            console.error('Handler error:', e);
          }
        });
      }
    } finally {
      this.isReading = false;
    }
  }

  /**
   * Process incoming data and extract complete packets
   */
  private processIncomingData(chunk: Uint8Array): void {
    // Append to buffer
    const newBuffer = new Uint8Array(this.readBuffer.length + chunk.length);
    newBuffer.set(this.readBuffer);
    newBuffer.set(chunk, this.readBuffer.length);
    this.readBuffer = newBuffer;

    // Try to extract complete packets
    while (this.readBuffer.length >= 5) {
      // Look for start byte
      const startIdx = this.readBuffer.indexOf(START_BYTE);
      
      if (startIdx === -1) {
        // No start byte found, clear buffer
        this.readBuffer = new Uint8Array(0);
        break;
      }

      if (startIdx > 0) {
        // Remove data before start byte
        this.readBuffer = this.readBuffer.slice(startIdx);
      }

      // Check if we have enough data for header
      if (this.readBuffer.length < 5) {
        break;
      }

      // Parse header
      const cmd = this.readBuffer[1];
      const len = (this.readBuffer[2] << 8) | this.readBuffer[3];
      const totalLen = 5 + len; // START + CMD + LEN(2) + PAYLOAD + CRC

      // Check if we have complete packet
      if (this.readBuffer.length < totalLen) {
        break;
      }

      // Extract packet
      const packet = this.readBuffer.slice(0, totalLen);
      this.readBuffer = this.readBuffer.slice(totalLen);

      // Verify CRC
      const receivedCrc = packet[totalLen - 1];
      const calculatedCrc = this.calculateCrc8(packet.slice(0, totalLen - 1));

      if (receivedCrc !== calculatedCrc) {
        console.warn('CRC mismatch, dropping packet');
        continue;
      }

      // Extract payload
      const payload = packet.slice(4, 4 + len);

      // Notify handlers
      this.notifyHandlers(cmd, payload);
    }
  }

  /**
   * Register message handler
   */
  onMessage(handler: (msg: Uint8Array) => void): () => void {
    this.messageHandlers.push(handler);
    return () => {
      const idx = this.messageHandlers.indexOf(handler);
      if (idx !== -1) {
        this.messageHandlers.splice(idx, 1);
      }
    };
  }

  /**
   * Notify all handlers
   */
  private notifyHandlers(cmd: number, payload: Uint8Array): void {
    const message = new Uint8Array(1 + payload.length);
    message[0] = cmd;
    message.set(payload, 1);

    this.messageHandlers.forEach(handler => {
      try {
        handler(message);
      } catch (error) {
        console.error('Handler error:', error);
      }
    });
  }

  /**
   * Sign a hash using the STM32 device
   * @param hash 32-byte Keccak-256 hash
   * @returns Promise resolving to signature bytes (65 bytes: r||s||v)
   */
  async signHash(hash: Uint8Array): Promise<Uint8Array> {
    if (hash.length !== 32) {
      throw new Error('Hash must be exactly 32 bytes');
    }

    if (!this.isConnected()) {
      throw new Error('Not connected to device');
    }

    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        cleanup();
        reject(new Error('Signature request timed out'));
      }, 30000); // 30 second timeout

      const cleanup = this.onMessage((msg) => {
        const cmd = msg[0];
        const payload = msg.slice(1);

        if (cmd === CMD_SIGNATURE) {
          clearTimeout(timeout);
          cleanup();
          
          if (payload.length === 65 || payload.length === 64) {
            resolve(payload);
          } else {
            reject(new Error(`Invalid signature length: ${payload.length} bytes`));
          }
        } else if (cmd === CMD_ERROR) {
          clearTimeout(timeout);
          cleanup();
          
          const errorCode = payload.length > 0 ? payload[0] : 0;
          reject(new Error(`Device error: code ${errorCode}`));
        }
      });

      // Send sign request
      const packet = this.encodePacket(CMD_SIGN_HASH, hash);
      this.send(packet).catch(error => {
        clearTimeout(timeout);
        cleanup();
        reject(error);
      });
    });
  }
}

// Export singleton instance
export const usbTransport = new UsbTransport();