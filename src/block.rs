use crate::{proofofwork::ProofOfWork, transaction::Transaction, utils};
use bincode;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize)]
pub struct Block {
    pub time_stamp: i64,
    pub transactions: Vec<Transaction>,
    pub prev_block_hash: Vec<u8>,
    pub hash: Vec<u8>,
    pub nonce: u32,
}

impl Block {
    pub fn new(transactions: Vec<Transaction>, prev_block_hash: Vec<u8>) -> Block {
        let mut block = Block {
            time_stamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            transactions,
            prev_block_hash,
            hash: Vec::new(),
            nonce: 0,
        };

        let pow = ProofOfWork::new(&block);
        let (nonce, hash) = pow.run();
        block.nonce = nonce;
        block.hash = hash;
        println!("hash: {}", utils::hex_string(&block.hash));
        block
    }

    /// deserializes a block from a byte array
    pub fn deserialize(data: &[u8]) -> Block {
        bincode::deserialize(data).expect("Failed to deserialize block")
    }

    /// serializes a block into a byte array
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).expect("Failed to serialize block")
    }

    /// generates a new genesis block
    pub fn new_genesis_block(coinbase: Vec<Transaction>) -> Block {
        Block::new(coinbase, vec![])
    }

    /// computes the hash of the block
    pub fn hash_transaction(&self) -> Vec<u8> {
        let mut tx_hashes = Vec::new();

        for tx in &self.transactions.clone() {
            let mut tx_hash = tx.id.clone();
            tx_hashes.append(&mut tx_hash);
        }

        utils::compute_sha256(&tx_hashes)
    }

    pub fn get_transactions(&self) -> Vec<Transaction> {
        self.transactions.clone()
    }

    pub fn get_pre_block_hash(&self) -> String {
        self.prev_block_hash.clone().into_iter().map(|x| format!("{:02x}", x)).collect()
    }

    pub fn get_hash(&self) -> String {
        self.hash.clone().into_iter().map(|x| format!("{:02x}", x)).collect()
    }
}
