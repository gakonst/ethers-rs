use std::collections::HashMap;
use std::str::FromStr;

use ethers_core::types::BlockNumber;
use serde::{Deserialize, Serialize};

use crate::{Client, EtherscanError, Response, Result};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlockNumberByTimestamp {
    pub timestamp: u64,
    pub block_number: BlockNumber,
}


impl Client {
    /// Returns block by timestamp
    pub async fn get_block_by_timestamp(&self, timestamp: u64, closest: &str) -> Result<BlockNumberByTimestamp> {
        let query = self.create_query(
            "block",
            "getblocknobytime",
            HashMap::from([("timestamp", timestamp.to_string()), ("closest", closest.to_string())]),
        );
        let response: Response<String> = self.get_json(&query).await?;

        match response.status.as_str() {
            "0" => Err(EtherscanError::BlockNumberByTimestampFailed),
            "1" => Ok(
                BlockNumberByTimestamp{
                    timestamp: timestamp,
                    block_number: response.result.parse::<BlockNumber>().unwrap()
                }
            ),
            err => Err(EtherscanError::BadStatusCode(err.to_string())),
        }
    }
}
