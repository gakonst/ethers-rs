//! Mod of types for ethereum logs
use ethers_core::{
    abi::{Error, RawLog},
    types::{Address, Log, TxHash, H256, U256, U64},
};
use serde::{Deserialize, Serialize};

/// A trait for types (events) that can be decoded from a `RawLog`
pub trait EthLogDecode: Send + Sync {
    /// decode from a `RawLog`
    fn decode_log(log: &RawLog) -> Result<Self, Error>
    where
        Self: Sized;
}

/// Decodes a series of logs into a vector
pub fn decode_logs<T: EthLogDecode>(logs: &[RawLog]) -> Result<Vec<T>, Error> {
    logs.iter().map(T::decode_log).collect()
}

/// Metadata inside a log
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogMeta {
    /// Address from which this log originated
    pub address: Address,

    /// The block in which the log was emitted
    pub block_number: U64,

    /// The block hash in which the log was emitted
    pub block_hash: H256,

    /// The transaction hash in which the log was emitted
    pub transaction_hash: TxHash,

    /// Transactions index position log was created from
    pub transaction_index: U64,

    /// Log index position in the block
    pub log_index: U256,
}

impl From<&Log> for LogMeta {
    fn from(src: &Log) -> Self {
        LogMeta {
            address: src.address,
            block_number: src.block_number.expect("should have a block number"),
            block_hash: src.block_hash.expect("should have a block hash"),
            transaction_hash: src.transaction_hash.expect("should have a tx hash"),
            transaction_index: src.transaction_index.expect("should have a tx index"),
            log_index: src.log_index.expect("should have a log index"),
        }
    }
}
