use std::env;
use crate::blockchain::Blockchain;

const CMD_ADDBLOCK: &str = "addblock";
const CMD_PRINTCHAIN: &str = "printchain";

enum Command {
    AddBlock(String),
    PrintChain,
    Invalid,
}

pub struct CLI {
    bc: Blockchain,
}

impl CLI {
    pub fn new(bc: Blockchain) -> Self {
        CLI { bc }
    }

    pub fn run(&mut self) {
        match self.parse_args() {
            Command::AddBlock(data) => self.add_block(&data),
            Command::PrintChain => self.bc.print_block(),
            Command::Invalid => self.print_usage(),
        }
    }

    fn parse_args(&self) -> Command {
        let args: Vec<String> = env::args().collect();
        if args.len() < 2 {
            return Command::Invalid;
        }

        match args[1].as_str() {
            CMD_ADDBLOCK if args.len() == 3 => Command::AddBlock(args[2].clone()),
            CMD_PRINTCHAIN => Command::PrintChain,
            _ => Command::Invalid,
        }
    }

    fn print_usage(&self) {
        println!("Usage:");
        println!("  {} <DATA> - add a block to the blockchain", CMD_ADDBLOCK);
        println!("  {} - print all the blocks of the blockchain", CMD_PRINTCHAIN);
    }

    fn add_block(&mut self, data: &str) {
        self.bc.add_block(data);
        println!("Block added successfully!");
    }
}