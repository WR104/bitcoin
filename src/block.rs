use std::time::{SystemTime, UNIX_EPOCH};
use crate::{utils, proofofwork::ProofOfWork};
use serde::{Serialize, Deserialize};
use bincode;

#[derive(Serialize, Deserialize)]
pub struct Block {
    pub time_stamp: i64,
    pub data: Vec<u8>,
    pub prev_block_hash: Vec<u8>,
    pub hash: Vec<u8>,
    pub nonce: u32
}

impl Block {
    pub fn new(data: &str, prev_block_hash: Vec<u8>) -> Block {
        let mut block = Block {
            time_stamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
                                        .as_secs() as i64,
            data: data.as_bytes().to_vec(),
            prev_block_hash,
            hash: Vec::new(),
            nonce: 0
        };

        let pow = ProofOfWork::new(&block);
        let (nonce, hash) = pow.run();
        block.nonce = nonce;
        block.hash = hash;
        block
    }


    pub fn new_genesis_block() -> Block {
        Block::new("Genesis Block", vec![])
    }

    pub fn print_content(&self) {
        println!("Timestamp: {}", self.time_stamp);
        println!("Data: {}", String::from_utf8_lossy(&self.data));
        println!("Previous Bloch Hash: {}", utils::hex_string(&self.prev_block_hash));
        println!("Hash {}", utils::hex_string(&self.hash));
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).expect("Failed to serialize block")
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(data)
    }
}