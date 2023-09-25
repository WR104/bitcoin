mod block;
mod blockchain;
mod proofofwork;
mod utils;
mod bcdb;
mod cli;
mod transaction;

use cli::CLI;

fn main() {
    let cli = CLI;
    cli.run();
}
