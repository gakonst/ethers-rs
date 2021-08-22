use ethers_core::types::{transaction::eip2718::TypedTransaction, BlockId};
use ethers_providers::{FromErr, Middleware, PendingTransaction};

use async_trait::async_trait;
use std::fmt::Debug;
use thiserror::Error;

#[async_trait]
pub trait Policy: Sync + Send + Debug {
    type Error: Sync + Send + Debug;

    async fn ensure_can_send(&self, tx: TypedTransaction) -> Result<TypedTransaction, Self::Error>;
}

/// A policy that does not restrict anything.
#[derive(Debug, Clone, Copy)]
pub struct AllowEverything;

#[async_trait]
impl Policy for AllowEverything {
    type Error = ();

    async fn ensure_can_send(&self, tx: TypedTransaction) -> Result<TypedTransaction, Self::Error> {
        Ok(tx)
    }
}

/// A policy that rejects all transactions.
#[derive(Debug, Clone, Copy)]
pub struct RejectAll;

#[async_trait]
impl Policy for RejectAll {
    type Error = ();

    async fn ensure_can_send(&self, _: TypedTransaction) -> Result<TypedTransaction, Self::Error> {
        Err(())
    }
}

#[derive(Clone, Debug)]
pub struct PolicyMiddleware<M, P> {
    pub(crate) inner: M,
    pub(crate) policy: P,
}

impl<M: Middleware, P: Policy> FromErr<M::Error> for PolicyMiddlewareError<M, P> {
    fn from(src: M::Error) -> PolicyMiddlewareError<M, P> {
        PolicyMiddlewareError::MiddlewareError(src)
    }
}

#[derive(Error, Debug)]
/// Error thrown when the client interacts with the blockchain
pub enum PolicyMiddlewareError<M: Middleware, P: Policy> {
    /// Thrown when the internal policy errors
    #[error("{0:?}")]
    PolicyError(P::Error),
    /// Thrown when an internal middleware errors
    #[error(transparent)]
    MiddlewareError(M::Error),
}

#[async_trait]
impl<M, P> Middleware for PolicyMiddleware<M, P>
where
    M: Middleware,
    P: Policy,
{
    type Error = PolicyMiddlewareError<M, P>;
    type Provider = M::Provider;
    type Inner = M;

    fn inner(&self) -> &M {
        &self.inner
    }

    async fn send_transaction<T: Into<TypedTransaction> + Send + Sync>(
        &self,
        tx: T,
        block: Option<BlockId>,
    ) -> Result<PendingTransaction<'_, Self::Provider>, Self::Error> {
        let tx = self
            .policy
            .ensure_can_send(tx.into())
            .await
            .map_err(PolicyMiddlewareError::PolicyError)?;
        self.inner
            .send_transaction(tx, block)
            .await
            .map_err(PolicyMiddlewareError::MiddlewareError)
    }
}
