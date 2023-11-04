use crate::{utils::{self}, wallets::Wallets, base58};
use core::fmt;
use secp256k1::{Message, PublicKey, Secp256k1, Signature};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, process};

use crate::blockchain::Blockchain;

// mining reward
pub const SUBSIDY: i32 = 10;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TXInput {
    pub txid: Vec<u8>,
    pub vout: i32,
    pub signature: Vec<u8>,
    pub pub_key: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TXOutput {
    pub value: i32,
    pub pub_key_hash: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub id: Vec<u8>,
    pub vin: Vec<TXInput>,
    pub vout: Vec<TXOutput>,
}

impl TXInput {
    pub fn uses_key(&self, pub_key_hash: &[u8]) -> bool {
        let locking_hash = utils::hash_public_key(&self.pub_key);
        locking_hash == pub_key_hash
    }
}

impl TXOutput {
    // Lock sings the output
    pub fn lock(&mut self, address: &str) {
        let pub_key_hash = base58::decode(address.as_bytes());
        self.pub_key_hash = pub_key_hash[1..pub_key_hash.len() - 4].to_vec();
    }

    // Check if the output is locked with the given key
    pub fn is_locked_with_key(&self, pub_key_hash: &[u8]) -> bool {
        self.pub_key_hash == pub_key_hash
    }

    pub fn new(value: i32, address: &str) -> Self {
        let mut tx_output = TXOutput {
            value,
            pub_key_hash: Vec::new(),
        };
        tx_output.lock(address);
        tx_output
    }
}

impl fmt::Debug for TXOutput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("TXOutput")
            .field("value", &self.value)
            .field("pub_key_hash", &self.pub_key_hash)
            .finish()
    }
}

impl fmt::Debug for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Transaction")
            .field("id", &self.id)
            .field("vin", &self.vin)
            .field("vout", &self.vout)
            .finish()
    }
}

#[allow(dead_code)]
impl Transaction {
    fn set_id(&mut self) {
        let encode = bincode::serialize(&self).unwrap_or_else(|e| {
            eprintln!("Serialization error: {}", e);
            Vec::new()
        });
        self.id = utils::compute_sha256(&encode);
    }

    /// Determines if the transaction is a coinbase transaction.
    ///
    /// A coinbase transaction is a special type of transaction used to reward miners.
    /// It has a single input, but no previous transaction ID and its output index is set to -1.
    ///
    /// # Returns
    ///
    /// * `true` if the transaction is a coinbase transaction.
    /// * `false` otherwise.
    pub fn is_coinbase(&self) -> bool {
        self.vin.len() == 1 && self.vin[0].txid.is_empty() && self.vin[0].vout == -1
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap_or_else(|e| {
            eprintln!("Serialization error: {}", e);
            Vec::new()
        })
    }

    pub fn hash(&self) -> Vec<u8> {
        let mut tx_copy = self.clone();
        tx_copy.id = vec![];
        utils::compute_sha256(&tx_copy.serialize())
    }

    /// Signs each input of a transaction.
    ///
    /// # Arguments
    ///
    /// * `private_key` - The private key of the sender.
    /// * `prev_txs` - The previous transactions.
    pub fn sign(&mut self, private_key: &[u8], prev_txs: &HashMap<String, Transaction>) {
        if self.is_coinbase() {
            return;
        }
    
        for vin in &self.vin {
            if prev_txs.get(&utils::hex_string(&vin.txid)).is_none() {
                eprintln!("ERROR: Previous transaction is not correct");
                process::exit(1);
            }
        }
    
        let mut tx_copy = self.trimmed_copy();
    
        for (in_id, vin) in tx_copy.vin.iter_mut().enumerate() {
            let prev_tx = prev_txs.get(&utils::hex_string(&vin.txid)).unwrap();

            vin.signature = Vec::new();
            vin.pub_key = prev_tx.vout[vin.vout as usize].pub_key_hash.clone();

            let mut data_to_sign = String::new();

            data_to_sign.push_str(&utils::hex_string(&tx_copy.id));

            let signature = utils::sign(private_key, &utils::string_hex(&data_to_sign));

            self.vin[in_id].signature = signature;

            // Reset the pub_key for the next iteration
            vin.pub_key = Vec::new();
        }
    }

    fn trimmed_copy(&self) -> Transaction {
        let mut inputs: Vec<TXInput> = Vec::new();
        let mut outputs: Vec<TXOutput> = Vec::new();
        for vin in &self.vin {
            inputs.push(TXInput {
                txid: vin.txid.clone(),
                vout: vin.vout,
                signature: Vec::new(),
                pub_key: Vec::new(),
            });
        }
        for vout in &self.vout {
            outputs.push(TXOutput {
                value: vout.value,
                pub_key_hash: vout.pub_key_hash.clone(),
            });
        }
        Transaction {
            id: self.id.clone(),
            vin: inputs,
            vout: outputs,
        }
    }

    /// Verifies the signatures of each input of a transaction.
    ///
    /// # Arguments
    ///
    /// * `prev_txs` - The previous transactions.
    ///
    /// # Returns
    ///
    /// * `true` if the signatures are valid.
    /// * `false` otherwise.
    pub fn verify(&self, prev_txs: &HashMap<Vec<u8>, Transaction>) -> Result<bool, Box<dyn Error>> {
        if self.is_coinbase() {
            return Ok(true);
        }

        for vin in &self.vin {
            if prev_txs.get(&vin.txid).is_none() {
                return Err("ERROR: Previous transaction is not correct".into());
            }
        }

        let mut tx_copy = self.trimmed_copy();
        let secp = Secp256k1::new();
        let mut success = Ok(());

        for (in_id, vin) in self.vin.iter().enumerate() {
            let prev_tx = &prev_txs[&vin.txid];
            tx_copy.vin[in_id].signature = Vec::new();
            tx_copy.vin[in_id].pub_key = prev_tx.vout[vin.vout as usize].pub_key_hash.clone();
            tx_copy.id = tx_copy.hash();
            tx_copy.vin[in_id].pub_key = Vec::new();

            let sig = Signature::from_compact(&vin.signature)?;
            let pub_key = PublicKey::from_slice(&vin.pub_key)?;
            let message = Message::from_slice(&tx_copy.id)?;

            if !secp.verify(&message, &sig, &pub_key).is_ok() {
                success = Err("ERROR: Invalid signature".into());
                break;
            }
        }

        success.map(|_| true)
    }

}

pub fn new_coinbase_tx(to: &str, data: &str) -> Transaction {
    if data.is_empty() {
        panic!("ERROR: Data must not be empty");
    }

    let txin = TXInput {
        txid: vec![],
        vout: -1,
        signature: Vec::new(),
        pub_key: vec![],
    };
    let txout = TXOutput::new(SUBSIDY, to);
    let mut tx = Transaction {
        id: vec![],
        vin: vec![txin],
        vout: vec![txout],
    };
    tx.set_id();
    tx
}

pub fn new_utxo_transaction(
    from: &str,
    to: &str,
    amount: i32,
    blockchain: &Blockchain,
) -> Result<Transaction, Box<dyn Error>> {
    let mut inputs: Vec<TXInput> = Vec::new();
    let mut outputs: Vec<TXOutput> = Vec::new();

    let wallets = Wallets::new();
    let wallet = wallets.get_wallet(from).ok_or("ERROR: Sender address not found")?;
    let pub_key_hash = utils::hash_public_key(&wallet.public_key);

    let (acc, valid_outputs) = blockchain.find_spendable_outputs(pub_key_hash.clone(), amount)?;

    if acc < amount {
        return Err("ERROR: Not enough funds".into());
    }

    for (txid, outs) in valid_outputs {
        let tx_id = utils::string_hex(&txid);
        for out in outs {
            let input = TXInput {
                txid: tx_id.clone(),
                vout: out,
                signature: Vec::new(),
                pub_key: pub_key_hash.clone(),
            };
            inputs.push(input);
        }
    }

    outputs.push(TXOutput::new(amount, to));
    if acc > amount {
        outputs.push(TXOutput::new(acc - amount, from));
    }

    let mut tx = Transaction {
        id: vec![],
        vin: inputs,
        vout: outputs,
    };
    tx.set_id();
    let blockchain = blockchain.clone();
    blockchain.sign_transaction(&mut tx, &wallet.private_key.clone());
    Ok(tx)
}