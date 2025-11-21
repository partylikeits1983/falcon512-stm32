use crate::falcon;
use rand_core::RngCore;

pub type SecretKey = falcon::SecretKey<512>;
pub type PublicKey = falcon::PublicKey<512>;
pub type Signature = falcon::Signature<512>;

pub fn keygen(seed: [u8; 32]) -> (SecretKey, PublicKey) {
    falcon::keygen(seed)
}

pub fn sign_with_rng(msg: &[u8], sk: &SecretKey, rng: &mut impl RngCore) -> Signature {
    falcon::sign_with_rng(msg, sk, rng)
}

pub fn verify(msg: &[u8], sig: &Signature, pk: &PublicKey) -> bool {
    falcon::verify(msg, sig, pk)
}
