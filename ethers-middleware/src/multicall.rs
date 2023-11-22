use std::sync::Arc;

use async_trait::async_trait;
use ethers_contract::{multicall::Multicall, BaseContract, ContractCall, FunctionCall};
use ethers_core::{
    abi::{Bytes, Tokenizable},
    types::{transaction::eip2718::TypedTransaction, BlockId},
};
use ethers_providers::{Middleware, MiddlewareError};
use instant::Duration;
use thiserror::Error;

use tokio::sync::mpsc;
use tokio::sync::oneshot;

type MulticallTx<M: Middleware> = (ContractCall<M, Bytes>, oneshot::Sender<Result<Bytes, MulticallError<M>>>);

/// Middleware used for transparently leveraging multicall functionality
pub struct MulticallMiddleware<M: Middleware> {
    inner: Arc<M>,
    contract: BaseContract,
    multicall: Multicall<M>,
    rx: mpsc::UnboundedReceiver<MulticallTx<M>>,
    tx: mpsc::UnboundedSender<MulticallTx<M>>,
    checkpoint: instant::Instant,
    frequency: Duration
}

impl<M> MulticallMiddleware<M>
where
    M: Middleware,
{
    /// Instantiates the nonce manager with a 0 nonce. The `address` should be the
    /// address which you'll be sending transactions from
    pub async fn new(inner: M, contract: BaseContract, frequency: Duration) -> Result<Self, MulticallError<M>> {
        // TODO: support custom multicall address
        let multicall = Multicall::new(inner, None).await?;

        let (tx, rx) = mpsc::unbounded_channel();

        let timestamp = instant::now();

        Ok(Self { inner: Arc::new(inner), multicall, contract, tx, rx, checkpoint: timestamp, frequency })
    }

    pub async fn run(&mut self) {
        while let Some((call, callback)) = self.rx.recv().await {
            self.multicall.add_call(call, false);

            let timestamp = instant::now();
            if timestamp.duration_since(self.checkpoint) > self.frequency {
                self.checkpoint = timestamp;

                let results = self.multicall.call_raw().await?;
                self.multicall.clear_calls();

                callback.send(results.pop());
            }
        }
    }

    fn call_from_tx<D: Tokenizable>(&self, tx: &TypedTransaction, block: Option<BlockId>) -> Option<ContractCall<M, D>> {
        if let Some(data) = tx.data() {
            if let Ok(function) = self.contract.get_fn_from_input(data) {
                return Some(FunctionCall::new(*tx, *function, self.inner, block));
            }
        }

        None
    }
}

#[derive(Error, Debug)]
/// Thrown when an error happens at the Multicall middleware
pub enum MulticallError<M: Middleware> {
    /// Thrown when the internal middleware errors
    #[error("{0}")]
    MiddlewareError(M::Error),
}

impl<M: Middleware> MiddlewareError for MulticallError<M> {
    type Inner = M::Error;

    fn from_err(src: M::Error) -> Self {
        MulticallError::MiddlewareError(src)
    }

    fn as_inner(&self) -> Option<&Self::Inner> {
        match self {
            MulticallError::MiddlewareError(e) => Some(e),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<M> Middleware for MulticallMiddleware<M>
where
    M: Middleware,
{
    type Error = MulticallError<M>;
    type Provider = M::Provider;
    type Inner = M;

    fn inner(&self) -> &M {
        &self.inner
    }

    async fn call(
        &self,
        tx: &TypedTransaction,
        block: Option<BlockId>,
    ) -> Result<Bytes, Self::Error> {
        if let Some(call) = self.call_from_tx(tx, block) {
            let (tx, rx) = oneshot::channel();

            self.tx.send((call, tx));

            return rx.await;
        }

        return self.inner.call(tx, block).await;
    }
}
