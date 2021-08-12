// Adapted from https://github.com/tomusdrw/rust-web3/blob/master/src/types/log.rs
use crate::{
    types::{Address, BlockNumber, Bytes, H256, U256, U64},
    utils::keccak256,
};
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use std::ops::{Range, RangeFrom, RangeTo};

/// A log produced by a transaction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Log {
    /// H160
    pub address: Address,

    /// topics: Array of 0 to 4 32 Bytes of indexed log arguments.
    /// (In solidity: The first topic is the hash of the signature of the event
    /// (e.g. `Deposit(address,bytes32,uint256)`), except you declared the event
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FilterBlockOption {
    Range {
        from_block: Option<BlockNumber>,
        to_block: Option<BlockNumber>,
    },
    AtBlockHash(H256),
}

impl From<BlockNumber> for FilterBlockOption {
    fn from(block: BlockNumber) -> Self {
        let block = Some(block);
        FilterBlockOption::Range {
            from_block: block,
            to_block: block,
        }
    }
}

impl From<U64> for FilterBlockOption {
    fn from(block: U64) -> Self {
        BlockNumber::from(block).into()
    }
}

impl From<u64> for FilterBlockOption {
    fn from(block: u64) -> Self {
        BlockNumber::from(block).into()
    }
}

impl<T: Into<BlockNumber>> From<Range<T>> for FilterBlockOption {
    fn from(r: Range<T>) -> Self {
        let from_block = Some(r.start.into());
        let to_block = Some(r.end.into());
        FilterBlockOption::Range {
            from_block,
            to_block,
        }
    }
}

impl<T: Into<BlockNumber>> From<RangeTo<T>> for FilterBlockOption {
    fn from(r: RangeTo<T>) -> Self {
        let to_block = Some(r.end.into());
        FilterBlockOption::Range {
            from_block: Some(BlockNumber::Earliest),
            to_block,
        }
    }
}

impl<T: Into<BlockNumber>> From<RangeFrom<T>> for FilterBlockOption {
    fn from(r: RangeFrom<T>) -> Self {
        let from_block = Some(r.start.into());
        FilterBlockOption::Range {
            from_block,
            to_block: Some(BlockNumber::Latest),
        }
    }
}

impl From<H256> for FilterBlockOption {
    fn from(hash: H256) -> Self {
        FilterBlockOption::AtBlockHash(hash)
    }
}

impl Default for FilterBlockOption {
    fn default() -> Self {
        FilterBlockOption::Range {
            from_block: None,
            to_block: None,
        }
    }
}

impl FilterBlockOption {
    pub fn set_from_block(&self, block: BlockNumber) -> Self {
        let to_block = if let FilterBlockOption::Range { to_block, .. } = self {
            *to_block
        } else {
            None
        };

        FilterBlockOption::Range {
            from_block: Some(block),
            to_block,
        }
    }

    pub fn set_to_block(&self, block: BlockNumber) -> Self {
        let from_block = if let FilterBlockOption::Range { from_block, .. } = self {
            *from_block
        } else {
            None
        };

        FilterBlockOption::Range {
            from_block,
            to_block: Some(block),
        }
    }

    pub fn set_hash(&self, hash: H256) -> Self {
        FilterBlockOption::AtBlockHash(hash)
    }
}

/// Filter for
#[derive(Default, Debug, PartialEq, Clone)]
pub struct Filter {
    /// Filter block options, specifying on which blocks the filter should
    /// match.
    // https://eips.ethereum.org/EIPS/eip-234
    pub block_option: FilterBlockOption,

    /// Address
    address: Option<ValueOrArray<Address>>,

    /// Topics
    // TODO: We could improve the low level API here by using ethabi's RawTopicFilter
    // and/or TopicFilter
    pub topics: [Option<ValueOrArray<H256>>; 4],

    /// Limit
    limit: Option<usize>,
}

impl Serialize for Filter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Filter", 5)?;
        match self.block_option {
            FilterBlockOption::Range {
                from_block,
                to_block,
            } => {
                if let Some(ref from_block) = from_block {
                    s.serialize_field("fromBlock", from_block)?;
                }

                if let Some(ref to_block) = to_block {
                    s.serialize_field("toBlock", to_block)?;
                }
            }

            FilterBlockOption::AtBlockHash(ref h) => s.serialize_field("blockHash", h)?,
        }

        if let Some(ref address) = self.address {
            s.serialize_field("address", address)?;
        }

        let mut filtered_topics = Vec::new();
        for i in 0..4 {
            if self.topics[i].is_some() {
                filtered_topics.push(&self.topics[i]);
            } else {
                // TODO: This can be optimized
                if self.topics[i + 1..].iter().any(|x| x.is_some()) {
                    filtered_topics.push(&None);
                }
            }
        }
        s.serialize_field("topics", &filtered_topics)?;

        if let Some(ref limit) = self.limit {
            s.serialize_field("limit", limit)?;
        }

        s.end()
    }
}

impl Filter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the inner filter object
    ///
    /// *NOTE:* ranges are always inclusive
    ///
    /// # Examples
    ///
    /// Match only a specific block
    ///
    /// ```rust
    /// # use ethers::types::Filter;
    /// # fn main() {
    /// let filter = Filter::new().select(69u64);
    /// # }
    /// ```
    /// This is the same as `Filter::new().from_block(1337u64).to_block(1337u64)`
    ///
    /// Match the latest block only
    ///
    /// ```rust
    /// # use ethers::types::{Filter, BlockNumber};
    /// # fn main() {
    /// let filter = Filter::new().select(BlockNumber::Latest);
    /// # }
    /// ```
    ///
    /// Match a block by its hash
    ///
    /// ```rust
    /// # use ethers::types::{Filter, H256};
    /// # fn main() {
    /// let filter = Filter::new().select(H256::zero());
    /// # }
    /// ```
    /// This is the same as `at_block_hash`
    ///
    /// Match a range of blocks
    ///
    /// ```rust
    /// # use ethers::types::{Filter, H256};
    /// # fn main() {
    /// let filter = Filter::new().select(0u64..100u64);
    /// # }
    /// ```
    ///
    /// Match all blocks in range `(1337..BlockNumber::Latest)`
    ///
    /// ```rust
    /// # use ethers::types::{Filter, H256};
    /// # fn main() {
    /// let filter = Filter::new().select(1337u64..);
    /// # }
    /// ```
    ///
    /// Match all blocks in range `(BlockNumber::Earliest..1337)`
    ///
    /// ```rust
    /// # use ethers::types::{Filter, H256};
    /// # fn main() {
    /// let filter = Filter::new().select(..1337u64);
    /// # }
    /// ```
    pub fn select(mut self, filter: impl Into<FilterBlockOption>) -> Self {
        self.block_option = filter.into();
        self
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn from_block<T: Into<BlockNumber>>(mut self, block: T) -> Self {
        self.block_option = self.block_option.set_from_block(block.into());
        self
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_block<T: Into<BlockNumber>>(mut self, block: T) -> Self {
        self.block_option = self.block_option.set_to_block(block.into());
        self
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn at_block_hash<T: Into<H256>>(mut self, hash: T) -> Self {
        self.block_option = self.block_option.set_hash(hash.into());
        self
    }

    pub fn address<T: Into<ValueOrArray<Address>>>(mut self, address: T) -> Self {
        self.address = Some(address.into());
        self
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::serialize;
    use serde_json::json;

    #[test]
    fn filter_serialization_test() {
        let t1 = "9729a6fbefefc8f6005933898b13dc45c3a2c8b7"
            .parse::<Address>()
            .unwrap();
        let t2 = H256::from([0; 32]);
        let t3 = U256::from(123);

        let t1_padded = H256::from(t1);
        let t3_padded = H256::from({
            let mut x = [0; 32];
            x[31] = 123;
            x
        });

        let event = "ValueChanged(address,string,string)";
        let t0 = H256::from(keccak256(event.as_bytes()));
        let addr: Address = "f817796F60D268A36a57b8D2dF1B97B14C0D0E1d".parse().unwrap();
        let filter = Filter::new();

        let ser = serialize(&filter);
        assert_eq!(ser, json!({ "topics": [] }));

        let filter = filter.address(ValueOrArray::Value(addr));

        let ser = serialize(&filter);
        assert_eq!(ser, json!({"address" : addr, "topics": []}));

        let filter = filter.event(event);

        // 0
        let ser = serialize(&filter);
        assert_eq!(ser, json!({ "address" : addr, "topics": [t0]}));

        // 1
        let ser = serialize(&filter.clone().topic1(t1));
        assert_eq!(ser, json!({ "address" : addr, "topics": [t0, t1_padded]}));

        // 2
        let ser = serialize(&filter.clone().topic2(t2));
        assert_eq!(ser, json!({ "address" : addr, "topics": [t0, null, t2]}));

        // 3
        let ser = serialize(&filter.clone().topic3(t3));
        assert_eq!(
            ser,
            json!({ "address" : addr, "topics": [t0, null, null, t3_padded]})
        );

        // 1 & 2
        let ser = serialize(&filter.clone().topic1(t1).topic2(t2));
        assert_eq!(
            ser,
            json!({ "address" : addr, "topics": [t0, t1_padded, t2]})
        );

        // 1 & 3
        let ser = serialize(&filter.clone().topic1(t1).topic3(t3));
        assert_eq!(
            ser,
            json!({ "address" : addr, "topics": [t0, t1_padded, null, t3_padded]})
        );

        // 2 & 3
        let ser = serialize(&filter.clone().topic2(t2).topic3(t3));
        assert_eq!(
            ser,
            json!({ "address" : addr, "topics": [t0, null, t2, t3_padded]})
        );

        // 1 & 2 & 3
        let ser = serialize(&filter.topic1(t1).topic2(t2).topic3(t3));
        assert_eq!(
            ser,
            json!({ "address" : addr, "topics": [t0, t1_padded, t2, t3_padded]})
        );
    }
}
