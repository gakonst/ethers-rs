use ethers_core::types::{transaction::eip2718::TypedTransaction, BlockId};
use ethers_providers::{Middleware, MiddlewareError, PendingTransaction};

use async_trait::async_trait;
use std::fmt::Debug;
use thiserror::Error;

/// Basic trait to ensure that transactions about to be sent follow certain rules.
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait Policy: Sync + Send + Debug {
    type Error: Sync + Send + Debug;

    /// Evaluates the transactions.
    ///
    /// Returns Ok with the `tx` or an Err otherwise.
    async fn ensure_can_send(&self, tx: TypedTransaction) -> Result<TypedTransaction, Self::Error>;
}

/// A policy that does not restrict anything.
#[derive(Debug, Clone, Copy)]
pub struct AllowEverything;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Policy for AllowEverything {
    type Error = ();

    async fn ensure_can_send(&self, tx: TypedTransaction) -> Result<TypedTransaction, Self::Error> {
        Ok(tx)
    }
}

/// A policy that rejects all transactions.
#[derive(Debug, Clone, Copy)]
pub struct RejectEverything;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Policy for RejectEverything {
    type Error = ();

    async fn ensure_can_send(&self, _: TypedTransaction) -> Result<TypedTransaction, Self::Error> {
        Err(())
    }
}

/// Middleware used to enforce certain policies for transactions.
#[derive(Clone, Debug)]
pub struct PolicyMiddleware<M, P> {
    pub(crate) inner: M,
    pub(crate) policy: P,
}

impl<M, P> PolicyMiddleware<M, P>
where
    M: Middleware,
    P: Policy,
{
    /// Creates a new client from the provider and policy.
    pub fn new(inner: M, policy: P) -> Self {
        Self { inner, policy }
    }
}

#[derive(Error, Debug)]
/// Error thrown when the client interacts with the policy middleware.
pub enum PolicyMiddlewareError<M: Middleware, P: Policy> {
    /// Thrown when the internal policy errors
    #[error("{0:?}")]
    PolicyError(P::Error),
    /// Thrown when an internal middleware errors
    #[error(transparent)]
    MiddlewareError(M::Error),
}

impl<M: Middleware, P: Policy> MiddlewareError for PolicyMiddlewareError<M, P> {
    type Inner = M::Error;

    fn from_err(src: M::Error) -> Self {
        PolicyMiddlewareError::MiddlewareError(src)
    }

    fn as_inner(&self) -> Option<&Self::Inner> {
        match self {
            PolicyMiddlewareError::MiddlewareError(e) => Some(e),
            _ => None,
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
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

    /// This ensures the tx complies with the registered policy.
    /// If so then this simply delegates the transaction to the inner middleware
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
        self.inner.send_transaction(tx, block).await.map_err(PolicyMiddlewareError::MiddlewareError)
    }
}
