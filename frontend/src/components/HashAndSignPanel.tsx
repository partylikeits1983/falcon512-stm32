/**
 * Hash and Sign Panel Component
 * Handles hashing messages and signing via STM32
 */

import { useState } from "react";
import type { Eip712Message } from "../lib/eip712";
import {
  hashEip712Message,
  hashEip712MessageBytes,
  parseSignature,
  formatSignatureForSolidity,
} from "../lib/eip712";
import { usbTransport } from "../lib/usbTransport";

interface HashAndSignPanelProps {
  message: Eip712Message | null;
  isUsbConnected: boolean;
}

type SigningStatus = "idle" | "hashing" | "signing" | "success" | "error";

export function HashAndSignPanel({
  message,
  isUsbConnected,
}: HashAndSignPanelProps) {
  const [status, setStatus] = useState<SigningStatus>("idle");
  const [hashHex, setHashHex] = useState<string>("");
  const [hashBytes, setHashBytes] = useState<Uint8Array | null>(null);
  const [signatureBytes, setSignatureBytes] = useState<Uint8Array | null>(null);
  const [signatureHex, setSignatureHex] = useState<string>("");
  const [signatureComponents, setSignatureComponents] = useState<{
    r: string;
    s: string;
    v: number;
  } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [signerAddress, setSignerAddress] = useState<string>(
    "0x0000000000000000000000000000000000000000",
  );

  const handleHashMessage = () => {
    if (!message) {
      setError("No message to hash");
      return;
    }

    setStatus("hashing");
    setError(null);

    try {
      // Hash the message
      const hash = hashEip712Message(message);
      const bytes = hashEip712MessageBytes(message);

      setHashHex(hash);
      setHashBytes(bytes);
      setStatus("idle");
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "Failed to hash message";
      setError(errorMessage);
      setStatus("error");
    }
  };

  const handleSignViaStm32 = async () => {
    if (!hashBytes) {
      setError("Please hash the message first");
      return;
    }

    if (!isUsbConnected) {
      setError("Not connected to STM32 device");
      return;
    }

    setStatus("signing");
    setError(null);

    try {
      // Send hash to STM32 for signing
      const signature = await usbTransport.signHash(hashBytes);

      // Parse signature
      const components = parseSignature(signature);
      const sigHex = formatSignatureForSolidity(signature);

      setSignatureBytes(signature);
      setSignatureHex(sigHex);
      setSignatureComponents(components);
      setStatus("success");
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "Failed to sign message";
      setError(errorMessage);
      setStatus("error");
    }
  };

  const copyToClipboard = (text: string, label: string) => {
    navigator.clipboard.writeText(text).then(
      () => alert(`${label} copied to clipboard!`),
      () => alert("Failed to copy to clipboard"),
    );
  };

  const getStatusColor = () => {
    switch (status) {
      case "success":
        return "#10b981";
      case "signing":
      case "hashing":
        return "#f59e0b";
      case "error":
        return "#ef4444";
      default:
        return "#6b7280";
    }
  };

  const getStatusText = () => {
    switch (status) {
      case "hashing":
        return "Hashing message...";
      case "signing":
        return "Waiting for device signature...";
      case "success":
        return "Signature received!";
      case "error":
        return "Error occurred";
      default:
        return "Ready";
    }
  };

  return (
    <div style={styles.container}>
      <div style={styles.card}>
        <h2 style={styles.title}>Hash & Sign</h2>

        <div style={styles.statusContainer}>
          <div style={styles.statusRow}>
            <span style={styles.statusLabel}>Status:</span>
            <div style={styles.statusBadge}>
              <span
                style={{
                  ...styles.statusDot,
                  backgroundColor: getStatusColor(),
                }}
              />
              <span style={styles.statusText}>{getStatusText()}</span>
            </div>
          </div>
        </div>

        {error && (
          <div style={styles.errorBox}>
            <strong>⚠️ Error:</strong>
            <p style={styles.errorText}>{error}</p>
          </div>
        )}

        {/* Step 1: Hash Message */}
        <div style={styles.section}>
          <h3 style={styles.sectionTitle}>Step 1: Hash Message</h3>
          <button
            onClick={handleHashMessage}
            disabled={!message || status === "hashing" || status === "signing"}
            style={{
              ...styles.buttonPrimary,
              opacity:
                !message || status === "hashing" || status === "signing"
                  ? 0.5
                  : 1,
            }}
          >
            {status === "hashing" ? "Hashing..." : "Hash Message (Keccak-256)"}
          </button>

          {hashHex && (
            <div style={styles.resultBox}>
              <div style={styles.resultHeader}>
                <strong>📝 Message Hash</strong>
                <button
                  onClick={() => copyToClipboard(hashHex, "Hash")}
                  style={styles.buttonCopy}
                >
                  Copy
                </button>
              </div>
              <div style={styles.resultContent}>
                <code style={styles.code}>{hashHex}</code>
              </div>
              <div style={styles.resultMeta}>
                Length: {hashBytes?.length || 0} bytes (32 bytes expected)
              </div>
            </div>
          )}
        </div>

        {/* Step 2: Sign via STM32 */}
        <div style={styles.section}>
          <h3 style={styles.sectionTitle}>Step 2: Sign via STM32</h3>
          <button
            onClick={handleSignViaStm32}
            disabled={!hashBytes || !isUsbConnected || status === "signing"}
            style={{
              ...styles.buttonPrimary,
              opacity:
                !hashBytes || !isUsbConnected || status === "signing" ? 0.5 : 1,
            }}
          >
            {status === "signing" ? "Signing..." : "Sign via STM32"}
          </button>

          {!isUsbConnected && hashBytes && (
            <div style={styles.warningBox}>
              ⚠️ Please connect to STM32 device first
            </div>
          )}

          {signatureHex && signatureComponents && (
            <div style={styles.resultBox}>
              <div style={styles.resultHeader}>
                <strong>✅ Signature</strong>
                <button
                  onClick={() => copyToClipboard(signatureHex, "Signature")}
                  style={styles.buttonCopy}
                >
                  Copy
                </button>
              </div>

              <div style={styles.signatureComponents}>
                <div style={styles.component}>
                  <span style={styles.componentLabel}>r:</span>
                  <code style={styles.componentValue}>
                    {signatureComponents.r}
                  </code>
                </div>
                <div style={styles.component}>
                  <span style={styles.componentLabel}>s:</span>
                  <code style={styles.componentValue}>
                    {signatureComponents.s}
                  </code>
                </div>
                <div style={styles.component}>
                  <span style={styles.componentLabel}>v:</span>
                  <code style={styles.componentValue}>
                    {signatureComponents.v}
                  </code>
                </div>
              </div>

              <div style={styles.resultContent}>
                <strong style={styles.resultSubtitle}>
                  Full Signature (Hex):
                </strong>
                <code style={styles.code}>{signatureHex}</code>
              </div>

              <div style={styles.resultMeta}>
                Length: {signatureBytes?.length || 0} bytes (65 bytes expected)
              </div>
            </div>
          )}
        </div>

        {/* Step 3: Web3 Payload Preview */}
        {signatureHex && (
          <div style={styles.section}>
            <h3 style={styles.sectionTitle}>Step 3: Web3 Payload Preview</h3>
            <div style={styles.infoBox}>
              <p style={styles.infoText}>
                <strong>📦 Ready for Solidity Contract</strong>
              </p>
              <p style={styles.infoDescription}>
                This signature can be verified on-chain using a Solidity
                contract with
                <code style={styles.inlineCode}>ecrecover</code> or similar
                verification logic.
              </p>
            </div>

            <div style={styles.payloadBox}>
              <div style={styles.payloadField}>
                <label style={styles.payloadLabel}>
                  Signer Address (on-chain identity):
                </label>
                <input
                  type="text"
                  value={signerAddress}
                  onChange={(e) => setSignerAddress(e.target.value)}
                  placeholder="0x..."
                  style={styles.input}
                />
              </div>

              <div style={styles.payloadField}>
                <label style={styles.payloadLabel}>Message Hash:</label>
                <code style={styles.payloadValue}>{hashHex}</code>
              </div>

              <div style={styles.payloadField}>
                <label style={styles.payloadLabel}>Signature:</label>
                <code style={styles.payloadValue}>{signatureHex}</code>
              </div>
            </div>

            <div style={styles.codePreview}>
              <strong style={styles.codeTitle}>Example Solidity Call:</strong>
              <pre style={styles.codeBlock}>
                {`// Pseudo-code for contract interaction
const contract = new ethers.Contract(
  contractAddress,
  contractAbi,
  provider
);

const isValid = await contract.verify(
  "${signerAddress}",
  message,
  "${signatureHex}"
);`}
              </pre>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

const styles = {
  container: {
    padding: "20px",
  },
  card: {
    backgroundColor: "#ffffff",
    borderRadius: "8px",
    padding: "24px",
    boxShadow: "0 1px 3px rgba(0, 0, 0, 0.1)",
    maxWidth: "800px",
  },
  title: {
    fontSize: "24px",
    fontWeight: "bold",
    marginBottom: "20px",
    color: "#1f2937",
  },
  statusContainer: {
    marginBottom: "20px",
  },
  statusRow: {
    display: "flex",
    alignItems: "center",
    gap: "12px",
  },
  statusLabel: {
    fontSize: "14px",
    fontWeight: "500",
    color: "#6b7280",
  },
  statusBadge: {
    display: "flex",
    alignItems: "center",
    gap: "8px",
    padding: "6px 12px",
    backgroundColor: "#f3f4f6",
    borderRadius: "6px",
  },
  statusDot: {
    width: "8px",
    height: "8px",
    borderRadius: "50%",
  },
  statusText: {
    fontSize: "14px",
    fontWeight: "500",
    color: "#1f2937",
  },
  errorBox: {
    backgroundColor: "#fef2f2",
    border: "1px solid #fecaca",
    borderRadius: "6px",
    padding: "16px",
    marginBottom: "20px",
    color: "#991b1b",
  },
  errorText: {
    margin: "8px 0 0 0",
    fontSize: "14px",
  },
  warningBox: {
    backgroundColor: "#fffbeb",
    border: "1px solid #fde68a",
    borderRadius: "6px",
    padding: "12px",
    marginTop: "12px",
    fontSize: "14px",
    color: "#92400e",
  },
  section: {
    marginBottom: "32px",
    paddingBottom: "32px",
    borderBottom: "1px solid #e5e7eb",
  },
  sectionTitle: {
    fontSize: "18px",
    fontWeight: "600",
    marginBottom: "16px",
    color: "#374151",
  },
  buttonPrimary: {
    backgroundColor: "#3b82f6",
    color: "white",
    padding: "12px 24px",
    borderRadius: "6px",
    border: "none",
    fontSize: "14px",
    fontWeight: "500",
    cursor: "pointer",
    transition: "background-color 0.2s",
  },
  buttonCopy: {
    backgroundColor: "#6b7280",
    color: "white",
    padding: "4px 12px",
    borderRadius: "4px",
    border: "none",
    fontSize: "12px",
    fontWeight: "500",
    cursor: "pointer",
  },
  resultBox: {
    marginTop: "16px",
    backgroundColor: "#f9fafb",
    border: "1px solid #e5e7eb",
    borderRadius: "6px",
    padding: "16px",
  },
  resultHeader: {
    display: "flex",
    justifyContent: "space-between",
    alignItems: "center",
    marginBottom: "12px",
    fontSize: "14px",
    color: "#374151",
  },
  resultContent: {
    marginBottom: "8px",
  },
  resultSubtitle: {
    display: "block",
    fontSize: "13px",
    color: "#6b7280",
    marginBottom: "8px",
  },
  code: {
    display: "block",
    backgroundColor: "#ffffff",
    border: "1px solid #e5e7eb",
    borderRadius: "4px",
    padding: "12px",
    fontSize: "12px",
    fontFamily: "monospace",
    wordBreak: "break-all" as const,
    overflowWrap: "break-word" as const,
  },
  resultMeta: {
    fontSize: "12px",
    color: "#6b7280",
    marginTop: "8px",
  },
  signatureComponents: {
    display: "flex",
    flexDirection: "column" as const,
    gap: "8px",
    marginBottom: "16px",
  },
  component: {
    display: "flex",
    gap: "8px",
    alignItems: "flex-start",
  },
  componentLabel: {
    fontSize: "13px",
    fontWeight: "600",
    color: "#374151",
    minWidth: "20px",
  },
  componentValue: {
    flex: 1,
    fontSize: "12px",
    fontFamily: "monospace",
    wordBreak: "break-all" as const,
  },
  infoBox: {
    backgroundColor: "#eff6ff",
    border: "1px solid #bfdbfe",
    borderRadius: "6px",
    padding: "16px",
    marginBottom: "16px",
  },
  infoText: {
    margin: "0 0 8px 0",
    fontSize: "14px",
    color: "#1e40af",
  },
  infoDescription: {
    margin: 0,
    fontSize: "14px",
    color: "#1e40af",
    lineHeight: "1.5",
  },
  inlineCode: {
    backgroundColor: "#dbeafe",
    padding: "2px 6px",
    borderRadius: "3px",
    fontSize: "13px",
    fontFamily: "monospace",
  },
  payloadBox: {
    backgroundColor: "#f9fafb",
    border: "1px solid #e5e7eb",
    borderRadius: "6px",
    padding: "16px",
    marginBottom: "16px",
  },
  payloadField: {
    marginBottom: "16px",
  },
  payloadLabel: {
    display: "block",
    fontSize: "13px",
    fontWeight: "500",
    color: "#374151",
    marginBottom: "6px",
  },
  input: {
    width: "100%",
    padding: "8px 12px",
    borderRadius: "4px",
    border: "1px solid #d1d5db",
    fontSize: "13px",
    fontFamily: "monospace",
    boxSizing: "border-box" as const,
  },
  payloadValue: {
    display: "block",
    fontSize: "12px",
    fontFamily: "monospace",
    wordBreak: "break-all" as const,
    padding: "8px",
    backgroundColor: "#ffffff",
    border: "1px solid #e5e7eb",
    borderRadius: "4px",
  },
  codePreview: {
    marginTop: "16px",
  },
  codeTitle: {
    display: "block",
    fontSize: "13px",
    color: "#374151",
    marginBottom: "8px",
  },
  codeBlock: {
    backgroundColor: "#1f2937",
    color: "#f9fafb",
    padding: "16px",
    borderRadius: "6px",
    fontSize: "13px",
    fontFamily: "monospace",
    overflow: "auto",
    lineHeight: "1.5",
  },
} as const;
