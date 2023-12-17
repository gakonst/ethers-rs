//! Types for `eth_syncing` RPC call

use crate::types::U64;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Structure used in `eth_syncing` RPC
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SyncingStatus {
    /// When client is synced to highest block, eth_syncing with return string "false"
    IsFalse,
    /// When client is still syncing past blocks we get IsSyncing information.
    IsSyncing(Box<SyncProgress>),
}

impl Serialize for SyncingStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            SyncingStatus::IsFalse => serializer.serialize_bool(false),
            SyncingStatus::IsSyncing(sync) => sync.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for SyncingStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Debug, Serialize, Deserialize)]
        #[serde(untagged)]
        pub enum SyncingStatusIntermediate {
            /// When client is synced to the highest block, eth_syncing with return string "false"
            IsFalse(bool),
            /// When client is still syncing past blocks we get IsSyncing information.
            IsSyncing(Box<SyncProgress>),
        }

        match SyncingStatusIntermediate::deserialize(deserializer)? {
            SyncingStatusIntermediate::IsFalse(false) => Ok(SyncingStatus::IsFalse),
            SyncingStatusIntermediate::IsFalse(true) => Err(serde::de::Error::custom(
                "eth_syncing returned `true` that is undefined value.",
            )),
            SyncingStatusIntermediate::IsSyncing(sync) => Ok(SyncingStatus::IsSyncing(sync)),
        }
    }
}

/// Represents the sync status of the node
///
/// **Note:** while the `eth_syncing` RPC response is defined as:
///
/// > Returns:
/// >
/// > Object|Boolean, An object with sync status data or FALSE, when not syncing:
///
/// > startingBlock: QUANTITY - The block at which the import started (will only be reset, after the
/// > sync reached his head)
/// > currentBlock: QUANTITY - The current block, same as eth_blockNumber
/// > highestBlock: QUANTITY - The estimated highest block
///
/// Geth returns additional fields: <https://github.com/ethereum/go-ethereum/blob/0ce494b60cd00d70f1f9f2dd0b9bfbd76204168a/ethclient/ethclient.go#L597-L617>
///
/// Erigon does not return the startingBlock field and also returns an additional staging field <https://github.com/ledgerwatch/erigon/blob/6a1bb1dff1deca7a50d66bfbc9e0b89bb4e335a1/turbo/jsonrpc/eth_system.go#L70-L74>
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncProgress {
    pub current_block: U64,
    pub highest_block: U64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stages: Option<Vec<SyncStage>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub starting_block: Option<U64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pulled_states: Option<U64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub known_states: Option<U64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub healed_bytecode_bytes: Option<U64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub healed_bytecodes: Option<U64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub healed_trienode_bytes: Option<U64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub healed_trienodes: Option<U64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub healing_bytecode: Option<U64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub healing_trienodes: Option<U64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub synced_account_bytes: Option<U64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub synced_accounts: Option<U64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub synced_bytecode_bytes: Option<U64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub synced_bytecodes: Option<U64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub synced_storage: Option<U64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub synced_storage_bytes: Option<U64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncStage {
    stage_name: String,
    block_number: U64,
}

#[cfg(test)]
mod tests {
    use super::*;

    // <https://github.com/gakonst/ethers-rs/issues/1623>
    #[test]
    fn deserialize_sync_geth() {
        let s = r#"{
        "currentBlock": "0xeaa2b4",
        "healedBytecodeBytes": "0xaad91fe",
        "healedBytecodes": "0x61d3",
        "healedTrienodeBytes": "0x156ac02b1",
        "healedTrienodes": "0x2885aa4",
        "healingBytecode": "0x0",
        "healingTrienodes": "0x454",
        "highestBlock": "0xeaa329",
        "startingBlock": "0xea97ee",
        "syncedAccountBytes": "0xa29fec90d",
        "syncedAccounts": "0xa7ed9ad",
        "syncedBytecodeBytes": "0xdec39008",
        "syncedBytecodes": "0x8d407",
        "syncedStorage": "0x2a517da1",
        "syncedStorageBytes": "0x23634dbedf"
    }"#;

        let sync: SyncingStatus = serde_json::from_str(s).unwrap();
        match sync {
            SyncingStatus::IsFalse => {
                panic!("unexpected variant")
            }
            SyncingStatus::IsSyncing(_) => {}
        }
    }

    // <https://github.com/ledgerwatch/erigon/blob/6a1bb1dff1deca7a50d66bfbc9e0b89bb4e335a1/turbo/jsonrpc/eth_system.go#L70-L74>
    #[test]
    fn deserialize_sync_erigon() {
        let s = r#"{
        "currentBlock": "0x11ef283",
        "highestBlock": "0x11ef284",
        "stages": [
            {
                "block_number": "0x11ef283",
                "stage_name": "Snapshots"
            },
            {
                "block_number": "0x11ef284",
                "stage_name": "Headers"
            },
            {
                "block_number": "0x11ef284",
                "stage_name": "BlockHashes"
            },
            {
                "block_number": "0x11ef284",
                "stage_name": "Bodies"
            },
            {
                "block_number": "0x11ef284",
                "stage_name": "Senders"
            },
            {
                "block_number": "0x11ef284",
                "stage_name": "Execution"
            },
            {
                "block_number": "0x0",
                "stage_name": "Translation"
            },
            {
                "block_number": "0x11ef284",
                "stage_name": "HashState"
            },
            {
                "block_number": "0x11ef284",
                "stage_name": "IntermediateHashes"
            },
            {
                "block_number": "0x11ef283",
                "stage_name": "AccountHistoryIndex"
            },
            {
                "block_number": "0x11ef283",
                "stage_name": "StorageHistoryIndex"
            },
            {
                "block_number": "0x11ef283",
                "stage_name": "LogIndex"
            },
            {
                "block_number": "0x11ef284",
                "stage_name": "CallTraces"
            },
            {
                "block_number": "0x11ef283",
                "stage_name": "TxLookup"
            },
            {
                "block_number": "0x11ef283",
                "stage_name": "Finish"
            }
        ]}"#;

        let sync: SyncingStatus = serde_json::from_str(s).unwrap();
        match sync {
            SyncingStatus::IsFalse => {
                panic!("unexpected variant")
            }
            SyncingStatus::IsSyncing(_) => {}
        }
    }

    #[test]
    fn deserialize_sync_minimal() {
        let s = r#"{
        "currentBlock": "0xeaa2b4",
        "highestBlock": "0xeaa329",
        "startingBlock": "0xea97ee"
    }"#;

        let sync: SyncingStatus = serde_json::from_str(s).unwrap();
        match sync {
            SyncingStatus::IsFalse => {
                panic!("unexpected variant")
            }
            SyncingStatus::IsSyncing(_) => {}
        }
    }

    #[test]
    fn deserialize_sync_false() {
        let s = r"false";

        let sync: SyncingStatus = serde_json::from_str(s).unwrap();
        match sync {
            SyncingStatus::IsFalse => {}
            SyncingStatus::IsSyncing(_) => {
                panic!("unexpected variant")
            }
        }
    }
}
