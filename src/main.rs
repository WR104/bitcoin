// mod block;
// mod blockchain;
// mod proofofwork;
mod utils;
// mod bcdb;
// mod cli;
// mod transaction;
mod wallet;


// use cli::CLI;

fn main() {
    // let cli = CLI;
    // cli.run();

    let wallet = wallet::Wallet::new();
    println!("Private key: {:?}", wallet.private_key);
    println!("Public key: {:?}", wallet.public_key);
    println!("Address: {}", wallet::Wallet::address());
}
