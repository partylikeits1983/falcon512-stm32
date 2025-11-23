/**
 * Main App Component
 * STM32 Falcon512 Signing Interface
 */

import { useState, useEffect } from "react";
import "./App.css";
import { keccak256, toUtf8Bytes } from "ethers";
import Falcon, { initFalcon } from "./lib/falcon";

function App() {
  const [port, setPort] = useState<SerialPort | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const [messageHash, setMessageHash] = useState("");
  const [rawMessage, setRawMessage] = useState("");
  const [isRawMode, setIsRawMode] = useState(false);
  const [status, setStatus] = useState("");
  const [signature, setSignature] = useState("");
  const [publicKey, setPublicKey] = useState("");
  const [isProcessing, setIsProcessing] = useState(false);
  const [isEditing, setIsEditing] = useState(false);
  const [forceHashUpdate, setForceHashUpdate] = useState(0);

  // Wallet mode and WASM keys
  const [walletMode, setWalletMode] = useState<"hardware" | "local">(
    "hardware",
  );
  const [isWasmInitialized, setIsWasmInitialized] = useState(false);
  const [localKeys, setLocalKeys] = useState<{
    publicKey: Uint8Array;
    secretKey: Uint8Array;
  } | null>(null);
  const [isGeneratingKeys, setIsGeneratingKeys] = useState(false);

  // 1inch order data (editable)
  const [orderData, setOrderData] = useState({
    from: "0x0000000000000000000000000000000000000001",
    send: "1000000000",
    sendToken: "0xA0b86a33E6441E6C7D3E4C5B4B6B8B8B8B8B8B8B",
    receiveMinimum: "500000000000000000",
    receiveToken: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
    to: "0x0000000000000000000000000000000000000002",
  });

  // JSON representation for editing
  const [editableJson, setEditableJson] = useState("");

  // Initialize WASM on component mount
  useEffect(() => {
    const initWasm = async () => {
      try {
        await initFalcon();
        setIsWasmInitialized(true);
      } catch (err) {
        console.error("Failed to initialize Falcon WASM:", err);
      }
    };
    initWasm();
  }, []);

  const generateLocalKeys = async () => {
    if (!isWasmInitialized) return;

    setIsGeneratingKeys(true);
    try {
      const newKeys = await Falcon.generateKeyPair();
      setLocalKeys(newKeys);
      setStatus(
        `✅ Local Falcon512 keys generated!\n📊 Public Key: ${newKeys.publicKey.length} bytes\n📊 Secret Key: ${newKeys.secretKey.length} bytes`,
      );
    } catch (err) {
      setStatus(`❌ Failed to generate keys: ${err}`);
    } finally {
      setIsGeneratingKeys(false);
    }
  };

  const signWithLocalKeys = async (messageToSign: string, _isHash: boolean) => {
    if (!localKeys) {
      setStatus("❌ No local keys available");
      return null;
    }

    try {
      const messageBytes = new TextEncoder().encode(messageToSign);
      const signature = await Falcon.sign(messageBytes, localKeys.secretKey);

      return {
        signature: Falcon.bytesToHex(signature),
        publicKey: Falcon.bytesToHex(localKeys.publicKey),
      };
    } catch (err) {
      setStatus(`❌ Failed to sign with local keys: ${err}`);
      return null;
    }
  };

  const connectToDevice = async () => {
    try {
      setStatus("Requesting port...");
      // Filter for STM32 device (VID: 0x0483, PID: 0x5740)
      const selectedPort = await navigator.serial.requestPort({
        filters: [
          {
            usbVendorId: 0x0483,
            usbProductId: 0x5740,
          },
        ],
      });

      setStatus("Opening port at 115200 baud...");
      await selectedPort.open({ baudRate: 115200 });

      // Get port info if available
      const info = selectedPort.getInfo();
      let portInfo = "";
      if (info.usbVendorId && info.usbProductId) {
        portInfo = `\nVID: 0x${info.usbVendorId.toString(16).padStart(4, "0")}, PID: 0x${info.usbProductId.toString(16).padStart(4, "0")}`;
      }

      setPort(selectedPort);
      setIsConnected(true);
      setStatus(`✅ Connected to STM32!${portInfo}\n\nReady to sign messages.`);
    } catch (error) {
      setStatus(
        `❌ Error: ${error instanceof Error ? error.message : "Failed to connect"}`,
      );
    }
  };

  const disconnect = async () => {
    if (port) {
      try {
        await port.close();
        setPort(null);
        setIsConnected(false);
        setStatus("Disconnected");
      } catch (error) {
        setStatus(
          `Error disconnecting: ${error instanceof Error ? error.message : "Unknown error"}`,
        );
      }
    }
  };

  // Generate ERC7730 plaintext and hash on component mount
  useEffect(() => {
    const generateERC7730Message = () => {
      // Create ERC7730 plaintext representation
      let plaintext = "ERC7730 Structured Data:\n\n";
      plaintext += `Domain:\n`;
      plaintext += `  Name: 1inch\n`;
      plaintext += `  Version: 1\n`;
      plaintext += `  Chain ID: 1\n`;
      plaintext += `  Verifying Contract: 0x119c71d3bbac22029622cbaec24854d3d32d2828\n\n`;

      plaintext += `Message Type: OrderStructure\n\n`;
      plaintext += `Message Data:\n`;
      plaintext += `  Salt: 123456789\n`;
      plaintext += `  Maker: ${orderData.from}\n`;
      plaintext += `  Receiver: ${orderData.to}\n`;
      plaintext += `  Maker Asset: ${orderData.sendToken}\n`;
      plaintext += `  Taker Asset: ${orderData.receiveToken}\n`;
      plaintext += `  Making Amount: ${orderData.send}\n`;
      plaintext += `  Taking Amount: ${orderData.receiveMinimum}\n`;
      plaintext += `  Maker Traits: 0\n`;

      // Generate keccak256 hash
      const hash = keccak256(toUtf8Bytes(plaintext));
      setMessageHash(hash);
    };

    generateERC7730Message();
  }, [orderData, forceHashUpdate]);

  // Handle raw message mode
  const handleRawMessageChange = (message: string) => {
    setRawMessage(message);
    if (message.trim() === "") {
      setIsRawMode(false);
      setMessageHash("");
    } else {
      setIsRawMode(true);
      // For raw mode, we don't hash - we send the message directly
      setMessageHash("");
    }
  };

  // Convert order data to JSON string for editing
  const orderToJson = () => {
    return JSON.stringify(
      {
        "1inch Order": {
          From: orderData.from,
          Send: `${orderData.send} (Token: ${orderData.sendToken})`,
          "Receive minimum": `${orderData.receiveMinimum} (Token: ${orderData.receiveToken})`,
          To: orderData.to,
        },
      },
      null,
      2,
    );
  };

  // Parse JSON back to order data or handle raw text
  const parseJsonToOrder = (jsonStr: string) => {
    const trimmed = jsonStr.trim();

    // If empty, clear everything
    if (trimmed === "") {
      handleRawMessageChange("");
      return;
    }

    // Try to parse as JSON first
    try {
      const parsed = JSON.parse(trimmed);
      const order = parsed["1inch Order"];

      if (order) {
        // It's valid ERC7730 JSON
        const sendMatch = order.Send?.match(
          /^(\d+) \(Token: (0x[a-fA-F0-9]{40})\)$/i,
        );
        const receiveMatch = order["Receive minimum"]?.match(
          /^(\d+) \(Token: (0x[a-fA-F0-9]{40})\)$/i,
        );

        if (sendMatch && receiveMatch && order.From && order.To) {
          const newOrderData = {
            from: order.From,
            send: sendMatch[1],
            sendToken: sendMatch[2],
            receiveMinimum: receiveMatch[1],
            receiveToken: receiveMatch[2],
            to: order.To,
          };

          setOrderData(newOrderData);
          setIsRawMode(false);
          setRawMessage("");
        }
      } else {
        // Valid JSON but not ERC7730 format - treat as raw message
        handleRawMessageChange(trimmed);
      }
    } catch (error) {
      // Not valid JSON - treat as raw text message
      handleRawMessageChange(trimmed);
    }
  };

  // Handle entering edit mode
  const enterEditMode = () => {
    if (isRawMode) {
      setEditableJson(rawMessage);
    } else {
      setEditableJson(orderToJson());
    }
    setIsEditing(true);
  };

  // Handle saving edits
  const saveEdits = () => {
    parseJsonToOrder(editableJson);
    setIsEditing(false);

    // Force hash regeneration by triggering a state update
    // This ensures the ERC7730 hash changes when save is clicked
    setForceHashUpdate((prev) => prev + 1);
  };

  // Handle canceling edits
  const cancelEdits = () => {
    setIsEditing(false);
  };

  const signMessage = async () => {
    // Check if we can sign (either connected to hardware or have local keys)
    if (walletMode === "hardware" && (!port || !isConnected)) {
      setStatus("❌ Not connected to hardware device");
      return;
    }

    if (walletMode === "local" && !localKeys) {
      setStatus("❌ No local keys generated");
      return;
    }

    let messageToSign = "";
    let isSigningHash = false;

    if (isRawMode && rawMessage.trim() !== "") {
      // Raw mode - sign the message directly
      messageToSign = rawMessage.trim();
      isSigningHash = false;
    } else if (!isRawMode && messageHash) {
      // ERC7730 mode - sign the hash
      messageToSign = messageHash.startsWith("0x")
        ? messageHash.slice(2)
        : messageHash;
      isSigningHash = true;
    } else {
      setStatus("❌ No message available to sign.");
      return;
    }

    setIsProcessing(true);
    setSignature("");
    setPublicKey("");

    if (walletMode === "local") {
      // Local WASM signing
      setStatus(
        isSigningHash
          ? "🔐 Signing message hash with local Falcon512 keys..."
          : "🔐 Signing raw message with local Falcon512 keys...",
      );

      try {
        const result = await signWithLocalKeys(messageToSign, isSigningHash);
        if (result) {
          setSignature(result.signature);
          setPublicKey(result.publicKey);

          // Verify the signature using WASM
          try {
            setStatus(
              "✅ Message signed locally!\n🔍 Verifying signature...\n� Signature: " +
                result.signature.length / 2 +
                " bytes\n📊 Public Key: " +
                result.publicKey.length / 2 +
                " bytes",
            );

            const messageBytes = new TextEncoder().encode(messageToSign);
            const signatureBytes = Falcon.hexToBytes(result.signature);
            const publicKeyBytes = Falcon.hexToBytes(result.publicKey);
            const isValid = await Falcon.verify(
              messageBytes,
              signatureBytes,
              publicKeyBytes,
            );

            const verificationText = isValid
              ? "✅ Signature Verified!"
              : "❌ Signature Verification Failed!";
            setStatus(
              "✅ Message signed locally!\n" +
                verificationText +
                "\n📊 Signature: " +
                result.signature.length / 2 +
                " bytes\n📊 Public Key: " +
                result.publicKey.length / 2 +
                " bytes",
            );
          } catch (verifyError) {
            setStatus(
              "✅ Message signed locally!\n❌ Signature Verification Failed!\n📊 Signature: " +
                result.signature.length / 2 +
                " bytes\n📊 Public Key: " +
                result.publicKey.length / 2 +
                " bytes\n⚠️ Verification error: " +
                verifyError,
            );
          }
        }
      } catch (error) {
        setStatus(
          `❌ Error: ${error instanceof Error ? error.message : "Unknown error"}`,
        );
      } finally {
        setIsProcessing(false);
      }
      return;
    }

    // Hardware signing (existing STM32 code)
    setStatus(
      isSigningHash
        ? "📤 Sending message hash to STM32..."
        : "📤 Sending raw message to STM32...",
    );

    try {
      const writer = port!.writable?.getWriter();
      if (!writer) {
        throw new Error("Port not writable");
      }

      const messageWithNewline = messageToSign + "\n";
      const encoder = new TextEncoder();
      await writer.write(encoder.encode(messageWithNewline));
      writer.releaseLock();

      setStatus(
        isSigningHash
          ? "⏳ Waiting for STM32 response...\n👆 Press button B0 on the STM32 board to sign the hash"
          : "⏳ Waiting for STM32 response...\n👆 Press button B0 on the STM32 board to sign the message",
      );

      // Read response
      const reader = port!.readable?.getReader();
      if (!reader) {
        throw new Error("Port not readable");
      }

      let response = "";
      const decoder = new TextDecoder();
      const timeout = setTimeout(() => {
        reader.cancel();
        setStatus("❌ Timeout waiting for response");
        setIsProcessing(false);
      }, 30000);

      try {
        // Read until we get a complete response
        // The STM32 sends: SIGNED:\n<message>\nSIGNATURE:\n<hex>\nPUBLIC_KEY:\n<hex>\n
        // We need to wait until we have received all the data
        let lastChunkTime = Date.now();
        const READ_TIMEOUT = 500; // 500ms without new data means we're done

        while (true) {
          const { value, done } = await reader.read();
          if (done) break;

          const chunk = decoder.decode(value, { stream: true });
          response += chunk;
          lastChunkTime = Date.now();

          // Check if we have the complete response structure
          // Wait a bit after seeing PUBLIC_KEY to ensure we got all the hex data
          if (response.includes("PUBLIC_KEY:")) {
            // Wait for a short period to ensure all data is received
            await new Promise((resolve) => setTimeout(resolve, 200));

            // Try to read any remaining data
            const controller = new AbortController();
            const timeoutId = setTimeout(
              () => controller.abort(),
              READ_TIMEOUT,
            );

            try {
              while (Date.now() - lastChunkTime < READ_TIMEOUT) {
                const result = await Promise.race([
                  reader.read(),
                  new Promise((_, reject) =>
                    setTimeout(
                      () => reject(new Error("timeout")),
                      READ_TIMEOUT,
                    ),
                  ),
                ]);

                if (result && typeof result === "object" && "value" in result) {
                  const { value: moreValue, done: moreDone } =
                    result as ReadableStreamReadResult<Uint8Array>;
                  if (moreDone) break;
                  if (moreValue) {
                    const moreChunk = decoder.decode(moreValue, {
                      stream: true,
                    });
                    response += moreChunk;
                    lastChunkTime = Date.now();
                  }
                } else {
                  break;
                }
              }
            } catch (e) {
              // Timeout or error means we're done reading
            } finally {
              clearTimeout(timeoutId);
            }

            break;
          }
        }
      } finally {
        clearTimeout(timeout);
        reader.releaseLock();
      }

      // Parse response
      // Extract signature: everything between "SIGNATURE:\n" and "\nPUBLIC_KEY:"
      const sigStart = response.indexOf("SIGNATURE:");
      const pkStart = response.indexOf("PUBLIC_KEY:");

      if (sigStart === -1 || pkStart === -1) {
        setStatus(
          "❌ Failed to parse response - missing headers:\n" + response,
        );
        return;
      }

      // Extract the hex strings
      const sigSection = response.substring(
        sigStart + "SIGNATURE:".length,
        pkStart,
      );
      const pkSection = response.substring(pkStart + "PUBLIC_KEY:".length);

      // Remove all whitespace (spaces, newlines, etc.) to get pure hex
      const sig = sigSection.replace(/\s/g, "");
      const pk = pkSection.replace(/\s/g, "");

      // Validate hex strings
      const hexPattern = /^[0-9a-fA-F]+$/;
      if (!sig || !pk) {
        setStatus(
          "❌ Failed to extract signature or public key from response:\n" +
            response,
        );
        return;
      }

      if (!hexPattern.test(sig)) {
        setStatus(
          "❌ Invalid signature format (not valid hex):\n" +
            sig.substring(0, 100),
        );
        return;
      }

      if (!hexPattern.test(pk)) {
        setStatus(
          "❌ Invalid public key format (not valid hex):\n" +
            pk.substring(0, 100),
        );
        return;
      }

      setSignature(sig);
      setPublicKey(pk);

      // Verify the signature using WASM
      try {
        setStatus(
          "✅ Signature received!\n🔍 Verifying signature...\n📊 Signature: " +
            sig.length / 2 +
            " bytes\n📊 Public Key: " +
            pk.length / 2 +
            " bytes",
        );

        const messageBytes = new TextEncoder().encode(messageToSign);
        const signatureBytes = Falcon.hexToBytes(sig);
        const publicKeyBytes = Falcon.hexToBytes(pk);
        const isValid = await Falcon.verify(
          messageBytes,
          signatureBytes,
          publicKeyBytes,
        );

        const verificationText = isValid
          ? "✅ Signature Verified!"
          : "❌ Signature Verification Failed!";
        setStatus(
          "✅ Signature received!\n" +
            verificationText +
            "\n📊 Signature: " +
            sig.length / 2 +
            " bytes\n📊 Public Key: " +
            pk.length / 2 +
            " bytes",
        );
      } catch (verifyError) {
        setStatus(
          "✅ Signature received!\n❌ Signature Verification Failed!\n📊 Signature: " +
            sig.length / 2 +
            " bytes\n📊 Public Key: " +
            pk.length / 2 +
            " bytes\n⚠️ Verification error: " +
            verifyError,
        );
      }
    } catch (error) {
      setStatus(
        `❌ Error: ${error instanceof Error ? error.message : "Unknown error"}`,
      );
    } finally {
      setIsProcessing(false);
    }
  };

  const isWebSerialSupported = "serial" in navigator;

  return (
    <div className="app">
      <div className="container">
        <header className="header">
          <h1>🔐 Post Quantum Hardware Wallet</h1>
          <p>Sign messages using Falcon512 DSA on your STM32 hardware device</p>
        </header>

        {!isWebSerialSupported ? (
          <div className="error-box">
            <h2>⚠️ Web Serial API Not Supported</h2>
            <p>Please use Chrome, Edge, or Opera browser.</p>
          </div>
        ) : (
          <div className="main-content">
            {/* Wallet Mode Selection */}
            <div className="card">
              <h2>🔐 Wallet Mode</h2>
              <div style={{ marginBottom: "1rem" }}>
                <label
                  style={{
                    display: "flex",
                    alignItems: "center",
                    marginBottom: "0.5rem",
                    cursor: "pointer",
                  }}
                >
                  <input
                    type="radio"
                    name="walletMode"
                    value="hardware"
                    checked={walletMode === "hardware"}
                    onChange={(e) =>
                      setWalletMode(e.target.value as "hardware" | "local")
                    }
                    style={{ marginRight: "0.5rem" }}
                  />
                  <span>🔌 Hardware Wallet (STM32)</span>
                </label>
                <label
                  style={{
                    display: "flex",
                    alignItems: "center",
                    cursor: "pointer",
                  }}
                >
                  <input
                    type="radio"
                    name="walletMode"
                    value="local"
                    checked={walletMode === "local"}
                    onChange={(e) =>
                      setWalletMode(e.target.value as "hardware" | "local")
                    }
                    style={{ marginRight: "0.5rem" }}
                  />
                  <span>💻 Local Keys (WASM)</span>
                </label>
              </div>

              {walletMode === "hardware" ? (
                <>
                  <div className="status-row">
                    <span className="status-label">Hardware Status:</span>
                    <span
                      className={`status-badge ${isConnected ? "connected" : "disconnected"}`}
                    >
                      {isConnected ? "🟢 Connected" : "⚫ Disconnected"}
                    </span>
                  </div>
                  <div className="button-row">
                    {!isConnected ? (
                      <button onClick={connectToDevice} className="btn-primary">
                        Connect to STM32
                      </button>
                    ) : (
                      <button onClick={disconnect} className="btn-secondary">
                        Disconnect
                      </button>
                    )}
                  </div>
                </>
              ) : (
                <>
                  <div className="status-row">
                    <span className="status-label">WASM Status:</span>
                    <span
                      className={`status-badge ${isWasmInitialized ? "connected" : "disconnected"}`}
                    >
                      {isWasmInitialized ? "🟢 Ready" : "⚫ Loading..."}
                    </span>
                  </div>
                  <div className="status-row">
                    <span className="status-label">Local Keys:</span>
                    <span
                      className={`status-badge ${localKeys ? "connected" : "disconnected"}`}
                    >
                      {localKeys ? "🔑 Generated" : "⚫ Not Generated"}
                    </span>
                  </div>
                  <div className="button-row">
                    <button
                      onClick={generateLocalKeys}
                      disabled={!isWasmInitialized || isGeneratingKeys}
                      className="btn-primary"
                    >
                      {isGeneratingKeys
                        ? "⏳ Generating..."
                        : "🔑 Generate Keys"}
                    </button>
                    {localKeys && (
                      <div
                        style={{
                          marginTop: "0.5rem",
                          fontSize: "0.8rem",
                          color: "#666",
                        }}
                      >
                        <p>
                          <strong>Public Key:</strong>{" "}
                          {Falcon.bytesToHex(localKeys.publicKey).substring(
                            0,
                            32,
                          )}
                          ...
                        </p>
                      </div>
                    )}
                  </div>
                </>
              )}
            </div>

            {/* What You're Signing */}
            <div className="card">
              <h2>What You're Signing EIP-7730</h2>
              {!isEditing ? (
                <div
                  onClick={enterEditMode}
                  style={{
                    backgroundColor: "#2d3748",
                    color: "#e2e8f0",
                    padding: "1rem",
                    borderRadius: "4px",
                    fontFamily: "monospace",
                    cursor: "pointer",
                    border: "2px dashed transparent",
                    transition: "border-color 0.2s",
                  }}
                  onMouseEnter={(e) =>
                    (e.currentTarget.style.borderColor = "#4a5568")
                  }
                  onMouseLeave={(e) =>
                    (e.currentTarget.style.borderColor = "transparent")
                  }
                >
                  {isRawMode ? (
                    <div>
                      <div
                        style={{
                          fontSize: "1.1rem",
                          fontWeight: "bold",
                          marginBottom: "1rem",
                        }}
                      >
                        Raw Message
                      </div>
                      <div
                        style={{ lineHeight: "1.6", whiteSpace: "pre-wrap" }}
                      >
                        {rawMessage || "No message"}
                      </div>
                    </div>
                  ) : (
                    <div>
                      <div
                        style={{
                          fontSize: "1.1rem",
                          fontWeight: "bold",
                          marginBottom: "1rem",
                        }}
                      >
                        1inch Order
                      </div>
                      <div style={{ lineHeight: "1.6" }}>
                        <div>
                          <strong>From:</strong> {orderData.from}
                        </div>
                        <div>
                          <strong>Send:</strong> {orderData.send} (Token:{" "}
                          {orderData.sendToken})
                        </div>
                        <div>
                          <strong>Receive minimum:</strong>{" "}
                          {orderData.receiveMinimum} (Token:{" "}
                          {orderData.receiveToken})
                        </div>
                        <div>
                          <strong>To:</strong> {orderData.to}
                        </div>
                      </div>
                    </div>
                  )}
                  <div
                    style={{
                      marginTop: "0.5rem",
                      fontSize: "0.8rem",
                      color: "#a0aec0",
                      fontStyle: "italic",
                    }}
                  >
                    Click to edit •{" "}
                    {isRawMode
                      ? "Clear text to use ERC7730 mode"
                      : "Enter any text for raw message mode"}
                  </div>
                </div>
              ) : (
                <div
                  style={{
                    backgroundColor: "#2d3748",
                    padding: "1rem",
                    borderRadius: "4px",
                  }}
                >
                  <textarea
                    value={editableJson}
                    onChange={(e) => setEditableJson(e.target.value)}
                    placeholder="Enter ERC7730 JSON or any raw text message..."
                    style={{
                      width: "100%",
                      height: "200px",
                      backgroundColor: "#1a202c",
                      color: "#e2e8f0",
                      border: "1px solid #4a5568",
                      borderRadius: "4px",
                      padding: "0.5rem",
                      fontFamily: "monospace",
                      fontSize: "0.9rem",
                      resize: "vertical",
                    }}
                  />
                  <div
                    style={{
                      marginTop: "0.5rem",
                      display: "flex",
                      gap: "0.5rem",
                    }}
                  >
                    <button
                      onClick={saveEdits}
                      style={{
                        backgroundColor: "#38a169",
                        color: "white",
                        border: "none",
                        borderRadius: "4px",
                        padding: "0.5rem 1rem",
                        cursor: "pointer",
                        fontFamily: "inherit",
                      }}
                    >
                      Save
                    </button>
                    <button
                      onClick={cancelEdits}
                      style={{
                        backgroundColor: "#e53e3e",
                        color: "white",
                        border: "none",
                        borderRadius: "4px",
                        padding: "0.5rem 1rem",
                        cursor: "pointer",
                        fontFamily: "inherit",
                      }}
                    >
                      Cancel
                    </button>
                  </div>
                </div>
              )}
            </div>

            {/* Message Hash Display or Raw Message */}
            {(messageHash || (isRawMode && rawMessage.trim())) && (
              <div className="card">
                <h2>
                  {isRawMode ? "Raw Message" : "Message Hash (Keccak256)"}
                </h2>
                <div className="result-box">
                  <code className="result-text">
                    {isRawMode ? rawMessage : messageHash}
                  </code>
                  <button
                    onClick={() =>
                      navigator.clipboard.writeText(
                        isRawMode ? rawMessage : messageHash,
                      )
                    }
                    className="btn-copy"
                  >
                    Copy
                  </button>
                </div>
                <button
                  onClick={signMessage}
                  disabled={
                    (walletMode === "hardware" &&
                      (!isConnected || isProcessing)) ||
                    (walletMode === "local" && (!localKeys || isProcessing))
                  }
                  className="btn-primary"
                  style={{ marginTop: "1rem" }}
                >
                  {isProcessing
                    ? "⏳ Processing..."
                    : isRawMode
                      ? `✍️ Sign Message ${walletMode === "local" ? "(Local)" : "(Hardware)"}`
                      : `✍️ Sign Hash ${walletMode === "local" ? "(Local)" : "(Hardware)"}`}
                </button>
              </div>
            )}

            {/* Status Section */}
            {status && (
              <div className="card">
                <h2>Status</h2>
                <pre className="status-text">{status}</pre>
              </div>
            )}

            {/* Results Section */}
            {signature && (
              <div className="card">
                <h2>Signature</h2>
                <div className="result-box">
                  <code className="result-text">{signature}</code>
                  <button
                    onClick={() => navigator.clipboard.writeText(signature)}
                    className="btn-copy"
                  >
                    Copy
                  </button>
                </div>
              </div>
            )}

            {publicKey && (
              <div className="card">
                <h2>Public Key</h2>
                <div className="result-box">
                  <code className="result-text">{publicKey}</code>
                  <button
                    onClick={() => navigator.clipboard.writeText(publicKey)}
                    className="btn-copy"
                  >
                    Copy
                  </button>
                </div>
              </div>
            )}

            {/* Instructions */}
            <div className="card info-card">
              <h3>📋 Quick Start</h3>

              <div style={{ display: "flex", gap: "1rem", marginTop: "1rem" }}>
                <div style={{ flex: 1 }}>
                  <h4 style={{ marginBottom: "0.5rem" }}>🔌 Hardware</h4>
                  <p style={{ fontSize: "0.9rem", lineHeight: "1.4" }}>
                    Connect STM32 → Select Hardware mode → Generate/sign with
                    physical button
                  </p>
                </div>
                <div style={{ flex: 1 }}>
                  <h4 style={{ marginBottom: "0.5rem" }}>💻 Local</h4>
                  <p style={{ fontSize: "0.9rem", lineHeight: "1.4" }}>
                    Select Local mode → Generate keys → Sign instantly in
                    browser
                  </p>
                </div>
              </div>

              <p
                style={{
                  marginTop: "1rem",
                  fontSize: "0.8rem",
                  color: "#666",
                  backgroundColor: "#f8f9fa",
                  padding: "0.5rem",
                  borderRadius: "4px",
                }}
              >
                <strong>⚠️ Note:</strong> Hardware mode requires STM32H750B-DK.
                Local mode uses post-quantum Falcon512 in WASM.
              </p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

export default App;
