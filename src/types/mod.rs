//! Various Ethereum Related Datatypes

// Re-export common ethereum datatypes with more specific names
pub use ethereum_types::{Address, H256, U256, U64};

mod transaction;
// TODO: Figure out some more intuitive way instead of having 3 similarly named structs
// with the same fields
pub use transaction::{Transaction, TransactionRequest, UnsignedTransaction};

mod keys;
pub use keys::{PrivateKey, PublicKey};

pub mod signature;
pub use signature::Signature;

mod bytes;
pub use bytes::Bytes;

mod block;
pub use block::BlockNumber;

use rustc_hex::{FromHex, ToHex};
use serde::{
    de::{Error, Unexpected},
    Deserialize, Deserializer, Serialize, Serializer,
};

/// Wrapper type 0round Vec<u8> to deserialize/serialize "0x" prefixed ethereum hex strings
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TxHash(
    #[serde(
        serialize_with = "serialize_h256",
        deserialize_with = "deserialize_h256"
    )]
    pub H256,
);

impl From<H256> for TxHash {
    fn from(src: H256) -> TxHash {
        TxHash(src)
    }
}

pub fn serialize_h256<S, T>(x: T, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: AsRef<[u8]>,
{
    s.serialize_str(&format!("0x{}", x.as_ref().to_hex::<String>()))
}

pub fn deserialize_h256<'de, D>(d: D) -> Result<H256, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(d)?;
    if value.len() >= 2 && &value[0..2] == "0x" {
        let slice: Vec<u8> = FromHex::from_hex(&value[2..])
            .map_err(|e| Error::custom(format!("Invalid hex: {}", e)))?;
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&slice[..32]);
        Ok(bytes.into())
    } else {
        Err(Error::invalid_value(Unexpected::Str(&value), &"0x prefix"))
    }
}
