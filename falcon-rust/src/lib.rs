//! Unofficial no-std rust implementation of the [Falcon] post-quantum
//! digital signature scheme.
//!
//! Falcon was submitted to the [NIST PQC]
//! standardization project and was [selected] for
//! standardization. The final standard is still outstanding. We do anticipate slight changes
//! between the standard and the submission, and these changes might break compatibility.
//!
//! Falcon comes in two variants. Falcon512 claims at least 108 bits of security, and
//! Falcon1024 claims at least 252 bits of security, both against quantum computers.
//!
//! This implementation was written following the [specification]
//! and using the [python implementation] as a guide, although later versions diverge from this
//! reference point.
//!
//! [Falcon]: https://falcon-sign.info/
//! [NIST PQC]: https://csrc.nist.gov/projects/post-quantum-cryptography
//! [selected]: https://csrc.nist.gov/Projects/post-quantum-cryptography/selected-algorithms-2022
//! [specification]: https://falcon-sign.info/falcon.pdf
//! [python implementation]: https://github.com/tprest/falcon.py
//!
//! # Usage (no-std with custom RNG)
//!
//! This crate is designed for embedded devices like STM32 microcontrollers.
//! You must provide your own RNG implementation via `rand_core::RngCore`.
//!
//! ```ignore
//! use falcon_rust::falcon512;
//! use rand_core::RngCore;
//!
//! // Your device's RNG implementation (e.g., STM32 hardware RNG)
//! let mut rng = your_device_rng();
//! let mut seed = [0u8; 32];
//! rng.fill_bytes(&mut seed);
//!
//! let msg = b"Hello, world!";
//! let (sk, pk) = falcon512::keygen(seed);
//! let sig = falcon512::sign_with_rng(msg, &sk, &mut rng);
//! assert!(falcon512::verify(msg, &sig, &pk));
//! ```
//!
//! For serialization / deserialization:
//! ```ignore
//! use falcon_rust::falcon512;
//!
//! let msg = b"Hello, world!";
//! let seed = [0u8; 32]; // Use proper randomness in production
//! let (sk, pk) = falcon512::keygen(seed);
//! let mut rng = your_device_rng();
//! let sig = falcon512::sign_with_rng(msg, &sk, &mut rng);
//!
//! let sk_buffer = sk.to_bytes();
//! let pk_buffer = pk.to_bytes();
//! let sig_buffer = sig.to_bytes();
//! falcon512::SecretKey::from_bytes(&sk_buffer);
//! falcon512::PublicKey::from_bytes(&pk_buffer);
//! falcon512::Signature::from_bytes(&sig_buffer);
//! ```

#![cfg_attr(not(test), no_std)]

extern crate alloc;

#[cfg(test)]
extern crate std;

pub(crate) mod cyclotomic_fourier;
pub(crate) mod encoding;
pub(crate) mod falcon;
pub mod falcon1024;
pub mod falcon512;
pub(crate) mod falcon_field;
pub(crate) mod fast_fft;
pub(crate) mod ffsampling;
pub(crate) mod inverse;
pub mod math; // pub for benching
pub mod polynomial; // pub for benching
pub(crate) mod samplerz;
pub(crate) mod u32_field;
pub mod workspace;
