use std::{cmp::Ordering, sync::Arc, time::Instant};

use async_trait::async_trait;
use ethers_contract::{multicall::Multicall, BaseContract, ContractCall, MulticallError};
use ethers_core::{
    abi::{Bytes, Token, Tokenizable},
    types::{transaction::eip2718::TypedTransaction, BlockId},
};
use ethers_providers::{Middleware, MiddlewareError};
use instant::Duration;
use thiserror::Error;

use tokio::sync::mpsc;
use tokio::sync::oneshot;

type MulticallResult = Result<Token, Bytes>;
type MulticallRequest<M: Middleware> = (ContractCall<M, Bytes>, oneshot::Sender<MulticallResult>);

#[derive(Debug)]
/// Middleware used for transparently leveraging multicall functionality
pub struct MulticallMiddleware<M: Middleware> {
    inner: Arc<M>,
    contracts: Vec<BaseContract>,
    multicall: Multicall<M>,
    callbacks: Vec<oneshot::Sender<MulticallResult>>,
    rx: mpsc::UnboundedReceiver<MulticallRequest<M>>,
    tx: mpsc::UnboundedSender<MulticallRequest<M>>,
    checkpoint: instant::Instant,
    frequency: Duration,
}

impl<M> MulticallMiddleware<M>
where
    M: Middleware,
{
    /// Instantiates the nonce manager with a 0 nonce. The `address` should be the
    /// address which you'll be sending transactions from
    /// TODO: support multiple contract ABIs // 4byte DB
    pub async fn new(
        inner: M,
        contracts: Vec<BaseContract>,
        batch_frequency: Duration,
    ) -> Result<Self, MulticallError<M>> {
        // TODO: support custom multicall address
        let multicall = Multicall::new(inner, None).await?;
        let callbacks = Vec::new();

        let (tx, rx) = mpsc::unbounded_channel();

        let timestamp = Instant::now();

        Ok(Self {
            inner,
            multicall,
            callbacks,
            contracts,
            tx,
            rx,
            checkpoint: timestamp,
            frequency: batch_frequency,
        })
    }

    pub async fn run(&mut self) -> Result<(), MulticallMiddlewareError<M>> {
        loop {
            let maybe_request = self.rx.try_recv();
            match maybe_request {
                Ok((call, callback)) => {
                    self.multicall.add_call(call, true);
                    self.callbacks.push(callback);
                    // keep filling batch until channel is empty (or closed)
                    continue;
                }
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    // TODO: exit?
                }
                Err(mpsc::error::TryRecvError::Empty) => {
                    // TODO: consider sleeping here?
                }
            }

            // check if batch is non-empty and frequency has elapsed since last batch was sent
            if self.callbacks.len() > 0
                && self.checkpoint.elapsed().cmp(&self.frequency) == Ordering::Greater
            {
                let maybe_results = self.multicall.call_raw().await;
                match maybe_results {
                    Ok(results) => {
                        self.multicall.clear_calls();

                        for (result, callback) in results.into_iter().zip(self.callbacks.drain(..))
                        {
                            callback.send(result);
                        }

                        self.checkpoint = Instant::now();
                    }
                    Err(MulticallError::ContractError(ce)) => {
                        // TODO: bubble up to callback?
                    }
                    Err(MulticallError::InvalidChainId(id)) => {
                        // TODO: exit?
                    }
                    Err(MulticallError::IllegalRevert) => {
                        // TODO: idk
                    }
                }
            }
        }
    }

    fn call_from_tx<D: Tokenizable>(
        &self,
        tx: &TypedTransaction,
        block: Option<BlockId>,
    ) -> Option<ContractCall<M, D>> {
        if let Some(data) = tx.data() {
            for contract in self.contracts.iter() {
                if let Ok(function) = contract.get_fn_from_input(data) {
                    return Some(ContractCall::new(*tx, *function, self.inner, block));
                }
            }
        }

        None
    }
}

#[derive(Error, Debug)]
/// Thrown when an error happens at the Multicall middleware
pub enum MulticallMiddlewareError<M: Middleware> {
    /// Thrown when the internal middleware errors
    #[error("{0}")]
    MiddlewareError(M::Error),
}

impl<M: Middleware> MiddlewareError for MulticallMiddlewareError<M> {
    type Inner = M::Error;

    fn from_err(src: M::Error) -> Self {
        MulticallMiddlewareError::MiddlewareError(src)
    }

    fn as_inner(&self) -> Option<&Self::Inner> {
        match self {
            MulticallMiddlewareError::MiddlewareError(e) => Some(e),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<M> Middleware for MulticallMiddleware<M>
where
    M: Middleware,
{
    type Error = MulticallMiddlewareError<M>;
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

            self.tx.send((call, tx))?;

            return rx.await;
        }

        return self.inner.call(tx, block).await;
    }
}
