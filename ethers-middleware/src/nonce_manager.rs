use async_trait::async_trait;
use ethers_core::types::*;
use ethers_providers::{FromErr, Middleware};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use thiserror::Error;

#[derive(Debug)]
/// Middleware used for calculating nonces locally, useful for signing multiple
/// consecutive transactions without waiting for them to hit the mempool
pub struct NonceManagerMiddleware<M> {
    pub inner: M,
    pub initialized: AtomicBool,
    pub nonce: AtomicU64,
    pub address: Address,
}

impl<M> NonceManagerMiddleware<M>
where
    M: Middleware,
{
    /// Instantiates the nonce manager with a 0 nonce.
    pub fn new(inner: M, address: Address) -> Self {
        Self {
            initialized: false.into(),
            nonce: 0.into(),
            inner,
            address,
        }
    }

    /// Returns the next nonce to be used
    pub fn next(&self) -> U256 {
        let nonce = self.nonce.fetch_add(1, Ordering::SeqCst);
        nonce.into()
    }

    async fn get_transaction_count_with_manager(
        &self,
        block: Option<BlockNumber>,
    ) -> Result<U256, NonceManagerError<M>> {
        // initialize the nonce the first time the manager is called
        if !self.initialized.load(Ordering::SeqCst) {
            let nonce = self
                .inner
                .get_transaction_count(self.address, block)
                .await
                .map_err(FromErr::from)?;
            self.nonce.store(nonce.as_u64(), Ordering::SeqCst);
            self.initialized.store(true, Ordering::SeqCst);
        }

        Ok(self.next())
    }
}

#[derive(Error, Debug)]
/// Thrown when an error happens at the Nonce Manager
pub enum NonceManagerError<M: Middleware> {
    /// Thrown when the internal middleware errors
    #[error("{0}")]
    MiddlewareError(M::Error),
}

impl<M: Middleware> FromErr<M::Error> for NonceManagerError<M> {
    fn from(src: M::Error) -> Self {
        NonceManagerError::MiddlewareError(src)
    }
}

#[async_trait]
impl<M> Middleware for NonceManagerMiddleware<M>
where
    M: Middleware,
{
    type Error = NonceManagerError<M>;
    type Provider = M::Provider;
    type Inner = M;

    fn inner(&self) -> &M {
        &self.inner
    }

    /// Signs and broadcasts the transaction. The optional parameter `block` can be passed so that
    /// gas cost and nonce calculations take it into account. For simple transactions this can be
    /// left to `None`.
    async fn send_transaction(
        &self,
        mut tx: TransactionRequest,
        block: Option<BlockNumber>,
    ) -> Result<TxHash, Self::Error> {
        if tx.nonce.is_none() {
            tx.nonce = Some(self.get_transaction_count_with_manager(block).await?);
        }

        let mut tx_clone = tx.clone();
        match self.inner.send_transaction(tx, block).await {
            Ok(tx_hash) => Ok(tx_hash),
            Err(err) => {
                let nonce = self.get_transaction_count(self.address, block).await?;
                if nonce != self.nonce.load(Ordering::SeqCst).into() {
                    // try re-submitting the transaction with the correct nonce if there
                    // was a nonce mismatch
                    self.nonce.store(nonce.as_u64(), Ordering::SeqCst);
                    tx_clone.nonce = Some(nonce);
                    self.inner
                        .send_transaction(tx_clone, block)
                        .await
                        .map_err(FromErr::from)
                } else {
                    // propagate the error otherwise
                    Err(FromErr::from(err))
                }
            }
        }
    }
}
