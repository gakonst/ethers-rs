use ethers_core::types::U256;
use serde::{de, Deserialize, Deserializer};

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
