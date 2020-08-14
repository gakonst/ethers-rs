mod eth_gas_station;
pub use eth_gas_station::EthGasStation;

mod etherchain;
pub use etherchain::Etherchain;

mod etherscan;
pub use etherscan::Etherscan;

use async_trait::async_trait;
use thiserror::Error;

/// `GasOracle` encapsulates a generic type that implements the `GasOracleFetch` trait.
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
/// let gas_oracle_1 = GasOracle::new(eth_gas_station_oracle);
///
/// let etherscan_oracle = EthGasStation::new(None);
/// let gas_oracle_2 = GasOracle::new(etherscan_oracle);
///
/// let data_1 = gas_oracle_1.fetch().await?;
/// let data_2 = gas_oracle_2.fetch().await?;
/// # Ok(())
/// # }
/// ```
pub struct GasOracle<G: GasOracleFetch>(G);

/// The response from a successful fetch from the `GasOracle`
#[derive(Debug)]
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
    HttpClientError(#[from] Box<dyn std::error::Error>),
}

impl<G: GasOracleFetch> GasOracle<G> {
    /// Initializes a new `GasOracle`
    ///
    /// # Example
    ///
    /// ```
    /// use ethers::providers::{
    ///     gas_oracle::{Etherchain, GasOracle},
    /// };
    ///
    /// let etherchain_oracle = GasOracle::new(Etherchain::new());
    /// ```
    pub fn new(oracle: G) -> Self {
        Self(oracle)
    }

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
    /// let etherchain_oracle = GasOracle::new(Etherchain::new());
    /// let data = etherchain_oracle.fetch().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn fetch(&self) -> Result<GasOracleResponse, GasOracleError> {
        Ok(self.0.fetch().await.map_err(Into::into)?)
    }
}

/// A common trait that an underlying gas oracle needs to implement.
#[async_trait]
pub trait GasOracleFetch {
    type Error: std::error::Error + Into<GasOracleError>;

    async fn fetch(&self) -> Result<GasOracleResponse, Self::Error>;
}
