use serde::{
    de::{self, Unexpected},
    ser::{SerializeSeq, SerializeStruct},
    Deserialize, Deserializer, Serialize, Serializer,
};

use ethers_core::types::{Address, Bytes, Transaction, H256, U256};

/// A block number or tag ("latest", "earliest" or "pending").
///
/// Most commonly, [`Latest`](BlockNumber::Latest) should be the preferred
/// choice, which is also the [`Default`] value.
///
/// # Examples
///
/// ```
/// # use ethers_connections::types::BlockNumber;
/// // there are numerous ways to construct a block number
/// let _: BlockNumber = "latest".into();
/// let _: BlockNumber = "pending".into();
/// let _: BlockNumber = 0xABCD.into();
/// let _: BlockNumber = Default::default();
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum BlockNumber {
    /// Latest block
    Latest,
    /// Earliest block (genesis)
    Earliest,
    /// Pending block (not yet part of the blockchain)
    Pending,
    /// Block by number from canon chain
    Number(u64),
}

impl Default for BlockNumber {
    fn default() -> Self {
        Self::Latest
    }
}

impl BlockNumber {
    /// Returns the numeric block number if explicitly set
    pub fn as_number(&self) -> Option<u64> {
        match *self {
            BlockNumber::Number(num) => Some(num),
            _ => None,
        }
    }
}

impl From<&str> for BlockNumber {
    fn from(tag: &str) -> Self {
        match tag {
            "earliest" => Self::Earliest,
            "latest" => Self::Latest,
            "pending" => Self::Pending,
            _ => panic!("invalid block tag, must be 'earliest', 'latest' or 'pending'"),
        }
    }
}

impl From<u64> for BlockNumber {
    fn from(num: u64) -> Self {
        BlockNumber::Number(num)
    }
}

impl Serialize for BlockNumber {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            BlockNumber::Number(ref x) => serializer.serialize_str(&format!("0x{:x}", x)),
            BlockNumber::Latest => serializer.serialize_str("latest"),
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
        Ok(match s.as_str() {
            "latest" => Self::Latest,
            "earliest" => Self::Earliest,
            "pending" => Self::Pending,
            num => BlockNumber::Number(
                u64::from_str_radix(num.trim_start_matches("0x"), 16).map_err(de::Error::custom)?,
            ),
        })
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Eq)]
#[serde(untagged)]
pub enum SyncStatus {
    #[serde(deserialize_with = "synced")]
    Synced,
    #[serde(rename_all = "camelCase")]
    Syncing { starting_block: U256, current_block: U256, highest_block: U256 },
}

impl SyncStatus {
    pub fn is_synced(&self) -> bool {
        matches!(self, Self::Synced)
    }
}

fn synced<'de, D: Deserializer<'de>>(deserializer: D) -> Result<(), D::Error> {
    let synced = bool::deserialize(deserializer)?;
    if synced {
        return Err(de::Error::invalid_value(Unexpected::Bool(true), &"false"));
    }

    Ok(())
}

/// A filter that can be installed using
/// [`Provider::new_filter`](crate::Provider::new_filter).
///
/// Topics are oder-dependent. A transaction with a log with topics `[A, B]`
/// will be matched by the following topic filters:
///
/// - `[]`:  "anything"
/// - `[A]`: "A in first position (and anything after)"
/// - `[null, B]`: "anything in first position AND B in second position
///   (and anything after)"
/// - `[[A, B], [A, B]]`: "(A OR B) in first position AND (A OR B) in second
///   position (and anything after)"
#[derive(Debug, Default)]
pub struct Filter {
    /// The first block from which to include logs.
    pub from_block: Option<BlockNumber>,
    /// The last block from which to include logs.
    pub to_block: Option<BlockNumber>,
    /// The contract address(es) to consider.
    pub address: ValueOrArray<Address>,
    /// The topic hashes to consider.
    pub topics: [ValueOrArray<H256>; 4],
}

impl Filter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the filter's `from_block`.
    pub fn from_block(mut self, from_block: BlockNumber) -> Self {
        self.from_block = Some(from_block);
        self
    }

    /// Sets the filter's `to_block`.
    pub fn to_block(mut self, to_block: BlockNumber) -> Self {
        self.to_block = Some(to_block);
        self
    }

    /// Sets the filter's `address`.
    pub fn address(mut self, address: ValueOrArray<Address>) -> Self {
        self.address = address;
        self
    }

    /// Sets the filter`s first topic (the event name) to the hash of `name`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ethers_connections::types::Filter;
    ///
    /// let filter = Filter::new().event("Transfer(uint256)");
    /// # drop(filter)
    /// ```
    pub fn event(mut self, event: &str) -> Self {
        let topic0 = H256(ethers_core::utils::keccak256(event)).into();
        self.topics[0] = topic0;
        self
    }

    pub fn topic0(mut self, topic: ValueOrArray<H256>) -> Self {
        self.topics[0] = topic;
        self
    }

    pub fn topic1(mut self, topic: ValueOrArray<H256>) -> Self {
        self.topics[1] = topic;
        self
    }

    pub fn topic2(mut self, topic: ValueOrArray<H256>) -> Self {
        self.topics[2] = topic;
        self
    }

    pub fn topic3(mut self, topic: ValueOrArray<H256>) -> Self {
        self.topics[3] = topic;
        self
    }
}

impl Serialize for Filter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_struct("Filter", 4)?;
        if let Some(from) = self.from_block {
            map.serialize_field("fromBlock", &from)?;
        }

        if let Some(to) = self.to_block {
            map.serialize_field("toBlock", &to)?;
        }

        if !self.address.is_empty() {
            map.serialize_field("address", &self.address)?;
        }

        let mut mask = 0;
        for i in 0..4 {
            mask |= (!self.topics[i].is_empty() as usize) << i;
        }

        if mask != 0 {
            struct Helper<'a>(&'a [ValueOrArray<H256>; 4], usize);
            impl Serialize for Helper<'_> {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    let &Helper(topics, mut mask) = self;
                    let mut seq = serializer.serialize_seq(Some(4))?;
                    for i in 0..4 {
                        seq.serialize_element(&topics[i])?;

                        mask >>= 1;
                        if mask == 0 {
                            break;
                        }
                    }

                    seq.end()
                }
            }

            map.serialize_field("topics", &Helper(&self.topics, mask))?;
        }

        map.end()
    }
}

#[derive(Debug, Default)]
pub struct ValueOrArray<T>(pub Vec<T>);

impl<T> ValueOrArray<T> {
    pub fn null() -> Self {
        Self(vec![])
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<T> From<T> for ValueOrArray<T> {
    fn from(val: T) -> Self {
        Self(vec![val])
    }
}

impl<T> From<Vec<T>> for ValueOrArray<T> {
    fn from(vec: Vec<T>) -> Self {
        Self(vec)
    }
}

impl<const N: usize, T: Clone> From<[T; N]> for ValueOrArray<T> {
    fn from(arr: [T; N]) -> Self {
        Self(arr.to_vec())
    }
}

impl<T: Clone> From<&[T]> for ValueOrArray<T> {
    fn from(slice: &[T]) -> Self {
        Self(slice.to_vec())
    }
}

impl<T: Serialize> Serialize for ValueOrArray<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.0.as_slice() {
            [] => ().serialize(serializer),
            [one] => one.serialize(serializer),
            many => many.serialize(serializer),
        }
    }
}

pub enum FilterChanges {
    Block,
    PendingTransactions,
    Logs,
}

/// The properties for a transaction to be simulated or replayed (see
/// [`Provider::call`](crate::Provider)).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionCall {
    /// The sender's address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<Address>,
    /// The recipient's address.
    pub to: Address,
    /// The maximum amount (limit) of gas.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas: Option<U256>,
    /// The gas price in wei.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<U256>,
    /// ....
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<U256>,
    /// The ...
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Bytes>,
}

/// The properties of the object returned by a call to
/// [`Provider::fill_transaction`](crate::Provider::fill_transaction).
#[derive(Debug, Clone, Deserialize)]
pub struct PreparedTransaction {
    /// The RLP encoded raw transaction.
    pub raw: Bytes,
    /// The transaction with its filled properties.
    pub tx: Transaction,
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use serde_json::Deserializer;

    use ethers_core::types::{H256, U256};

    use super::{Filter, SyncStatus};

    #[test]
    fn deserialize_sync_status() {
        let de = &mut Deserializer::from_str("false");
        let status: SyncStatus = Deserialize::deserialize(de).unwrap();
        assert_eq!(status, SyncStatus::Synced);

        let de = &mut Deserializer::from_str(
            r###"{"startingBlock":"0x5555","currentBlock":"0x5588","highestBlock":"0x6000"}"###,
        );
        let status: SyncStatus = Deserialize::deserialize(de).unwrap();
        assert_eq!(
            status,
            SyncStatus::Syncing {
                starting_block: U256::from(0x5555),
                current_block: U256::from(0x5588),
                highest_block: U256::from(0x6000),
            }
        );
    }

    #[test]
    fn serialize_filter() {
        let filter = Filter::new();
        assert_eq!(serde_json::to_string(&filter).unwrap(), "{}");

        let filter = Filter::new()
            .from_block("latest".into())
            .to_block("pending".into())
            .topic1(vec![H256::zero(), H256::zero()].into())
            .topic3(H256::zero().into());

        let zero = serde_json::to_string(&H256::zero()).unwrap();
        let json = format!(
            r##"{{"fromBlock":"latest","toBlock":"pending","topics":[null,[{zero},{zero}],null,{zero}]}}"##
        );
        assert_eq!(serde_json::to_string(&filter).unwrap(), json);
    }
}
