use std::env;
use crate::{blockchain::Blockchain, transaction::new_utxo_transaction};

pub struct CLI;

impl CLI {
    pub fn run(&self) {
        let args: Vec<String> = env::args().collect();
        if args.len() < 2 {
            self.print_usage();
            return;
        }

        match args[1].as_str() {
            "createblockchain" => {
                if let Some(address) = args.get(3) {
                    self.create_blockchain(address);
                } else {
                    println!("Usage: createblockchain -address ADDRESS - Create a blockchain and send genesis block reward to ADDRESS");
                }
            },
            "getbalance" => {
                if let Some(address) = args.get(3) {
                    self.get_balance(address);
                } else {
                    println!("Usage: getbalance -address ADDRESS - Get balance of ADDRESS");
                }
            },
            "send" => {
                if args.len() < 8 || args[3].is_empty() || args[5].is_empty() || args[7].parse::<i32>().is_err() {
                    println!("Usage: send -from FROM -to TO -amount AMOUNT - Send AMOUNT of coins from FROM address to TO");
                    return;
                }
                let amount = args[7].parse::<i32>().unwrap();
                self.mine_block(&args[3], &args[5], amount);
            },
            "printchain" => {
                self.print_chain();
            },
            _ => {
                self.print_usage();
            },
        }
    }

    fn print_usage(&self) {
        println!("Usage:");
        println!("  getbalance -address ADDRESS - Get balance of ADDRESS");
        println!("  createblockchain -address ADDRESS - Create a blockchain and send genesis block reward to ADDRESS");
        println!("  printchain - Print all the blocks of the blockchain");
        println!("  send -from FROM -to TO -amount AMOUNT - Send AMOUNT of coins from FROM address to TO");
    }

    pub fn mine_block(&self, from: &str, to: &str, amount: i32) {
        let mut bc = Blockchain::new(from);
        let tx = new_utxo_transaction(from, to, amount, &bc);
        bc.mine_block(vec![tx]);
        println!("Success!");
    }

    pub fn create_blockchain(&self, address: &str) {
        Blockchain::new(address);
        println!("Done");
    }

    pub fn get_balance(&self, address: &str) {
        let bc = Blockchain::new(address);
        let balance: i32 = bc.find_utxo(address).iter().map(|utxo| utxo.value).sum();
        println!("Balance of {}: {}", address, balance);
    }

    fn print_chain(&self) {
        let bc = Blockchain::new("");
        bc.print_blocks();
    }
}
