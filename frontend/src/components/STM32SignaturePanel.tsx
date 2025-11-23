import React, { useState, useEffect } from "react";
import Falcon, { initFalcon } from "../lib/falcon";

interface STM32Response {
  message: string;
  signature: Uint8Array;
  publicKey: Uint8Array;
}

export const STM32SignaturePanel: React.FC = () => {
  const [isInitialized, setIsInitialized] = useState(false);
  const [message, setMessage] = useState("Hello, STM32 Falcon512!");
  const [isConnected, setIsConnected] = useState(false);
  const [port, setPort] = useState<SerialPort | null>(null);
  const [response, setResponse] = useState<STM32Response | null>(null);
  const [verificationResult, setVerificationResult] = useState<boolean | null>(
    null,
  );
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState<string>("");

  useEffect(() => {
    const initWasm = async () => {
      try {
        await initFalcon();
        setIsInitialized(true);
      } catch (err) {
        setError(`Failed to initialize Falcon WASM: ${err}`);
      }
    };
    initWasm();
  }, []);

  const connectToSTM32 = async () => {
    if (!("serial" in navigator)) {
      setError("Web Serial API not supported in this browser");
      return;
    }

    try {
      setIsLoading(true);
      setError(null);
      setStatus("Connecting to STM32...");

      const selectedPort = await navigator.serial.requestPort();
      await selectedPort.open({ baudRate: 115200 });

      setPort(selectedPort);
      setIsConnected(true);
      setStatus("✅ Connected to STM32");
    } catch (err) {
      setError(`Failed to connect: ${err}`);
      setStatus("❌ Connection failed");
    } finally {
      setIsLoading(false);
    }
  };

  const disconnectFromSTM32 = async () => {
    if (port) {
      try {
        await port.close();
        setPort(null);
        setIsConnected(false);
        setStatus("Disconnected");
        setResponse(null);
        setVerificationResult(null);
      } catch (err) {
        setError(`Failed to disconnect: ${err}`);
      }
    }
  };

  const parseSTM32Response = (responseText: string): STM32Response | null => {
    try {
      const lines = responseText
        .split("\n")
        .map((line) => line.trim())
        .filter((line) => line);

      let originalMessage = "";
      let signatureHex = "";
      let publicKeyHex = "";
      let currentSection = "";

      for (const line of lines) {
        if (line === "SIGNED:") {
          currentSection = "message";
          continue;
        } else if (line === "SIGNATURE:") {
          currentSection = "signature";
          continue;
        } else if (line === "PUBLIC_KEY:") {
          currentSection = "publickey";
          continue;
        }

        if (currentSection === "message") {
          originalMessage = line;
        } else if (currentSection === "signature") {
          signatureHex += line;
        } else if (currentSection === "publickey") {
          publicKeyHex += line;
        }
      }

      if (!originalMessage || !signatureHex || !publicKeyHex) {
        throw new Error("Incomplete response from STM32");
      }

      const signature = Falcon.hexToBytes(signatureHex);
      const publicKey = Falcon.hexToBytes(publicKeyHex);

      return {
        message: originalMessage,
        signature,
        publicKey,
      };
    } catch (err) {
      console.error("Failed to parse STM32 response:", err);
      return null;
    }
  };

  const sendMessageToSTM32 = async () => {
    if (!port || !message) return;

    setIsLoading(true);
    setError(null);
    setResponse(null);
    setVerificationResult(null);
    setStatus("📤 Sending message to STM32...");

    try {
      const writer = port.writable?.getWriter();
      const reader = port.readable?.getReader();

      if (!writer || !reader) {
        throw new Error("Failed to get port streams");
      }

      // Send message to STM32
      const messageToSend = message + "\n";
      await writer.write(new TextEncoder().encode(messageToSend));
      writer.releaseLock();

      setStatus("⏳ Waiting for button press on STM32...");

      // Read response
      let responseText = "";
      const timeout = setTimeout(() => {
        reader.releaseLock();
        setError("Timeout waiting for STM32 response");
        setStatus("❌ Timeout");
        setIsLoading(false);
      }, 60000); // 60 second timeout

      try {
        while (true) {
          const { value, done } = await reader.read();
          if (done) break;

          const chunk = new TextDecoder().decode(value);
          responseText += chunk;

          // Check if we have a complete response
          if (
            responseText.includes("PUBLIC_KEY:") &&
            responseText.includes("\n", responseText.lastIndexOf("PUBLIC_KEY:"))
          ) {
            break;
          }
        }
      } finally {
        clearTimeout(timeout);
        reader.releaseLock();
      }

      setStatus("📥 Response received, parsing...");

      // Parse the response
      const parsedResponse = parseSTM32Response(responseText);
      if (!parsedResponse) {
        throw new Error("Failed to parse STM32 response");
      }

      setResponse(parsedResponse);
      setStatus(
        `✅ Signature received!\n📊 Signature: ${parsedResponse.signature.length} bytes\n📊 Public Key: ${parsedResponse.publicKey.length} bytes`,
      );

      // Automatically verify the signature
      setStatus((prev) => prev + "\n🔍 Verifying signature...");

      const messageBytes = new TextEncoder().encode(parsedResponse.message);
      const isValid = await Falcon.verify(
        messageBytes,
        parsedResponse.signature,
        parsedResponse.publicKey,
      );

      setVerificationResult(isValid);
      setStatus(
        (prev) =>
          prev +
          `\n${isValid ? "✅ Signature verified!" : "❌ Signature verification failed!"}`,
      );
    } catch (err) {
      setError(`Failed to communicate with STM32: ${err}`);
      setStatus("❌ Communication failed");
    } finally {
      setIsLoading(false);
    }
  };

  if (!isInitialized) {
    return (
      <div className="p-6 bg-white rounded-lg shadow-md">
        <h2 className="text-xl font-bold mb-4">STM32 Falcon512 Signing</h2>
        <p>Initializing Falcon WASM module...</p>
      </div>
    );
  }

  return (
    <div className="p-6 bg-white rounded-lg shadow-md">
      <h2 className="text-xl font-bold mb-4">
        STM32 Falcon512 Hardware Signing
      </h2>

      {error && (
        <div className="mb-4 p-3 bg-red-100 border border-red-400 text-red-700 rounded">
          {error}
        </div>
      )}

      <div className="space-y-4">
        {/* Connection Status */}
        <div>
          <div className="flex items-center gap-2 mb-2">
            <span
              className={`w-3 h-3 rounded-full ${isConnected ? "bg-green-500" : "bg-gray-400"}`}
            ></span>
            <span className="font-medium">
              {isConnected ? "Connected to STM32" : "Not connected"}
            </span>
          </div>

          {!isConnected ? (
            <button
              onClick={connectToSTM32}
              disabled={isLoading}
              className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:opacity-50"
            >
              {isLoading ? "Connecting..." : "Connect to STM32"}
            </button>
          ) : (
            <button
              onClick={disconnectFromSTM32}
              className="px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600"
            >
              Disconnect
            </button>
          )}
        </div>

        {/* Message Input */}
        {isConnected && (
          <>
            <div>
              <label className="block text-sm font-medium mb-1">
                Message to Sign:
              </label>
              <input
                type="text"
                value={message}
                onChange={(e) => setMessage(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                placeholder="Enter message to sign"
                disabled={isLoading}
              />
            </div>

            {/* Sign Button */}
            <div>
              <button
                onClick={sendMessageToSTM32}
                disabled={!message || isLoading}
                className="px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600 disabled:opacity-50"
              >
                {isLoading ? "Processing..." : "Send to STM32 for Signing"}
              </button>
            </div>
          </>
        )}

        {/* Status Display */}
        {status && (
          <div className="p-3 bg-gray-100 rounded">
            <h3 className="font-semibold mb-1">Status:</h3>
            <pre className="text-sm whitespace-pre-wrap">{status}</pre>
          </div>
        )}

        {/* Response Details */}
        {response && (
          <div className="p-3 bg-blue-50 rounded">
            <h3 className="font-semibold mb-2">Signature Details:</h3>
            <div className="text-sm space-y-1">
              <p>
                <strong>Original Message:</strong> {response.message}
              </p>
              <p>
                <strong>Signature:</strong>{" "}
                {Falcon.bytesToHex(response.signature).substring(0, 64)}...
              </p>
              <p>
                <strong>Public Key:</strong>{" "}
                {Falcon.bytesToHex(response.publicKey).substring(0, 64)}...
              </p>
              {verificationResult !== null && (
                <p
                  className={`font-semibold ${verificationResult ? "text-green-600" : "text-red-600"}`}
                >
                  <strong>Verification:</strong>{" "}
                  {verificationResult ? "✅ Valid" : "❌ Invalid"}
                </p>
              )}
            </div>
          </div>
        )}

        {/* Instructions */}
        <div className="mt-6 p-4 bg-gray-100 rounded">
          <h3 className="font-semibold mb-2">Instructions:</h3>
          <ol className="text-sm space-y-1 list-decimal list-inside">
            <li>Connect to your STM32 device via USB</li>
            <li>Enter a message to sign</li>
            <li>Click "Send to STM32 for Signing"</li>
            <li>Press the button on your STM32 when the LED flashes</li>
            <li>The signature will be automatically verified</li>
          </ol>
        </div>
      </div>
    </div>
  );
};

export default STM32SignaturePanel;
