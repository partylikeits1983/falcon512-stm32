/**
 * ERC7730 Message Builder Component
 * Allows users to build structured messages according to ERC7730 specification
 */

import { useState, useEffect } from "react";
import { keccak256, toUtf8Bytes } from "ethers";

// Import the 1inch limit order schema
import limitOrderSchema from "../1inch-limit-order.json";

interface ERC7730Domain {
  name: string;
  version: string;
  chainId: number;
  verifyingContract: string;
}

interface OrderStructure {
  salt: string;
  maker: string;
  receiver: string;
  makerAsset: string;
  takerAsset: string;
  makingAmount: string;
  takingAmount: string;
  makerTraits: string;
}

interface ERC7730MessageBuilderProps {
  onMessageChange: (
    plaintext: string,
    hash: string,
    humanReadable: string,
  ) => void;
}

export default function ERC7730MessageBuilder({
  onMessageChange,
}: ERC7730MessageBuilderProps) {
  const [domain, setDomain] = useState<ERC7730Domain>({
    name: "1inch",
    version: "1",
    chainId: 1,
    verifyingContract: "0x119c71d3bbac22029622cbaec24854d3d32d2828",
  });

  const [message, setMessage] = useState<OrderStructure>({
    salt: "123456789",
    maker: "0x0000000000000000000000000000000000000001",
    receiver: "0x0000000000000000000000000000000000000002",
    makerAsset: "0xA0b86a33E6441E6C7D3E4C5B4B6B8B8B8B8B8B8B", // Example USDC
    takerAsset: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // Example WETH
    makingAmount: "1000000000", // 1000 USDC (6 decimals)
    takingAmount: "500000000000000000", // 0.5 ETH
    makerTraits: "0",
  });

  // Get the EIP712 types from the schema
  // const getEIP712Types = () => {
  //   const schema = limitOrderSchema.context.eip712.schemas[0];
  //   return schema.types;
  // };

  // Generate ERC7730 plaintext representation
  const generatePlaintext = (
    domain: ERC7730Domain,
    message: OrderStructure,
  ): string => {
    // const types = getEIP712Types();

    // Create the structured data object
    // const structuredData = {
    //   types,
    //   primaryType: "OrderStructure",
    //   domain,
    //   message,
    // };

    // Generate human-readable plaintext according to ERC7730
    let plaintext = "ERC7730 Structured Data:\n\n";
    plaintext += `Domain:\n`;
    plaintext += `  Name: ${domain.name}\n`;
    plaintext += `  Version: ${domain.version}\n`;
    plaintext += `  Chain ID: ${domain.chainId}\n`;
    plaintext += `  Verifying Contract: ${domain.verifyingContract}\n\n`;

    plaintext += `Message Type: OrderStructure\n\n`;
    plaintext += `Message Data:\n`;
    plaintext += `  Salt: ${message.salt}\n`;
    plaintext += `  Maker: ${message.maker}\n`;
    plaintext += `  Receiver: ${message.receiver}\n`;
    plaintext += `  Maker Asset: ${message.makerAsset}\n`;
    plaintext += `  Taker Asset: ${message.takerAsset}\n`;
    plaintext += `  Making Amount: ${message.makingAmount}\n`;
    plaintext += `  Taking Amount: ${message.takingAmount}\n`;
    plaintext += `  Maker Traits: ${message.makerTraits}\n`;

    return plaintext;
  };

  // Generate human-readable display using ERC7730 display rules
  const generateHumanReadable = (message: OrderStructure): string => {
    const displayFormat = limitOrderSchema.display.formats.OrderStructure;

    let readable = `${displayFormat.intent}\n\n`;

    // Apply display formatting rules
    displayFormat.fields.forEach((field) => {
      const value = message[field.path as keyof OrderStructure];
      let displayValue = value;

      // Apply formatting based on field format
      if (field.format === "tokenAmount") {
        // For demo purposes, show raw amount with token info
        const tokenPath = field.params?.tokenPath;
        const tokenAddress = tokenPath
          ? message[tokenPath as keyof OrderStructure]
          : "Unknown";
        displayValue = `${value} (Token: ${tokenAddress})`;
      }

      readable += `${field.label}: ${displayValue}\n`;
    });

    return readable;
  };

  // Update the message and notify parent
  useEffect(() => {
    const plaintext = generatePlaintext(domain, message);
    const hash = keccak256(toUtf8Bytes(plaintext));
    const humanReadable = generateHumanReadable(message);

    onMessageChange(plaintext, hash, humanReadable);
  }, [domain, message, onMessageChange]);

  const updateMessage = (field: keyof OrderStructure, value: string) => {
    setMessage((prev) => ({
      ...prev,
      [field]: value,
    }));
  };

  const updateDomain = (field: keyof ERC7730Domain, value: string | number) => {
    setDomain((prev) => ({
      ...prev,
      [field]: value,
    }));
  };

  return (
    <div className="card">
      <h2>ERC7730 Message Builder</h2>

      {/* Domain Section */}
      <div style={{ marginBottom: "1rem" }}>
        <h3>EIP712 Domain</h3>
        <div
          style={{
            display: "grid",
            gridTemplateColumns: "1fr 1fr",
            gap: "0.5rem",
          }}
        >
          <div>
            <label>Name:</label>
            <input
              type="text"
              value={domain.name}
              onChange={(e) => updateDomain("name", e.target.value)}
              style={{ width: "100%", padding: "0.25rem" }}
            />
          </div>
          <div>
            <label>Version:</label>
            <input
              type="text"
              value={domain.version}
              onChange={(e) => updateDomain("version", e.target.value)}
              style={{ width: "100%", padding: "0.25rem" }}
            />
          </div>
          <div>
            <label>Chain ID:</label>
            <input
              type="number"
              value={domain.chainId}
              onChange={(e) =>
                updateDomain("chainId", parseInt(e.target.value))
              }
              style={{ width: "100%", padding: "0.25rem" }}
            />
          </div>
          <div>
            <label>Verifying Contract:</label>
            <input
              type="text"
              value={domain.verifyingContract}
              onChange={(e) =>
                updateDomain("verifyingContract", e.target.value)
              }
              style={{ width: "100%", padding: "0.25rem" }}
              placeholder="0x..."
            />
          </div>
        </div>
      </div>

      {/* Message Section */}
      <div>
        <h3>Order Structure</h3>
        <div
          style={{
            display: "grid",
            gridTemplateColumns: "1fr 1fr",
            gap: "0.5rem",
          }}
        >
          <div>
            <label>Salt:</label>
            <input
              type="text"
              value={message.salt}
              onChange={(e) => updateMessage("salt", e.target.value)}
              style={{ width: "100%", padding: "0.25rem" }}
            />
          </div>
          <div>
            <label>Maker Traits:</label>
            <input
              type="text"
              value={message.makerTraits}
              onChange={(e) => updateMessage("makerTraits", e.target.value)}
              style={{ width: "100%", padding: "0.25rem" }}
            />
          </div>
          <div>
            <label>Maker Address:</label>
            <input
              type="text"
              value={message.maker}
              onChange={(e) => updateMessage("maker", e.target.value)}
              style={{ width: "100%", padding: "0.25rem" }}
              placeholder="0x..."
            />
          </div>
          <div>
            <label>Receiver Address:</label>
            <input
              type="text"
              value={message.receiver}
              onChange={(e) => updateMessage("receiver", e.target.value)}
              style={{ width: "100%", padding: "0.25rem" }}
              placeholder="0x..."
            />
          </div>
          <div>
            <label>Maker Asset:</label>
            <input
              type="text"
              value={message.makerAsset}
              onChange={(e) => updateMessage("makerAsset", e.target.value)}
              style={{ width: "100%", padding: "0.25rem" }}
              placeholder="0x... (Token Address)"
            />
          </div>
          <div>
            <label>Taker Asset:</label>
            <input
              type="text"
              value={message.takerAsset}
              onChange={(e) => updateMessage("takerAsset", e.target.value)}
              style={{ width: "100%", padding: "0.25rem" }}
              placeholder="0x... (Token Address)"
            />
          </div>
          <div>
            <label>Making Amount:</label>
            <input
              type="text"
              value={message.makingAmount}
              onChange={(e) => updateMessage("makingAmount", e.target.value)}
              style={{ width: "100%", padding: "0.25rem" }}
              placeholder="Amount in wei/smallest unit"
            />
          </div>
          <div>
            <label>Taking Amount:</label>
            <input
              type="text"
              value={message.takingAmount}
              onChange={(e) => updateMessage("takingAmount", e.target.value)}
              style={{ width: "100%", padding: "0.25rem" }}
              placeholder="Amount in wei/smallest unit"
            />
          </div>
        </div>
      </div>

      <div style={{ marginTop: "1rem", fontSize: "0.9rem", color: "#666" }}>
        <p>
          <strong>Note:</strong> This builds a 1inch Limit Order according to
          ERC7730 specification. The plaintext representation will be hashed
          with keccak256 before signing.
        </p>
      </div>
    </div>
  );
}
