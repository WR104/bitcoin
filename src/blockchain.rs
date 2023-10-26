use std::collections::HashMap;

use crate::{
    bcdb::BlockchainDb,
    block::Block,
    proofofwork::ProofOfWork,
    transaction::{Transaction, TXOutput},
    utils,
};

const DB_FILE: &str = "blockchain.db";
const GENESIS_COINBASE_DATA: &str =
    "The Times 03/Jan/2009 Chancellor on brink of second bailout for banks";

pub struct Blockchain {
    // last block hsh
    pub tip: Vec<u8>,
    pub db: BlockchainDb,
}

impl Blockchain {
    pub fn new(address: &str) -> Self {
        let mut db = BlockchainDb::new(DB_FILE).expect("Failed to initialize the database");

        let tip = if let Some(last_hash) = db.read(b"1").unwrap() {
            last_hash
        } else if address == "" {
            panic!("Please create blockchain first");
        } else {
            println!("No existing blockchain found. Creating a new one...");
            let coinbase = Transaction::new_coinbase_tx(address, GENESIS_COINBASE_DATA);
            let genesis_block = Block::new_genesis_block(vec![coinbase]);
            db.write(&genesis_block.hash, &genesis_block.serialize())
                .unwrap();
            db.write(b"1", &genesis_block.hash).unwrap();
            genesis_block.hash
        };

        Blockchain { tip, db }
    }

    pub fn mine_block(&mut self, transactions: Vec<Transaction>) {
        let last_hash = match self.db.read(b"1").unwrap() {
            Some(hash) => hash,
            None => panic!("No last hash found in the database"),
        };
    
        let last_block_serialized = self.db.read(&last_hash).unwrap().expect("Failed to read last block");
        let last_block = Block::deserialize(&last_block_serialized).expect("Failed to deserialize last block");
    
        let pow = ProofOfWork::new(&last_block);
    
        if pow.validate() {
            let new_block = Block::new(transactions, last_hash);
    
            self.db.write(&new_block.hash, &new_block.serialize())
                .expect("Failed to write new block to the database");
    
            self.db.write(b"1", &new_block.hash)
                .expect("Failed to update the tip in the database");
    
            self.tip = new_block.hash;
        } else {
            panic!("Failed to validate proof of work");
        }
    }
       
    pub fn find_unspent_transactions(&self, pub_key_hash: &Vec<u8>) -> Vec<Transaction> {
        let mut unspent_txs = Vec::new();
        let mut spent_txos: HashMap<String, Vec<i32>> = HashMap::new();
        let mut blockchain_iterator = BlockchainIterator {
            prev_block_hash: self.tip.clone(),
            db: &self.db,
        };        

        loop {
            let block = match blockchain_iterator.next() {
                Some(block) => block,
                None => break,
            };
    
            for tx in &block.transaction {
                let tx_id = hex::encode(&tx.id);
    
                'outputs: for (out_idx, out) in tx.vout.iter().enumerate() {
                    if let Some(spent_outputs) = spent_txos.get(&tx_id) {
                        if spent_outputs.contains(&(out_idx as i32)) {
                            continue 'outputs;
                        }
                    }
    
                    if out.is_locked_with_key(&pub_key_hash) {
                        unspent_txs.push(tx.clone());
                    }
                }
    
                if !tx.is_coinbase() {
                    for tx_input in &tx.vin {
                        if tx_input.uses_key(&pub_key_hash) {
                            let in_tx_id = hex::encode(&tx_input.txid);
                            spent_txos.entry(in_tx_id)
                                .or_insert_with(Vec::new)
                                .push(tx_input.vout as i32);
                        }
                    }
                }
            }
    
            if block.prev_block_hash.is_empty() {
                break;
            }
        }
    
        unspent_txs
    }


    pub fn find_utxo(&self, pub_key_hash: Vec<u8>) -> Vec<TXOutput> {
        let mut utxo = Vec::new();
        let unspent_txs = self.find_unspent_transactions(&pub_key_hash);
    
        for tx in unspent_txs {
            for out in tx.vout {
                if out.is_locked_with_key(&pub_key_hash) {
                    utxo.push(out);
                }
            }
        }
    
        utxo
    }

    /// Finds all unspent outputs for the given address and ensures they store enough value.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to find unspent outputs for.
    /// * `amount` - The minimum amount of value to find.
    ///
    /// # Returns
    ///
    /// * A tuple containing the accumulated value and a map of transaction IDs to their unspent output indices.
    pub fn find_spendable_outputs(
        &self,
        pub_key_hash: Vec<u8>,
        amount: i32,
    ) -> Result<(i32, HashMap<String, Vec<i32>>), Box<dyn std::error::Error>> {
        let mut unspent_outputs = HashMap::new();
        let unspent_txs = self.find_unspent_transactions(&pub_key_hash);
        let mut accumulated = 0;
    
        'Work:
        for tx in unspent_txs {
            let txid = utils::hex_string(&tx.id);
    
            for (out_idx, out) in tx.vout.iter().enumerate() {
                if out.is_locked_with_key(&pub_key_hash) && accumulated < amount {
                    accumulated += out.value;
                    unspent_outputs.entry(txid.clone()).or_insert(vec![]).push(out_idx as i32);
    
                    if accumulated >= amount {
                        break 'Work;
                    }
                }
            }
        }
    
        Ok((accumulated, unspent_outputs))
    }

    fn find_transaction(&self, id: &[u8]) -> Option<Transaction> {
        let mut blockchain_iterator = BlockchainIterator {
            prev_block_hash: self.tip.clone(),
            db: &self.db,
        };

        while let Some(block) = blockchain_iterator.next() {
            for tx in &block.transaction {
                if tx.id == id.to_vec() {
                    return Some(tx.clone());
                }
            }
        }

        None
    }

    fn find_prev_txs(&self, tx: &Transaction) -> HashMap<Vec<u8>, Transaction> {
        let mut prev_txs = HashMap::new();
        for vin in &tx.vin {
            let prev_tx = self.find_transaction(&vin.txid).unwrap();
            prev_txs.insert(prev_tx.id.clone(), prev_tx);
        }
        prev_txs
    }

    pub fn sign_transaction(&self, tx: &mut Transaction, private_key: &[u8]) {
        let prev_txs = self.find_prev_txs(tx);
        tx.sign(private_key, &prev_txs);
    }

    

    pub fn print_blocks(&self) {
        let mut blockchain_iterator = BlockchainIterator {
            prev_block_hash: self.tip.clone(),
            db: &self.db,
        };

        while let Some(block) = blockchain_iterator.next() {
            block.print_content();
            println!();
        }
    }
}

pub struct BlockchainIterator<'a> {
    prev_block_hash: Vec<u8>,
    db: &'a BlockchainDb,
}

impl<'a> BlockchainIterator<'a> {
    pub fn next(&mut self) -> Option<Block> {
        if let Some(encoded_block) = self.db.read(&self.prev_block_hash).unwrap() {
            let block = Block::deserialize(&encoded_block).unwrap();
            self.prev_block_hash = block.prev_block_hash.clone();
            Some(block)
        } else {
            None
        }
    }
}
