use crate::{Address, BlockNumber, Bytes, H256, U256, U64};
use ethers_utils::keccak256;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_hash: Option<H256>,

    /// Block Number
    #[serde(rename = "blockNumber")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_number: Option<U64>,

    /// Transaction Hash
    #[serde(rename = "transactionHash")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_hash: Option<H256>,

    /// Transaction Index
    #[serde(rename = "transactionIndex")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_index: Option<U64>,

    /// Integer of the log index position in the block. Noe if it's a pending log.
    #[serde(rename = "logIndex")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_index: Option<U256>,

    /// Integer of the transactions index position log was created from.
    /// None when it's a pending log.
    #[serde(rename = "transactionLogIndex")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_log_index: Option<U256>,

    /// Log Type
    #[serde(rename = "logType")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_type: Option<String>,

    /// True when the log was removed, due to a chain reorganization.
    /// false if its a valid log.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removed: Option<bool>,
}

/// Filter
#[derive(Default, Debug, PartialEq, Clone, Serialize)]
pub struct Filter {
    /// From Block
    #[serde(rename = "fromBlock", skip_serializing_if = "Option::is_none")]
    pub from_block: Option<BlockNumber>,

    /// To Block
    #[serde(rename = "toBlock", skip_serializing_if = "Option::is_none")]
    pub to_block: Option<BlockNumber>,

    /// Address
    #[serde(skip_serializing_if = "Option::is_none")]
    // TODO: The spec says that this can also be an array, do we really want to
    // monitor for the same event for multiple contracts?
    address: Option<Address>,

    /// Topics
    #[serde(skip_serializing_if = "Vec::is_empty")]
    // TODO: Split in an event name + 3 topics
    pub topics: Vec<ValueOrArray<H256>>,

    /// Limit
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<usize>,
}

impl Filter {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn from_block<T: Into<BlockNumber>>(mut self, block: T) -> Self {
        self.from_block = Some(block.into());
        self
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_block<T: Into<BlockNumber>>(mut self, block: T) -> Self {
        self.to_block = Some(block.into());
        self
    }

    pub fn address<T: Into<Address>>(mut self, address: T) -> Self {
        self.address = Some(address.into());
        self
    }

    pub fn address_str(mut self, address: &str) -> Result<Self, rustc_hex::FromHexError> {
        self.address = Some(Address::from_str(address)?);
        Ok(self)
    }

    /// given the event in string form, it hashes it and adds it to the topics to monitor
    pub fn event(self, event_name: &str) -> Self {
        let hash = H256::from(keccak256(event_name.as_bytes()));
        self.topic(hash)
    }

    pub fn topic<T: Into<ValueOrArray<H256>>>(mut self, topic: T) -> Self {
        self.topics.push(topic.into());
        self
    }

    pub fn topics(mut self, topics: &[ValueOrArray<H256>]) -> Self {
        self.topics.extend_from_slice(topics);
        self
    }

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

impl From<H256> for ValueOrArray<H256> {
    fn from(src: H256) -> Self {
        ValueOrArray::Value(src)
    }
}

impl From<Address> for ValueOrArray<H256> {
    fn from(src: Address) -> Self {
        let mut bytes = [0; 32];
        bytes[12..32].copy_from_slice(src.as_bytes());
        ValueOrArray::Value(H256::from(bytes))
    }
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
