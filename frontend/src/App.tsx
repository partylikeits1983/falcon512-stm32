/**
 * Main App Component
 * STM32 Falcon512 Signing Interface
 */

import { useState } from 'react';
import './App.css';

function App() {
  const [port, setPort] = useState<SerialPort | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const [message, setMessage] = useState('Hello, STM32!');
  const [status, setStatus] = useState('');
  const [signature, setSignature] = useState('');
  const [publicKey, setPublicKey] = useState('');
  const [isProcessing, setIsProcessing] = useState(false);

  const connectToDevice = async () => {
    try {
      setStatus('Requesting port...');
      const selectedPort = await navigator.serial.requestPort();
      
      setStatus('Opening port at 115200 baud...');
      await selectedPort.open({ baudRate: 115200 });
      
      // Get port info if available
      const info = selectedPort.getInfo();
      let portInfo = '';
      if (info.usbVendorId && info.usbProductId) {
        portInfo = `\nVID: 0x${info.usbVendorId.toString(16).padStart(4, '0')}, PID: 0x${info.usbProductId.toString(16).padStart(4, '0')}`;
      }
      
      setPort(selectedPort);
      setIsConnected(true);
      setStatus(`✅ Connected to STM32!${portInfo}\n\nReady to sign messages.`);
    } catch (error) {
      setStatus(`❌ Error: ${error instanceof Error ? error.message : 'Failed to connect'}`);
    }
  };

  const disconnect = async () => {
    if (port) {
      try {
        await port.close();
        setPort(null);
        setIsConnected(false);
        setStatus('Disconnected');
      } catch (error) {
        setStatus(`Error disconnecting: ${error instanceof Error ? error.message : 'Unknown error'}`);
      }
    }
  };

  const signMessage = async () => {
    if (!port || !isConnected) {
      setStatus('❌ Not connected to device');
      return;
    }

    setIsProcessing(true);
    setStatus('📤 Sending message to STM32...');
    setSignature('');
    setPublicKey('');

    try {
      // Send message with newline
      const writer = port.writable?.getWriter();
      if (!writer) {
        throw new Error('Port not writable');
      }

      const messageWithNewline = message + '\n';
      const encoder = new TextEncoder();
      await writer.write(encoder.encode(messageWithNewline));
      writer.releaseLock();

      setStatus('⏳ Waiting for STM32 response...\n👆 Press button B0 on the STM32 board to sign');

      // Read response
      const reader = port.readable?.getReader();
      if (!reader) {
        throw new Error('Port not readable');
      }

      let response = '';
      const decoder = new TextDecoder();
      const timeout = setTimeout(() => {
        reader.cancel();
        setStatus('❌ Timeout waiting for response');
        setIsProcessing(false);
      }, 30000);

      try {
        while (true) {
          const { value, done } = await reader.read();
          if (done) break;
          
          const chunk = decoder.decode(value, { stream: true });
          response += chunk;

          // Check if we have complete response
          if (response.includes('PUBLIC_KEY:') && response.includes('\n', response.indexOf('PUBLIC_KEY:'))) {
            break;
          }
        }
      } finally {
        clearTimeout(timeout);
        reader.releaseLock();
      }

      // Parse response
      const sigMatch = response.match(/SIGNATURE:\s*([0-9a-fA-F\s\n]+)PUBLIC_KEY:/);
      const pkMatch = response.match(/PUBLIC_KEY:\s*([0-9a-fA-F\s\n]+)/);

      if (sigMatch && pkMatch) {
        const sig = sigMatch[1].replace(/\s/g, '');
        const pk = pkMatch[1].replace(/\s/g, '');
        
        setSignature(sig);
        setPublicKey(pk);
        setStatus('✅ Signature received!\n📊 Signature: ' + sig.length / 2 + ' bytes\n📊 Public Key: ' + pk.length / 2 + ' bytes');
      } else {
        setStatus('❌ Failed to parse response:\n' + response);
      }
    } catch (error) {
      setStatus(`❌ Error: ${error instanceof Error ? error.message : 'Unknown error'}`);
    } finally {
      setIsProcessing(false);
    }
  };

  const isWebSerialSupported = 'serial' in navigator;

  return (
    <div className="app">
      <div className="container">
        <header className="header">
          <h1>🔐 STM32 Falcon512 Signer</h1>
          <p>Sign messages using your STM32 hardware device</p>
        </header>

        {!isWebSerialSupported ? (
          <div className="error-box">
            <h2>⚠️ Web Serial API Not Supported</h2>
            <p>Please use Chrome, Edge, or Opera browser.</p>
          </div>
        ) : (
          <div className="main-content">
            {/* Connection Section */}
            <div className="card">
              <h2>USB Connection</h2>
              <div className="status-row">
                <span className="status-label">Status:</span>
                <span className={`status-badge ${isConnected ? 'connected' : 'disconnected'}`}>
                  {isConnected ? '🟢 Connected' : '⚫ Disconnected'}
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
            </div>

            {/* Message Input Section */}
            <div className="card">
              <h2>Message to Sign</h2>
              <textarea
                value={message}
                onChange={(e) => setMessage(e.target.value)}
                placeholder="Enter message to sign..."
                className="message-input"
                rows={4}
              />
              <button
                onClick={signMessage}
                disabled={!isConnected || isProcessing}
                className="btn-primary"
              >
                {isProcessing ? '⏳ Processing...' : '✍️ Sign Message'}
              </button>
            </div>

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
              <h3>📋 Instructions</h3>
              <ol>
                <li>Connect your STM32H750B-DK board via USB (CN13 connector)</li>
                <li>Click "Connect to STM32" and select the serial port</li>
                <li>Enter a message to sign</li>
                <li>Click "Sign Message"</li>
                <li>Press button B0 on the STM32 board when prompted</li>
                <li>Wait for the signature to appear</li>
              </ol>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

export default App;
