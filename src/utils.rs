extern crate bs58;

use sha2::{Digest as SHA256Digest, Sha256};
use ripemd::Ripemd160;
use ring::rand::SystemRandom;
use ring::signature::{EcdsaKeyPair, ECDSA_P256_SHA256_FIXED, ECDSA_P256_SHA256_FIXED_SIGNING};

/// Encodes a byte slice into its hexadecimal representation.
pub fn hex_string(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

/// Computes the SHA-256 hash of the given data.
pub fn compute_sha256(data: &[u8]) -> Vec<u8> {
    Sha256::digest(data).as_slice().to_vec()
}


/// Computes the RIPEMD-160 hash of the given data.
pub fn compute_ripemd160(data: &[u8]) -> Vec<u8> {
    let mut hasher = Ripemd160::new();
    hasher.update(data);
    let result = hasher.finalize();
    result.to_vec()
}

/// Base58 encodes the given data.
pub fn base58_encode(data: &[u8]) -> String {
    bs58::encode(data).into_string()
}

/// Base58 decodes the given string.
pub fn base58_decode(s: &str) -> Vec<u8> {
    bs58::decode(s).into_vec().unwrap_or_else(|e| {
        eprint!("Failed to decode base58 string: {}", e);
        Vec::new()
    })
}

/// Generates a new key pair using the ECDSA P-256 algorithm.
pub fn generate_key_pair() -> Vec<u8> {
    let rng = SystemRandom::new();
    let pkcs8 = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &rng).unwrap();
    pkcs8.as_ref().to_vec()
}

/// Sign the given message using ECDSA P256 SHA256
pub fn ecdsa_p256_sha256_sign(pkcs8: &[u8], message: &[u8]) -> Vec<u8> {
    let rng = ring::rand::SystemRandom::new();
    let key_pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, pkcs8, &rng).unwrap();
    let rng = ring::rand::SystemRandom::new();
    key_pair.sign(&rng, message).unwrap().as_ref().to_vec()
}

/// Verify the given signature using ECDSA P256 SHA256
pub fn ecdsa_p256_sha256_sign_verify(public_key: &[u8], signature: &[u8], message: &[u8]) -> bool {
    let peer_public_key =
        ring::signature::UnparsedPublicKey::new(&ECDSA_P256_SHA256_FIXED, public_key);
    let result = peer_public_key.verify(message, signature.as_ref());
    result.is_ok()
}

pub fn hash_pub_key(pub_key: &[u8]) -> Vec<u8> {
    let pub_key_sha256 = compute_sha256(pub_key);
    compute_ripemd160(&pub_key_sha256)
}