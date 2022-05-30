mod fee_history;
mod filter;
mod transaction;

use std::mem;

use ethers_core::types::{Address, Bloom, Bytes, Log, H256, U256, U64};
use serde::{
    de,
    ser::{SerializeSeq, SerializeStruct},
    Deserialize, Deserializer, Serialize, Serializer,
};

/// A block number or tag ("latest", "earliest" or "pending").
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum BlockNumber {
    /// Latest block
    Latest,
    /// Earliest block (genesis)
    Earliest,
    /// Pending block (not yet part of the blockchain)
    Pending,
    /// Block by number from canon chain
    Number(U64),
}

impl BlockNumber {
    /// Returns the numeric block number if explicitly set
    pub fn as_number(&self) -> Option<u64> {
        match *self {
            BlockNumber::Number(num) => Some(num.low_u64()),
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
        BlockNumber::Number(num.into())
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
            num => BlockNumber::Number(num.parse().map_err(serde::de::Error::custom)?),
        })
    }
}

/// The current sync status of the provider.
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatus {
    pub starting_block: U256,
    pub current_block: U256,
    pub highest_block: U256,
}

pub(crate) fn deserialize_sync_status<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<SyncStatus>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Helper {
        Synced(bool),
        Syncing(SyncStatus),
    }

    match Deserialize::deserialize(deserializer)? {
        Helper::Synced(false) => Ok(None),
        Helper::Syncing(status) => Ok(Some(status)),
        Helper::Synced(true) => Err(de::Error::custom("`eth_syncing` can not return `true`")),
    }
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

// FIXME: should be in a separate PR?

/// The properties for a transaction to be simulated or replayed (see
/// [`Provider::call`](crate::Provider)).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionCall {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<Address>,
    pub to: Address,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas: Option<U256>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<U256>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<U256>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Bytes>,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionRequest {
    /// The sender address or ENS name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<Address>,
    /// Recipient address (None for contract creation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<Address>,
    /// Supplied gas (None for sensible default)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas: Option<U256>,
    /// Transfered value (None for no transfer)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<U256>,
    /// The compiled code of a contract OR the first 4 bytes of the hash of the
    /// invoked method signature and encoded parameters. For details see Ethereum Contract ABI
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Bytes>,
    /// Transaction nonce (None for next available nonce)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<U256>,
    /// The transaction type.
    #[serde(flatten)]
    pub transaction_type: TransactionType,
}

impl TransactionRequest {
    /// Creates new (empty) [`Legacy`](TransactionType::Legacy) transaction.
    pub fn legacy() -> Self {
        Self { transaction_type: TransactionType::Legacy { gas_price: None }, ..Default::default() }
    }

    /// Creates new (empty) [`Eip2930`](TransactionType::Eip2930) (access list)
    /// transaction.
    pub fn eip2930(access_list: Vec<()>) -> Self {
        Self {
            transaction_type: TransactionType::Eip2930 { gas_price: None, access_list },
            ..Default::default()
        }
    }

    /// Creates new (empty) [`Eip1559`](TransactionType::Eip1559) (dynamic fee)
    /// transaction.
    pub fn eip1559() -> Self {
        Self {
            transaction_type: TransactionType::Eip1559 {
                max_fee_per_gas: None,
                max_priority_fee_per_gas: None,
                access_list: vec![],
            },
            ..Default::default()
        }
    }

    /// Sets the `from` address.
    pub fn from(mut self, from: Address) -> Self {
        self.from = Some(from);
        self
    }

    /// Sets the `to` address.
    pub fn to(mut self, to: Address) -> Self {
        self.to = Some(to);
        self
    }

    /// Sets the `gas` limit.
    pub fn gas(mut self, gas: U256) -> Self {
        self.gas = Some(gas);
        self
    }

    pub fn value(mut self, value: U256) -> Self {
        self.nonce = Some(value);
        self
    }

    pub fn data(mut self, data: Bytes) -> Self {
        self.data = Some(data);
        self
    }

    pub fn nonce(mut self, nonce: U256) -> Self {
        self.nonce = Some(nonce);
        self
    }

    pub fn access_list(mut self, access_list: Vec<()>) -> Self {
        match &mut self.transaction_type {
            TransactionType::Legacy { gas_price } => {
                let gas_price = *gas_price;
                self.transaction_type = TransactionType::Eip2930 { gas_price, access_list }
            }
            TransactionType::Eip2930 { access_list: al, .. } => *al = access_list,
            TransactionType::Eip1559 { access_list: al, .. } => *al = access_list,
        };
        self
    }

    pub fn gas_price(mut self, gas_price: U256) -> Self {
        match &mut self.transaction_type {
            TransactionType::Legacy { gas_price: gp } => *gp = Some(gas_price),
            TransactionType::Eip2930 { gas_price: gp, .. } => *gp = Some(gas_price),
            TransactionType::Eip1559 { access_list, .. } => {
                let access_list = mem::replace(access_list, vec![]);
                self.transaction_type =
                    TransactionType::Eip2930 { gas_price: Some(gas_price), access_list };
            }
        }
        self
    }
}

/// The type of a transactions and its respective unique properties.
#[derive(Deserialize, Serialize)]
pub enum TransactionType {
    /// A legacy transaction (with `gasPrice`).
    #[serde(rename = "0x0")]
    Legacy { gas_price: Option<U256> },
    /// An access list transaction (with `gasPrice`).
    #[serde(rename = "0x1")]
    Eip2930 {
        gas_price: Option<U256>,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        access_list: Vec<()>,
    },
    /// A dynamic fee transaction (with `maxPriorityFeePerGas` and `maxFeePerGas`).
    #[serde(rename = "0x2")]
    Eip1559 {
        max_priority_fee_per_gas: Option<U256>,
        max_fee_per_gas: Option<U256>,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        access_list: Vec<()>,
    },
}

impl Default for TransactionType {
    fn default() -> Self {
        Self::Eip1559 { max_priority_fee_per_gas: None, max_fee_per_gas: None, access_list: vec![] }
    }
}

/// The receipt for a confirmed transaction.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TransactionReceipt {
    /// The transaction hash.
    pub transaction_hash: H256,
    /// The index within the block.
    pub transaction_index: U64,
    /// The hash of the block this transaction was included within.
    pub block_hash: H256,
    /// The number of the block this transaction was included within.
    pub block_number: U64,
    /// The address of the sender.
    pub from: Address,
    // The address of the receiver (`None` if contract creation).
    pub to: Option<Address>,
    /// Cumulative gas used within the block after this was executed.
    pub cumulative_gas_used: U256,
    /// Gas used by this transaction alone (`None` if light client).
    pub gas_used: Option<U256>,
    /// Created contract address (`None` if not a deployment).
    pub contract_address: Option<Address>,
    /// Logs generated within this transaction.
    pub logs: Vec<Log>,
    /// The transaction status, 0x1 for success, 0x0 for failure (only present
    /// after [EIP-658](https://eips.ethereum.org/EIPS/eip-658)).
    pub status: Option<U64>,
    /// State root. Only present before activation of [EIP-658](https://eips.ethereum.org/EIPS/eip-658)
    pub root: Option<H256>,
    /// Logs bloom
    pub logs_bloom: Bloom,
    /// The transaction type, `None` for Legacy, `Some(1)` for access list
    /// transaction (EIP-2930), `Some(2)` for dynamic fee transaction (EIP-1559).
    pub transaction_type: Option<U64>,
    /// The price paid post-execution by the transaction (i.e. base fee + priority fee).
    /// Both fields in 1559-style transactions are *maximums* (max fee + max priority fee), the
    /// amount that's actually paid by users can only be determined post-execution
    pub effective_gas_price: Option<U256>,
}

#[cfg(test)]
mod tests {
    use serde_json::Deserializer;

    use ethers_core::types::H256;

    use super::{Filter, SyncStatus};

    #[test]
    fn deserialize_sync_status() {
        assert_eq!(
            super::deserialize_sync_status(&mut Deserializer::from_str("false")).unwrap(),
            None
        );

        assert_eq!(
            super::deserialize_sync_status(&mut Deserializer::from_str(
                r###"{"startingBlock":"0x5555","currentBlock":"0x5588","highestBlock":"0x6000"}"###
            ))
            .unwrap(),
            Some(SyncStatus {
                starting_block: 0x5555.into(),
                current_block: 0x5588.into(),
                highest_block: 0x6000.into(),
            })
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

//
