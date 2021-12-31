use async_trait::async_trait;

use ethers_core::types::U256;
use ethers_etherscan::Client;

use crate::gas_oracle::{GasCategory, GasOracle, GasOracleError, GWEI_TO_WEI};

/// A client over HTTP for the [Etherscan](https://api.etherscan.io/api?module=gastracker&action=gasoracle) gas tracker API
/// that implements the `GasOracle` trait
#[derive(Clone, Debug)]
pub struct Etherscan {
    client: Client,
    gas_category: GasCategory,
}

impl Etherscan {
    /// Creates a new [Etherscan](https://etherscan.io/gastracker) gas price oracle.
    pub fn new(client: Client) -> Self {
        Etherscan { client, gas_category: GasCategory::Standard }
    }

    /// Sets the gas price category to be used when fetching the gas price.
    #[must_use]
    pub fn category(mut self, gas_category: GasCategory) -> Self {
        self.gas_category = gas_category;
        self
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl GasOracle for Etherscan {
    async fn fetch(&self) -> Result<U256, GasOracleError> {
        if matches!(self.gas_category, GasCategory::Fastest) {
            return Err(GasOracleError::GasCategoryNotSupported)
        }

        let result = self.client.gas_oracle().await?;

        match self.gas_category {
            GasCategory::SafeLow => Ok(U256::from(result.safe_gas_price * GWEI_TO_WEI)),
            GasCategory::Standard => Ok(U256::from(result.propose_gas_price * GWEI_TO_WEI)),
            GasCategory::Fast => Ok(U256::from(result.fast_gas_price * GWEI_TO_WEI)),
            _ => Err(GasOracleError::GasCategoryNotSupported),
        }
    }

    async fn estimate_eip1559_fees(&self) -> Result<(U256, U256), GasOracleError> {
        Err(GasOracleError::Eip1559EstimationNotSupported)
    }
}
