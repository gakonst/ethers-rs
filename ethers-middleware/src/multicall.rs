use std::{cmp::Ordering, sync::Arc, time::Instant};

use async_trait::async_trait;
use ethers_contract::{
    multicall::Multicall, BaseContract, ContractCall, ContractError,
    MulticallError,
};
use ethers_core::{
    abi::{encode, Token, Tokenizable},
    types::{transaction::eip2718::TypedTransaction, Address, BlockId},
};
use ethers_providers::{Middleware, MiddlewareError};
use instant::Duration;
use thiserror::Error;

use tokio::sync::oneshot;
use tokio::{sync::mpsc, time::sleep};

type MulticallResult<M> = Result<Token, MulticallError<M>>;
type MulticallRequest<M> = (ContractCall<M, Token>, oneshot::Sender<MulticallResult<M>>);

#[derive(Debug)]
/// Middleware used for transparently leveraging multicall functionality
pub struct MulticallProcessor<M: Middleware> {
    inner: Arc<M>,
    multicall_address: Option<Address>,
    frequency: Duration,
    rx: mpsc::UnboundedReceiver<MulticallRequest<M>>,
}

#[derive(Debug, Clone)]
pub struct MulticallMiddleware<M: Middleware> {
    inner: Arc<M>,
    contracts: Vec<BaseContract>,
    tx: mpsc::UnboundedSender<MulticallRequest<M>>,
}

#[derive(Error, Debug)]
/// Thrown when an error happens at the Multicall middleware
pub enum MulticallMiddlewareError<M: Middleware> {
    /// Thrown when the internal middleware errors
    #[error("{0}")]
    MiddlewareError(M::Error),
    /// Thrown when the internal multicall errors
    #[error(transparent)]
    MulticallError(#[from] MulticallError<M>),
    /// Thrown when a revert reason is decoded from the contract
    #[error("{0}")]
    RevertReason(String),
}

impl<M: Middleware> MiddlewareError for MulticallMiddlewareError<M> {
    type Inner = M::Error;

    fn from_err(src: M::Error) -> Self {
        MulticallMiddlewareError::MiddlewareError(src)
    }

    fn as_inner(&self) -> Option<&Self::Inner> {
        match self {
            MulticallMiddlewareError::MiddlewareError(e) => Some(e),
            MulticallMiddlewareError::MulticallError(e) => e.as_middleware_error(),
            MulticallMiddlewareError::RevertReason(_) => None,
        }
    }
}

impl<M> MulticallProcessor<M>
where
    M: Middleware,
{
    pub async fn run(mut self) -> Result<(), MulticallMiddlewareError<M>> {
        let mut multicall = Multicall::new(self.inner, self.multicall_address).await?;
        let mut callbacks = Vec::new();
        let mut checkpoint = Instant::now();

        loop {
            let maybe_request = self.rx.try_recv();
            match maybe_request {
                Err(mpsc::error::TryRecvError::Empty) => {
                    sleep(self.frequency).await;
                    continue;
                }
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    panic!("multicall channel disconnected");
                }
                Ok((call, callback)) => {
                    multicall.add_call(call, true);
                    callbacks.push(callback);
                }
            }

            // check if batch is non-empty and frequency has elapsed since last batch was sent
            if callbacks.len() > 0 && checkpoint.elapsed().cmp(&self.frequency) == Ordering::Greater
            {
                let results = multicall.call_raw().await?;
                multicall.clear_calls();

                for (result, callback) in results.into_iter().zip(callbacks.drain(..)) {
                    let response =
                        result.map_err(|e| MulticallError::ContractError(ContractError::Revert(e)));
                    if let Err(e) = callback.send(response) {
                        panic!("oneshot channel closed: {:?}", e);
                    }
                }

                checkpoint = Instant::now();
            }
        }
    }
}

impl<M> MulticallMiddleware<M>
where
    M: Middleware,
{
    /// Instantiates the multicall middleware to recognize the given `contracts` selectors
    /// and batch calls in a single inner call every `frequency` interval
    pub fn new(
        inner: M,
        contracts: Vec<BaseContract>,
        frequency: Duration,
        multicall_address: Option<Address>,
    ) -> (Self, MulticallProcessor<M>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let client = Arc::new(inner);

        (
            Self { inner: client.clone(), tx, contracts },
            MulticallProcessor { inner: client, rx, frequency, multicall_address },
        )
    }

    fn call_from_tx<D: Tokenizable>(
        &self,
        tx: &TypedTransaction,
        block: Option<BlockId>,
    ) -> Option<ContractCall<M, D>> {
        if let Some(data) = tx.data() {
            for contract in self.contracts.iter() {
                if let Ok(function) = contract.get_fn_from_input(data) {
                    return Some(ContractCall::new(
                        tx.clone(),
                        function.clone(),
                        self.inner.clone(),
                        block,
                    ));
                }
            }
        }
        None
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
    ) -> Result<ethers_core::types::Bytes, Self::Error> {
        if let Some(call) = self.call_from_tx(tx, block) {
            let (tx, rx) = oneshot::channel();

            if let Err(e) = self.tx.send((call, tx)) {
                panic!("multicall channel disconnected: {:?}", e);
            };

            match rx.await {
                Err(e) => panic!("multicall channel disconnected: {:?}", e),
                Ok(response) => {
                    return response.map(|token| encode(&[token]).into()).map_err(|e| {
                        if let Some(reason) = e.decode_revert::<String>() {
                            MulticallMiddlewareError::RevertReason(reason)
                        } else {
                            MulticallMiddlewareError::MulticallError(e)
                        }
                    });
                }
            }
        }

        return self.inner.call(tx, block).await.map_err(MulticallMiddlewareError::from_err);
    }

    // TODO: support other Multicall methods (blocknumber, balance, etc)
}
