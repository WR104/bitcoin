extern crate bs58;

use sha2::{Digest, Sha256};
use ripemd::Ripemd160;

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
#[allow(dead_code)]
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

/// Encodes a byte slice into its base58 representation.
/// 
/// # Arguments
/// 
/// * `payload` - A byte slice to be encoded.
/// 
/// # Returns
/// 
/// * A `String` containing the base58 representation of the byte slice.
// pub fn base58_encode(payload: &[u8]) -> String {
//     let payload = payload.to_vec();
//     let mut result = Vec::new();
//     let alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz".as_bytes();
//     let base = alphabet.len();
//     let mut leading_zeroes_count = 0;
//     for b in payload.iter() {
//         if *b == 0 {
//             leading_zeroes_count += 1;
//         } else {
//             break;
//         }
//     }
//     let mut x = BigUint::from_bytes_be(&payload);
//     while x > BigUint::from(0 as u8) {
//         let rem = x.clone() % base;
//         result.push(alphabet[rem.to_usize().unwrap()]);
//         x = x / base;
//     }
//     for _ in 0..leading_zeroes_count {
//         result.push(alphabet[0]);
//     }
//     result.reverse();
//     String::from_utf8(result).unwrap()
// }

// pub fn base58_decode(input: &str) -> Vec<u8> {
//     let alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
//     let base = BigUint::from(alphabet.len());

//     let mut result = BigUint::zero();
//     let mut leading_zeroes = 0;
//     let mut leading_zeroes_handled = false;

//     for &byte in input.as_bytes() {
//         if byte == b'1' && !leading_zeroes_handled {
//             leading_zeroes += 1;
//         } else {
//             leading_zeroes_handled = true;
//             let char_index = alphabet.find(byte as char)
//                 .unwrap_or_else(|| panic!("Invalid character in Base58 string: {}", byte as char));
//             result = result * &base + BigUint::from(char_index);
//         }
//     }

//     let mut result_bytes = result.to_bytes_be();
//     let mut bytes = vec![0; leading_zeroes];
//     bytes.append(&mut result_bytes);

//     bytes
// }

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