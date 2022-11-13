pub use ethabi;
use ethabi::{ethereum_types::U256, Address};
// Re-export
pub use hex; // Re-export

pub struct TakeLastXBytes(pub usize);

/// Represents a data type in solidity
/// ```rust
/// use eth_encode_packed::SolidityDataType;
/// use eth_encode_packed::TakeLastXBytes;
/// use eth_encode_packed::ethabi::ethereum_types::{U256, Address};
/// // Uint24
/// SolidityDataType::NumberWithShift(U256::from(3838), TakeLastXBytes(24));
/// // String
/// SolidityDataType::String("ipfs-cid-url-very-long");
/// // Bool
/// SolidityDataType::Bool(true);
/// // Address
/// use std::convert::TryInto;
///
/// let address = hex::decode("d8b934580fcE35a11B58C6D73aDeE468a2833fa8").unwrap();
/// let address: [u8; 20] = address.try_into().unwrap();
/// SolidityDataType::Address(Address::from(address));
/// ```
pub enum SolidityDataType<'a> {
    String(&'a str),
    Address(Address),
    Bytes(&'a [u8]),
    Bool(bool),
    Number(U256),
    NumberWithShift(U256, TakeLastXBytes),
}

pub mod abi {

    use crate::abi::encode_packed::SolidityDataType;

    /// Pack a single `SolidityDataType` into bytes
    fn pack<'a>(data_type: &'a SolidityDataType) -> Vec<u8> {
        let mut res = Vec::new();
        match data_type {
            SolidityDataType::String(s) => {
                res.extend(s.as_bytes());
            }
            SolidityDataType::Address(a) => {
                res.extend(a.0);
            }
            SolidityDataType::Number(n) => {
                for b in n.0.iter().rev() {
                    let bytes = b.to_be_bytes();
                    res.extend(bytes);
                }
            }
            SolidityDataType::Bytes(b) => {
                res.extend(*b);
            }
            SolidityDataType::Bool(b) => {
                if *b {
                    res.push(1);
                } else {
                    res.push(0);
                }
            }
            SolidityDataType::NumberWithShift(n, to_take) => {
                let local_res = n.0.iter().rev().fold(vec![], |mut acc, i| {
                    let bytes = i.to_be_bytes();
                    acc.extend(bytes);
                    acc
                });

                let to_skip = local_res.len() - (to_take.0 / 8);
                let local_res = local_res.into_iter().skip(to_skip).collect::<Vec<u8>>();
                res.extend(local_res);
            }
        };
        return res
    }

    /// ```rust
    /// use eth_encode_packed::hex;
    /// use eth_encode_packed::SolidityDataType;
    /// use eth_encode_packed::TakeLastXBytes;
    /// use eth_encode_packed::abi;
    /// use eth_encode_packed::ethabi::ethereum_types::{Address, U256};
    /// use std::convert::TryInto;
    ///
    /// let address = hex::decode("d8b934580fcE35a11B58C6D73aDeE468a2833fa8").unwrap();
    /// let address: [u8; 20] = address.try_into().unwrap();
    /// let input = vec![
    ///     SolidityDataType::NumberWithShift(U256::from(3838), TakeLastXBytes(24)),
    ///     SolidityDataType::Number(U256::from(4001)),
    ///     SolidityDataType::String("this-is-a-sample-string"),
    ///     SolidityDataType::Address(Address::from(address)),
    ///     SolidityDataType::Number(U256::from(1)),
    /// ];
    /// let (_bytes, hash) = abi::encode_packed(&input);
    /// let hash = format!("0x{:}", hash);
    /// let expected = "0x000efe0000000000000000000000000000000000000000000000000000000000000fa1746869732d69732d612d73616d706c652d737472696e67d8b934580fce35a11b58c6d73adee468a2833fa80000000000000000000000000000000000000000000000000000000000000001";
    /// assert_eq!(hash, expected);
    /// ```
    pub fn encode_packed(items: &[SolidityDataType]) -> (Vec<u8>, String) {
        let res = items.iter().fold(Vec::new(), |mut acc, i| {
            let pack = pack(i);
            acc.push(pack);
            acc
        });
        let res = res.join(&[][..]);
        let hexed = hex::encode(&res);
        (res, hexed)
    }
}
