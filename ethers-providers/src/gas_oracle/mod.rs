mod eth_gas_station;
pub use eth_gas_station::EthGasStation;

mod etherchain;
pub use etherchain::Etherchain;

mod etherscan;
pub use etherscan::Etherscan;

use async_trait::async_trait;
use reqwest::Error as ReqwestError;
use thiserror::Error;

/// The response from a successful fetch from the `GasOracle`
#[derive(Clone, Debug)]
pub struct GasOracleResponse {
    pub block: Option<u64>,
    pub safe_low: Option<u64>,
    pub standard: Option<u64>,
    pub fast: Option<u64>,
    pub fastest: Option<u64>,
}

#[derive(Error, Debug)]
/// Error thrown when fetching data from the `GasOracle`
pub enum GasOracleError {
    /// An internal error in the HTTP request made from the underlying
    /// gas oracle
    #[error(transparent)]
    // HttpClientError(#[from] Box<dyn std::error::Error>),
    HttpClientError(#[from] ReqwestError),
}

/// `GasOracle` is a trait that an underlying gas oracle needs to implement.
///
/// # Example
///
/// ```no_run
/// use ethers::providers::{
///     gas_oracle::{EthGasStation, Etherscan, GasOracle},
/// };
///
/// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
/// let eth_gas_station_oracle = EthGasStation::new(Some("my-api-key"));
/// let etherscan_oracle = EthGasStation::new(None);
///
/// let data_1 = eth_gas_station_oracle.fetch().await?;
/// let data_2 = etherscan_oracle.fetch().await?;
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait GasOracle: std::fmt::Debug {
    /// Makes an asynchronous HTTP query to the underlying `GasOracle`
    ///
    /// # Example
    ///
    /// ```
    /// use ethers::providers::{
    ///     gas_oracle::{Etherchain, GasOracle},
    /// };
    ///
    /// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
    /// let etherchain_oracle = Etherchain::new();
    /// let data = etherchain_oracle.fetch().await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn fetch(&self) -> Result<GasOracleResponse, GasOracleError>;
}
