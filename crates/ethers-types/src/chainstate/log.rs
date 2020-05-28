// Adapted from https://github.com/tomusdrw/rust-web3/blob/master/src/types/log.rs
use crate::{Address, BlockNumber, Bytes, H256, U256, U64};
use ethers_utils::keccak256;
use serde::{ser::SerializeSeq, Deserialize, Serialize, Serializer};
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

/// Filter for
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
    // TODO: We could improve the low level API here by using ethabi's RawTopicFilter
    // and/or TopicFilter
    #[serde(serialize_with = "skip_nones")]
    pub topics: [Option<ValueOrArray<H256>>; 4],

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
        self.topic0(hash)
    }

    /// Sets topic0 (the event name for non-anonymous events)
    pub fn topic0<T: Into<ValueOrArray<H256>>>(mut self, topic: T) -> Self {
        self.topics[0] = Some(topic.into());
        self
    }

    /// Sets the 1st indexed topic
    pub fn topic1<T: Into<ValueOrArray<H256>>>(mut self, topic: T) -> Self {
        self.topics[1] = Some(topic.into());
        self
    }

    /// Sets the 2nd indexed topic
    pub fn topic2<T: Into<ValueOrArray<H256>>>(mut self, topic: T) -> Self {
        self.topics[2] = Some(topic.into());
        self
    }

    /// Sets the 3rd indexed topic
    pub fn topic3<T: Into<ValueOrArray<H256>>>(mut self, topic: T) -> Self {
        self.topics[3] = Some(topic.into());
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// Union type for representing a single value or a vector of values inside a filter
#[derive(Debug, PartialEq, Clone)]
pub enum ValueOrArray<T> {
    /// A single value
    Value(T),
    /// A vector of values
    Array(Vec<T>),
}

// TODO: Implement more common types - or adjust this to work with all Tokenizable items

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

impl From<U256> for ValueOrArray<H256> {
    fn from(src: U256) -> Self {
        let mut bytes = [0; 32];
        src.to_big_endian(&mut bytes);
        ValueOrArray::Value(H256::from(bytes))
    }
}

impl<T> Serialize for ValueOrArray<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ValueOrArray::Value(inner) => inner.serialize(serializer),
            ValueOrArray::Array(inner) => inner.serialize(serializer),
        }
    }
}

// adapted from https://github.com/serde-rs/serde/issues/550#issuecomment-246746639
fn skip_nones<T, S>(elements: &[Option<T>], serializer: S) -> Result<S::Ok, S::Error>
where
    T: Serialize,
    S: Serializer,
{
    // get number of Some elements
    let len = elements.iter().filter(|opt| opt.is_some()).count();

    let mut seq = serializer.serialize_seq(Some(len))?;
    for elem in elements {
        if elem.is_some() {
            seq.serialize_element(elem)?;
        }
    }
    seq.end()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethers_utils::serialize;

    #[test]
    fn filter_serialization_test() {
        let t1 = "9729a6fbefefc8f6005933898b13dc45c3a2c8b7"
            .parse::<Address>()
            .unwrap();
        let t3 = U256::from(123);
        let filter = Filter::new()
            .address_str("f817796F60D268A36a57b8D2dF1B97B14C0D0E1d")
            .unwrap()
            .event("ValueChanged(address,string,string)") // event name
            .topic1(t1)
            .topic2(t3);

        dbg!(&filter);
        let ser = serialize(&filter).to_string();
        assert_eq!(ser, "{\"address\":\"0xf817796f60d268a36a57b8d2df1b97b14c0d0e1d\",\"topics\":[\"0xe826f71647b8486f2bae59832124c70792fba044036720a54ec8dacdd5df4fcb\",\"0x0000000000000000000000009729a6fbefefc8f6005933898b13dc45c3a2c8b7\",\"0x000000000000000000000000000000000000000000000000000000000000007b\"]}");
    }
}
