use crate::{block::Block, bcdb::BlockchainDb, proofofwork::ProofOfWork};

const DB_FILE: &str = "blockchain.db";

pub struct Blockchain {
    // last block hsh
    tip: Vec<u8>,
    db: BlockchainDb, 
}

impl Blockchain {
    pub fn new() -> Self {
        let mut db = BlockchainDb::new(DB_FILE).expect("Failed to initialize the database");
        
        let tip = if let Some(last_hash) = db.read(b"1").unwrap() {
            last_hash
        } else {
            println!("No existing blockchain found. Creating a new one...");
            let genesis_block = Block::new_genesis_block();
            db.write(&genesis_block.hash, &genesis_block.serialize()).unwrap();
            db.write(b"1", &genesis_block.hash).unwrap();
            genesis_block.hash
        };

        Blockchain { tip, db }
    }

    pub fn add_block(&mut self, data: &str) {
        let last_hash = self.db.read(b"1").unwrap().expect("Failed to read the last hash");
        let last_block_serialized = self.db.read(&last_hash).unwrap().expect("Failed to read the last block");
        let last_block = Block::deserialize(&last_block_serialized).unwrap();

        let pow = ProofOfWork::new(&last_block);
        if pow.validate() {
            let new_block = Block::new(data, last_hash);
            self.db.write(&new_block.hash, &new_block.serialize()).unwrap();
            self.db.write(b"1", &new_block.hash).unwrap();
            self.tip = new_block.hash;
        } else {
            panic!("PoW validation error. Stopping block addition")
        }
    }

    pub fn print_block(&self) {
        let mut blockchain_iterator = BlockchainIterator {
            prev_block_hash: self.tip.clone(),
            db: &self.db,
        };

        while let Some(block) = blockchain_iterator.next() {
            block.print_content();
            println!("-------------");
        }
    }
}

struct BlockchainIterator<'a> {
    prev_block_hash: Vec<u8>,
    db: &'a BlockchainDb,
}

impl<'a> BlockchainIterator<'a> {
    fn next(&mut self) -> Option<Block> {
        if let Some(encoded_block) = self.db.read(&self.prev_block_hash).unwrap() {
            let block = Block::deserialize(&encoded_block).unwrap();
            self.prev_block_hash = block.prev_block_hash.clone();
            Some(block)
        } else {
            None
        }
    }
}
