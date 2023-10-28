use crate::block::Block;
use num_bigint::BigUint;
use sha2::{Digest, Sha256};

const TARGET_BITS: u16 = 16;
const MAT_NONCE: u32 = u32::MAX;

pub struct ProofOfWork<'a> {
    block: &'a Block,
    target: BigUint,
}

impl<'a> ProofOfWork<'a> {
    pub fn new(block: &'a Block) -> Self {
        let mut target = BigUint::from(1u32);
        target = target << (256 - TARGET_BITS);
        ProofOfWork { block, target }
    }

    fn prepare_data(&self, nonce: u32) -> Vec<u8> {
        let data = vec![
            &self.block.time_stamp.to_le_bytes()[..],
            &self.block.hash_transaction(),
            &self.block.prev_block_hash[..],
            &TARGET_BITS.to_le_bytes(),
            &nonce.to_be_bytes(),
        ]
        .concat();
        data
    }

    /// Calculates the proof of work, returning the correct nonce and hash.
    ///
    /// # Returns
    ///
    /// * A tuple containing the correct nonce and the resulting hash as a `Vec<u8>`.
    pub fn run(&self) -> (u32, Vec<u8>) {
        let mut hasher = Sha256::new();

        for nonce in 0..MAT_NONCE {
            let data = self.prepare_data(nonce);
            hasher.update(&data);
            let hash = hasher.finalize_reset().to_vec();

            let hash_int = BigUint::from_bytes_be(&hash);

            if hash_int < self.target {
                return (nonce, hash);
            }
        }

        (0, Vec::new()) // Return a default value if no solution is found within MAT_NONCE
    }

    #[allow(dead_code)]
    pub fn validate(&self) -> bool {
        let data: Vec<u8> = self.prepare_data(self.block.nonce);
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hasher.finalize().to_vec();
        let hash_int = BigUint::from_bytes_be(&hash);

        hash_int < self.target
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        let bytes: &[u8] = &[0x78, 0x56, 0x12, 0x44];
        println!("len value: {}", bytes.len());
        println!("byte val: {:?}", bytes);

        let last_four = &bytes[bytes.len() - 4..];
        println!("{:?}", last_four);
    }
}
