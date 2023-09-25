use crate::{proofofwork::ProofOfWork, transaction::Transaction, utils};
use bincode;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize)]
pub struct Block {
    pub time_stamp: i64,
    pub transaction: Vec<Transaction>,
    pub prev_block_hash: Vec<u8>,
    pub hash: Vec<u8>,
    pub nonce: u32,
}

impl Block {
    pub fn new(transaction: Vec<Transaction>, prev_block_hash: Vec<u8>) -> Block {
        let mut block = Block {
            time_stamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            transaction,
            prev_block_hash,
            hash: Vec::new(),
            nonce: 0,
        };

        let pow = ProofOfWork::new(&block);
        let (nonce, hash) = pow.run();
        block.nonce = nonce;
        block.hash = hash;
        block
    }

    pub fn new_genesis_block(coinbase: Vec<Transaction>) -> Block {
        Block::new(coinbase, vec![])
    }

    /// Computes a unique hash for all transactions in a block.
    ///
    /// Transactions in a block are uniquely identified by concatenating the hashes of each transaction
    /// and then hashing the concatenated combination.
    ///
    /// # Returns
    ///
    /// * A `Vec<u8>` containing the unique hash for all transactions in the block.
    pub fn hash_transaction(&self) -> Vec<u8> {
        let tx_hashes: Vec<u8> = self
            .transaction
            .iter()
            .flat_map(|tx| tx.id.clone())
            .collect();
        utils::compute_sha256(&tx_hashes)
    }

    pub fn print_content(&self) {
        println!("Timestamp: {}", self.time_stamp);
        //println!("Data: {}", String::from_utf8_lossy(&self.data));
        println!(
            "Previous Bloch Hash: {}",
            utils::hex_string(&self.prev_block_hash)
        );
        println!("Hash {}", utils::hex_string(&self.hash));
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).expect("Failed to serialize block")
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(data)
    }
}
