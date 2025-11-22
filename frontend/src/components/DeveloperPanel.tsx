/**
 * Developer Panel Component
 * Shows raw USB packets and debug information
 */

import { useState, useEffect } from 'react';
import { usbTransport } from '../lib/usbTransport';
import { bytesToHex } from '../lib/eip712';

interface PacketLog {
  timestamp: string;
  direction: 'sent' | 'received';
  data: Uint8Array;
  description: string;
}

export function DeveloperPanel() {
  const [isExpanded, setIsExpanded] = useState(false);
  const [packets, setPackets] = useState<PacketLog[]>([]);
  const [maxLogs, setMaxLogs] = useState(50);

  useEffect(() => {
    // Listen for incoming messages
    const cleanup = usbTransport.onMessage((msg) => {
      const cmd = msg[0];
      const payload = msg.slice(1);
      
      let description = 'Unknown command';
      if (cmd === 0x02) {
        description = `SIGNATURE (${payload.length} bytes)`;
      } else if (cmd === 0xff) {
        description = `ERROR (code: ${payload.length > 0 ? payload[0] : 'unknown'})`;
      }

      addPacket('received', msg, description);
    });

    return cleanup;
  }, []);

  const addPacket = (direction: 'sent' | 'received', data: Uint8Array, description: string) => {
    const timestamp = new Date().toLocaleTimeString('en-US', {
      hour12: false,
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
      fractionalSecondDigits: 3,
    });

    setPackets((prev) => {
      const newPackets = [
        { timestamp, direction, data, description },
        ...prev,
      ];
      return newPackets.slice(0, maxLogs);
    });
  };

  const clearLogs = () => {
    setPackets([]);
  };

  const exportLogs = () => {
    const logsText = packets
      .map((p) => {
        const hex = bytesToHex(p.data, false);
        return `[${p.timestamp}] ${p.direction.toUpperCase()}: ${p.description}\nHex: ${hex}\n`;
      })
      .join('\n');

    const blob = new Blob([logsText], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `usb-logs-${Date.now()}.txt`;
    a.click();
    URL.revokeObjectURL(url);
  };

  if (!isExpanded) {
    return (
      <div style={styles.collapsedContainer}>
        <button onClick={() => setIsExpanded(true)} style={styles.expandButton}>
          🔧 Developer Mode
        </button>
      </div>
    );
  }

  return (
    <div style={styles.container}>
      <div style={styles.card}>
        <div style={styles.header}>
          <h2 style={styles.title}>🔧 Developer Mode</h2>
          <button onClick={() => setIsExpanded(false)} style={styles.collapseButton}>
            Collapse
          </button>
        </div>

        <div style={styles.controls}>
          <button onClick={clearLogs} style={styles.buttonSecondary}>
            Clear Logs
          </button>
          <button onClick={exportLogs} style={styles.buttonSecondary} disabled={packets.length === 0}>
            Export Logs
          </button>
          <div style={styles.maxLogsControl}>
            <label style={styles.label}>Max logs:</label>
            <input
              type="number"
              value={maxLogs}
              onChange={(e) => setMaxLogs(Math.max(10, parseInt(e.target.value) || 50))}
              style={styles.numberInput}
              min="10"
              max="500"
            />
          </div>
        </div>

        <div style={styles.infoBox}>
          <p style={styles.infoText}>
            <strong>ℹ️ About:</strong> This panel shows raw USB packets exchanged with the STM32 device.
            Useful for debugging communication issues.
          </p>
        </div>

        <div style={styles.logsContainer}>
          <div style={styles.logsHeader}>
            <strong>Packet Logs ({packets.length})</strong>
          </div>
          
          {packets.length === 0 ? (
            <div style={styles.emptyState}>
              No packets logged yet. Connect to device and perform operations to see logs.
            </div>
          ) : (
            <div style={styles.logsList}>
              {packets.map((packet, idx) => (
                <div
                  key={idx}
                  style={{
                    ...styles.logEntry,
                    ...(packet.direction === 'sent' ? styles.logEntrySent : styles.logEntryReceived),
                  }}
                >
                  <div style={styles.logHeader}>
                    <span style={styles.logTimestamp}>{packet.timestamp}</span>
                    <span
                      style={{
                        ...styles.logDirection,
                        ...(packet.direction === 'sent'
                          ? styles.logDirectionSent
                          : styles.logDirectionReceived),
                      }}
                    >
                      {packet.direction === 'sent' ? '→ SENT' : '← RECEIVED'}
                    </span>
                    <span style={styles.logDescription}>{packet.description}</span>
                  </div>
                  <div style={styles.logData}>
                    <div style={styles.logDataLabel}>Hex ({packet.data.length} bytes):</div>
                    <code style={styles.logDataHex}>{bytesToHex(packet.data, false)}</code>
                  </div>
                  <div style={styles.logData}>
                    <div style={styles.logDataLabel}>Bytes:</div>
                    <code style={styles.logDataBytes}>
                      [{Array.from(packet.data).join(', ')}]
                    </code>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        <div style={styles.protocolInfo}>
          <strong style={styles.protocolTitle}>Protocol Reference:</strong>
          <div style={styles.protocolTable}>
            <div style={styles.protocolRow}>
              <span style={styles.protocolLabel}>Start Byte:</span>
              <code style={styles.protocolValue}>0xAA</code>
            </div>
            <div style={styles.protocolRow}>
              <span style={styles.protocolLabel}>CMD_SIGN_HASH:</span>
              <code style={styles.protocolValue}>0x01</code>
            </div>
            <div style={styles.protocolRow}>
              <span style={styles.protocolLabel}>CMD_SIGNATURE:</span>
              <code style={styles.protocolValue}>0x02</code>
            </div>
            <div style={styles.protocolRow}>
              <span style={styles.protocolLabel}>CMD_ERROR:</span>
              <code style={styles.protocolValue}>0xFF</code>
            </div>
          </div>
          <div style={styles.protocolFormat}>
            <strong>Packet Format:</strong>
            <code style={styles.protocolFormatCode}>
              START (1) | CMD (1) | LEN_HIGH (1) | LEN_LOW (1) | PAYLOAD (N) | CRC8 (1)
            </code>
          </div>
        </div>
      </div>
    </div>
  );
}

const styles = {
  collapsedContainer: {
    padding: '20px',
  },
  expandButton: {
    backgroundColor: '#6b7280',
    color: 'white',
    padding: '10px 20px',
    borderRadius: '6px',
    border: 'none',
    fontSize: '14px',
    fontWeight: '500',
    cursor: 'pointer',
  },
  container: {
    padding: '20px',
  },
  card: {
    backgroundColor: '#ffffff',
    borderRadius: '8px',
    padding: '24px',
    boxShadow: '0 1px 3px rgba(0, 0, 0, 0.1)',
    maxWidth: '1000px',
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '20px',
  },
  title: {
    fontSize: '24px',
    fontWeight: 'bold',
    color: '#1f2937',
    margin: 0,
  },
  collapseButton: {
    backgroundColor: '#6b7280',
    color: 'white',
    padding: '8px 16px',
    borderRadius: '6px',
    border: 'none',
    fontSize: '14px',
    fontWeight: '500',
    cursor: 'pointer',
  },
  controls: {
    display: 'flex',
    gap: '12px',
    marginBottom: '16px',
    alignItems: 'center',
  },
  buttonSecondary: {
    backgroundColor: '#6b7280',
    color: 'white',
    padding: '8px 16px',
    borderRadius: '6px',
    border: 'none',
    fontSize: '14px',
    fontWeight: '500',
    cursor: 'pointer',
  },
  maxLogsControl: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    marginLeft: 'auto',
  },
  label: {
    fontSize: '14px',
    color: '#374151',
  },
  numberInput: {
    width: '80px',
    padding: '6px 10px',
    borderRadius: '4px',
    border: '1px solid #d1d5db',
    fontSize: '14px',
  },
  infoBox: {
    backgroundColor: '#eff6ff',
    border: '1px solid #bfdbfe',
    borderRadius: '6px',
    padding: '12px',
    marginBottom: '16px',
  },
  infoText: {
    margin: 0,
    fontSize: '14px',
    color: '#1e40af',
  },
  logsContainer: {
    backgroundColor: '#f9fafb',
    border: '1px solid #e5e7eb',
    borderRadius: '6px',
    marginBottom: '20px',
  },
  logsHeader: {
    padding: '12px 16px',
    borderBottom: '1px solid #e5e7eb',
    fontSize: '14px',
    fontWeight: '600',
    color: '#374151',
  },
  emptyState: {
    padding: '40px 16px',
    textAlign: 'center' as const,
    color: '#6b7280',
    fontSize: '14px',
  },
  logsList: {
    maxHeight: '400px',
    overflowY: 'auto' as const,
  },
  logEntry: {
    padding: '12px 16px',
    borderBottom: '1px solid #e5e7eb',
  },
  logEntrySent: {
    backgroundColor: '#fef3c7',
  },
  logEntryReceived: {
    backgroundColor: '#dbeafe',
  },
  logHeader: {
    display: 'flex',
    gap: '12px',
    alignItems: 'center',
    marginBottom: '8px',
    fontSize: '13px',
  },
  logTimestamp: {
    fontFamily: 'monospace',
    color: '#6b7280',
  },
  logDirection: {
    padding: '2px 8px',
    borderRadius: '4px',
    fontSize: '11px',
    fontWeight: '600',
  },
  logDirectionSent: {
    backgroundColor: '#fbbf24',
    color: '#78350f',
  },
  logDirectionReceived: {
    backgroundColor: '#3b82f6',
    color: 'white',
  },
  logDescription: {
    color: '#374151',
    fontWeight: '500',
  },
  logData: {
    marginBottom: '6px',
  },
  logDataLabel: {
    fontSize: '11px',
    color: '#6b7280',
    marginBottom: '4px',
  },
  logDataHex: {
    display: 'block',
    fontSize: '11px',
    fontFamily: 'monospace',
    wordBreak: 'break-all' as const,
    color: '#1f2937',
  },
  logDataBytes: {
    display: 'block',
    fontSize: '11px',
    fontFamily: 'monospace',
    wordBreak: 'break-all' as const,
    color: '#1f2937',
  },
  protocolInfo: {
    backgroundColor: '#f9fafb',
    border: '1px solid #e5e7eb',
    borderRadius: '6px',
    padding: '16px',
  },
  protocolTitle: {
    display: 'block',
    fontSize: '14px',
    color: '#374151',
    marginBottom: '12px',
  },
  protocolTable: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '6px',
    marginBottom: '12px',
  },
  protocolRow: {
    display: 'flex',
    justifyContent: 'space-between',
    fontSize: '13px',
  },
  protocolLabel: {
    color: '#6b7280',
  },
  protocolValue: {
    fontFamily: 'monospace',
    backgroundColor: '#ffffff',
    padding: '2px 6px',
    borderRadius: '3px',
    fontSize: '12px',
  },
  protocolFormat: {
    marginTop: '12px',
    paddingTop: '12px',
    borderTop: '1px solid #e5e7eb',
  },
  protocolFormatCode: {
    display: 'block',
    marginTop: '8px',
    fontSize: '12px',
    fontFamily: 'monospace',
    backgroundColor: '#ffffff',
    padding: '8px',
    borderRadius: '4px',
  },
} as const;