use std::collections::HashMap;

use crate::{
    bcdb::BlockchainDb,
    block::Block,
    transaction::{TXOutput, Transaction, self},
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
            let coinbase = transaction::new_coinbase_tx(address, GENESIS_COINBASE_DATA);
            let genesis_block = Block::new_genesis_block(vec![coinbase]);
            db.write(&genesis_block.hash, &genesis_block.serialize())
                .unwrap();
            db.write(b"1", &genesis_block.hash).unwrap();
            genesis_block.hash
        };

        Blockchain { tip, db }
    }

    pub fn mine_block(&mut self, transactions: Vec<Transaction>) {
        // Fetch the last hash, and log an error if it fails.
        let last_hash = match self.db.get_last_hash() {
            Ok(Some(hash)) => hash,
            Ok(None) => {
                println!("No last hash found");
                return;
            }
            Err(_) => {
                println!("Failed to get last hash");
                return;
            }
        };
    
        // Verify each transaction, logging an error for any invalid transaction.
        for tx in &transactions {
            if !self.verify_transaction(&tx) {
                println!("Invalid transaction: {:?}", tx);
                return;
            }
        }
    
        // Create a new block with the provided transactions and the last hash.
        let new_block = Block::new(transactions, last_hash);
    
        // Attempt to write the new block to the database, logging any errors.
        if let Err(_) = self.db.write(&new_block.hash, &new_block.serialize()) {
            println!("Failed to write block");
            return;
        }
    
        // Update the tip of the blockchain, logging any errors.
        if let Err(_) = self.db.write(b"1", &new_block.hash) {
            println!("Failed to update last hash");
            return;
        }
    
        self.tip = new_block.hash;
        // If all operations are successful, the function simply ends.
    }
    
    pub fn find_unspent_transactions(&self, pub_key_hash: &[u8]) -> Vec<Transaction> {
        let mut unspent_txs = Vec::new();
        let mut spent_txos: HashMap<String, Vec<i32>> = HashMap::new();
        let mut blockchain_iterator = BlockchainIterator {
            prev_block_hash: self.tip.clone(),
            db: &self.db,
        };

        while let Some(block) = blockchain_iterator.next() {
            for tx in &block.transaction {
                let tx_id = utils::hex_string(&tx.id);

                'outputs: for (out_idx, out) in tx.vout.iter().enumerate() {
                    if spent_txos.get(&tx_id).map_or(false, |spent| spent.contains(&(out_idx as i32))) {
                        continue 'outputs;
                    }
                    if out.is_locked_with_key(pub_key_hash) {
                        unspent_txs.push(tx.clone());
                    }
                }

                if !tx.is_coinbase() {
                    for tx_input in &tx.vin {
                        if tx_input.uses_key(pub_key_hash) {
                            let in_tx_id = utils::hex_string(&tx_input.txid);
                            spent_txos
                                .entry(in_tx_id)
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

        'work: for tx in unspent_txs {
            let txid = utils::hex_string(&tx.id);

            for (out_idx, out) in tx.vout.iter().enumerate() {
                if out.is_locked_with_key(&pub_key_hash) && accumulated < amount {
                    accumulated += out.value;
                    unspent_outputs
                        .entry(txid.clone())
                        .or_insert_with(Vec::new)
                        .push(out_idx as i32);

                    if accumulated >= amount {
                        break 'work;
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

    fn find_prev_txs(&self, tx: &Transaction) -> HashMap<String, Transaction> {
        let mut prev_txs: HashMap<String, Transaction> = HashMap::new();
        for vin in &tx.vin {
            let prev_tx = self.find_transaction(&vin.txid).unwrap();
            prev_txs.insert(utils::hex_string(&prev_tx.id), prev_tx);
        }
        prev_txs
    }

    pub fn sign_transaction(&self, tx: &mut Transaction, private_key: &[u8]) {
        let prev_txs = self.find_prev_txs(tx);
        tx.sign(private_key, &prev_txs);
    }

    /// Verifies that the transaction is valid.
    fn verify_transaction(&self, tx: &Transaction) -> bool {
        if tx.is_coinbase() {
            return true;
        }

        let mut prev_txs = HashMap::new();

        for vin in &tx.vin {
            let prev_tx = match self.find_transaction(&vin.txid) {
                Some(prev_tx) => prev_tx,
                None => return false,
            };
            prev_txs.insert(prev_tx.id.clone(), prev_tx);
        }

        // Need to work on this funciton
        // tx.verify(&prev_txs)
        true
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