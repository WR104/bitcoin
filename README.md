# bitcoin-rs

A simple blockchain implementation allowing users to create a blockchain, manage wallets, and conduct transactions between wallets. Below is a detailed explanation of how to use each functionality.

## Usage

```
cargo run createwallet
cargo run createblockchain
cargo run getbalance <ADDRESS>
cargo run listaddresses
cargo run printchain
cargo run send <FROM> <TO> <AMOUNT>
```

🚧 The send feature is currently under development and not available for use. Need to work on sign_transaction() and sign().