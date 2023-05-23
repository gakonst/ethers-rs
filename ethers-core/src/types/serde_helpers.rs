//! Some convenient serde helpers

use crate::types::{BlockNumber, U256, U64};
use serde::{Deserialize, Deserializer};
use std::{
    convert::{TryFrom, TryInto},
    str::FromStr,
};

/// Helper type to parse both `u64` and `U256`
#[derive(Copy, Clone, Deserialize)]
#[serde(untagged)]
pub enum Numeric {
    U256(U256),
    Num(u64),
}

impl From<Numeric> for U256 {
    fn from(n: Numeric) -> U256 {
        match n {
            Numeric::U256(n) => n,
            Numeric::Num(n) => U256::from(n),
        }
    }
}

impl FromStr for Numeric {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(val) = s.parse::<u128>() {
            Ok(Numeric::U256(val.into()))
        } else if s.starts_with("0x") {
            U256::from_str(s).map(Numeric::U256).map_err(|err| err.to_string())
        } else {
            U256::from_dec_str(s).map(Numeric::U256).map_err(|err| err.to_string())
        }
    }
}

/// Helper type to parse numeric strings, `u64` and `U256`
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum StringifiedNumeric {
    String(String),
    U256(U256),
    Num(serde_json::Number),
}

impl TryFrom<StringifiedNumeric> for U256 {
    type Error = String;

    fn try_from(value: StringifiedNumeric) -> Result<Self, Self::Error> {
        match value {
            StringifiedNumeric::U256(n) => Ok(n),
            StringifiedNumeric::Num(n) => {
                Ok(U256::from_dec_str(&n.to_string()).map_err(|err| err.to_string())?)
            }
            StringifiedNumeric::String(s) => {
                if let Ok(val) = s.parse::<u128>() {
                    Ok(val.into())
                } else if s.starts_with("0x") {
                    U256::from_str(&s).map_err(|err| err.to_string())
                } else {
                    U256::from_dec_str(&s).map_err(|err| err.to_string())
                }
            }
        }
    }
}

impl TryFrom<StringifiedNumeric> for U64 {
    type Error = String;

    fn try_from(value: StringifiedNumeric) -> Result<Self, Self::Error> {
        let value = U256::try_from(value)?;
        let mut be_bytes = [0u8; 32];
        value.to_big_endian(&mut be_bytes);
        U64::try_from(&be_bytes[value.leading_zeros() as usize / 8..])
            .map_err(|err| err.to_string())
    }
}

/// Supports parsing numbers as strings
///
/// See <https://github.com/gakonst/ethers-rs/issues/1507>
pub fn deserialize_stringified_numeric<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
    D: Deserializer<'de>,
{
    let num = StringifiedNumeric::deserialize(deserializer)?;
    num.try_into().map_err(serde::de::Error::custom)
}

/// Supports parsing numbers as strings
///
/// See <https://github.com/gakonst/ethers-rs/issues/1507>
pub fn deserialize_stringified_numeric_opt<'de, D>(
    deserializer: D,
) -> Result<Option<U256>, D::Error>
where
    D: Deserializer<'de>,
{
    if let Some(num) = Option::<StringifiedNumeric>::deserialize(deserializer)? {
        num.try_into().map(Some).map_err(serde::de::Error::custom)
    } else {
        Ok(None)
    }
}

/// Supports parsing ethereum-types U64
///
/// See <https://github.com/gakonst/ethers-rs/issues/1507>
pub fn deserialize_stringified_eth_u64<'de, D>(deserializer: D) -> Result<U64, D::Error>
where
    D: Deserializer<'de>,
{
    let num = StringifiedNumeric::deserialize(deserializer)?;
    num.try_into().map_err(serde::de::Error::custom)
}

/// Supports parsing ethereum-types `Option<U64>`
///
/// See <https://github.com/gakonst/ethers-rs/issues/1507>
pub fn deserialize_stringified_eth_u64_opt<'de, D>(deserializer: D) -> Result<Option<U64>, D::Error>
where
    D: Deserializer<'de>,
{
    if let Some(num) = Option::<StringifiedNumeric>::deserialize(deserializer)? {
        let num: U64 = num.try_into().map_err(serde::de::Error::custom)?;
        Ok(Some(num))
    } else {
        Ok(None)
    }
}

/// Supports parsing u64
///
/// See <https://github.com/gakonst/ethers-rs/issues/1507>
pub fn deserialize_stringified_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let num = StringifiedNumeric::deserialize(deserializer)?;
    let num: U256 = num.try_into().map_err(serde::de::Error::custom)?;
    num.try_into().map_err(serde::de::Error::custom)
}

/// Supports parsing u64
///
/// See <https://github.com/gakonst/ethers-rs/issues/1507>
pub fn deserialize_stringified_u64_opt<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    if let Some(num) = Option::<StringifiedNumeric>::deserialize(deserializer)? {
        let num: U256 = num.try_into().map_err(serde::de::Error::custom)?;
        let num: u64 = num.try_into().map_err(serde::de::Error::custom)?;
        Ok(Some(num))
    } else {
        Ok(None)
    }
}

/// Helper type to deserialize sequence of numbers
#[derive(Deserialize)]
#[serde(untagged)]
pub enum NumericSeq {
    Seq([Numeric; 1]),
    U256(U256),
    Num(u64),
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
        NumericSeq::Seq(seq) => seq[0].into(),
        NumericSeq::U256(n) => n,
        NumericSeq::Num(n) => U256::from(n),
    };

    Ok(num)
}

/// Helper type to parse numeric strings, `u64` and `U256`
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum StringifiedBlockNumber {
    Numeric(StringifiedNumeric),
    BlockNumber(BlockNumber),
}

impl TryFrom<StringifiedBlockNumber> for BlockNumber {
    type Error = String;

    fn try_from(value: StringifiedBlockNumber) -> Result<Self, Self::Error> {
        match value {
            StringifiedBlockNumber::Numeric(num) => {
                let num = U256::try_from(num)
                    .and_then(|num| u64::try_from(num).map_err(str::to_string))?;
                Ok(BlockNumber::Number(num.into()))
            }
            StringifiedBlockNumber::BlockNumber(b) => Ok(b),
        }
    }
}

/// Supports parsing block number as strings
///
/// See <https://github.com/gakonst/ethers-rs/issues/1507>
pub fn deserialize_stringified_block_number<'de, D>(
    deserializer: D,
) -> Result<BlockNumber, D::Error>
where
    D: Deserializer<'de>,
{
    let num = StringifiedBlockNumber::deserialize(deserializer)?;
    num.try_into().map_err(serde::de::Error::custom)
}

/// Various block number representations, See [`lenient_block_number()`]
#[derive(Clone, Copy, Deserialize)]
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
/// <https://github.com/ethereum/EIPs/blob/master/EIPS/eip-1898.md>
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
    let num = <[LenientBlockNumber; 1]>::deserialize(deserializer)?[0].into();
    Ok(num)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::U256;

    #[test]
    fn test_deserialize_string_chain_id() {
        use crate::types::transaction::eip712::EIP712Domain;

        let val = serde_json::json!(
                  {
          "name": "Seaport",
          "version": "1.1",
          "chainId": "137",
          "verifyingContract": "0x00000000006c3852cbEf3e08E8dF289169EdE581"
        }
              );

        let domain: EIP712Domain = serde_json::from_value(val).unwrap();
        assert_eq!(domain.chain_id, Some(137u64.into()));
    }

    // <https://github.com/gakonst/ethers-rs/issues/2353>
    #[test]
    fn deserialize_stringified() {
        #[derive(Debug, Deserialize, Eq, PartialEq)]
        struct TestValues {
            #[serde(deserialize_with = "deserialize_stringified_numeric")]
            value_1: U256,
            #[serde(deserialize_with = "deserialize_stringified_numeric")]
            value_2: U256,
            #[serde(deserialize_with = "deserialize_stringified_numeric")]
            value_3: U256,
            #[serde(deserialize_with = "deserialize_stringified_numeric")]
            value_4: U256,
        }

        let data = r#"
        {
            "value_1": "750000000000000000",
            "value_2": "21000000000000000",
            "value_3": "0",
            "value_4": "1"
        }
    "#;

        let deserialized: TestValues = serde_json::from_str(data).unwrap();
        let expected = TestValues {
            value_1: U256::from(750_000_000_000_000_000u64),
            value_2: U256::from(21_000_000_000_000_000u64),
            value_3: U256::from(0u64),
            value_4: U256::from(1u64),
        };
        assert_eq!(deserialized, expected);
    }
}
