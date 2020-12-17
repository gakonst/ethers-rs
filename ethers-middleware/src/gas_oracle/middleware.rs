use super::{GasOracle, GasOracleError};
use async_trait::async_trait;
use ethers_core::types::*;
use ethers_providers::{FromErr, Middleware, PendingTransaction};
use thiserror::Error;

#[derive(Debug)]
/// Middleware used for fetching gas prices over an API instead of `eth_gasPrice`
pub struct GasOracleMiddleware<M, G> {
    inner: M,
    gas_oracle: G,
}

impl<M, G> GasOracleMiddleware<M, G>
where
    M: Middleware,
    G: GasOracle,
{
    pub fn new(inner: M, gas_oracle: G) -> Self {
        Self { inner, gas_oracle }
    }
}

#[derive(Error, Debug)]
pub enum MiddlewareError<M: Middleware> {
    #[error(transparent)]
    GasOracleError(#[from] GasOracleError),

    #[error("{0}")]
    MiddlewareError(M::Error),
}

impl<M: Middleware> FromErr<M::Error> for MiddlewareError<M> {
    fn from(src: M::Error) -> MiddlewareError<M> {
        MiddlewareError::MiddlewareError(src)
    }
}

#[async_trait]
impl<M, G> Middleware for GasOracleMiddleware<M, G>
where
    M: Middleware,
    G: GasOracle,
{
    type Error = MiddlewareError<M>;
    type Provider = M::Provider;
    type Inner = M;

    // OVERRIDEN METHODS

    fn inner(&self) -> &M {
        &self.inner
    }

    async fn get_gas_price(&self) -> Result<U256, Self::Error> {
        Ok(self.gas_oracle.fetch().await?)
    }

    async fn send_transaction(
        &self,
        mut tx: TransactionRequest,
        block: Option<BlockNumber>,
    ) -> Result<PendingTransaction<'_, Self::Provider>, Self::Error> {
        if tx.gas_price.is_none() {
            tx.gas_price = Some(self.get_gas_price().await?);
        }
        self.inner
            .send_transaction(tx, block)
            .await
            .map_err(MiddlewareError::MiddlewareError)
    }
}
