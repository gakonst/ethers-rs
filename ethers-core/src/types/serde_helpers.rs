//! Some convenient serde helpers

use crate::types::{BlockNumber, U256};
use serde::{Deserialize, Deserializer};

/// Helper type to parse both `u64` and `U256`
#[derive(Deserialize)]
#[serde(untagged)]
pub enum Numeric {
    U256(U256),
    Num(u128),
    #[serde(deserialize_with = "deserialize_dec_string")]
    Decimal(U256),
}

impl From<Numeric> for U256 {
    fn from(n: Numeric) -> U256 {
        match n {
            Numeric::U256(n) => n,
            Numeric::Num(n) => U256::from(n),
            Numeric::Decimal(n) => n,
        }
    }
}

/// Helper type to deserialize sequence of numbers
#[derive(Deserialize)]
#[serde(untagged)]
pub enum NumericSeq {
    Seq([Numeric; 1]),
    U256(U256),
    Num(u128),
    #[serde(deserialize_with = "deserialize_dec_string")]
    Decimal(U256),
}

/// Deserializes a number from hex or int
fn deserialize_dec_string<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    U256::from_dec_str(&s).map_err(serde::de::Error::custom)
}

/// Deserializes a number from hex or int
pub fn deserialize_number<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
    D: Deserializer<'de>,
{
    Numeric::deserialize(deserializer).map(Into::into)
}

/// Deserializes a number from hex or int, but optionally
pub fn deserialize_number_opt<'de, D>(deserializer: D) -> Result<Option<U256>, D::Error>
where
    D: Deserializer<'de>,
{
    let num = match Option::<Numeric>::deserialize(deserializer)? {
        Some(Numeric::U256(n)) => Some(n),
        Some(Numeric::Num(n)) => Some(U256::from(n)),
        _ => None,
    };

    Ok(num)
}

/// Deserializes single integer params: `1, [1], ["0x01"]`
pub fn deserialize_number_seq<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
    D: Deserializer<'de>,
{
    let num = match NumericSeq::deserialize(deserializer)? {
        NumericSeq::Seq(seq) => seq.into_iter().next().unwrap().into(),
        NumericSeq::U256(n) => n,
        NumericSeq::Num(n) => U256::from(n),
        NumericSeq::Decimal(n) => n,
    };

    Ok(num)
}

/// Various block number representations, See [`lenient_block_number()`]
#[derive(Deserialize)]
#[serde(untagged)]
pub enum LenientBlockNumber {
    BlockNumber(BlockNumber),
    Num(u64),
}

impl From<LenientBlockNumber> for BlockNumber {
    fn from(b: LenientBlockNumber) -> Self {
        match b {
            LenientBlockNumber::BlockNumber(b) => b,
            LenientBlockNumber::Num(b) => b.into(),
        }
    }
}

/// Following the spec the block parameter is either:
///
/// > HEX String - an integer block number
/// > String "earliest" for the earliest/genesis block
/// > String "latest" - for the latest mined block
/// > String "pending" - for the pending state/transactions
///
/// and with EIP-1898:
/// > blockNumber: QUANTITY - a block number
/// > blockHash: DATA - a block hash
///
/// https://github.com/ethereum/EIPs/blob/master/EIPS/eip-1898.md
///
/// EIP-1898 does not all calls that use `BlockNumber` like `eth_getBlockByNumber` and doesn't list
/// raw integers as supported.
///
/// However, there are dev node implementations that support integers, such as ganache: <https://github.com/foundry-rs/foundry/issues/1868>
///
/// N.B.: geth does not support ints in `eth_getBlockByNumber`
pub fn lenient_block_number<'de, D>(deserializer: D) -> Result<BlockNumber, D::Error>
where
    D: Deserializer<'de>,
{
    LenientBlockNumber::deserialize(deserializer).map(Into::into)
}

/// Same as `lenient_block_number` but requires to be `[num; 1]`
pub fn lenient_block_number_seq<'de, D>(deserializer: D) -> Result<BlockNumber, D::Error>
where
    D: Deserializer<'de>,
{
    let num =
        <[LenientBlockNumber; 1]>::deserialize(deserializer)?.into_iter().next().unwrap().into();
    Ok(num)
}
