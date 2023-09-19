mod block;
mod blockchain;
mod proofofwork;
mod utils;
mod bcdb;
mod cli;

use blockchain::Blockchain;
use cli::CLI;

fn main() {
    let blockchian = Blockchain::new();

    let mut cli = CLI::new(blockchian);
    cli.run();
}
