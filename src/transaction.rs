use crate::utils;
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
        let pub_key_hash = utils::base58_decode(address);
        self.pub_key_hash = pub_key_hash[1..20].to_vec();
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
    pub fn sign(&mut self, private_key: &[u8], prev_txs: &HashMap<Vec<u8>, Transaction>) {
        if self.is_coinbase() {
            return;
        }
    
        for vin in &self.vin {
            if prev_txs.get(&vin.txid).is_none() {
                eprintln!("ERROR: Previous transaction is not correct");
                process::exit(1);
            }
        }
    
        let mut tx_copy = self.trimmed_copy();
    
        for (_in_id, vin) in tx_copy.vin.iter_mut().enumerate() {
            let prev_tx = match prev_txs.get(&vin.txid) {
                Some(tx) => tx,
                None => {
                    eprintln!("ERROR: Previous transaction is not correct");
                    process::exit(1);
                }
            };
    
            let mut prev_tx_copy = prev_tx.trimmed_copy();
            prev_tx_copy.vout[vin.vout as usize].value = 0;
            prev_tx_copy.id = prev_tx_copy.hash();
    
            vin.signature = utils::sign(private_key, &prev_tx_copy.id);
            vin.pub_key = utils::hash_public_key(&utils::get_public_key(private_key));
        }
    
        self.vin = tx_copy.vin;
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

        let _tx_copy = self.trimmed_copy();
        let secp = Secp256k1::new();

        for (_in_id, vin) in self.vin.iter().enumerate() {
            let prev_tx = &prev_txs[&vin.txid];
            let mut prev_tx_copy = prev_tx.trimmed_copy();
            prev_tx_copy.vout[vin.vout as usize].value = 0;
            prev_tx_copy.id = prev_tx_copy.hash();

            let message = Message::from_slice(&prev_tx_copy.id)
            .map_err(|_| "ERROR: Failed to create message from slice")?;
    
        let public_key = PublicKey::from_slice(&vin.pub_key)
            .map_err(|_| "ERROR: Failed to create public key from slice")?;
    
        // Convert vin.signature from Vec<u8> to Signature
        let signature = Signature::from_der(&vin.signature)
            .map_err(|_| "ERROR: Failed to create signature from DER")?;
    
        // Now, pass the converted signature to the verify method
        let verification_result = secp.verify(&message, &signature, &public_key);
        if verification_result.is_err() {
            return Err("ERROR: Signature verification failed".into());
        }
        }

        Ok(true)
    }

    pub fn new_coinbase_tx(to: &str, data: &str) -> Self {
        if data.is_empty() {
            panic!("Data must not be empty");
        }
        let txin = TXInput {
            txid: vec![],
            vout: -1,
            signature: data.as_bytes().to_vec(),
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
        sender: &str,
        receiver: &str,
        amount: i32,
        blockchain: &Blockchain,
    ) -> Result<Self, Box<dyn Error>> {
        let mut inputs: Vec<TXInput> = Vec::new();
        let mut outputs: Vec<TXOutput> = Vec::new();

        let (acc, valid_outputs) = blockchain.find_spendable_outputs(
            utils::base58_decode(sender),
            amount,
        )?;

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
                    pub_key: utils::base58_decode(sender),
                };
                inputs.push(input);
            }
        }

        outputs.push(TXOutput::new(amount, receiver));
        if acc > amount {
            outputs.push(TXOutput::new(acc - amount, sender));
        }

        let mut tx = Transaction {
            id: vec![],
            vin: inputs,
            vout: outputs,
        };

        tx.id = tx.hash();
        blockchain.sign_transaction(&mut tx, &utils::string_hex(sender));
        Ok(tx)
    }
}
