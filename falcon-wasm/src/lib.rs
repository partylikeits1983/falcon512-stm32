use wasm_bindgen::prelude::*;
use falcon_rust::falcon512;

// Import the `console.log` function from the `console` module
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Define a macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
pub struct FalconKeyPair {
    secret_key: falcon512::SecretKey,
    public_key: falcon512::PublicKey,
}

#[wasm_bindgen]
impl FalconKeyPair {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: &[u8]) -> Result<FalconKeyPair, JsValue> {
        if seed.len() != 32 {
            return Err(JsValue::from_str("Seed must be exactly 32 bytes"));
        }
        
        let mut seed_array = [0u8; 32];
        seed_array.copy_from_slice(seed);
        
        let (secret_key, public_key) = falcon512::keygen(seed_array);
        
        Ok(FalconKeyPair {
            secret_key,
            public_key,
        })
    }

    #[wasm_bindgen(getter)]
    pub fn public_key(&self) -> Vec<u8> {
        self.public_key.to_bytes()
    }

    #[wasm_bindgen(getter)]
    pub fn secret_key(&self) -> Vec<u8> {
        self.secret_key.to_bytes()
    }
}

#[wasm_bindgen]
pub struct FalconSignature {
    signature: falcon512::Signature,
}

#[wasm_bindgen]
impl FalconSignature {
    #[wasm_bindgen(getter)]
    pub fn bytes(&self) -> Vec<u8> {
        self.signature.to_bytes()
    }
}

// Simple RNG implementation using Web Crypto API
struct WebRng;

impl rand_core::RngCore for WebRng {
    fn next_u32(&mut self) -> u32 {
        let mut bytes = [0u8; 4];
        self.fill_bytes(&mut bytes);
        u32::from_le_bytes(bytes)
    }

    fn next_u64(&mut self) -> u64 {
        let mut bytes = [0u8; 8];
        self.fill_bytes(&mut bytes);
        u64::from_le_bytes(bytes)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        getrandom::getrandom(dest).expect("Failed to get random bytes");
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

#[wasm_bindgen]
pub fn sign_message(message: &[u8], secret_key_bytes: &[u8]) -> Result<FalconSignature, JsValue> {
    let secret_key = falcon512::SecretKey::from_bytes(secret_key_bytes)
        .map_err(|_| JsValue::from_str("Invalid secret key"))?;
    
    let mut rng = WebRng;
    let signature = falcon512::sign_with_rng(message, &secret_key, &mut rng);
    
    Ok(FalconSignature { signature })
}

#[wasm_bindgen]
pub fn verify_signature(message: &[u8], signature_bytes: &[u8], public_key_bytes: &[u8]) -> Result<bool, JsValue> {
    let signature = falcon512::Signature::from_bytes(signature_bytes)
        .map_err(|_| JsValue::from_str("Invalid signature"))?;
    
    let public_key = falcon512::PublicKey::from_bytes(public_key_bytes)
        .map_err(|_| JsValue::from_str("Invalid public key"))?;
    
    Ok(falcon512::verify(message, &signature, &public_key))
}

#[wasm_bindgen]
pub fn generate_keypair_from_random() -> Result<FalconKeyPair, JsValue> {
    let mut seed = [0u8; 32];
    getrandom::getrandom(&mut seed).map_err(|_| JsValue::from_str("Failed to generate random seed"))?;
    
    let (secret_key, public_key) = falcon512::keygen(seed);
    
    Ok(FalconKeyPair {
        secret_key,
        public_key,
    })
}

// Initialize the WASM module
#[wasm_bindgen(start)]
pub fn main() {
    console_log!("Falcon512 WASM module initialized");
}