//! Various utilities for manipulating Ethereum related dat
use crate::types::H256;
use tiny_keccak::{Hasher, Keccak};

const PREFIX: &str = "\x19Ethereum Signed Message:\n";

/// Hash a message according to EIP-191.
///
/// The data is a UTF-8 encoded string and will enveloped as follows:
/// `"\x19Ethereum Signed Message:\n" + message.length + message` and hashed
/// using keccak256.
pub fn hash_message<S>(message: S) -> H256
where
    S: AsRef<[u8]>,
{
    let message = message.as_ref();

    let mut eth_message = format!("{}{}", PREFIX, message.len()).into_bytes();
    eth_message.extend_from_slice(message);

    keccak256(&eth_message).into()
}

/// Compute the Keccak-256 hash of input bytes.
// TODO: Add Solidity Keccak256 packing support
pub fn keccak256(bytes: &[u8]) -> [u8; 32] {
    let mut output = [0u8; 32];
    let mut hasher = Keccak::v256();
    hasher.update(bytes);
    hasher.finalize(&mut output);
    output
}

/// Gets the first 4 bytes
pub fn id(name: &str) -> [u8; 4] {
    let mut output = [0u8; 4];

    let mut hasher = Keccak::v256();
    hasher.update(name.as_bytes());
    hasher.finalize(&mut output);

    output
}

/// Serialize a type. Panics if the type is returns error during serialization.
pub fn serialize<T: serde::Serialize>(t: &T) -> serde_json::Value {
    serde_json::to_value(t).expect("Types never fail to serialize.")
}

#[cfg(test)]
mod tests {
    use super::*;

    // test vector taken from:
    // https://web3js.readthedocs.io/en/v1.2.2/web3-eth-accounts.html#hashmessage
    #[test]
    fn test_hash_message() {
        let hash = hash_message("Hello World");

        assert_eq!(
            hash,
            "a1de988600a42c4b4ab089b619297c17d53cffae5d5120d82d8a92d0bb3b78f2"
                .parse()
                .unwrap()
        );
    }
}
