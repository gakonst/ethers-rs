use crate::{Client, EtherscanError, Response, Result};
use chrono::{DateTime, NaiveDate, NaiveTime, TimeZone, Utc};
use ethers_core::{types::U256, utils::parse_units};
use serde::{Deserialize, Deserializer};
use std::str::FromStr;

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct EthSupply2 {
    /// The current amount of ETH in circulation
    #[serde(deserialize_with = "deserialize_number_from_string")]
    #[serde(rename = "EthSupply")]
    pub eth_supply: u128,
    /// The current amount of ETH2 Staking rewards
    #[serde(deserialize_with = "deserialize_number_from_string")]
    #[serde(rename = "Eth2Staking")]
    pub eth2_staking: u128,
    /// The current amount of EIP1559 burnt fees
    #[serde(deserialize_with = "deser_wei_amount")]
    #[serde(rename = "BurntFees")]
    pub burnt_fees: U256,
    /// Total withdrawn ETH from the beacon chain
    #[serde(deserialize_with = "deserialize_number_from_string")]
    #[serde(rename = "WithdrawnTotal")]
    pub withdrawn_total: u128,
}

#[derive(Deserialize, Clone, Debug)]
pub struct EthPrice {
    /// ETH-to-BTC exchange rate
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub ethbtc: f64,
    /// Last updated timestamp for the ETH-to-BTC exchange rate
    #[serde(deserialize_with = "deserialize_datetime_from_string")]
    pub ethbtc_timestamp: DateTime<Utc>,
    /// ETH-to-USD exchange rate
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub ethusd: f64,
    /// Last updated timestamp for the ETH-to-USD exchange rate
    #[serde(deserialize_with = "deserialize_datetime_from_string")]
    pub ethusd_timestamp: DateTime<Utc>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct NodeCount {
    /// Last updated date for the total number of discoverable Ethereum nodes
    #[serde(rename = "UTCDate")]
    #[serde(deserialize_with = "deserialize_utc_date_from_string")]
    pub utc_date: DateTime<Utc>,
    /// The total number of discoverable Ethereum nodes
    #[serde(rename = "TotalNodeCount")]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub total_node_count: usize,
}

// This function is used to deserialize a string or number into a U256 with an
// amount of wei. If the contents is a number, deserialize it. If the contents
// is a string, attempt to deser as first a decimal f64 then a decimal U256.
fn deser_wei_amount<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt {
        Number(u64),
        String(String),
    }

    match StringOrInt::deserialize(deserializer)? {
        StringOrInt::Number(i) => Ok(U256::from(i)),
        StringOrInt::String(s) => {
            parse_units(s, "wei").map(Into::into).map_err(serde::de::Error::custom)
        }
    }
}

fn deserialize_number_from_string<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + serde::Deserialize<'de>,
    <T as FromStr>::Err: std::fmt::Display,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt<T> {
        String(String),
        Number(T),
    }

    match StringOrInt::<T>::deserialize(deserializer)? {
        StringOrInt::String(s) => s.parse::<T>().map_err(serde::de::Error::custom),
        StringOrInt::Number(i) => Ok(i),
    }
}

fn deserialize_utc_date_from_string<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;

    let naive_date = NaiveDate::parse_from_str(&s, "%Y-%m-%d").expect("Invalid date format");
    let naive_time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();

    Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive_date.and_time(naive_time), Utc))
}

fn deserialize_datetime_from_string<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt {
        String(String),
        Number(i64),
    }

    match StringOrInt::deserialize(deserializer)? {
        StringOrInt::String(s) => {
            let i = s.parse::<i64>().unwrap();
            Ok(Utc.timestamp_opt(i, 0).unwrap())
        }
        StringOrInt::Number(i) => Ok(Utc.timestamp_opt(i, 0).unwrap()),
    }
}

impl Client {
    /// Returns the current amount of Ether in circulation excluding ETH2 Staking rewards
    /// and EIP1559 burnt fees.
    pub async fn eth_supply(&self) -> Result<u128> {
        let query = self.create_query("stats", "ethsupply", serde_json::Value::Null);
        let response: Response<String> = self.get_json(&query).await?;

        if response.status == "1" {
            Ok(u128::from_str(&response.result).map_err(|_| EtherscanError::EthSupplyFailed)?)
        } else {
            Err(EtherscanError::EthSupplyFailed)
        }
    }

    /// Returns the current amount of Ether in circulation, ETH2 Staking rewards,
    /// EIP1559 burnt fees, and total withdrawn ETH from the beacon chain.
    pub async fn eth_supply2(&self) -> Result<EthSupply2> {
        let query = self.create_query("stats", "ethsupply2", serde_json::Value::Null);
        let response: Response<EthSupply2> = self.get_json(&query).await?;

        Ok(response.result)
    }

    /// Returns the latest price of 1 ETH.
    pub async fn eth_price(&self) -> Result<EthPrice> {
        let query = self.create_query("stats", "ethprice", serde_json::Value::Null);
        let response: Response<EthPrice> = self.get_json(&query).await?;

        Ok(response.result)
    }

    /// Returns the total number of discoverable Ethereum nodes.
    pub async fn node_count(&self) -> Result<NodeCount> {
        let query = self.create_query("stats", "nodecount", serde_json::Value::Null);
        let response: Response<NodeCount> = self.get_json(&query).await?;

        Ok(response.result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_works() {
        // Sample Response from the etherscan documentation
        // https://docs.etherscan.io/api-endpoints/stats-1#get-total-supply-of-ether-2
        let v = r#"{
                    "status":"1",
                    "message":"OK",
                    "result":{
                        "EthSupply":"122373866217800000000000000",
                        "Eth2Staking":"1157529105115885000000000",
                        "BurntFees":"3102505506455601519229842",
                        "WithdrawnTotal":"1170200333006131000000000"
                    }
                }"#;
        let eth_supply2: Response<EthSupply2> = serde_json::from_str(v).unwrap();
        assert_eq!(eth_supply2.message, "OK");
        assert_eq!(eth_supply2.result.eth_supply, 122373866217800000000000000);
        assert_eq!(eth_supply2.result.eth2_staking, 1157529105115885000000000);
        assert_eq!(
            eth_supply2.result.burnt_fees,
            parse_units("3102505506455601519229842", "wei").map(Into::into).unwrap()
        );
        assert_eq!(eth_supply2.result.withdrawn_total, 1170200333006131000000000);

        // Sample Response from the etherscan documentation
        // https://docs.etherscan.io/api-endpoints/stats-1#get-ether-last-price
        let v = r#"{
                    "status":"1",
                    "message":"OK",
                    "result":{
                        "ethbtc":"0.06116",
                        "ethbtc_timestamp":"1624961308",
                        "ethusd":"2149.18",
                        "ethusd_timestamp":"1624961308"
                    }
                }"#;
        let eth_price: Response<EthPrice> = serde_json::from_str(v).unwrap();
        assert_eq!(eth_price.message, "OK");
        assert_eq!(eth_price.result.ethbtc, 0.06116);
        assert_eq!(eth_price.result.ethusd, 2149.18);

        // Sample Response from the etherscan documentation
        // https://docs.etherscan.io/api-endpoints/stats-1#get-total-nodes-count
        let v = r#"{
                    "status":"1",
                    "message":"OK",
                    "result":{
                        "UTCDate":"2021-06-29",
                        "TotalNodeCount":"6413"
                    }
                }"#;
        let node_count: Response<NodeCount> = serde_json::from_str(v).unwrap();
        assert_eq!(node_count.message, "OK");
        assert_eq!(node_count.result.total_node_count, 6413);
    }
}
