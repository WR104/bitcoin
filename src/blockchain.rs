use crate::{
    bcdb::BlockchainDb,
    block::Block,
    transaction::{self, TXOutput, Transaction},
    utils, wallet,
};

use data_encoding::HEXLOWER;
use std::collections::HashMap;

const DB_FILE: &str = "blockchain.db";

pub struct Blockchain {
    pub tip: Vec<u8>, // hash of the last block
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
            let coinbase = transaction::new_coinbase_tx(address);
            let genesis_block = Block::new_genesis_block(vec![coinbase]);
            db.write(&genesis_block.hash, &genesis_block.serialize())
                .unwrap();
            db.write(b"1", &genesis_block.hash).unwrap();
            genesis_block.hash
        };

        Blockchain { tip, db }
    }

    pub fn mine_block(&mut self, transactions: Vec<Transaction>) {
        // Verify each transaction, logging an error for any invalid transaction.
        for tx in &transactions {
            if tx.verify(self) == false {
                panic!("ERROR: Invalid transaction");
            }
        }

        let last_hash = self.tip.clone();
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
    }

    pub fn find_spendable_outputs(
        &self,
        pub_key_hash: &[u8],
        amount: i32,
    ) -> (i32, HashMap<String, Vec<usize>>) {
        let unspent_transaction = self.find_unspent_transactions(pub_key_hash);

        let mut accumulated: i32 = 0;
        let mut unspent_outputs: HashMap<String, Vec<usize>> = HashMap::new();

        'outer: for tx in &unspent_transaction {
            let txid_hex = HEXLOWER.encode(tx.get_id().as_slice());
            for idx in 0..tx.get_vout().len() {
                let txout = tx.get_vout()[idx].clone();
                if txout.is_locked_with_key(pub_key_hash) {
                    accumulated += txout.get_value();
                    if unspent_outputs.contains_key(txid_hex.as_str()) {
                        unspent_outputs
                            .get_mut(txid_hex.as_str())
                            .unwrap()
                            .push(idx);
                    } else {
                        unspent_outputs.insert(txid_hex.clone(), vec![idx]);
                    }
                    if accumulated >= amount {
                        break 'outer;
                    }
                }
            }
        }

        (accumulated, unspent_outputs)
    }

    /// Finds all unspent transaction outputs and returns transactions with spent outputs removed.
    /// 1. Some outputs are not tied to an input, such as coinbase mining rewards.
    /// 2. The input of a transaction can refer to the output of multiple previous transactions.
    /// 3. An input must reference an output.
    pub fn find_unspent_transactions(&self, pub_key_hash: &[u8]) -> Vec<Transaction> {
        let mut unspent_txs = Vec::new();
        let mut spent_txos: HashMap<String, Vec<usize>> = HashMap::new();
        let mut blockchain_iterator = BlockchainIterator {
            prev_block_hash: self.tip.clone(),
            db: &self.db,
        };

        loop {
            let block = blockchain_iterator.next();
            if block.is_none() {
                break;
            }

            for tx in block.unwrap().get_transactions() {
                let txid_hex = HEXLOWER.encode(tx.get_id().as_slice());
                let txout = tx.get_vout();
                'outer: for idx in 0..txout.len() {
                    let txout = txout[idx].clone();

                    // filter out the spent txos
                    if spent_txos.contains_key(txid_hex.as_str()) {
                        let outs = spent_txos.get(txid_hex.as_str()).unwrap();
                        for out in outs {
                            if out.eq(&idx) {
                                continue 'outer;
                            }
                        }
                    }
                    if txout.is_locked_with_key(pub_key_hash) {
                        unspent_txs.push(tx.clone());
                    }
                }

                if tx.is_coinbase() {
                    continue;
                }

                for txin in tx.get_vin() {
                    if txin.uses_key(pub_key_hash) {
                        let txid_hex = HEXLOWER.encode(&txin.get_txid().as_slice());
                        if spent_txos.contains_key(txid_hex.as_str()) {
                            let outs = spent_txos.get_mut(txid_hex.as_str()).unwrap();
                            outs.push(txin.get_vout());
                        } else {
                            spent_txos.insert(txid_hex, vec![txin.get_vout()]);
                        }
                    }
                }
            }
        }

        unspent_txs
    }

    pub fn find_utxo(&self, pub_key_hash: Vec<u8>) -> Vec<TXOutput> {
        let mut utxo = Vec::new();
        let unspent_txs = self.find_unspent_transactions(&pub_key_hash);

        for tx in unspent_txs {
            for out in tx.get_vout() {
                if out.is_locked_with_key(&pub_key_hash) {
                    utxo.push(out);
                }
            }
        }

        utxo
    }

    pub fn find_transaction(&self, txid: &[u8]) -> Option<Transaction> {
        let mut blockchain_iterator = BlockchainIterator {
            prev_block_hash: self.tip.clone(),
            db: &self.db,
        };

        loop {
            let option = blockchain_iterator.next();
            if option.is_none() {
                break;
            }
            let block = option.unwrap();
            for transaction in &block.get_transactions() {
                if txid.eq(transaction.get_id().as_slice()) {
                    return Some(transaction.clone());
                }
            }
        }
        None
    }

    pub fn print_chain(&self) {
        let mut blockchain_iterator = BlockchainIterator {
            prev_block_hash: self.tip.clone(),
            db: &self.db,
        };

        loop {
            let option = blockchain_iterator.next();
            if option.is_none() {
                break;
            }
            let block = option.unwrap();
            println!("Pre block hash: {}", block.get_pre_block_hash());
            println!("Cur block hash: {}", block.get_hash());
            for tx in block.get_transactions() {
                for input in tx.get_vin() {
                    let txid_hex = HEXLOWER.encode(&input.get_txid());
                    let pub_key_hash = utils::hash_pub_key(&input.get_pub_key());
                    let address = wallet::calc_address(&pub_key_hash);
                    println!(
                        "Transaction input txid = {}, vout = {}, from = {}",
                        txid_hex,
                        input.get_vout(),
                        address,
                    )
                }
                let cur_txid_hex = HEXLOWER.encode(&tx.get_id());
                for output in tx.get_vout() {
                    let pub_key_hash = output.get_pub_key_hash();
                    let address = wallet::calc_address(&pub_key_hash);
                    println!(
                        "Transaction output current txid = {}, value = {}, to = {}",
                        cur_txid_hex,
                        output.get_value(),
                        address,
                    )
                }
            }
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
            let block = Block::deserialize(&encoded_block);
            self.prev_block_hash = block.prev_block_hash.clone();
            Some(block)
        } else {
            None
        }
    }
}
