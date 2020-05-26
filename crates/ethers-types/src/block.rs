use crate::{Address, Bloom, Bytes, H256, U256, U64};
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};

/// The block type returned from RPC calls.
/// This is generic over a `TX` type which will be either the hash or the
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct Block<TX> {
    /// Hash of the block
    pub hash: Option<H256>,
    /// Hash of the parent
    #[serde(rename = "parentHash")]
    pub parent_hash: H256,
    /// Hash of the uncles
    #[serde(rename = "sha3Uncles")]
    pub uncles_hash: H256,
    /// Miner/author's address.
    #[serde(rename = "miner")]
    pub author: Address,
    /// State root hash
    #[serde(rename = "stateRoot")]
    pub state_root: H256,
    /// Transactions root hash
    #[serde(rename = "transactionsRoot")]
    pub transactions_root: H256,
    /// Transactions receipts root hash
    #[serde(rename = "receiptsRoot")]
    pub receipts_root: H256,
    /// Block number. None if pending.
    pub number: Option<U64>,
    /// Gas Used
    #[serde(rename = "gasUsed")]
    pub gas_used: U256,
    /// Gas Limit
    #[serde(rename = "gasLimit")]
    pub gas_limit: U256,
    /// Extra data
    #[serde(rename = "extraData")]
    pub extra_data: Bytes,
    /// Logs bloom
    #[serde(rename = "logsBloom")]
    pub logs_bloom: Option<Bloom>,
    /// Timestamp
    pub timestamp: U256,
    /// Difficulty
    pub difficulty: U256,
    /// Total difficulty
    #[serde(rename = "totalDifficulty")]
    pub total_difficulty: Option<U256>,
    /// Seal fields
    #[serde(default, rename = "sealFields")]
    pub seal_fields: Vec<Bytes>,
    /// Uncles' hashes
    pub uncles: Vec<H256>,
    /// Transactions
    pub transactions: Vec<TX>,
    /// Size in bytes
    pub size: Option<U256>,
    /// Mix Hash
    #[serde(rename = "mixHash")]
    pub mix_hash: Option<H256>,
    /// Nonce
    pub nonce: Option<U64>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BlockId {
    Hash(H256),
    Number(BlockNumber),
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
                s.serialize_field("blockHash", &format!("{:?}", x))?;
                s.end()
            }
            BlockId::Number(ref num) => num.serialize(serializer),
        }
    }
}

/// Block Number
#[derive(Copy, Clone, Debug, PartialEq)]
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
            BlockNumber::Number(ref x) => serializer.serialize_str(&format!("0x{:x}", x)),
            BlockNumber::Latest => serializer.serialize_str("latest"),
            BlockNumber::Earliest => serializer.serialize_str("earliest"),
            BlockNumber::Pending => serializer.serialize_str("pending"),
        }
    }
}
