mod blocknative;
pub use blocknative::BlockNative;

mod eth_gas_station;
pub use eth_gas_station::EthGasStation;

mod etherchain;
pub use etherchain::Etherchain;

mod etherscan;
pub use etherscan::Etherscan;

mod middleware;
pub use middleware::{GasOracleMiddleware, MiddlewareError};

mod median;
pub use median::Median;

mod cache;
pub use cache::Cache;

mod polygon;
pub use polygon::Polygon;

mod gas_now;
pub use gas_now::GasNow;

mod provider_oracle;
pub use provider_oracle::ProviderOracle;

use ethers_core::types::U256;

use async_trait::async_trait;
use auto_impl::auto_impl;
use reqwest::Error as ReqwestError;
use std::error::Error;
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

    /// An error decoding JSON response from gas oracle
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),

    /// An error with oracle response type
    #[error("invalid oracle response")]
    InvalidResponse,

    /// An internal error in the Etherscan client request made from the underlying
    /// gas oracle
    #[error(transparent)]
    EtherscanError(#[from] ethers_etherscan::errors::EtherscanError),

    /// An internal error thrown when the required gas category is not
    /// supported by the gas oracle API
    #[error("gas category not supported")]
    GasCategoryNotSupported,

    #[error("EIP-1559 gas estimation not supported")]
    Eip1559EstimationNotSupported,

    #[error("None of the oracles returned a value")]
    NoValues,

    #[error("Chain is not supported by the oracle")]
    UnsupportedChain,

    /// Error thrown when the provider failed.
    #[error("Chain is not supported by the oracle")]
    ProviderError(#[from] Box<dyn Error + Send + Sync>),
}

/// `GasOracle` is a trait that an underlying gas oracle needs to implement.
///
/// # Example
///
/// ```no_run
/// use ethers_middleware::{
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
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[auto_impl(&, Box, Arc)]
pub trait GasOracle: Send + Sync + std::fmt::Debug {
    /// Makes an asynchronous HTTP query to the underlying `GasOracle`
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_middleware::{
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

    async fn estimate_eip1559_fees(&self) -> Result<(U256, U256), GasOracleError>;
}
