use super::{GasCategory, GasOracle, GasOracleError, Result};
use async_trait::async_trait;
use ethers_core::types::U256;
use ethers_etherscan::Client;
use std::ops::{Deref, DerefMut};

/// A client over HTTP for the [Etherscan](https://api.etherscan.io/api?module=gastracker&action=gasoracle) gas tracker API
/// that implements the `GasOracle` trait
#[derive(Clone, Debug)]
#[must_use]
pub struct Etherscan {
    client: Client,
    gas_category: GasCategory,
}

impl Deref for Etherscan {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl DerefMut for Etherscan {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl GasOracle for Etherscan {
    async fn fetch(&self) -> Result<U256> {
        // handle unsupported gas categories before making the request
        match self.gas_category {
            GasCategory::SafeLow | GasCategory::Standard | GasCategory::Fast => {}
            GasCategory::Fastest => return Err(GasOracleError::GasCategoryNotSupported),
        }

        let result = self.query().await?;
        let gas_price = match self.gas_category {
            GasCategory::SafeLow => result.safe_gas_price,
            GasCategory::Standard => result.propose_gas_price,
            GasCategory::Fast => result.fast_gas_price,
            _ => unreachable!(),
        };
        Ok(gas_price)
    }

    async fn estimate_eip1559_fees(&self) -> Result<(U256, U256)> {
        Err(GasOracleError::Eip1559EstimationNotSupported)
    }
}

impl Etherscan {
    /// Creates a new [Etherscan](https://etherscan.io/gastracker) gas price oracle.
    pub fn new(client: Client) -> Self {
        Etherscan { client, gas_category: GasCategory::Standard }
    }

    /// Sets the gas price category to be used when fetching the gas price.
    pub fn category(mut self, gas_category: GasCategory) -> Self {
        self.gas_category = gas_category;
        self
    }

    /// Perform a request to the gas price API and deserialize the response.
    pub async fn query(&self) -> Result<ethers_etherscan::gas::GasOracle> {
        Ok(self.client.gas_oracle().await?)
    }
}
