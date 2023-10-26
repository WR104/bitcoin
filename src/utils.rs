use num_bigint::BigUint;
use num_traits::ToPrimitive;
use sha2::{Digest, Sha256};

/// Encodes a byte slice into its hexadecimal representation.
///
/// # Arguments
///
/// * `bytes` - A byte slice to be encoded.
///
/// # Returns
///
/// * A `String` containing the hexadecimal representation of the byte slice.
pub fn hex_string(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

/// Decodes a hexadecimal string into its byte representation.
///
/// # Arguments
///
/// * `s` - A string slice containing the hexadecimal representation.
///
/// # Returns
///
/// * A `Vec<u8>` containing the byte representation of the hexadecimal string.
pub fn string_hex(s: &str) -> Vec<u8> {
    hex::decode(s).unwrap_or_else(|e| {
        eprint!("Failed to decode hex string: {}", e);
        Vec::new()
    })
}

/// Computes the SHA-256 hash of the given data.
///
/// # Arguments
///
/// * `data` - A byte slice to be hashed.
///
/// # Returns
///
/// * A `Vec<u8>` containing the SHA-256 hash of the data.
pub fn compute_sha256(data: &[u8]) -> Vec<u8> {
    Sha256::digest(data).as_slice().to_vec()
}

pub fn base58_encode(payload: &[u8]) -> String {
    let mut payload = payload.to_vec();
    let mut result = Vec::new();
    let alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz".as_bytes();
    let base = alphabet.len();
    let mut leading_zeroes_count = 0;
    for b in payload.iter() {
        if *b == 0 {
            leading_zeroes_count += 1;
        } else {
            break;
        }
    }
    let mut x = BigUint::from_bytes_be(&payload);
    while x > BigUint::from(0 as u8) {
        let rem = x.clone() % base;
        result.push(alphabet[rem.to_usize().unwrap()]);
        x = x / base;
    }
    for _ in 0..leading_zeroes_count {
        result.push(alphabet[0]);
    }
    result.reverse();
    String::from_utf8(result).unwrap()
}