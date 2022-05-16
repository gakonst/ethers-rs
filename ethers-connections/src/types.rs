use ethers_core::types::{Address, Bytes, U256, U64};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

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

/// TODO.
#[derive(Debug, Clone, Deserialize, Serialize)]
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

// FIXME: should be in a separate PR

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
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
    transaction_type: TransactionType,
}

impl Transaction {
    pub fn from(mut self, from: Address) -> Self {
        self.from = Some(from);
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
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TransactionType {
    Legacy {
        gas_price: Option<U256>,
    },
    Eip2930 {
        gas_price: Option<U256>,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        access_list: Vec<()>,
    },
    Eip1559 {
        max_priority_fee_per_gas: Option<U256>,
        max_fee_per_gas: Option<U256>,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        access_list: Vec<()>,
    },
}

#[cfg(test)]
mod tests {
    use serde_json::Deserializer;

    use crate::types::SyncStatus;

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
}
