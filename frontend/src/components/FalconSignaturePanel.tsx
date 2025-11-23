import React, { useState, useEffect } from 'react';
import Falcon, { initFalcon } from '../lib/falcon';

export const FalconSignaturePanel: React.FC = () => {
  const [isInitialized, setIsInitialized] = useState(false);
  const [keys, setKeys] = useState<{ publicKey: Uint8Array; secretKey: Uint8Array } | null>(null);
  const [message, setMessage] = useState('Hello, Falcon512!');
  const [signature, setSignature] = useState<Uint8Array | null>(null);
  const [verificationResult, setVerificationResult] = useState<boolean | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

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

  const generateKeys = async () => {
    if (!isInitialized) return;
    
    setIsLoading(true);
    setError(null);
    try {
      const newKeys = await Falcon.generateKeyPair();
      setKeys(newKeys);
      setSignature(null);
      setVerificationResult(null);
    } catch (err) {
      setError(`Failed to generate keys: ${err}`);
    } finally {
      setIsLoading(false);
    }
  };

  const signMessage = async () => {
    if (!keys || !message) return;
    
    setIsLoading(true);
    setError(null);
    try {
      const messageBytes = new TextEncoder().encode(message);
      const sig = await Falcon.sign(messageBytes, keys.secretKey);
      setSignature(sig);
      setVerificationResult(null);
    } catch (err) {
      setError(`Failed to sign message: ${err}`);
    } finally {
      setIsLoading(false);
    }
  };

  const verifySignature = async () => {
    if (!keys || !signature || !message) return;
    
    setIsLoading(true);
    setError(null);
    try {
      const messageBytes = new TextEncoder().encode(message);
      const isValid = await Falcon.verify(messageBytes, signature, keys.publicKey);
      setVerificationResult(isValid);
    } catch (err) {
      setError(`Failed to verify signature: ${err}`);
    } finally {
      setIsLoading(false);
    }
  };

  if (!isInitialized) {
    return (
      <div className="p-6 bg-white rounded-lg shadow-md">
        <h2 className="text-xl font-bold mb-4">Falcon512 Signatures</h2>
        <p>Initializing Falcon WASM module...</p>
      </div>
    );
  }

  return (
    <div className="p-6 bg-white rounded-lg shadow-md">
      <h2 className="text-xl font-bold mb-4">Falcon512 Post-Quantum Signatures</h2>
      
      {error && (
        <div className="mb-4 p-3 bg-red-100 border border-red-400 text-red-700 rounded">
          {error}
        </div>
      )}

      <div className="space-y-4">
        {/* Key Generation */}
        <div>
          <button
            onClick={generateKeys}
            disabled={isLoading}
            className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:opacity-50"
          >
            {isLoading ? 'Generating...' : 'Generate New Key Pair'}
          </button>
          
          {keys && (
            <div className="mt-2 text-sm">
              <p><strong>Public Key:</strong> {Falcon.bytesToHex(keys.publicKey).substring(0, 32)}...</p>
              <p><strong>Secret Key:</strong> {Falcon.bytesToHex(keys.secretKey).substring(0, 32)}...</p>
            </div>
          )}
        </div>

        {/* Message Input */}
        <div>
          <label className="block text-sm font-medium mb-1">Message to Sign:</label>
          <input
            type="text"
            value={message}
            onChange={(e) => setMessage(e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
            placeholder="Enter message to sign"
          />
        </div>

        {/* Sign Button */}
        <div>
          <button
            onClick={signMessage}
            disabled={!keys || !message || isLoading}
            className="px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600 disabled:opacity-50"
          >
            {isLoading ? 'Signing...' : 'Sign Message'}
          </button>
          
          {signature && (
            <div className="mt-2 text-sm">
              <p><strong>Signature:</strong> {Falcon.bytesToHex(signature).substring(0, 64)}...</p>
              <p><strong>Signature Length:</strong> {signature.length} bytes</p>
            </div>
          )}
        </div>

        {/* Verify Button */}
        {signature && (
          <div>
            <button
              onClick={verifySignature}
              disabled={isLoading}
              className="px-4 py-2 bg-purple-500 text-white rounded hover:bg-purple-600 disabled:opacity-50"
            >
              {isLoading ? 'Verifying...' : 'Verify Signature'}
            </button>
            
            {verificationResult !== null && (
              <div className={`mt-2 p-2 rounded ${verificationResult ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'}`}>
                <strong>Verification Result:</strong> {verificationResult ? '✅ Valid' : '❌ Invalid'}
              </div>
            )}
          </div>
        )}

        {/* Info */}
        <div className="mt-6 p-4 bg-gray-100 rounded">
          <h3 className="font-semibold mb-2">About Falcon512</h3>
          <ul className="text-sm space-y-1">
            <li>• Post-quantum digital signature scheme</li>
            <li>• Based on lattice cryptography (NTRU)</li>
            <li>• Selected by NIST for standardization</li>
            <li>• Provides ~108 bits of quantum security</li>
            <li>• Fast verification, compact signatures</li>
          </ul>
        </div>
      </div>
    </div>
  );
};

export default FalconSignaturePanel;