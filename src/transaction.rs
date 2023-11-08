use crate::{utils, wallets::Wallets, wallet};
use serde::{Deserialize, Serialize};
use data_encoding::HEXLOWER;

use crate::blockchain::Blockchain;

// mining reward
pub const SUBSIDY: i32 = 10;

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct TXInput {
    pub txid: Vec<u8>,
    pub vout: usize,
    pub signature: Vec<u8>,
    pub pub_key: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TXOutput {
    pub value: i32,     // amount of coins
    pub pub_key_hash: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub id: Vec<u8>, 
    pub vin: Vec<TXInput>,
    pub vout: Vec<TXOutput>,
}

impl TXInput {
    pub fn get_txid(&self) -> Vec<u8> {
        self.txid.clone()
    }

    pub fn get_vout(&self) -> usize {
        self.vout
    }

    pub fn get_pub_key(&self) -> Vec<u8> {
        self.pub_key.clone()
    }

    pub fn uses_key(&self, pub_key_hash: &[u8]) -> bool {
        let locking_hash = utils::hash_pub_key(&self.pub_key.as_slice());
        locking_hash.eq(pub_key_hash)
    }
}

impl TXOutput {
    pub fn new(value: i32, address: &str) -> TXOutput {
        let mut output = TXOutput {
            value,
            pub_key_hash: Vec::new(),
        };
        output.lock(address);
        output
    }

    pub fn get_value(&self) -> i32 {
        self.value
    }

    pub fn get_pub_key_hash(&self) -> Vec<u8> {
        self.pub_key_hash.clone()
    }

    pub fn lock(&mut self, address: &str) {
        let payload = utils::base58_decode(address);
        let pub_key_hash = payload[1..payload.len() - wallet::CHECKSUM_LENGTH].to_vec();
        self.pub_key_hash = pub_key_hash;
    }

    pub fn is_locked_with_key(&self, pub_key_hash: &[u8]) -> bool {
        self.pub_key_hash.eq(pub_key_hash)
    }
    
}

#[allow(dead_code)]
impl Transaction {
    fn hash(&mut self) -> Vec<u8>{
        let tx_copy = Transaction {
            id: vec![],
            vin: self.vin.clone(),
            vout: self.vout.clone(),
        };
        let data = bincode::serialize(&tx_copy).unwrap();
        utils::compute_sha256(data.as_slice())
    }

    /// Determines if the transaction is a coinbase transaction.
    pub fn is_coinbase(&self) -> bool {
        self.vin.len() == 1 && self.vin[0].txid.len() == 0 && self.vin[0].vout == 0
    }

    pub fn get_id(&self) -> Vec<u8> {
        self.id.clone()
    }

    pub fn get_vin(&self) -> Vec<TXInput> {
        self.vin.clone()
    }

    pub fn get_vout(&self) -> Vec<TXOutput> {
        self.vout.clone()
    }

    /// Creates a trimmed copy of the transaction to be used in signing.
    fn trimmed_copy(&self) -> Transaction {
        let mut inputs: Vec<TXInput> = Vec::new();
        let mut outputs: Vec<TXOutput> = Vec::new();
        for input in &self.vin {
            inputs.push(TXInput {
                txid: input.get_txid(),
                vout: input.get_vout(),
                signature: Vec::new(),
                pub_key: Vec::new(),
            });
        }
        for output in &self.vout {
            outputs.push(output.clone());
        }
        Transaction {
            id: self.id.clone(),
            vin: inputs,
            vout: outputs,
        }
    }

    /// Signs each input of a transaction.
    fn sign(&mut self, blockchain: &Blockchain, private_key: Vec<u8>) {
        let mut tx_copy = self.trimmed_copy();

        for (id, vin) in self.vin.iter_mut().enumerate() {
            // find the previous transaction
            let prev_tx_option = blockchain.find_transaction(&vin.get_txid().as_slice());
            if prev_tx_option.is_none() {
                panic!("ERROR: Previous transaction is not correct");
            }

            let prev_tx = prev_tx_option.unwrap();
            tx_copy.vin[id].signature = Vec::new();
            tx_copy.vin[id].pub_key = prev_tx.vout[vin.vout].pub_key_hash.clone();
            tx_copy.id = tx_copy.hash();
            tx_copy.vin[id].pub_key = Vec::new();

            // Sign the transaction using the private key
            let tx_bytes = bincode::serialize(&tx_copy).expect("ERROR: Failed to serialize transaction");
            let signature = utils::ecdsa_p256_sha256_sign(&private_key.as_slice(), tx_bytes.as_slice());
            vin.signature = signature
        }
    }

    /// Verifies the signatures of each input of a transaction.
    pub fn verify(&self, blockchain: &Blockchain) -> bool {
        if self.is_coinbase() {
            return true;
        }

        let mut tx_copy = self.trimmed_copy();
        
        for (idx, vin) in  self.vin.iter().enumerate() {
            let prev_tx_option = blockchain.find_transaction(&vin.get_txid().as_slice());
            if prev_tx_option.is_none() {
                panic!("ERROR: Previous transaction is not correct");
            }
            let prev_tx = prev_tx_option.unwrap();
            tx_copy.vin[idx].signature = Vec::new();
            tx_copy.vin[idx].pub_key = prev_tx.vout[vin.vout].pub_key_hash.clone();
            tx_copy.id = tx_copy.hash();
            tx_copy.vin[idx].pub_key = Vec::new();

            // Verify the transaction using the public key
            let tx_bytes = bincode::serialize(&tx_copy).expect("ERROR: Failed to serialize transaction");
            let verify = utils::ecdsa_p256_sha256_sign_verify(
                &vin.pub_key.as_slice(), 
                &vin.signature.as_slice(),
                 tx_bytes.as_slice()
            );
            if !verify {
                return false;
            }
        }

        true
    }

}

/// Creates a new coinbase transaction. It has no inputs, produce one output.
pub fn new_coinbase_tx(to: &str) -> Transaction {
    let txin = TXInput::default();
    let txout = TXOutput::new(SUBSIDY, to);
    let mut tx = Transaction {
        id: vec![],
        vin: vec![txin],
        vout: vec![txout],
    };
    tx.id = tx.hash();
    tx
}

pub fn new_utxo_transaction(
    from: &str,
    to: &str,
    amount: i32,
    blockchain: &Blockchain,
) -> Transaction {
    // 1. find wallet
    let binding = Wallets::new();
    let wallet = binding
        .get_wallet(from)
        .expect("ERROR: Wallet not found");
    let public_key_hash = utils::hash_pub_key(wallet.get_public_key().as_slice());

    //2. find unspent outputs
    let (accumlated, valid_outputs) =
        blockchain.find_spendable_outputs(public_key_hash.as_slice(), amount);
    if accumlated < amount {
        panic!("ERROR: Not enough funds");
    }

    let mut inputs = Vec::new();
    for (txid_hex, outs) in valid_outputs {
        let txid = HEXLOWER.decode(txid_hex.as_bytes()).unwrap();
        for out in outs {
            let input = TXInput {
                txid: txid.clone(), // last transaction ID
                vout: out,
                signature: Vec::new(),
                pub_key: wallet.get_public_key(),
            };
            inputs.push(input);
        }
    }

    let mut outputs = vec![TXOutput::new(amount, to)];
    if accumlated > amount {
        outputs.push(TXOutput::new(accumlated - amount, from));
    }

    let mut tx = Transaction {
        id: Vec::new(),
        vin: inputs,
        vout: outputs,
    };

    tx.id = tx.hash();

    tx.sign(blockchain, wallet.get_private_key());

    tx
}