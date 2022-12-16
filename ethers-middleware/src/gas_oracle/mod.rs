pub mod blocknative;
pub use blocknative::BlockNative;

pub mod eth_gas_station;
#[allow(deprecated)]
pub use eth_gas_station::EthGasStation;

pub mod etherchain;
pub use etherchain::Etherchain;

pub mod etherscan;
pub use etherscan::Etherscan;

pub mod middleware;
pub use middleware::{GasOracleMiddleware, MiddlewareError};

pub mod median;
pub use median::Median;

pub mod cache;
pub use cache::Cache;

pub mod polygon;
pub use polygon::Polygon;

pub mod gas_now;
pub use gas_now::GasNow;

pub mod provider_oracle;
pub use provider_oracle::ProviderOracle;

use async_trait::async_trait;
use auto_impl::auto_impl;
use ethers_core::types::U256;
use reqwest::Error as ReqwestError;
use std::{error::Error, fmt::Debug};
use thiserror::Error;

pub(crate) const GWEI_TO_WEI: u64 = 1_000_000_000;
pub(crate) const GWEI_TO_WEI_U256: U256 = U256([0, 0, 0, GWEI_TO_WEI]);

pub type Result<T, E = GasOracleError> = std::result::Result<T, E>;

/// Generic gas price categories received from a [`GasOracle`].
#[derive(Clone, Copy, Default, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum GasCategory {
    SafeLow,
    #[default]
    Standard,
    Fast,
    Fastest,
}

/// Error thrown by a [`GasOracle`].
#[derive(Debug, Error)]
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
    #[error("Provider error: {0}")]
    ProviderError(#[from] Box<dyn Error + Send + Sync>),
}

/// An Ethereum gas price oracle.
///
/// # Example
///
/// ```no_run
/// use ethers_core::types::U256;
/// use ethers_middleware::gas_oracle::{GasCategory, GasNow, GasOracle};
///
/// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
/// let etherscan_oracle = GasNow::default().category(GasCategory::SafeLow);
///
/// let gas_price = etherscan_oracle.fetch().await?;
/// assert!(gas_price > U256::zero());
/// # Ok(())
/// # }
/// ```
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[auto_impl(&, Box, Arc)]
pub trait GasOracle: Send + Sync + Debug {
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
    async fn fetch(&self) -> Result<U256>;

    async fn estimate_eip1559_fees(&self) -> Result<(U256, U256)>;
}

#[inline]
#[doc(hidden)]
pub(crate) fn from_gwei_f64(gwei: f64) -> U256 {
    ethers_core::types::u256_from_f64_saturating(gwei) * GWEI_TO_WEI_U256
}
