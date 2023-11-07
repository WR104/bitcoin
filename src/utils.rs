extern crate bs58;

use sha2::{Digest as SHA256Digest, Sha256};
use ripemd::Ripemd160;
use crypto::digest::Digest;
use std::iter::repeat;
use ring::rand::SystemRandom;
use ring::signature::{EcdsaKeyPair, ECDSA_P256_SHA256_FIXED, ECDSA_P256_SHA256_FIXED_SIGNING};

/// Encodes a byte slice into its hexadecimal representation.
pub fn hex_string(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

/// Decodes a hexadecimal string into its byte representation.
#[allow(dead_code)]
pub fn string_hex(s: &str) -> Vec<u8> {
    hex::decode(s).unwrap_or_else(|e| {
        eprint!("Failed to decode hex string: {}", e);
        Vec::new()
    })
}

/// Computes the SHA-256 hash of the given data.
pub fn compute_sha256(data: &[u8]) -> Vec<u8> {
    Sha256::digest(data).as_slice().to_vec()
}


/// Computes the RIPEMD-160 hash of the given data.
pub fn ripemd160_digest(data: &[u8]) -> Vec<u8> {
    let mut ripemd160 = crypto::ripemd160::Ripemd160::new();
    ripemd160.input(data);
    let mut buf: Vec<u8> = repeat(0).take(ripemd160.output_bytes()).collect();
    ripemd160.result(&mut buf);
    return buf;
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
    let key_pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, pkcs8).unwrap();
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

/// Computes the RIPEMD-160 hash of the given data.
/// 
/// # Arguments
/// 
/// * `data` - A byte slice to be hashed.
/// 
/// # Returns
/// 
/// * A `Vec<u8>` containing the RIPEMD-160 hash of the data.
pub fn hash_public_key(public_key: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(public_key);
    let public_sha256 = hasher.finalize();

    let mut ripemd160 = Ripemd160::new();
    ripemd160.update(public_sha256);
    ripemd160.finalize().to_vec()
}

/// gets the public key from the given private key.
/// 
/// # Arguments
/// 
/// * `private_key` - A byte slice containing the private key.
/// 
/// # Returns
/// 
/// * A `Vec<u8>` containing the public key.
#[allow(dead_code)]
pub fn get_public_key(private_key: &[u8]) -> Vec<u8> {
    let secp = secp256k1::Secp256k1::new();
    let secret_key = secp256k1::SecretKey::from_slice(private_key).unwrap();
    let public_key = secp256k1::PublicKey::from_secret_key(&secp, &secret_key);
    public_key.serialize().to_vec()
}

/// signs the given data with the given private key.
/// 
/// # Arguments
/// 
/// * `private_key` - A byte slice containing the private key.
/// * `data` - A byte slice containing the data to be signed.
/// 
/// # Returns
/// 
/// * A `Vec<u8>` containing the signature.
pub fn sign(private_key: &[u8], data: &[u8]) -> Vec<u8> {
    let secp = secp256k1::Secp256k1::new();
    let message = secp256k1::Message::from_slice(data).unwrap();
    let secret_key = secp256k1::SecretKey::from_slice(private_key).unwrap();
    let sig = secp.sign(&message, &secret_key);
    sig.serialize_der().to_vec()
}


pub fn validate_address(address: &str) -> bool {
    // Base58 decode the address
    let decoded = match bs58::decode(address).into_vec() {
        Ok(v) => v,
        Err(_) => return false,
    };

    // Ensure the length is correct to avoid panicking while slicing
    if decoded.len() <= 4 {
        return false;
    }

    // Split into payload and checksum
    let (payload, checksum) = decoded.split_at(decoded.len() - 4);

    // Double SHA-256 hash the payload
    let hash1 = Sha256::digest(payload);
    let hash2 = Sha256::digest(&hash1);

    // Compare the checksums and return the result
    checksum == &hash2[0..4]
}