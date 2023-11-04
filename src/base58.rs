extern crate num_bigint;
extern crate num_traits;

use num_bigint::{BigInt, ToBigInt};
use num_integer::Integer;
use num_traits::{Zero, ToPrimitive};

const B58_ALPHABET: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

pub fn encode(input: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut x = BigInt::from_bytes_be(num_bigint::Sign::Plus, input);
    let base = B58_ALPHABET.len().to_bigint().unwrap();
    let zero = Zero::zero();

    while x > zero {
        let (div, modu) = x.div_rem(&base);
        x = div;
        result.push(B58_ALPHABET[modu.to_usize().unwrap()]);
    }

    result.reverse();
    for &_byte in input.iter().take_while(|&&b| b == 0) {
        result.insert(0, B58_ALPHABET[0]);
    }

    result
}

pub fn decode(input: &[u8]) -> Vec<u8> {
    let mut result: BigInt = Zero::zero();
    let base = B58_ALPHABET.len().to_bigint().unwrap();
    let mut zero_bytes = 0;

    for &byte in input.iter() {
        if byte == B58_ALPHABET[0] {
            zero_bytes += 1;
        } else {
            break;
        }
    }

    let payload = &input[zero_bytes..];
    for &b in payload {
        let char_index = B58_ALPHABET.iter().position(|&c| c == b)
            .expect("Invalid character in Base58 string");
        result = result * &base + char_index.to_bigint().unwrap();
    }

    let mut decoded = result.to_bytes_be().1;
    let mut leading_zeros = vec![0u8; zero_bytes];
    leading_zeros.append(&mut decoded);

    leading_zeros
}
