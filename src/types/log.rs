use crate::types::{Address, BlockNumber, Bytes, H256, U256, U64};
use serde::{Deserialize, Serialize};

/// A log produced by a transaction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Log {
    /// H160
    pub address: Address,

    /// topics: Array of 0 to 4 32 Bytes of indexed log arguments.
    /// (In solidity: The first topic is the hash of the signature of the event
    /// (e.g. Deposit(address,bytes32,uint256)), except you declared the event
    /// with the anonymous specifier.)
    pub topics: Vec<H256>,

    /// Data
    pub data: Bytes,

    /// Block Hash
    #[serde(rename = "blockHash")]
    pub block_hash: Option<H256>,

    /// Block Number
    #[serde(rename = "blockNumber")]
    pub block_number: Option<U64>,

    /// Transaction Hash
    #[serde(rename = "transactionHash")]
    pub transaction_hash: Option<H256>,

    /// Transaction Index
    #[serde(rename = "transactionIndex")]
    pub transaction_index: Option<U64>,

    /// Integer of the log index position in the block. Noe if it's a pending log.
    #[serde(rename = "logIndex")]
    pub log_index: Option<U256>,

    /// Integer of the transactions index position log was created from.
    /// None when it's a pending log.
    #[serde(rename = "transactionLogIndex")]
    pub transaction_log_index: Option<U256>,

    /// Log Type
    #[serde(rename = "logType")]
    pub log_type: Option<String>,

    /// True when the log was removed, due to a chain reorganization.
    /// false if its a valid log.
    pub removed: Option<bool>,
}

/// Filter
#[derive(Default, Debug, PartialEq, Clone, Serialize)]
pub struct Filter {
    /// From Block
    #[serde(rename = "fromBlock", skip_serializing_if = "Option::is_none")]
    from_block: Option<BlockNumber>,
    /// To Block
    #[serde(rename = "toBlock", skip_serializing_if = "Option::is_none")]
    to_block: Option<BlockNumber>,
    /// Address
    #[serde(skip_serializing_if = "Option::is_none")]
    address: Option<ValueOrArray<Address>>,
    /// Topics
    #[serde(skip_serializing_if = "Option::is_none")]
    topics: Option<Vec<Option<ValueOrArray<H256>>>>,
    /// Limit
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<usize>,
}

impl Filter {
    pub fn from_block<T: Into<BlockNumber>>(mut self, block: BlockNumber) -> Self {
        self.from_block = Some(block.into());
        self
    }

    pub fn to_block<T: Into<BlockNumber>>(mut self, block: BlockNumber) -> Self {
        self.to_block = Some(block.into());
        self
    }

    // pub fn address<T: Into<Address>>(mut self, block: BlockNumber) -> Self {
    //     self.to_block = Some(block.into());
    //     self
    // }
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ValueOrArray<T> {
    Value(T),
    Array(Vec<T>),
}

impl<T> Serialize for ValueOrArray<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            ValueOrArray::Value(inner) => inner.serialize(serializer),
            ValueOrArray::Array(inner) => inner.serialize(serializer),
        }
    }
}
