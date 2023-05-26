use crate::{Client, EtherscanError, Response, Result};
use ethers_core::types::BlockNumber;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlockNumberByTimestamp {
    pub timestamp: u64,
    pub block_number: BlockNumber,
}

impl Client {
    /// Returns either (1) the oldest block since a particular timestamp occurred or (2) the newest
    /// block that occurred prior to that timestamp
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn foo(client: ethers_etherscan::Client) -> Result<(), Box<dyn std::error::Error>> {
    /// // The newest block that occurred prior to 1 January 2020
    /// let block_number_before = client.get_block_by_timestamp(1577836800, "before");
    /// // The oldest block that occurred after 1 January 2020
    /// let block_number_after = client.get_block_by_timestamp(1577836800, "after");
    /// # Ok(()) }
    /// ```
    pub async fn get_block_by_timestamp(
        &self,
        timestamp: u64,
        closest: &str,
    ) -> Result<BlockNumberByTimestamp> {
        let query = self.create_query(
            "block",
            "getblocknobytime",
            HashMap::from([("timestamp", timestamp.to_string()), ("closest", closest.to_string())]),
        );
        let response: Response<String> = self.get_json(&query).await?;

        match response.status.as_str() {
            "0" => Err(EtherscanError::BlockNumberByTimestampFailed),
            "1" => Ok(BlockNumberByTimestamp {
                timestamp,
                block_number: response.result.parse::<BlockNumber>().unwrap(),
            }),
            err => Err(EtherscanError::BadStatusCode(err.to_string())),
        }
    }
}
