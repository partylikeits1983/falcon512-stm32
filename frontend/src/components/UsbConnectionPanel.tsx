/**
 * USB Connection Panel Component
 * Handles connection/disconnection to STM32 device via Web Serial API
 */

import { useState, useEffect } from "react";
import { usbTransport, UsbTransport } from "../lib/usbTransport";

export type ConnectionStatus =
  | "disconnected"
  | "connecting"
  | "connected"
  | "error";

interface UsbConnectionPanelProps {
  onConnectionChange?: (connected: boolean) => void;
}

export function UsbConnectionPanel({
  onConnectionChange,
}: UsbConnectionPanelProps) {
  const [status, setStatus] = useState<ConnectionStatus>("disconnected");
  const [error, setError] = useState<string | null>(null);
  const [isSupported, setIsSupported] = useState(true);

  useEffect(() => {
    // Check if Web Serial API is supported
    setIsSupported(UsbTransport.isSupported());
  }, []);

  const handleConnect = async () => {
    setStatus("connecting");
    setError(null);

    try {
      await usbTransport.connect();
      setStatus("connected");
      onConnectionChange?.(true);
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "Unknown error occurred";
      setError(errorMessage);
      setStatus("error");
      onConnectionChange?.(false);
    }
  };

  const handleDisconnect = async () => {
    try {
      await usbTransport.disconnect();
      setStatus("disconnected");
      setError(null);
      onConnectionChange?.(false);
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "Unknown error occurred";
      setError(errorMessage);
    }
  };

  const getStatusColor = () => {
    switch (status) {
      case "connected":
        return "#10b981"; // green
      case "connecting":
        return "#f59e0b"; // amber
      case "error":
        return "#ef4444"; // red
      default:
        return "#6b7280"; // gray
    }
  };

  const getStatusText = () => {
    switch (status) {
      case "connected":
        return "Connected";
      case "connecting":
        return "Connecting...";
      case "error":
        return "Error";
      default:
        return "Disconnected";
    }
  };

  if (!isSupported) {
    return (
      <div style={styles.container}>
        <div style={styles.card}>
          <h2 style={styles.title}>USB Connection</h2>
          <div style={{ ...styles.alert, ...styles.alertError }}>
            <strong>⚠️ Web Serial API Not Supported</strong>
            <p style={styles.alertText}>
              Your browser doesn't support the Web Serial API. Please use:
            </p>
            <ul style={styles.list}>
              <li>Chrome 89+</li>
              <li>Edge 89+</li>
              <li>Opera 75+</li>
            </ul>
            <p style={styles.alertText}>
              Make sure to enable the feature flag if needed:
              <code style={styles.code}>
                chrome://flags/#enable-experimental-web-platform-features
              </code>
            </p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div style={styles.container}>
      <div style={styles.card}>
        <h2 style={styles.title}>USB Connection</h2>

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
          <div style={{ ...styles.alert, ...styles.alertError }}>
            <strong>Error:</strong>
            <p style={styles.alertText}>{error}</p>
          </div>
        )}

        <div style={styles.buttonContainer}>
          {status === "disconnected" || status === "error" ? (
            <button onClick={handleConnect} style={styles.buttonPrimary}>
              Connect to STM32
            </button>
          ) : status === "connected" ? (
            <button onClick={handleDisconnect} style={styles.buttonSecondary}>
              Disconnect
            </button>
          ) : (
            <button style={{ ...styles.buttonPrimary, opacity: 0.6 }} disabled>
              Connecting...
            </button>
          )}
        </div>

        <div style={styles.infoBox}>
          <p style={styles.infoText}>
            <strong>ℹ️ Instructions:</strong>
          </p>
          <ol style={styles.list}>
            <li>Connect your STM32 device via USB</li>
            <li>Click "Connect to STM32"</li>
            <li>Select the correct serial port from the browser dialog</li>
            <li>Wait for connection to establish</li>
          </ol>
        </div>
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
    maxWidth: "600px",
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
  alert: {
    padding: "16px",
    borderRadius: "6px",
    marginBottom: "20px",
  },
  alertError: {
    backgroundColor: "#fef2f2",
    border: "1px solid #fecaca",
    color: "#991b1b",
  },
  alertText: {
    margin: "8px 0",
    fontSize: "14px",
    lineHeight: "1.5",
  },
  list: {
    marginLeft: "20px",
    marginTop: "8px",
    fontSize: "14px",
    lineHeight: "1.6",
  },
  code: {
    backgroundColor: "#f3f4f6",
    padding: "2px 6px",
    borderRadius: "4px",
    fontSize: "13px",
    fontFamily: "monospace",
    display: "inline-block",
    marginTop: "8px",
  },
  buttonContainer: {
    display: "flex",
    gap: "12px",
    marginBottom: "20px",
  },
  buttonPrimary: {
    backgroundColor: "#3b82f6",
    color: "white",
    padding: "10px 20px",
    borderRadius: "6px",
    border: "none",
    fontSize: "14px",
    fontWeight: "500",
    cursor: "pointer",
    transition: "background-color 0.2s",
  },
  buttonSecondary: {
    backgroundColor: "#ef4444",
    color: "white",
    padding: "10px 20px",
    borderRadius: "6px",
    border: "none",
    fontSize: "14px",
    fontWeight: "500",
    cursor: "pointer",
    transition: "background-color 0.2s",
  },
  infoBox: {
    backgroundColor: "#eff6ff",
    border: "1px solid #bfdbfe",
    borderRadius: "6px",
    padding: "16px",
  },
  infoText: {
    margin: "0 0 8px 0",
    fontSize: "14px",
    color: "#1e40af",
  },
} as const;
