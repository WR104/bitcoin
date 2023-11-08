use crate::wallet::Wallet;

use std::collections::HashMap;
use std::env::current_dir;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Read, Write};

pub const WALLET_FILE: &str = "wallets.dat";

pub struct Wallets {
    wallets: HashMap<String, Wallet>,
}

impl Wallets{
    pub fn new() -> Self {
        let mut wallets = Wallets {
            wallets: HashMap::new(),
        };
        wallets.load_from_file();
        wallets
    }

    pub fn create_wallet(&mut self) -> String {
        let wallet = Wallet::new();
        let address = wallet.address();
        self.wallets.insert(address.clone(), wallet);
        address
    }

    pub fn get_addresses(&self) -> Vec<String> {
        let mut addresses = Vec::new();
        for (address, _) in &self.wallets {
            addresses.push(address.clone());
        }
        addresses
    }

    pub fn get_wallet(&self, address: &str) -> Option<&Wallet> {
        self.wallets.get(address)
    }

    pub fn load_from_file(&mut self) {
        let path = current_dir().unwrap().join(WALLET_FILE);
        if !path.exists() {
            return;
        }
        let mut file = File::open(path).unwrap();
        let metadata = file.metadata().expect("Unable to read metadata");
        let mut buffer = vec![0; metadata.len() as usize];
        let _ = file.read(&mut buffer).expect("buffer overflow");
        let wallets = bincode::deserialize(&buffer[..]).expect("Unable to deserialize file data");
        self.wallets = wallets;
    }

    pub fn save_to_file(&self) {
        let path = current_dir().unwrap().join(WALLET_FILE);
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&path)
            .expect("Unable to open wallet.data");
        let mut writer = BufWriter::new(&file);
        let wallets_bytes = bincode::serialize(&self.wallets).expect("Unable to serialize wallets");
        writer.write(wallets_bytes.as_slice()).expect("Unable to write wallets to file");
        let _ = writer.flush();
    }
}