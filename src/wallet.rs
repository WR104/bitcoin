extern crate ripemd;
extern crate sha2;

use std::collections::HashMap;
use secp256k1::{SecretKey, Secp256k1, rand::rngs::OsRng};
use ripemd::{Ripemd160, Digest};

use crate::utils;


pub struct Wallet {
    pub private_key: Vec<u8>,
    pub public_key: Vec<u8>,
}

pub struct Wallets {
    pub wallets: HashMap<String, Wallet>,
}

impl Wallet {
    pub fn new() -> Self {
        let (private_key, public_key) = Self::generate_key_pair();
        Wallet {
            private_key,
            public_key,
        }
    }

    fn generate_key_pair() -> (Vec<u8>, Vec<u8>) {
        let secp = Secp256k1::new();
        let mut rng = OsRng::new().unwrap();
        let private_key = SecretKey::new(&mut rng);
        let public_key = secp256k1::PublicKey::from_secret_key(&secp, &private_key);
        let private_key = private_key[..].to_vec();
        let public_key = public_key.serialize().to_vec();
        (private_key, public_key)   
    }

    fn hash_public_key(public_key: &[u8]) -> Vec<u8> {
        let public_sha256 = utils::compute_sha256(public_key);

        // Compute RIPEMD-160 hash of the SHA-256 hash
        let mut hasher = Ripemd160::new();
        hasher.update(public_sha256);
        hasher.finalize().to_vec()
    }

    fn checksum(payload: &[u8]) -> Vec<u8> {
        let first_sha256 = utils::compute_sha256(payload);
        let second_sha256 = utils::compute_sha256(&first_sha256);
        second_sha256[..4].to_vec()
    } 

    pub fn address() -> String {
        let wallet = Self::new();
        let public_key_hash = Self::hash_public_key(&wallet.public_key);
        let versioned_payload = [0x00].to_vec();
        let mut payload = versioned_payload.clone();
        payload.extend(public_key_hash);
        let checksum = Self::checksum(&payload);
        payload.extend(checksum);
        utils::base58_encode(&payload)
    }

}