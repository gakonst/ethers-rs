use ethers_core::types::U256;
use serde::{de, Deserialize, Deserializer};

#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatus {
    pub starting_block: U256,
    pub current_block: U256,
    pub highest_block: U256,
}

pub(crate) fn deserialize_sync_status<'de, D>(
    deserializer: Deserializer<'de>,
) -> Option<SyncStatus> {
    #[derive(Deserialize)]
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
