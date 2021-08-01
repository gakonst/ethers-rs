use super::{Transformer, TransformerError};
use async_trait::async_trait;
use ethers_core::types::{transaction::eip2718::TypedTransaction, *};
use ethers_providers::{FromErr, Middleware, PendingTransaction};
use thiserror::Error;

#[derive(Debug)]
/// Middleware used for intercepting transaction requests and transforming them to be executed by
/// the underneath `Transformer` instance.
pub struct TransformerMiddleware<M, T> {
    inner: M,
    transformer: T,
}

impl<M, T> TransformerMiddleware<M, T>
where
    M: Middleware,
    T: Transformer,
{
    /// Creates a new TransformerMiddleware that intercepts transactions, modifying them to be sent
    /// through the Transformer.
    pub fn new(inner: M, transformer: T) -> Self {
        Self { inner, transformer }
    }
}

#[derive(Error, Debug)]
pub enum TransformerMiddlewareError<M: Middleware> {
    #[error(transparent)]
    TransformerError(#[from] TransformerError),

    #[error("{0}")]
    MiddlewareError(M::Error),
}

impl<M: Middleware> FromErr<M::Error> for TransformerMiddlewareError<M> {
    fn from(src: M::Error) -> TransformerMiddlewareError<M> {
        TransformerMiddlewareError::MiddlewareError(src)
    }
}

#[async_trait]
impl<M, T> Middleware for TransformerMiddleware<M, T>
where
    M: Middleware,
    T: Transformer,
{
    type Error = TransformerMiddlewareError<M>;
    type Provider = M::Provider;
    type Inner = M;

    fn inner(&self) -> &M {
        &self.inner
    }

    async fn send_transaction(
        &self,
        mut tx: TypedTransaction,
        block: Option<BlockId>,
    ) -> Result<PendingTransaction<'_, Self::Provider>, Self::Error> {
        self.fill_transaction(&mut tx, block).await?;

        // construct the appropriate proxy tx.
        self.transformer.transform(&mut tx)?;

        // send the proxy tx.
        self.inner
            .send_transaction(tx, block)
            .await
            .map_err(TransformerMiddlewareError::MiddlewareError)
    }
}
