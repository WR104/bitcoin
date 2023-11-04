use secp256k1::{SecretKey, Secp256k1, rand::rngs::OsRng};
use serde::{Serialize, Deserialize};
use sha2::{Digest, Sha256};

use crate::{utils, base58};

#[derive(Serialize, Deserialize)]
pub struct Wallet {
    pub private_key: Vec<u8>,
    pub public_key: Vec<u8>,
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

    fn checksum(payload: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(payload);
        let first_hash = hasher.finalize();

        hasher = Sha256::new();
        hasher.update(first_hash);
        let second_hash = hasher.finalize();

        second_hash[..4].to_vec()
    } 

    pub fn address(&self) -> String {
        let pub_key_hash = utils::hash_public_key(&self.public_key);
        let versioned_payload = [vec![0x00], pub_key_hash].concat();
        let checksum = Self::checksum(&versioned_payload);
        let full_payload = [versioned_payload, checksum].concat();
        let address = base58::encode(&full_payload);
        String::from_utf8(address).unwrap()
    }

}