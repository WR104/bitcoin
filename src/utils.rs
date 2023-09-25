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
