use std::{cmp::Ordering, ops::Deref, sync::Arc, time::Instant};

use async_trait::async_trait;
use ethers_contract::{
    multicall::{
        contract::{
            GetBlockNumberCall, GetBlockNumberReturn, GetEthBalanceCall, GetEthBalanceReturn,
            MULTICALL3_ABI,
        },
        Multicall,
    },
    BaseContract, ContractCall, ContractError, EthCall, MulticallError,
};
use ethers_core::{
    abi::{encode, Abi, AbiDecode, Token, Tokenizable},
    types::{
        transaction::eip2718::TypedTransaction, Address, BlockId, Bytes, NameOrAddress,
        TransactionRequest,
    },
};
use ethers_providers::{Middleware, MiddlewareError};
use instant::Duration;
use thiserror::Error;

use tokio::{
    sync::{mpsc, oneshot},
    time::sleep,
};

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
    multicall: BaseContract,
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
                Err(mpsc::error::TryRecvError::Empty) => {}
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    return Ok(());
                }
                Ok((mut call, callback)) => {
                    // use to: None as sentinel for system calls to get block number, etc
                    if call.tx.to().is_none() {
                        call.tx.set_to(multicall.contract.address());
                        multicall.add_call(call, false);
                    } else {
                        multicall.add_call(call, true);
                    }

                    callbacks.push(callback);

                    // keep filling up the batch until channel is empty
                    continue;
                }
            }

            // check if batch is non-empty and frequency has elapsed since last batch was sent
            if callbacks.len() > 0 && checkpoint.elapsed().cmp(&self.frequency) == Ordering::Greater
            {
                checkpoint = Instant::now();

                let results = multicall.call_raw().await?;
                multicall.clear_calls();

                for (result, callback) in results.into_iter().zip(callbacks.drain(..)) {
                    let response =
                        result.map_err(|e| MulticallError::ContractError(ContractError::Revert(e)));

                    // ignore send errors, as the receiver may have dropped
                    let _ = callback.send(response);
                }
            }
        }
    }
}

impl<M> MulticallMiddleware<M>
where
    M: Middleware,
{
    /// Instantiates the multicall middleware to recognize the given `match_abis`
    /// and batch calls in a single inner call every `frequency` interval
    pub fn new(
        inner: M,
        match_abis: Vec<Abi>,
        frequency: Duration,
        multicall_address: Option<Address>,
    ) -> (Self, MulticallProcessor<M>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let client = Arc::new(inner);

        let contracts = match_abis.iter().map(|abi| abi.clone().into()).collect();

        let multicall: BaseContract = MULTICALL3_ABI.clone().into();

        (
            Self { inner: client.clone(), tx, contracts, multicall },
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

    async fn batch_call(&self, call: ContractCall<M, Token>) -> Result<Bytes, MulticallError<M>> {
        let (tx, rx) = oneshot::channel();

        if let Err(e) = self.tx.send((call, tx)) {
            panic!("multicall processor disconnected: {:?}", e);
        };

        match rx.await {
            Err(e) => panic!("multicall processor disconnected: {:?}", e),
            Ok(response) => response.map(|token| encode(&[token]).into()),
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
    ) -> Result<ethers_core::types::Bytes, Self::Error> {
        if let Some(call) = self.call_from_tx(tx, block) {
            return self.batch_call(call).await.map_err(|e| {
                if let Some(reason) = e.decode_revert::<String>() {
                    MulticallMiddlewareError::RevertReason(reason)
                } else {
                    MulticallMiddlewareError::MulticallError(e)
                }
            });
        }

        return self.inner.call(tx, block).await.map_err(MulticallMiddlewareError::from_err);
    }

    async fn get_block_number(&self) -> Result<ethers_core::types::U64, Self::Error> {
        let get_block_fn =
            self.multicall.get_fn_from_selector(GetBlockNumberCall::selector()).unwrap();
        let data = get_block_fn.encode_input(&vec![]).unwrap();
        let call =
            ContractCall::new(
                TypedTransaction::Legacy(TransactionRequest::new().data(data)),
                get_block_fn.to_owned(),
                self.inner.clone(),
                None,
            );
        return self
            .batch_call(call)
            .await
            .map(|b| GetBlockNumberReturn::decode(b.deref()).unwrap().block_number.as_u64().into())
            .map_err(MulticallMiddlewareError::MulticallError);
    }

    async fn get_balance<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        address_or_name: T,
        block: Option<BlockId>,
    ) -> Result<ethers_core::types::U256, Self::Error> {
        let address = *address_or_name.into().as_address().unwrap();
        let get_balance_fn =
            self.multicall.get_fn_from_selector(GetEthBalanceCall::selector()).unwrap();
        let data = get_balance_fn.encode_input(&vec![Token::Address(address)]).unwrap();
        let call = ContractCall::new(
            TypedTransaction::Legacy(TransactionRequest::new().data(data)),
            get_balance_fn.to_owned(),
            self.inner.clone(),
            block,
        );
        return self
            .batch_call(call)
            .await
            .map(|b| GetEthBalanceReturn::decode(b.deref()).unwrap().balance)
            .map_err(MulticallMiddlewareError::MulticallError);
    }

    // TODO: implement more middleware functions?
}
