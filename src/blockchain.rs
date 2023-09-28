use std::collections::HashMap;

use crate::{
    bcdb::BlockchainDb,
    block::Block,
    proofofwork::ProofOfWork,
    transaction::{new_coinbase_tx, TXOutput, Transaction},
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
            let coinbase = new_coinbase_tx(address, GENESIS_COINBASE_DATA);
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
       

    pub fn find_unspent_transactions(&self, address: &str) -> Vec<Transaction> {
        let mut unspent_txs: Vec<Transaction> = Vec::new();
        let mut spent_txos: HashMap<String, Vec<i32>> = HashMap::new();
        let mut blockchain_iterator = BlockchainIterator {
            prev_block_hash: self.tip.clone(),
            db: &self.db,
        };

        while let Some(block) = blockchain_iterator.next() {
            for tx in &block.transaction {
                let txid = utils::hex_string(&tx.id);

                for (tx_output_index, tx_output) in tx.vout.iter().enumerate() {
                    if Blockchain::is_spent_output(tx_output_index, &spent_txos, &txid) {
                        continue;
                    }

                    if tx_output.can_be_unlocked_with(address) {
                        unspent_txs.push(tx.clone());
                    }

                    if !tx.is_coinbase() {
                        for tx_input in &tx.vin {
                            if tx_input.can_unlock_output_with(address) {
                                let in_txid = utils::hex_string(&tx_input.txid);
                                spent_txos
                                    .entry(in_txid)
                                    .or_insert_with(Vec::new)
                                    .push(tx_input.vout);
                            }
                        }
                    }
                }
            }
        }

        unspent_txs
    }

    pub fn find_utxo(&self, address: &str) -> Vec<TXOutput> {
        self.find_unspent_transactions(address)
            .into_iter()
            .flat_map(|tx| tx.vout.into_iter())
            .filter(|out| out.can_be_unlocked_with(address))
            .collect()
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
        address: &str,
        amount: i32,
    ) -> (i32, HashMap<String, Vec<i32>>) {
        let mut unspent_outputs = HashMap::new();
        let unspent_transactions = self.find_unspent_transactions(address);
        let mut accumulated = 0;

        for unspent_transaction in unspent_transactions {
            let txid = utils::hex_string(&unspent_transaction.id);

            for (tx_output_index, tx_output) in unspent_transaction.vout.iter().enumerate() {
                if tx_output.can_be_unlocked_with(address) && accumulated < amount {
                    accumulated += tx_output.value;

                    unspent_outputs
                        .entry(txid.clone())
                        .or_insert_with(Vec::new)
                        .push(tx_output_index as i32);

                    if accumulated >= amount {
                        return (accumulated, unspent_outputs);
                    }
                }
            }
        }

        (accumulated, unspent_outputs)
    }

    /// Checks if a transaction output has been spent.
    ///
    /// This function determines whether a specific transaction output, identified by its index and transaction ID,
    /// is present in the provided map of spent transaction outputs.
    ///
    /// # Arguments
    ///
    /// * `tx_output_index` - The index of the transaction output in question.
    /// * `spent_txo` - A reference to a `HashMap` containing transaction IDs mapped to a vector of spent output indices.
    /// * `txid` - A reference to the transaction ID string.
    ///
    /// # Returns
    ///
    /// * `true` if the transaction output is found in the map of spent outputs.
    /// * `false` otherwise.
    fn is_spent_output(
        tx_output_index: usize,
        spent_txo: &HashMap<String, Vec<i32>>,
        txid: &str,
    ) -> bool {
        // Check if the transaction ID exists in the map. If it does, check if the output index is in the associated vector.
        spent_txo.get(txid).map_or(false, |spent_outs| {
            spent_outs.contains(&(tx_output_index as i32))
        })
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
