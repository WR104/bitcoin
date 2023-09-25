use crate::utils;
use serde::{Deserialize, Serialize};
use std::process;

use crate::blockchain::Blockchain;

// mining reward
pub const SUBSIDY: i32 = 10;

#[derive(Serialize, Deserialize, Clone)]
pub struct TXInput {
    pub txid: Vec<u8>,
    pub vout: i32,
    script_sig: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TXOuput {
    pub value: i32,
    pub script_pub_key: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub id: Vec<u8>,
    pub vin: Vec<TXInput>,
    pub vout: Vec<TXOuput>,
}

impl TXInput {
    pub fn can_unlock_output_with(&self, unlocking_data: &str) -> bool {
        self.script_sig == unlocking_data
    }
}

impl TXOuput {
    pub fn can_be_unlocked_with(&self, unlocking_data: &str) -> bool {
        self.script_pub_key == unlocking_data
    }
}

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
}

pub fn new_coinbase_tx(to: &str, data: &str) -> Transaction {
    let input_data = if data.is_empty() {
        format!("Reward to {}", to)
    } else {
        data.to_string()
    };

    let mut tx = Transaction {
        id: vec![],
        vin: vec![TXInput {
            txid: vec![],
            vout: -1,
            script_sig: input_data,
        }],
        vout: vec![TXOuput {
            value: SUBSIDY,
            script_pub_key: to.to_string(),
        }],
    };

    tx.set_id();
    tx
}

pub fn new_utxo_transaction(from: &str, to: &str, amount: i32, bc: &Blockchain) -> Transaction {
    let mut txs_inputs = Vec::new();
    let mut tsx_outputs = Vec::new();

    let (acc, valid_outputs) = bc.find_spendable_outputs(from, amount);

    if acc < amount {
        eprintln!("Error: Not enough funds");
        process::exit(-1);
    }

    // Build a list of inputs
    for (txid, outs) in valid_outputs.iter() {
        for out in outs {
            let input = TXInput {
                txid: utils::string_hex(txid),
                vout: *out,
                script_sig: from.to_string(),
            };
            txs_inputs.push(input);
        }
    }

    // transfer utxo to the "to" address
    tsx_outputs.push(TXOuput {
        value: amount,
        script_pub_key: to.to_string(),
    });

    if acc > amount {
        tsx_outputs.push(TXOuput {
            value: acc - amount,
            script_pub_key: from.to_string(),
        });
    }

    let mut tx = Transaction {
        id: Vec::new(),
        vin: txs_inputs,
        vout: tsx_outputs,
    };
    tx.set_id();

    tx
}
