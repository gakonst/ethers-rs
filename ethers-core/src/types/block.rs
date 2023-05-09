// Modified from <https://github.com/tomusdrw/rust-web3/blob/master/src/types/block.rs>

#[cfg(not(feature = "celo"))]
use crate::types::Withdrawal;
use crate::types::{Address, Bloom, Bytes, Transaction, TxHash, H256, U256, U64};
use chrono::{DateTime, TimeZone, Utc};
use serde::{
    de::{MapAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::{fmt, fmt::Formatter, str::FromStr};
use thiserror::Error;

/// The block type returned from RPC calls.
///
/// This is generic over a `TX` type which will be either the hash or the full transaction,
/// i.e. `Block<TxHash>` or `Block<Transaction>`.
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Block<TX> {
    /// Hash of the block
    pub hash: Option<H256>,
    /// Hash of the parent
    #[serde(default, rename = "parentHash")]
    pub parent_hash: H256,
    /// Hash of the uncles
    #[cfg(not(feature = "celo"))]
    #[serde(default, rename = "sha3Uncles")]
    pub uncles_hash: H256,
    /// Miner/author's address. None if pending.
    #[serde(default, rename = "miner")]
    pub author: Option<Address>,
    /// State root hash
    #[serde(default, rename = "stateRoot")]
    pub state_root: H256,
    /// Transactions root hash
    #[serde(default, rename = "transactionsRoot")]
    pub transactions_root: H256,
    /// Transactions receipts root hash
    #[serde(default, rename = "receiptsRoot")]
    pub receipts_root: H256,
    /// Block number. None if pending.
    pub number: Option<U64>,
    /// Gas Used
    #[serde(default, rename = "gasUsed")]
    pub gas_used: U256,
    /// Gas Limit
    #[cfg(not(feature = "celo"))]
    #[serde(default, rename = "gasLimit")]
    pub gas_limit: U256,
    /// Extra data
    #[serde(default, rename = "extraData")]
    pub extra_data: Bytes,
    /// Logs bloom
    #[serde(rename = "logsBloom")]
    pub logs_bloom: Option<Bloom>,
    /// Timestamp
    #[serde(default)]
    pub timestamp: U256,
    /// Difficulty
    #[cfg(not(feature = "celo"))]
    #[serde(default)]
    pub difficulty: U256,
    /// Total difficulty
    #[serde(rename = "totalDifficulty")]
    pub total_difficulty: Option<U256>,
    /// Seal fields
    #[serde(default, rename = "sealFields", deserialize_with = "deserialize_null_default")]
    pub seal_fields: Vec<Bytes>,
    /// Uncles' hashes
    #[cfg(not(feature = "celo"))]
    #[serde(default)]
    pub uncles: Vec<H256>,
    /// Transactions
    #[serde(bound = "TX: Serialize + serde::de::DeserializeOwned", default)]
    pub transactions: Vec<TX>,
    /// Size in bytes
    pub size: Option<U256>,
    /// Mix Hash
    #[serde(rename = "mixHash")]
    #[cfg(not(feature = "celo"))]
    pub mix_hash: Option<H256>,
    /// Nonce
    #[cfg(not(feature = "celo"))]
    pub nonce: Option<crate::types::H64>,
    /// Base fee per unit of gas (if past London)
    #[serde(rename = "baseFeePerGas")]
    pub base_fee_per_gas: Option<U256>,
    /// Withdrawals root hash (if past Shanghai)
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "withdrawalsRoot")]
    #[cfg(not(feature = "celo"))]
    pub withdrawals_root: Option<H256>,
    /// Withdrawals (if past Shanghai)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg(not(feature = "celo"))]
    pub withdrawals: Option<Vec<Withdrawal>>,

    #[cfg(feature = "celo")]
    #[cfg_attr(docsrs, doc(cfg(feature = "celo")))]
    /// The block's randomness
    pub randomness: Randomness,

    /// BLS signatures with a SNARK-friendly hash function
    #[cfg(feature = "celo")]
    #[cfg_attr(docsrs, doc(cfg(feature = "celo")))]
    #[serde(rename = "epochSnarkData", default)]
    pub epoch_snark_data: Option<EpochSnarkData>,

    /// Captures unknown fields such as additional fields used by L2s
    #[cfg(not(feature = "celo"))]
    #[serde(flatten)]
    pub other: crate::types::OtherFields,
}

fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

/// Error returned by [`Block::time`].
#[derive(Clone, Copy, Debug, Error)]
pub enum TimeError {
    /// Timestamp is zero.
    #[error("timestamp is zero")]
    TimestampZero,

    /// Timestamp is too large for [`DateTime<Utc>`].
    #[error("timestamp is too large")]
    TimestampOverflow,
}

// ref <https://eips.ethereum.org/EIPS/eip-1559>
#[cfg(not(feature = "celo"))]
pub const ELASTICITY_MULTIPLIER: U256 = U256([2u64, 0, 0, 0]);
// max base fee delta is 12.5%
#[cfg(not(feature = "celo"))]
pub const BASE_FEE_MAX_CHANGE_DENOMINATOR: U256 = U256([8u64, 0, 0, 0]);

impl<TX> Block<TX> {
    /// The target gas usage as per EIP-1559
    #[cfg(not(feature = "celo"))]
    pub fn gas_target(&self) -> U256 {
        self.gas_limit / ELASTICITY_MULTIPLIER
    }

    /// The next block's base fee, it is a function of parent block's base fee and gas usage.
    /// Reference: <https://eips.ethereum.org/EIPS/eip-1559>
    #[cfg(not(feature = "celo"))]
    pub fn next_block_base_fee(&self) -> Option<U256> {
        use core::cmp::Ordering;

        let target_usage = self.gas_target();
        let base_fee_per_gas = self.base_fee_per_gas?;

        match self.gas_used.cmp(&target_usage) {
            Ordering::Greater => {
                let gas_used_delta = self.gas_used - self.gas_target();
                let base_fee_per_gas_delta = U256::max(
                    base_fee_per_gas * gas_used_delta /
                        target_usage /
                        BASE_FEE_MAX_CHANGE_DENOMINATOR,
                    U256::from(1u32),
                );
                let expected_base_fee_per_gas = base_fee_per_gas + base_fee_per_gas_delta;
                Some(expected_base_fee_per_gas)
            }
            Ordering::Less => {
                let gas_used_delta = self.gas_target() - self.gas_used;
                let base_fee_per_gas_delta = base_fee_per_gas * gas_used_delta /
                    target_usage /
                    BASE_FEE_MAX_CHANGE_DENOMINATOR;
                let expected_base_fee_per_gas = base_fee_per_gas - base_fee_per_gas_delta;
                Some(expected_base_fee_per_gas)
            }
            Ordering::Equal => self.base_fee_per_gas,
        }
    }

    /// Parse [`Self::timestamp`] into a [`DateTime<Utc>`].
    ///
    /// # Errors
    ///
    /// * [`TimeError::TimestampZero`] if the timestamp is zero, or
    /// * [`TimeError::TimestampOverflow`] if the timestamp is too large to be represented as a
    ///   [`DateTime<Utc>`].
    pub fn time(&self) -> Result<DateTime<Utc>, TimeError> {
        if self.timestamp.is_zero() {
            return Err(TimeError::TimestampZero)
        }
        if self.timestamp.bits() > 63 {
            return Err(TimeError::TimestampOverflow)
        }
        // Casting to i64 is safe because the timestamp is guaranteed to be less than 2^63.
        // TODO: It would be nice if there was `TryInto<i64> for U256`.
        let secs = self.timestamp.as_u64() as i64;
        Ok(Utc.timestamp_opt(secs, 0).unwrap())
    }
}

impl Block<TxHash> {
    /// Converts this block that only holds transaction hashes into a full block with `Transaction`
    pub fn into_full_block(self, transactions: Vec<Transaction>) -> Block<Transaction> {
        #[cfg(not(feature = "celo"))]
        {
            let Block {
                hash,
                parent_hash,
                uncles_hash,
                author,
                state_root,
                transactions_root,
                receipts_root,
                number,
                gas_used,
                gas_limit,
                extra_data,
                logs_bloom,
                timestamp,
                difficulty,
                total_difficulty,
                seal_fields,
                uncles,
                size,
                mix_hash,
                nonce,
                base_fee_per_gas,
                withdrawals_root,
                withdrawals,
                other,
                ..
            } = self;
            Block {
                hash,
                parent_hash,
                uncles_hash,
                author,
                state_root,
                transactions_root,
                receipts_root,
                number,
                gas_used,
                gas_limit,
                extra_data,
                logs_bloom,
                timestamp,
                difficulty,
                total_difficulty,
                seal_fields,
                uncles,
                size,
                mix_hash,
                nonce,
                base_fee_per_gas,
                withdrawals_root,
                withdrawals,
                transactions,
                other,
            }
        }

        #[cfg(feature = "celo")]
        {
            let Block {
                hash,
                parent_hash,
                author,
                state_root,
                transactions_root,
                receipts_root,
                number,
                gas_used,
                extra_data,
                logs_bloom,
                timestamp,
                total_difficulty,
                seal_fields,
                size,
                base_fee_per_gas,
                randomness,
                epoch_snark_data,
                ..
            } = self;

            Block {
                hash,
                parent_hash,
                author,
                state_root,
                transactions_root,
                receipts_root,
                number,
                gas_used,
                extra_data,
                logs_bloom,
                timestamp,
                total_difficulty,
                seal_fields,
                size,
                base_fee_per_gas,
                randomness,
                epoch_snark_data,
                transactions,
            }
        }
    }
}

impl From<Block<Transaction>> for Block<TxHash> {
    fn from(full: Block<Transaction>) -> Self {
        #[cfg(not(feature = "celo"))]
        {
            let Block {
                hash,
                parent_hash,
                uncles_hash,
                author,
                state_root,
                transactions_root,
                receipts_root,
                number,
                gas_used,
                gas_limit,
                extra_data,
                logs_bloom,
                timestamp,
                difficulty,
                total_difficulty,
                seal_fields,
                uncles,
                transactions,
                size,
                mix_hash,
                nonce,
                base_fee_per_gas,
                withdrawals_root,
                withdrawals,
                other,
            } = full;
            Block {
                hash,
                parent_hash,
                uncles_hash,
                author,
                state_root,
                transactions_root,
                receipts_root,
                number,
                gas_used,
                gas_limit,
                extra_data,
                logs_bloom,
                timestamp,
                difficulty,
                total_difficulty,
                seal_fields,
                uncles,
                size,
                mix_hash,
                nonce,
                base_fee_per_gas,
                withdrawals_root,
                withdrawals,
                transactions: transactions.iter().map(|tx| tx.hash).collect(),
                other,
            }
        }

        #[cfg(feature = "celo")]
        {
            let Block {
                hash,
                parent_hash,
                author,
                state_root,
                transactions_root,
                receipts_root,
                number,
                gas_used,
                extra_data,
                logs_bloom,
                timestamp,
                total_difficulty,
                seal_fields,
                transactions,
                size,
                base_fee_per_gas,
                randomness,
                epoch_snark_data,
            } = full;

            Block {
                hash,
                parent_hash,
                author,
                state_root,
                transactions_root,
                receipts_root,
                number,
                gas_used,
                extra_data,
                logs_bloom,
                timestamp,
                total_difficulty,
                seal_fields,
                size,
                base_fee_per_gas,
                randomness,
                epoch_snark_data,
                transactions: transactions.iter().map(|tx| tx.hash).collect(),
            }
        }
    }
}

/// Commit-reveal data for generating randomness in the
/// [Celo protocol](https://docs.celo.org/celo-codebase/protocol/identity/randomness)
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg(feature = "celo")]
pub struct Randomness {
    /// The committed randomness for that block
    pub committed: Bytes,
    /// The revealed randomness for that block
    pub revealed: Bytes,
}

/// SNARK-friendly epoch block signature and bitmap
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg(feature = "celo")]
pub struct EpochSnarkData {
    /// The bitmap showing which validators signed on the epoch block
    pub bitmap: Bytes,
    /// Signature using a SNARK-friendly hash
    pub signature: Bytes,
}

/// A [block hash](H256) or [block number](BlockNumber).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BlockId {
    // TODO: May want to expand this to include the requireCanonical field
    // <https://github.com/ethereum/EIPs/blob/master/EIPS/eip-1898.md>
    /// A block hash
    Hash(H256),
    /// A block number
    Number(BlockNumber),
}

impl From<u64> for BlockId {
    fn from(num: u64) -> Self {
        BlockNumber::Number(num.into()).into()
    }
}

impl From<U64> for BlockId {
    fn from(num: U64) -> Self {
        BlockNumber::Number(num).into()
    }
}

impl From<BlockNumber> for BlockId {
    fn from(num: BlockNumber) -> Self {
        BlockId::Number(num)
    }
}

impl From<H256> for BlockId {
    fn from(hash: H256) -> Self {
        BlockId::Hash(hash)
    }
}

impl Serialize for BlockId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            BlockId::Hash(ref x) => {
                let mut s = serializer.serialize_struct("BlockIdEip1898", 1)?;
                s.serialize_field("blockHash", &format!("{x:?}"))?;
                s.end()
            }
            BlockId::Number(ref num) => num.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for BlockId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct BlockIdVisitor;

        impl<'de> Visitor<'de> for BlockIdVisitor {
            type Value = BlockId;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("Block identifier following EIP-1898")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(BlockId::Number(v.parse().map_err(serde::de::Error::custom)?))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut number = None;
                let mut hash = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "blockNumber" => {
                            if number.is_some() || hash.is_some() {
                                return Err(serde::de::Error::duplicate_field("blockNumber"))
                            }
                            number = Some(BlockId::Number(map.next_value::<BlockNumber>()?))
                        }
                        "blockHash" => {
                            if number.is_some() || hash.is_some() {
                                return Err(serde::de::Error::duplicate_field("blockHash"))
                            }
                            hash = Some(BlockId::Hash(map.next_value::<H256>()?))
                        }
                        key => {
                            return Err(serde::de::Error::unknown_field(
                                key,
                                &["blockNumber", "blockHash"],
                            ))
                        }
                    }
                }

                number.or(hash).ok_or_else(|| {
                    serde::de::Error::custom("Expected `blockNumber` or `blockHash`")
                })
            }
        }

        deserializer.deserialize_any(BlockIdVisitor)
    }
}

impl FromStr for BlockId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("0x") && s.len() == 66 {
            let hash = s.parse::<H256>().map_err(|e| e.to_string());
            hash.map(Self::Hash)
        } else {
            s.parse().map(Self::Number)
        }
    }
}

/// A block number or tag.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum BlockNumber {
    /// Latest block
    #[default]
    Latest,
    /// Finalized block accepted as canonical
    Finalized,
    /// Safe head block
    Safe,
    /// Earliest block (genesis)
    Earliest,
    /// Pending block (not yet part of the blockchain)
    Pending,
    /// Block by number from canon chain
    Number(U64),
}

impl BlockNumber {
    /// Returns the numeric block number if explicitly set
    pub fn as_number(&self) -> Option<U64> {
        match *self {
            BlockNumber::Number(num) => Some(num),
            _ => None,
        }
    }

    /// Returns `true` if a numeric block number is set
    pub fn is_number(&self) -> bool {
        matches!(self, BlockNumber::Number(_))
    }

    /// Returns `true` if it's "latest"
    pub fn is_latest(&self) -> bool {
        matches!(self, BlockNumber::Latest)
    }

    /// Returns `true` if it's "finalized"
    pub fn is_finalized(&self) -> bool {
        matches!(self, BlockNumber::Finalized)
    }

    /// Returns `true` if it's "safe"
    pub fn is_safe(&self) -> bool {
        matches!(self, BlockNumber::Safe)
    }

    /// Returns `true` if it's "pending"
    pub fn is_pending(&self) -> bool {
        matches!(self, BlockNumber::Pending)
    }

    /// Returns `true` if it's "earliest"
    pub fn is_earliest(&self) -> bool {
        matches!(self, BlockNumber::Earliest)
    }
}

impl<T: Into<U64>> From<T> for BlockNumber {
    fn from(num: T) -> Self {
        BlockNumber::Number(num.into())
    }
}

impl Serialize for BlockNumber {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            BlockNumber::Number(ref x) => serializer.serialize_str(&format!("0x{x:x}")),
            BlockNumber::Latest => serializer.serialize_str("latest"),
            BlockNumber::Finalized => serializer.serialize_str("finalized"),
            BlockNumber::Safe => serializer.serialize_str("safe"),
            BlockNumber::Earliest => serializer.serialize_str("earliest"),
            BlockNumber::Pending => serializer.serialize_str("pending"),
        }
    }
}

impl<'de> Deserialize<'de> for BlockNumber {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?.to_lowercase();
        s.parse().map_err(serde::de::Error::custom)
    }
}

impl FromStr for BlockNumber {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "latest" => Ok(Self::Latest),
            "finalized" => Ok(Self::Finalized),
            "safe" => Ok(Self::Safe),
            "earliest" => Ok(Self::Earliest),
            "pending" => Ok(Self::Pending),
            // hex
            n if n.starts_with("0x") => n.parse().map(Self::Number).map_err(|e| e.to_string()),
            // decimal
            n => n.parse::<u64>().map(|n| Self::Number(n.into())).map_err(|e| e.to_string()),
        }
    }
}

impl fmt::Display for BlockNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlockNumber::Number(ref x) => format!("0x{x:x}").fmt(f),
            BlockNumber::Latest => f.write_str("latest"),
            BlockNumber::Finalized => f.write_str("finalized"),
            BlockNumber::Safe => f.write_str("safe"),
            BlockNumber::Earliest => f.write_str("earliest"),
            BlockNumber::Pending => f.write_str("pending"),
        }
    }
}

#[cfg(test)]
#[cfg(not(feature = "celo"))]
mod tests {
    use super::*;
    use crate::types::{Transaction, TxHash};

    #[test]
    fn can_parse_eip1898_block_ids() {
        let num = serde_json::json!(
            { "blockNumber": "0x0" }
        );
        let id = serde_json::from_value::<BlockId>(num).unwrap();
        assert_eq!(id, BlockId::Number(BlockNumber::Number(0u64.into())));

        let num = serde_json::json!(
            { "blockNumber": "pending" }
        );
        let id = serde_json::from_value::<BlockId>(num).unwrap();
        assert_eq!(id, BlockId::Number(BlockNumber::Pending));

        let num = serde_json::json!(
            { "blockNumber": "latest" }
        );
        let id = serde_json::from_value::<BlockId>(num).unwrap();
        assert_eq!(id, BlockId::Number(BlockNumber::Latest));

        let num = serde_json::json!(
            { "blockNumber": "finalized" }
        );
        let id = serde_json::from_value::<BlockId>(num).unwrap();
        assert_eq!(id, BlockId::Number(BlockNumber::Finalized));

        let num = serde_json::json!(
            { "blockNumber": "safe" }
        );
        let id = serde_json::from_value::<BlockId>(num).unwrap();
        assert_eq!(id, BlockId::Number(BlockNumber::Safe));

        let num = serde_json::json!(
            { "blockNumber": "earliest" }
        );
        let id = serde_json::from_value::<BlockId>(num).unwrap();
        assert_eq!(id, BlockId::Number(BlockNumber::Earliest));

        let num = serde_json::json!("0x0");
        let id = serde_json::from_value::<BlockId>(num).unwrap();
        assert_eq!(id, BlockId::Number(BlockNumber::Number(0u64.into())));

        let num = serde_json::json!("pending");
        let id = serde_json::from_value::<BlockId>(num).unwrap();
        assert_eq!(id, BlockId::Number(BlockNumber::Pending));

        let num = serde_json::json!("latest");
        let id = serde_json::from_value::<BlockId>(num).unwrap();
        assert_eq!(id, BlockId::Number(BlockNumber::Latest));

        let num = serde_json::json!("finalized");
        let id = serde_json::from_value::<BlockId>(num).unwrap();
        assert_eq!(id, BlockId::Number(BlockNumber::Finalized));

        let num = serde_json::json!("safe");
        let id = serde_json::from_value::<BlockId>(num).unwrap();
        assert_eq!(id, BlockId::Number(BlockNumber::Safe));

        let num = serde_json::json!("earliest");
        let id = serde_json::from_value::<BlockId>(num).unwrap();
        assert_eq!(id, BlockId::Number(BlockNumber::Earliest));

        let num = serde_json::json!(
            { "blockHash": "0xd4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3" }
        );
        let id = serde_json::from_value::<BlockId>(num).unwrap();
        assert_eq!(
            id,
            BlockId::Hash(
                "0xd4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3"
                    .parse()
                    .unwrap()
            )
        );
    }

    #[test]
    fn serde_block_number() {
        for b in &[
            BlockNumber::Latest,
            BlockNumber::Finalized,
            BlockNumber::Safe,
            BlockNumber::Earliest,
            BlockNumber::Pending,
        ] {
            let b_ser = serde_json::to_string(&b).unwrap();
            let b_de: BlockNumber = serde_json::from_str(&b_ser).unwrap();
            assert_eq!(b_de, *b);
        }

        let b = BlockNumber::Number(1042u64.into());
        let b_ser = serde_json::to_string(&b).unwrap();
        let b_de: BlockNumber = serde_json::from_str(&b_ser).unwrap();
        assert_eq!(b_ser, "\"0x412\"");
        assert_eq!(b_de, b);
    }

    #[test]
    fn deserialize_blk_no_txs() {
        let block = r#"{"number":"0x3","hash":"0xda53da08ef6a3cbde84c33e51c04f68c3853b6a3731f10baa2324968eee63972","parentHash":"0x689c70c080ca22bc0e681694fa803c1aba16a69c8b6368fed5311d279eb9de90","mixHash":"0x0000000000000000000000000000000000000000000000000000000000000000","nonce":"0x0000000000000000","sha3Uncles":"0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347","logsBloom":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","transactionsRoot":"0x7270c1c4440180f2bd5215809ee3d545df042b67329499e1ab97eb759d31610d","stateRoot":"0x29f32984517a7d25607da485b23cefabfd443751422ca7e603395e1de9bc8a4b","receiptsRoot":"0x056b23fbba480696b65fe5a59b8f2148a1299103c4f57df839233af2cf4ca2d2","miner":"0x0000000000000000000000000000000000000000","difficulty":"0x0","totalDifficulty":"0x0","extraData":"0x","size":"0x3e8","gasLimit":"0x6691b7","gasUsed":"0x5208","timestamp":"0x5ecedbb9","transactions":["0xc3c5f700243de37ae986082fd2af88d2a7c2752a0c0f7b9d6ac47c729d45e067"],"uncles":[]}"#;
        let _block: Block<TxHash> = serde_json::from_str(block).unwrap();
    }

    #[test]
    fn deserialize_blk_with_txs() {
        let block = r#"{"number":"0x3","hash":"0xda53da08ef6a3cbde84c33e51c04f68c3853b6a3731f10baa2324968eee63972","parentHash":"0x689c70c080ca22bc0e681694fa803c1aba16a69c8b6368fed5311d279eb9de90","mixHash":"0x0000000000000000000000000000000000000000000000000000000000000000","nonce":"0x0000000000000000","sha3Uncles":"0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347","logsBloom":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","transactionsRoot":"0x7270c1c4440180f2bd5215809ee3d545df042b67329499e1ab97eb759d31610d","stateRoot":"0x29f32984517a7d25607da485b23cefabfd443751422ca7e603395e1de9bc8a4b","receiptsRoot":"0x056b23fbba480696b65fe5a59b8f2148a1299103c4f57df839233af2cf4ca2d2","miner":"0x0000000000000000000000000000000000000000","difficulty":"0x0","totalDifficulty":"0x0","extraData":"0x","size":"0x3e8","gasLimit":"0x6691b7","gasUsed":"0x5208","timestamp":"0x5ecedbb9","transactions":[{"hash":"0xc3c5f700243de37ae986082fd2af88d2a7c2752a0c0f7b9d6ac47c729d45e067","nonce":"0x2","blockHash":"0xda53da08ef6a3cbde84c33e51c04f68c3853b6a3731f10baa2324968eee63972","blockNumber":"0x3","transactionIndex":"0x0","from":"0xfdcedc3bfca10ecb0890337fbdd1977aba84807a","to":"0xdca8ce283150ab773bcbeb8d38289bdb5661de1e","value":"0x0","gas":"0x15f90","gasPrice":"0x4a817c800","input":"0x","v":"0x25","r":"0x19f2694eb9113656dbea0b925e2e7ceb43df83e601c4116aee9c0dd99130be88","s":"0x73e5764b324a4f7679d890a198ba658ba1c8cd36983ff9797e10b1b89dbb448e"}],"uncles":[]}"#;
        let _block: Block<Transaction> = serde_json::from_str(block).unwrap();
    }

    #[test]
    // <https://github.com/tomusdrw/rust-web3/commit/3a32ee962c0f2f8d50a5e25be9f2dfec7ae0750d>
    fn post_london_block() {
        let json = serde_json::json!(
        {
            "baseFeePerGas": "0x7",
            "miner": "0x0000000000000000000000000000000000000001",
            "number": "0x1b4",
            "hash": "0x0e670ec64341771606e55d6b4ca35a1a6b75ee3d5145a99d05921026d1527331",
            "parentHash": "0x9646252be9520f6e71339a8df9c55e4d7619deeb018d2a3f2d21fc165dde5eb5",
            "mixHash": "0x1010101010101010101010101010101010101010101010101010101010101010",
            "nonce": "0x0000000000000000",
            "sealFields": [
              "0xe04d296d2460cfb8472af2c5fd05b5a214109c25688d3704aed5484f9a7792f2",
              "0x0000000000000042"
            ],
            "sha3Uncles": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
            "logsBloom":  "0x0e670ec64341771606e55d6b4ca35a1a6b75ee3d5145a99d05921026d15273310e670ec64341771606e55d6b4ca35a1a6b75ee3d5145a99d05921026d15273310e670ec64341771606e55d6b4ca35a1a6b75ee3d5145a99d05921026d15273310e670ec64341771606e55d6b4ca35a1a6b75ee3d5145a99d05921026d15273310e670ec64341771606e55d6b4ca35a1a6b75ee3d5145a99d05921026d15273310e670ec64341771606e55d6b4ca35a1a6b75ee3d5145a99d05921026d15273310e670ec64341771606e55d6b4ca35a1a6b75ee3d5145a99d05921026d15273310e670ec64341771606e55d6b4ca35a1a6b75ee3d5145a99d05921026d1527331",
            "transactionsRoot": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
            "receiptsRoot": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
            "stateRoot": "0xd5855eb08b3387c0af375e9cdb6acfc05eb8f519e419b874b6ff2ffda7ed1dff",
            "difficulty": "0x27f07",
            "totalDifficulty": "0x27f07",
            "extraData": "0x0000000000000000000000000000000000000000000000000000000000000000",
            "size": "0x27f07",
            "gasLimit": "0x9f759",
            "minGasPrice": "0x9f759",
            "gasUsed": "0x9f759",
            "timestamp": "0x54e34e8e",
            "transactions": [],
            "uncles": []
          }
        );

        let block: Block<()> = serde_json::from_value(json).unwrap();
        assert_eq!(block.base_fee_per_gas, Some(U256::from(7)));
    }

    #[test]
    fn test_next_block_base_fee() {
        // <https://etherscan.io/block/14402566>
        let block_14402566: Block<TxHash> = Block {
            number: Some(U64::from(14402566u64)),
            base_fee_per_gas: Some(U256::from(36_803_013_756u128)),
            gas_limit: U256::from(30_087_887u128),
            gas_used: U256::from(2_023_848u128),
            ..Default::default()
        };

        assert_eq!(block_14402566.base_fee_per_gas, Some(U256::from(36_803_013_756u128)));
        assert_eq!(block_14402566.gas_target(), U256::from(15_043_943u128));
        // next block decreasing base fee https://etherscan.io/block/14402567
        assert_eq!(block_14402566.next_block_base_fee(), Some(U256::from(32_821_521_542u128)));

        // https://etherscan.io/block/14402712
        let block_14402712: Block<TxHash> = Block {
            number: Some(U64::from(14402712u64)),
            base_fee_per_gas: Some(U256::from(24_870_031_149u128)),
            gas_limit: U256::from(30_000_000u128),
            gas_used: U256::from(29_999_374u128),
            ..Default::default()
        };

        assert_eq!(block_14402712.base_fee_per_gas, Some(U256::from(24_870_031_149u128)));
        assert_eq!(block_14402712.gas_target(), U256::from(15_000_000u128));
        // next block increasing base fee https://etherscan.io/block/14402713
        assert_eq!(block_14402712.next_block_base_fee(), Some(U256::from(27_978_655_303u128)));
    }

    #[test]
    fn pending_block() {
        let json = serde_json::json!(
        {
          "baseFeePerGas": "0x3a460775a",
          "difficulty": "0x329b1f81605a4a",
          "extraData": "0xd983010a0d846765746889676f312e31362e3130856c696e7578",
          "gasLimit": "0x1c950d9",
          "gasUsed": "0x1386f81",
          "hash": null,
          "logsBloom": "0x5a3fc3425505bf83b1ebe6ffead1bbfdfcd6f9cfbd1e5fb7fbc9c96b1bbc2f2f6bfef959b511e4f0c7d3fbc60194fbff8bcff8e7b8f6ba9a9a956fe36473ed4deec3f1bc67f7dabe48f71afb377bdaa47f8f9bb1cd56930c7dfcbfddf283f9697fb1db7f3bedfa3e4dfd9fae4fb59df8ac5d9c369bff14efcee59997df8bb16d47d22f0bfbafb29fbfff6e1e41bca61e37e7bdfde1fe27b9fd3a7adfcb74fe98e6dbcc5f5bb3bd4d4bb6ccd29fd3bd446c7f38dcaf7ff78fb3f3aa668cbffe56291d7fbbebd2549fdfd9f223b3ba61dee9e92ebeb5dc967f711d039ff1cb3c3a8fb3b7cbdb29e6d1e79e6b95c596dfe2be36fd65a4f6fdeebe7efbe6e38037d7",
          "miner": null,
          "mixHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
          "nonce": null,
          "number": "0xe1a6ee",
          "parentHash": "0xff1a940068dfe1f9e3f5514e6b7ff5092098d21d706396a9b19602f0f2b11d44",
          "receiptsRoot": "0xe7df36675953a2d2dd22ec0e44ff59818891170875ebfba5a39f6c85084a6f10",
          "sha3Uncles": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
          "size": "0xd8d6",
          "stateRoot": "0x2ed13b153d1467deb397a789aabccb287590579cf231bf4f1ff34ac3b4905ce2",
          "timestamp": "0x6282b31e",
          "totalDifficulty": null,
          "transactions": [
            "0xaad0d53ae350dd06481b42cd097e3f3a4a31e0ac980fac4663b7dd913af20d9b"
          ],
          "transactionsRoot": "0xfc5c28cb82d5878172cd1b97429d44980d1cee03c782950ee73e83f1fa8bfb49",
          "uncles": []
        }
        );
        let block: Block<H256> = serde_json::from_value(json).unwrap();
        assert!(block.author.is_none());
    }

    #[test]
    fn can_deserialize_with_sealed_fields() {
        let json = serde_json::json!({
          "number": "0x1b4",
          "hash": "0xdc0818cf78f21a8e70579cb46a43643f78291264dda342ae31049421c82d21ae",
          "parentHash": "0xe99e022112df268087ea7eafaf4790497fd21dbeeb6bd7a1721df161a6657a54",
          "sha3Uncles": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
          "logsBloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
          "transactionsRoot": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
          "stateRoot": "0xddc8b0234c2e0cad087c8b389aa7ef01f7d79b2570bccb77ce48648aa61c904d",
          "receiptsRoot": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
          "miner": "0xbb7b8287f3f0a933474a79eae42cbca977791171",
          "difficulty": "0x4ea3f27bc",
          "totalDifficulty": "0x78ed983323d",
          "sealFields": null,
          "nonce": "0x689056015818adbe",
          "mixHash": "0x4fffe9ae21f1c9e15207b1f472d5bbdd68c9595d461666602f2be20daf5e7843",
          "extraData": "0x476574682f4c5649562f76312e302e302f6c696e75782f676f312e342e32",
          "size": "0x0",
          "gasLimit": "0x1388",
          "gasUsed": "0x0",
          "timestamp": "0x55ba467c",
          "transactions": [],
          "uncles": []
        }
              );
        let _block: Block<TxHash> = serde_json::from_value(json).unwrap();
    }
}

#[cfg(test)]
#[cfg(feature = "celo")]
mod celo_tests {
    use super::*;
    use crate::types::Transaction;

    #[test]
    fn block_without_snark_data() {
        let block = r#"{"extraData":"0xd983010000846765746889676f312e31332e3130856c696e7578000000000000f8b2c0c080b841cfa11585812ec794c4baa46178690971b3c72e367211d68a9ea318ff500c5aeb7099cafc965240e3b57cf7355341cf76bdca74530334658370d2df7b2e030ab200f582027db017810fa05b4f35927f968f6be1a61e322d4ace3563feb8a489690a91c031fda640c55c216f6712a7bdde994338a5610080f58203ffb093cd643f5154979952791ff714eb885df0f18012f2211fb8c29a8947130dc3adf4ecb48a3c4a142a0faa51e5c60b048180","gasUsed":"0xbef6","hash":"0x37ac2818e50e61f0566caea102ed98677f2552fa86fed53443315ed11fe0eaad","logsBloom":"0x00000800000200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000020000000100000000000000000000000000000000000000000000000000000000000000080000000000001020000400000000000000000000000000000000000000000000000000000000000080000000000000000000000000400000000000000000000000000000000000000100000040004000000000000800000000000000000084000000000000000000000000000000000020000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000","miner":"0xcda518f6b5a797c3ec45d37c65b83e0b0748edca","number":"0x1b4","parentHash":"0xa6b4775f600c2981f9142cbc1361db02c7ba8c185a1110537b255356876301a2","randomness":{"committed":"0x049e84c89f1aa0e3a770b2545b05a30eb814dae322e7247fd2bf27e6cacb1f51","revealed":"0x5a8826bf59a7ed1ee86a9d6464fa9c1fcece78ffa7cf32b11a03ad251ddcefe6"},"receiptsRoot":"0x1724dc3e7c2bfa03974c1deedf5ea20ad30b72e25f3c62fbb5fd06fc107068d7","size":"0x3a0","stateRoot":"0xc45fa03e69dccb54b4981d23d77328ab8906ddd7a0d8238b9c54ae1a14df4d1c","timestamp":"0x5e90166d","totalDifficulty":"0x1b5","transactions":[{"blockHash":"0x37ac2818e50e61f0566caea102ed98677f2552fa86fed53443315ed11fe0eaad","blockNumber":"0x1b4","from":"0x456f41406b32c45d59e539e4bba3d7898c3584da","gas":"0x1312d00","gasPrice":"0x174876e800","feeCurrency":null,"gatewayFeeRecipient":null,"gatewayFee":"0x0","hash":"0xf7b1b588b1fc03305f556805812273d80fb61fc0ba7f812de27189e95c5ecfc5","input":"0xed385274000000000000000000000000b9ff7ab50a2f0fd3e2fb2814b016ac90c91df98f03386ba30000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000be951906eba2aa800000","nonce":"0x147","to":"0xa12a699c641cc875a7ca57495861c79c33d293b4","transactionIndex":"0x0","value":"0x0","v":"0x15e08","r":"0x5787d040d09a34cb2b9ffcd096be7fe66aa6a3ed0632f182d1f3045640a9ef8b","s":"0x7897f58740f2a1c645826579106a620c306fc56381520ae2f28880bb284c4abd"}],"transactionsRoot":"0xbc8cb40b809914b9cd735b12e9b1802cf5d85de5223a22bbdb249a7e8b45ec93"}"#;
        let block: Block<Transaction> = serde_json::from_str(block).unwrap();
        assert_eq!(block.epoch_snark_data, None);
    }

    #[test]
    fn block_with_snark_data() {
        let block = r#"{"extraData":"0xd983010000846765746889676f312e31332e3130856c696e7578000000000000f8b2c0c080b841cfa11585812ec794c4baa46178690971b3c72e367211d68a9ea318ff500c5aeb7099cafc965240e3b57cf7355341cf76bdca74530334658370d2df7b2e030ab200f582027db017810fa05b4f35927f968f6be1a61e322d4ace3563feb8a489690a91c031fda640c55c216f6712a7bdde994338a5610080f58203ffb093cd643f5154979952791ff714eb885df0f18012f2211fb8c29a8947130dc3adf4ecb48a3c4a142a0faa51e5c60b048180","gasUsed":"0xbef6","hash":"0x37ac2818e50e61f0566caea102ed98677f2552fa86fed53443315ed11fe0eaad","logsBloom":"0x00000800000200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000020000000100000000000000000000000000000000000000000000000000000000000000080000000000001020000400000000000000000000000000000000000000000000000000000000000080000000000000000000000000400000000000000000000000000000000000000100000040004000000000000800000000000000000084000000000000000000000000000000000020000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000","miner":"0xcda518f6b5a797c3ec45d37c65b83e0b0748edca","number":"0x1b4","parentHash":"0xa6b4775f600c2981f9142cbc1361db02c7ba8c185a1110537b255356876301a2","randomness":{"committed":"0x049e84c89f1aa0e3a770b2545b05a30eb814dae322e7247fd2bf27e6cacb1f51","revealed":"0x5a8826bf59a7ed1ee86a9d6464fa9c1fcece78ffa7cf32b11a03ad251ddcefe6"},"receiptsRoot":"0x1724dc3e7c2bfa03974c1deedf5ea20ad30b72e25f3c62fbb5fd06fc107068d7","size":"0x3a0","stateRoot":"0xc45fa03e69dccb54b4981d23d77328ab8906ddd7a0d8238b9c54ae1a14df4d1c","timestamp":"0x5e90166d","totalDifficulty":"0x1b5","transactions":[{"blockHash":"0x37ac2818e50e61f0566caea102ed98677f2552fa86fed53443315ed11fe0eaad","blockNumber":"0x1b4","from":"0x456f41406b32c45d59e539e4bba3d7898c3584da","gas":"0x1312d00","gasPrice":"0x174876e800","feeCurrency":null,"gatewayFeeRecipient":null,"gatewayFee":"0x0","hash":"0xf7b1b588b1fc03305f556805812273d80fb61fc0ba7f812de27189e95c5ecfc5","input":"0xed385274000000000000000000000000b9ff7ab50a2f0fd3e2fb2814b016ac90c91df98f03386ba30000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000be951906eba2aa800000","nonce":"0x147","to":"0xa12a699c641cc875a7ca57495861c79c33d293b4","transactionIndex":"0x0","value":"0x0","v":"0x15e08","r":"0x5787d040d09a34cb2b9ffcd096be7fe66aa6a3ed0632f182d1f3045640a9ef8b","s":"0x7897f58740f2a1c645826579106a620c306fc56381520ae2f28880bb284c4abd"}],"transactionsRoot":"0xbc8cb40b809914b9cd735b12e9b1802cf5d85de5223a22bbdb249a7e8b45ec93","epochSnarkData":{"bitmap": "0x01a72267ae3fe9fffb","signature": "0xcd803565d415c14b42d3aee51c5de1f6fd7d33cd036f03178c104c787a6ceafb8dd2b357d5fb5992fc2a23706625c800"}}"#;
        let _block: Block<Transaction> = serde_json::from_str(block).unwrap();
    }
}
