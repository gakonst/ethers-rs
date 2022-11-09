//! Various utilities for manipulating Ethereum related dat
use ethabi::ethereum_types::H256;
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

    let mut eth_message = format!("{PREFIX}{}", message.len()).into_bytes();
    eth_message.extend_from_slice(message);

    keccak256(&eth_message).into()
}

/// Compute the Keccak-256 hash of input bytes.
// TODO: Add Solidity Keccak256 packing support
pub fn keccak256<S>(bytes: S) -> [u8; 32]
where
    S: AsRef<[u8]>,
{
    let mut output = [0u8; 32];
    let mut hasher = Keccak::v256();
    hasher.update(bytes.as_ref());
    hasher.finalize(&mut output);
    output
}

/// Calculate the function selector as per the contract ABI specification. This
/// is defined as the first 4 bytes of the Keccak256 hash of the function
/// signature.
pub fn id<S: AsRef<str>>(signature: S) -> [u8; 4] {
    let mut output = [0u8; 4];

    let mut hasher = Keccak::v256();
    hasher.update(signature.as_ref().as_bytes());
    hasher.finalize(&mut output);

    output
}

/// Serialize a type.
///
/// # Panics
///
/// If the type returns an error during serialization.
pub fn serialize<T: serde::Serialize>(t: &T) -> serde_json::Value {
    serde_json::to_value(t).expect("Types never fail to serialize.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // from https://emn178.github.io/online-tools/keccak_256.html
    fn test_keccak256() {
        assert_eq!(
            hex::encode(keccak256(b"hello")),
            "1c8aff950685c2ed4bc3174f3472287b56d9517b9c948127319a09a7a36deac8"
        );
    }

    // test vector taken from:
    // https://web3js.readthedocs.io/en/v1.2.2/web3-eth-accounts.html#hashmessage
    #[test]
    fn test_hash_message() {
        let hash = hash_message("Hello World");

        assert_eq!(
            hash,
            "a1de988600a42c4b4ab089b619297c17d53cffae5d5120d82d8a92d0bb3b78f2".parse().unwrap()
        );
    }

    #[test]
    fn simple_function_signature() {
        // test vector retrieved from
        // https://web3js.readthedocs.io/en/v1.2.4/web3-eth-abi.html#encodefunctionsignature
        assert_eq!(id("myMethod(uint256,string)"), [0x24, 0xee, 0x00, 0x97],);
    }

    #[test]
    fn revert_function_signature() {
        assert_eq!(id("Error(string)"), [0x08, 0xc3, 0x79, 0xa0]);
    }
}
