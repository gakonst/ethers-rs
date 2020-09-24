mod eth_gas_station;
pub use eth_gas_station::EthGasStation;

mod etherchain;
pub use etherchain::Etherchain;

mod etherscan;
pub use etherscan::Etherscan;

mod gas_now;
pub use gas_now::GasNow;

mod middleware;
pub use middleware::{GasOracleMiddleware, MiddlewareError};

use ethers_core::types::U256;

use async_trait::async_trait;
use reqwest::Error as ReqwestError;
use thiserror::Error;

const GWEI_TO_WEI: u64 = 1000000000;

/// Various gas price categories. Choose one of the available
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GasCategory {
    SafeLow,
    Standard,
    Fast,
    Fastest,
}

#[derive(Error, Debug)]
/// Error thrown when fetching data from the `GasOracle`
pub enum GasOracleError {
    /// An internal error in the HTTP request made from the underlying
    /// gas oracle
    #[error(transparent)]
    HttpClientError(#[from] ReqwestError),

    /// An internal error thrown when the required gas category is not
    /// supported by the gas oracle API
    #[error("gas category not supported")]
    GasCategoryNotSupported,
}

/// `GasOracle` is a trait that an underlying gas oracle needs to implement.
///
/// # Example
///
/// ```no_run
/// use ethers::middleware::{
///     gas_oracle::{EthGasStation, Etherscan, GasCategory, GasOracle},
/// };
///
/// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
/// let eth_gas_station_oracle = EthGasStation::new(Some("my-api-key"));
/// let etherscan_oracle = EthGasStation::new(None).category(GasCategory::SafeLow);
///
/// let data_1 = eth_gas_station_oracle.fetch().await?;
/// let data_2 = etherscan_oracle.fetch().await?;
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait GasOracle: Send + Sync + std::fmt::Debug {
    /// Makes an asynchronous HTTP query to the underlying `GasOracle`
    ///
    /// # Example
    ///
    /// ```
    /// use ethers::middleware::{
    ///     gas_oracle::{Etherchain, GasCategory, GasOracle},
    /// };
    ///
    /// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
    /// let etherchain_oracle = Etherchain::new().category(GasCategory::Fastest);
    /// let data = etherchain_oracle.fetch().await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn fetch(&self) -> Result<U256, GasOracleError>;
}
