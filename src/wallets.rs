use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

use crate::wallet::Wallet;

const PATH: &str = "wallet.json";

pub struct Wallets {
    wallets: HashMap<String, Wallet>,
}

impl Wallets{
    pub fn new() -> Self {
        let mut wallets = Wallets {
            wallets: HashMap::new(),
        };
        wallets.load_file().unwrap();
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

    fn load_file(&mut self) -> Result<(), Box<dyn Error>> {
        let path = Path::new(PATH);
        if path.exists() {
            let contents = std::fs::read_to_string(path)?;
            self.wallets = serde_json::from_str(&contents)?;
        }
        Ok(())
    }

    pub fn save_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        let contents = serde_json::to_string(&self.wallets)?;
        std::fs::write(PATH, contents)?;
        Ok(())
    }
}