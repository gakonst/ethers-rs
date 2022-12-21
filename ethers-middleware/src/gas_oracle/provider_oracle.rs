use super::{GasOracle, GasOracleError, Result};
use async_trait::async_trait;
use ethers_core::types::U256;
use ethers_providers::Middleware;
use std::fmt::Debug;

/// Gas oracle from a [`Middleware`] implementation such as an
/// Ethereum RPC provider.
#[derive(Clone, Debug)]
#[must_use]
pub struct ProviderOracle<M: Middleware> {
    provider: M,
}

impl<M: Middleware> ProviderOracle<M> {
    pub fn new(provider: M) -> Self {
        Self { provider }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<M: Middleware> GasOracle for ProviderOracle<M>
where
    M::Error: 'static,
{
    async fn fetch(&self) -> Result<U256> {
        self.provider
            .get_gas_price()
            .await
            .map_err(|err| GasOracleError::ProviderError(Box::new(err)))
    }

    async fn estimate_eip1559_fees(&self) -> Result<(U256, U256)> {
        // TODO: Allow configuring different estimation functions.
        self.provider
            .estimate_eip1559_fees(None)
            .await
            .map_err(|err| GasOracleError::ProviderError(Box::new(err)))
    }
}
