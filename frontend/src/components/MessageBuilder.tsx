/**
 * Message Builder Component
 * Allows users to construct EIP-712 messages
 */

import { useState } from 'react';
import type { Eip712Message } from '../lib/eip712';
import { createSampleMessage, validateMessage } from '../lib/eip712';

interface MessageBuilderProps {
  onMessageChange?: (message: Eip712Message) => void;
}

export function MessageBuilder({ onMessageChange }: MessageBuilderProps) {
  const [message, setMessage] = useState<Eip712Message>(createSampleMessage());
  const [errors, setErrors] = useState<string[]>([]);

  const handleFieldChange = (field: keyof Eip712Message, value: string) => {
    const newMessage = { ...message, [field]: value };
    setMessage(newMessage);
    
    // Validate and update errors
    const validation = validateMessage(newMessage);
    setErrors(validation.errors);
    
    // Notify parent if valid
    if (validation.valid) {
      onMessageChange?.(newMessage);
    }
  };

  const loadSampleMessage = () => {
    const sample = createSampleMessage();
    setMessage(sample);
    setErrors([]);
    onMessageChange?.(sample);
  };

  return (
    <div style={styles.container}>
      <div style={styles.card}>
        <div style={styles.header}>
          <h2 style={styles.title}>Message Builder</h2>
          <button onClick={loadSampleMessage} style={styles.buttonSecondary}>
            Load Sample
          </button>
        </div>

        <div style={styles.typeSelector}>
          <span style={styles.label}>Message Type:</span>
          <span style={styles.badge}>EIP-712</span>
          <span style={styles.hint}>(ERC-7730 coming soon)</span>
        </div>

        <div style={styles.form}>
          <div style={styles.field}>
            <label style={styles.fieldLabel}>
              From Address
              <span style={styles.required}>*</span>
            </label>
            <input
              type="text"
              value={message.from}
              onChange={(e) => handleFieldChange('from', e.target.value)}
              placeholder="0x..."
              style={styles.input}
            />
            <span style={styles.fieldHint}>Sender's Ethereum address</span>
          </div>

          <div style={styles.field}>
            <label style={styles.fieldLabel}>
              To Address
              <span style={styles.required}>*</span>
            </label>
            <input
              type="text"
              value={message.to}
              onChange={(e) => handleFieldChange('to', e.target.value)}
              placeholder="0x..."
              style={styles.input}
            />
            <span style={styles.fieldHint}>Recipient's Ethereum address</span>
          </div>

          <div style={styles.field}>
            <label style={styles.fieldLabel}>
              Value (wei)
              <span style={styles.required}>*</span>
            </label>
            <input
              type="text"
              value={message.value}
              onChange={(e) => handleFieldChange('value', e.target.value)}
              placeholder="1000000000000000000"
              style={styles.input}
            />
            <span style={styles.fieldHint}>
              Amount in wei (1 ETH = 1000000000000000000 wei)
            </span>
          </div>

          <div style={styles.field}>
            <label style={styles.fieldLabel}>
              Nonce
              <span style={styles.required}>*</span>
            </label>
            <input
              type="text"
              value={message.nonce}
              onChange={(e) => handleFieldChange('nonce', e.target.value)}
              placeholder="0"
              style={styles.input}
            />
            <span style={styles.fieldHint}>Transaction nonce (usually incrementing)</span>
          </div>
        </div>

        {errors.length > 0 && (
          <div style={styles.errorBox}>
            <strong>⚠️ Validation Errors:</strong>
            <ul style={styles.errorList}>
              {errors.map((error, idx) => (
                <li key={idx}>{error}</li>
              ))}
            </ul>
          </div>
        )}

        <div style={styles.previewBox}>
          <div style={styles.previewHeader}>
            <strong>📄 Message Preview (JSON)</strong>
          </div>
          <pre style={styles.previewContent}>
            {JSON.stringify(message, null, 2)}
          </pre>
        </div>

        <div style={styles.infoBox}>
          <p style={styles.infoText}>
            <strong>ℹ️ About EIP-712:</strong>
          </p>
          <p style={styles.infoDescription}>
            EIP-712 is a standard for hashing and signing typed structured data. This message
            will be hashed using the Keccak-256 algorithm and then signed by your STM32 device.
          </p>
        </div>
      </div>
    </div>
  );
}

const styles = {
  container: {
    padding: '20px',
  },
  card: {
    backgroundColor: '#ffffff',
    borderRadius: '8px',
    padding: '24px',
    boxShadow: '0 1px 3px rgba(0, 0, 0, 0.1)',
    maxWidth: '600px',
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
  buttonSecondary: {
    backgroundColor: '#6b7280',
    color: 'white',
    padding: '8px 16px',
    borderRadius: '6px',
    border: 'none',
    fontSize: '14px',
    fontWeight: '500',
    cursor: 'pointer',
    transition: 'background-color 0.2s',
  },
  typeSelector: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    marginBottom: '24px',
    padding: '12px',
    backgroundColor: '#f9fafb',
    borderRadius: '6px',
  },
  label: {
    fontSize: '14px',
    fontWeight: '500',
    color: '#374151',
  },
  badge: {
    backgroundColor: '#3b82f6',
    color: 'white',
    padding: '4px 12px',
    borderRadius: '4px',
    fontSize: '12px',
    fontWeight: '600',
  },
  hint: {
    fontSize: '12px',
    color: '#9ca3af',
    fontStyle: 'italic',
  },
  form: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '20px',
    marginBottom: '20px',
  },
  field: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '6px',
  },
  fieldLabel: {
    fontSize: '14px',
    fontWeight: '500',
    color: '#374151',
  },
  required: {
    color: '#ef4444',
    marginLeft: '4px',
  },
  input: {
    padding: '10px 12px',
    borderRadius: '6px',
    border: '1px solid #d1d5db',
    fontSize: '14px',
    fontFamily: 'monospace',
    transition: 'border-color 0.2s',
  },
  fieldHint: {
    fontSize: '12px',
    color: '#6b7280',
  },
  errorBox: {
    backgroundColor: '#fef2f2',
    border: '1px solid #fecaca',
    borderRadius: '6px',
    padding: '16px',
    marginBottom: '20px',
    color: '#991b1b',
  },
  errorList: {
    marginTop: '8px',
    marginLeft: '20px',
    fontSize: '14px',
  },
  previewBox: {
    marginBottom: '20px',
  },
  previewHeader: {
    fontSize: '14px',
    fontWeight: '500',
    color: '#374151',
    marginBottom: '8px',
  },
  previewContent: {
    backgroundColor: '#f9fafb',
    border: '1px solid #e5e7eb',
    borderRadius: '6px',
    padding: '16px',
    fontSize: '13px',
    fontFamily: 'monospace',
    overflow: 'auto',
    maxHeight: '200px',
  },
  infoBox: {
    backgroundColor: '#eff6ff',
    border: '1px solid #bfdbfe',
    borderRadius: '6px',
    padding: '16px',
  },
  infoText: {
    margin: '0 0 8px 0',
    fontSize: '14px',
    color: '#1e40af',
  },
  infoDescription: {
    margin: 0,
    fontSize: '14px',
    color: '#1e40af',
    lineHeight: '1.5',
  },
} as const;