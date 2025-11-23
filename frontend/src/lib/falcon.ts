import init, {
  FalconKeyPair,
  sign_message,
  verify_signature,
  generate_keypair_from_random,
} from "../falcon-wasm/falcon_wasm.js";

let wasmInitialized = false;

export async function initFalcon() {
  if (!wasmInitialized) {
    await init();
    wasmInitialized = true;
  }
}

export interface FalconKeys {
  publicKey: Uint8Array;
  secretKey: Uint8Array;
}

export class Falcon {
  private static async ensureInit() {
    await initFalcon();
  }

  /**
   * Generate a new Falcon512 key pair from a 32-byte seed
   */
  static async generateKeyPair(seed?: Uint8Array): Promise<FalconKeys> {
    await this.ensureInit();

    let keyPair: FalconKeyPair;

    if (seed) {
      if (seed.length !== 32) {
        throw new Error("Seed must be exactly 32 bytes");
      }
      keyPair = new FalconKeyPair(seed);
    } else {
      keyPair = generate_keypair_from_random();
    }

    return {
      publicKey: new Uint8Array(keyPair.public_key),
      secretKey: new Uint8Array(keyPair.secret_key),
    };
  }

  /**
   * Sign a message with a Falcon512 secret key
   */
  static async sign(
    message: Uint8Array,
    secretKey: Uint8Array,
  ): Promise<Uint8Array> {
    await this.ensureInit();

    const signature = sign_message(message, secretKey);
    return new Uint8Array(signature.bytes);
  }

  /**
   * Verify a Falcon512 signature
   */
  static async verify(
    message: Uint8Array,
    signature: Uint8Array,
    publicKey: Uint8Array,
  ): Promise<boolean> {
    await this.ensureInit();

    return verify_signature(message, signature, publicKey);
  }

  /**
   * Convert a hex string to Uint8Array
   */
  static hexToBytes(hex: string): Uint8Array {
    const bytes = new Uint8Array(hex.length / 2);
    for (let i = 0; i < hex.length; i += 2) {
      bytes[i / 2] = parseInt(hex.substr(i, 2), 16);
    }
    return bytes;
  }

  /**
   * Convert Uint8Array to hex string
   */
  static bytesToHex(bytes: Uint8Array): string {
    return Array.from(bytes)
      .map((b) => b.toString(16).padStart(2, "0"))
      .join("");
  }
}

export default Falcon;
