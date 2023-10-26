use std::collections::HashMap;
use leveldb::database::Database;

use crate::wallet::Wallet;

pub struct Wallets {
    wallets: HashMap<String, Wallet>,
}

impl Wallets{
    pub fn new() -> Self {
        let wallets = HashMap::new();
        Wallets { wallets }
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

}