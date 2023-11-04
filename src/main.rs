mod utils;
mod base58;

mod block;
mod blockchain;
mod proofofwork;
mod bcdb;
mod cli;
mod transaction;
mod wallet;
mod wallets;


use cli::CLI;

fn main() {
    let cli = CLI;
    cli.run();
}
