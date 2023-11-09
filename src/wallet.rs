use crate::utils;

use ring::signature::{EcdsaKeyPair, KeyPair, ECDSA_P256_SHA256_FIXED_SIGNING};
use serde::{Serialize, Deserialize};

const VERSION: u8 = 0x00;
pub const CHECKSUM_LENGTH: usize = 4;

#[derive(Serialize, Deserialize)]
pub struct Wallet {
    pub private_key: Vec<u8>,
    pub public_key: Vec<u8>,
}

impl Wallet {
    pub fn new() -> Self {
        let pkcs8: Vec<u8> = utils::generate_key_pair();
        let rng = ring::rand::SystemRandom::new();
        let key_pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, pkcs8.as_ref(), &rng).unwrap();
        let public_key = key_pair.public_key().as_ref().to_vec();
        Wallet {
            private_key: pkcs8,
            public_key,
        }
    }

    pub fn address(&self) -> String {
        let pub_key_hash = utils::hash_pub_key(&self.public_key);
        let mut playload: Vec<u8> = Vec::new();
        playload.push(VERSION);
        playload.extend(&pub_key_hash);
        let checksum = checksum(&playload);
        playload.extend(&checksum);
        utils::base58_encode(&playload)
    }

    pub fn get_public_key(&self) -> Vec<u8> {
        self.public_key.clone()
    }

    pub fn get_private_key(&self) -> Vec<u8> {
        self.private_key.clone()
    }
}

/// Computes the checksum of the given payload.
fn checksum(payload: &[u8]) -> Vec<u8> {
    let first_sha256 = utils::compute_sha256(payload);
    let second_sha256 = utils::compute_sha256(&first_sha256);
    second_sha256[0..CHECKSUM_LENGTH].to_vec()
}

/// Validates that the given address is valid.
pub fn validate_address(address: &str) -> bool {
    let payload = utils::base58_decode(address);
    let actual_checksum = payload[payload.len() - CHECKSUM_LENGTH..].to_vec();
    let version = payload[0];
    let pub_key_hash = payload[1..payload.len() - CHECKSUM_LENGTH].to_vec();

    let mut target_vec = vec![];
    target_vec.push(version);
    target_vec.extend(pub_key_hash);
    let target_checksum = checksum(&target_vec);
    actual_checksum.eq(&target_checksum)
}

/// Calculates the address of the given public key.
pub fn calc_address(pub_hash_key: &[u8]) -> String {
    let mut playload: Vec<u8> = Vec::new();
    playload.push(VERSION);
    playload.extend(pub_hash_key);
    let checksum = checksum(&playload);
    playload.extend(&checksum);
    utils::base58_encode(&playload)
}

