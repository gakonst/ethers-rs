use super::{ProxyWallet, ProxyWalletError};
use async_trait::async_trait;
use ethers_core::types::*;
use ethers_providers::{FromErr, Middleware, PendingTransaction};
use thiserror::Error;

#[derive(Debug)]
pub struct ProxyWalletMiddleware<M, P> {
    inner: M,
    proxy: P,
}

impl<M, P> ProxyWalletMiddleware<M, P>
where
    M: Middleware,
    P: ProxyWallet,
{
    /// Creates a new ProxyWalletMiddleware that intercepts transactions, modifying them to be sent
    /// through the ProxyWallet.
    pub fn new(inner: M, proxy: P) -> Self {
        Self { inner, proxy }
    }
}

#[derive(Error, Debug)]
pub enum ProxyWalletMiddlewareError<M: Middleware> {
    #[error(transparent)]
    ProxyWalletError(#[from] ProxyWalletError),

    #[error("{0}")]
    MiddlewareError(M::Error),
}

impl<M: Middleware> FromErr<M::Error> for ProxyWalletMiddlewareError<M> {
    fn from(src: M::Error) -> ProxyWalletMiddlewareError<M> {
        ProxyWalletMiddlewareError::MiddlewareError(src)
    }
}

#[async_trait]
impl<M, P> Middleware for ProxyWalletMiddleware<M, P>
where
    M: Middleware,
    P: ProxyWallet,
{
    type Error = ProxyWalletMiddlewareError<M>;
    type Provider = M::Provider;
    type Inner = M;

    fn inner(&self) -> &M {
        &self.inner
    }

    async fn send_transaction(
        &self,
        mut tx: TransactionRequest,
        block: Option<BlockNumber>,
    ) -> Result<PendingTransaction<'_, Self::Provider>, Self::Error> {
        // resolve the to field if that's an ENS name.
        if let Some(ref to) = tx.to {
            if let NameOrAddress::Name(ens_name) = to {
                let addr = self
                    .inner
                    .resolve_name(&ens_name)
                    .await
                    .map_err(ProxyWalletMiddlewareError::MiddlewareError)?;
                tx.to = Some(addr.into())
            }
        }

        // construct the appropriate proxy tx.
        let proxy_tx = self.proxy.get_proxy_tx(tx)?;

        // send the proxy tx.
        self.inner
            .send_transaction(proxy_tx, block)
            .await
            .map_err(ProxyWalletMiddlewareError::MiddlewareError)
    }
}
