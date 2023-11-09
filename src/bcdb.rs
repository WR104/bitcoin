use std::env;
use leveldb::kv::KV;
use leveldb::options::{Options, WriteOptions, ReadOptions};
use leveldb::database::Database;
use byteorder::{ByteOrder, LittleEndian};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub struct BlockchainDb {
    database: Database<i32>,
}

impl BlockchainDb {
    pub fn new(local_path: &str) -> Result<Self> {
        let mut dir_path = env::current_dir()?;
        dir_path.push(local_path);

        let mut opts = Options::new();
        opts.create_if_missing = true;

        let database = Database::open(&dir_path, opts)?;

        Ok(BlockchainDb { database })
    }

    pub fn write(&mut self, key: &[u8], val: &[u8]) -> Result<()> {
        let write_opts = WriteOptions::new();
        self.database.put(write_opts, from_u8(key), val).map_err(Into::into)
    }

    pub fn read(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let read_options = ReadOptions::new();
        self.database.get(read_options, from_u8(key)).map_err(Into::into)
    }
}

/// Converts the last 4 bytes (or fewer) of a byte slice into an `i32` using little-endian order.
fn from_u8(key: &[u8]) -> i32 {
    let mut buffer = [0u8; 4];
    let key_end = key.len().min(4);
    let buffer_start = 4 - key_end;

    buffer[buffer_start..].copy_from_slice(&key[key.len() - key_end..]);

    LittleEndian::read_i32(&buffer)
}