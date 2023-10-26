use std::env;
use clap::{App, Arg, SubCommand};

use crate::{blockchain::Blockchain, utils, wallets::Wallets, transaction::Transaction};

pub struct CLI;

impl CLI {
    fn print_usage() {
        println!("Usage:");
        println!("  createblockchain -address ADDRESS - Create a blockchain and send genesis block reward to ADDRESS");
        println!("  createwallet - Generates a new key-pair and saves it into the wallet file");
        println!("  getbalance -address ADDRESS - Get balance of ADDRESS");
        println!("  listaddresses - Lists all addresses from the wallet file");
        println!("  printchain - Print all the blocks of the blockchain");
        println!("  send -from FROM -to TO -amount AMOUNT - Send AMOUNT of coins from FROM address to TO");
    }

    pub fn run(&self) {
        let args: Vec<String> = env::args().collect();
        if args.len() < 2 {
            Self::print_usage();
            return;
        }

        let matches = App::new("Blockchain CLI")
            .version("1.0")
            .author("Zhenyu Jia <jzhenyu3@gmail.com>")
            .about("Blockchain Command Line Interface")
            .subcommand(SubCommand::with_name("getbalance")
                .about("Get balance of ADDRESS")
                .arg(Arg::with_name("ADDRESS")
                     .required(true)
                     .index(1)))
            .subcommand(SubCommand::with_name("createblockchain")
                .about("Create a blockchain and send genesis block reward to ADDRESS")
                .arg(Arg::with_name("ADDRESS")
                     .required(true)
                     .index(1)))
            .subcommand(SubCommand::with_name("createwallet")
                .about("Generates a new key-pair and saves it into the wallet file"))
            .subcommand(SubCommand::with_name("listaddresses")
                .about("Lists all addresses from the wallet file"))
            .subcommand(SubCommand::with_name("printchain")
                .about("Print all the blocks of the blockchain"))
            .subcommand(SubCommand::with_name("send")
                .about("Send AMOUNT of coins from FROM address to TO")
                .arg(Arg::with_name("FROM")
                     .required(true)
                     .index(1))
                .arg(Arg::with_name("TO")
                     .required(true)
                     .index(2))
                .arg(Arg::with_name("AMOUNT")
                     .required(true)
                     .index(3)))
            .get_matches();

        // Match the subcommands and execute the corresponding code
        match matches.subcommand() {
            ("getbalance", Some(sub_m)) => {
                let address = sub_m.value_of("ADDRESS").unwrap();
                self.get_balance(address);
            }
            ("createblockchain", Some(sub_m)) => {
                let address = sub_m.value_of("ADDRESS").unwrap();
                self.create_blockchain(address);
            }
            ("createwallet", Some(_)) => {
                self.create_wallet();
            }
            ("listaddresses", Some(_)) => {
                self.list_addresses();
            }
            ("printchain", Some(_)) => {
                self.print_chain();
            }
            ("send", Some(sub_m)) => {
                let from = sub_m.value_of("FROM").unwrap();
                let to = sub_m.value_of("TO").unwrap();
                let amount = sub_m.value_of("AMOUNT").unwrap().parse::<i32>().unwrap();
                self.send(from, to, amount);
            }
            _ => {
                eprintln!("Invalid command. Use --help for usage information.");
            }
        }
    }

    pub fn get_balance(&self, address: &str) {
        if !utils::validate_address(address) {
            eprintln!("Invalid address");
            return;
        }

        let blockchain = Blockchain::new(address);
        let mut balance = 0;
        let pub_key_hash = utils::base58_decode(address);
        let pub_key_hash = &pub_key_hash[1..20];
        let utxos = blockchain.find_utxo(pub_key_hash.clone().to_vec());

        for out in utxos {
            balance += out.value;
        }

        println!("Balance of '{}' is {}", address, balance);
    }

    pub fn create_blockchain(&self, address: &str) {
        if !utils::validate_address(address) {
            eprintln!("Invalid address");
            return;
        }

        let blockchain = Blockchain::new(address);
        println!("Done!");
    }

    pub fn create_wallet(&self) {
        let mut wallets = Wallets::new();
        let address = wallets.create_wallet();
        println!("Your new address: {}", address);
    }

    pub fn list_addresses(&self) {
        let wallets = Wallets::new();
        let addresses = wallets.get_addresses();
        for address in addresses {
            println!("{}", address);
        }
    }

    pub fn print_chain(&self) {
        let blockchain = Blockchain::new("");
        blockchain.print_blocks();
    }

    pub fn send(&self, from: &str, to: &str, amount: i32) {
        if !utils::validate_address(from) {
            eprintln!("Invalid address");
            return;
        }
        if !utils::validate_address(to) {
            eprintln!("Invalid address");
            return;
        }

        let mut blockchain = Blockchain::new(from);
        let tx = Transaction::new_utxo_transaction(from, to, amount, &blockchain).unwrap();
        blockchain.mine_block(vec![tx]);
        println!("Success!");
    }
}
