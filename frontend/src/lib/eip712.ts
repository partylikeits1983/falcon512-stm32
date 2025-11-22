/**
 * EIP-712 Typed Data Hashing Utilities
 * Provides helpers for creating and hashing EIP-712 messages
 */

import { TypedDataEncoder, keccak256, toUtf8Bytes } from "ethers";

/**
 * EIP-712 Domain definition
 */
export const domain = {
  name: "STM32SignerDemo",
  version: "1",
  chainId: 1,
  verifyingContract: "0x0000000000000000000000000000000000000000",
} as const;

/**
 * Message type definition for EIP-712
 */
export const types = {
  Message: [
    { name: "from", type: "address" },
    { name: "to", type: "address" },
    { name: "value", type: "uint256" },
    { name: "nonce", type: "uint256" },
  ],
};

/**
 * Message structure
 */
export interface Eip712Message {
  from: string;
  to: string;
  value: string;
  nonce: string;
}

/**
 * Validate Ethereum address format
 */
export function isValidAddress(address: string): boolean {
  return /^0x[0-9a-fA-F]{40}$/.test(address);
}

/**
 * Validate uint256 value
 */
export function isValidUint256(value: string): boolean {
  try {
    const num = BigInt(value);
    return num >= 0n && num <= 2n ** 256n - 1n;
  } catch {
    return false;
  }
}

/**
 * Validate EIP-712 message
 */
export function validateMessage(message: Eip712Message): {
  valid: boolean;
  errors: string[];
} {
  const errors: string[] = [];

  if (!isValidAddress(message.from)) {
    errors.push('Invalid "from" address format');
  }

  if (!isValidAddress(message.to)) {
    errors.push('Invalid "to" address format');
  }

  if (!isValidUint256(message.value)) {
    errors.push('Invalid "value" - must be a valid uint256');
  }

  if (!isValidUint256(message.nonce)) {
    errors.push('Invalid "nonce" - must be a valid uint256');
  }

  return {
    valid: errors.length === 0,
    errors,
  };
}

/**
 * Hash an EIP-712 message
 * @param message The message to hash
 * @returns The 32-byte hash as hex string (0x-prefixed)
 */
export function hashEip712Message(message: Eip712Message): string {
  const validation = validateMessage(message);
  if (!validation.valid) {
    throw new Error(`Invalid message: ${validation.errors.join(", ")}`);
  }

  return TypedDataEncoder.hash(domain, types, message);
}

/**
 * Convert hex string to Uint8Array
 * @param hex Hex string (with or without 0x prefix)
 * @returns Uint8Array
 */
export function hexToBytes(hex: string): Uint8Array {
  // Remove 0x prefix if present
  const cleanHex = hex.startsWith("0x") ? hex.slice(2) : hex;

  if (cleanHex.length % 2 !== 0) {
    throw new Error("Hex string must have even length");
  }

  const bytes = new Uint8Array(cleanHex.length / 2);
  for (let i = 0; i < cleanHex.length; i += 2) {
    bytes[i / 2] = parseInt(cleanHex.slice(i, i + 2), 16);
  }

  return bytes;
}

/**
 * Convert Uint8Array to hex string
 * @param bytes Uint8Array
 * @param prefix Whether to include 0x prefix
 * @returns Hex string
 */
export function bytesToHex(bytes: Uint8Array, prefix = true): string {
  const hex = Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");

  return prefix ? `0x${hex}` : hex;
}

/**
 * Get the hash as bytes (Uint8Array)
 * @param message The message to hash
 * @returns 32-byte hash as Uint8Array
 */
export function hashEip712MessageBytes(message: Eip712Message): Uint8Array {
  const hashHex = hashEip712Message(message);
  return hexToBytes(hashHex);
}

/**
 * Create a sample message for testing
 */
export function createSampleMessage(): Eip712Message {
  return {
    from: "0x0000000000000000000000000000000000000001",
    to: "0x0000000000000000000000000000000000000002",
    value: "1000000000000000000", // 1 ETH in wei
    nonce: "0",
  };
}

/**
 * Format message for display
 */
export function formatMessage(message: Eip712Message): string {
  return JSON.stringify(message, null, 2);
}

/**
 * Parse signature bytes into r, s, v components
 * @param signature 64 or 65 byte signature
 * @returns Object with r, s, v as hex strings
 */
export function parseSignature(signature: Uint8Array): {
  r: string;
  s: string;
  v: number;
} {
  if (signature.length !== 64 && signature.length !== 65) {
    throw new Error(`Invalid signature length: ${signature.length} bytes`);
  }

  const r = bytesToHex(signature.slice(0, 32));
  const s = bytesToHex(signature.slice(32, 64));

  // If 65 bytes, last byte is v
  // If 64 bytes, v needs to be derived (typically 27 or 28)
  const v = signature.length === 65 ? signature[64] : 27;

  return { r, s, v };
}

/**
 * Format signature for Solidity
 * @param signature 64 or 65 byte signature
 * @returns Hex string suitable for Solidity (0x + 130 chars for 65 bytes)
 */
export function formatSignatureForSolidity(signature: Uint8Array): string {
  if (signature.length === 64) {
    // Add v = 27 as default
    const fullSig = new Uint8Array(65);
    fullSig.set(signature);
    fullSig[64] = 27;
    return bytesToHex(fullSig);
  }

  return bytesToHex(signature);
}

/**
 * Compute Keccak-256 hash of arbitrary data
 * @param data Data to hash (string or Uint8Array)
 * @returns 32-byte hash as hex string
 */
export function keccak256Hash(data: string | Uint8Array): string {
  if (typeof data === "string") {
    return keccak256(toUtf8Bytes(data));
  }
  return keccak256(data);
}
