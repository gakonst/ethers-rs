#![cfg(not(target_arch = "wasm32"))]

use std::{ops::Deref, sync::Arc};

use async_trait::async_trait;
use ethers_contract::{
    multicall::{
        contract::{
            GetBlockNumberCall, GetBlockNumberReturn, GetEthBalanceCall, GetEthBalanceReturn,
            MULTICALL3_ABI,
        },
        Multicall,
    },
    BaseContract, ContractCall, ContractError, MulticallError,
};
use ethers_core::{
    abi::{encode, Abi, AbiDecode, AbiEncode, Token, Tokenizable},
    types::{
        transaction::eip2718::TypedTransaction, Address, BlockId, Bytes, NameOrAddress,
        TransactionRequest,
    },
};
use ethers_providers::{Middleware, MiddlewareError};
use thiserror::Error;

use tokio::sync::{mpsc, oneshot};

type MulticallResult<M> = Result<Token, Arc<MulticallError<M>>>;
type MulticallRequest<M> = (ContractCall<M, Token>, oneshot::Sender<MulticallResult<M>>);

/// Processor for multicall middleware requests
#[derive(Debug)]
pub struct MulticallProcessor<M: Middleware> {
    inner: Arc<M>,
    multicall_address: Option<Address>,
    max_batch_size: usize,
    rx: mpsc::UnboundedReceiver<MulticallRequest<M>>,
}

/// Middleware used for transparently leveraging multicall functionality
#[derive(Debug, Clone)]
pub struct MulticallMiddleware<M: Middleware> {
    inner: Arc<M>,
    contracts: Arc<Vec<BaseContract>>,
    tx: mpsc::UnboundedSender<MulticallRequest<M>>,
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
            MulticallMiddlewareError::ProcessorNotRunning => None,
        }
    }
}

/// Thrown when an error happens at the Multicall middleware
#[derive(Error, Debug)]
pub enum MulticallMiddlewareError<M: Middleware> {
    /// Thrown when the internal middleware errors
    #[error("{0}")]
    MiddlewareError(M::Error),
    /// Thrown when the internal multicall errors
    #[error(transparent)]
    MulticallError(#[from] Arc<MulticallError<M>>),
    /// Thrown when the processor isn't running
    #[error("Processor is not running")]
    ProcessorNotRunning,
}

impl<M> MulticallProcessor<M>
where
    M: Middleware,
{
    /// Should be run in a separate task to process requests
    pub async fn run(mut self) -> Result<(), MulticallError<M>> {
        let mut multicall: Multicall<M> =
            Multicall::new(self.inner.clone(), self.multicall_address).await?;

        loop {
            let mut requests = Vec::new();

            // wait for the first request
            match self.rx.recv().await {
                Some(request) => requests.push(request),
                None => break,
            }

            // attempt to batch more requests, up to the max batch size
            while requests.len() < self.max_batch_size {
                match self.rx.try_recv() {
                    Ok(request) => requests.push(request),

                    // For both errors (Disconnected and Empty), the correct action
                    // is to process the items.  If the error was Disconnected, on
                    // the next iteration rx.recv().await will be None and we'll
                    // break from the outer loop anyway.
                    Err(_) => break,
                }
            }

            let (calls, callbacks): (Vec<_>, Vec<_>) = requests.into_iter().unzip();

            multicall.clear_calls();
            for mut call in calls.into_iter() {
                // use `to: None` as sentinel for system calls to get block number, etc
                if call.tx.to().is_none() {
                    call.tx.set_to(multicall.contract.address());
                    // do not allow reverts for system calls
                    multicall.add_call(call, false);
                } else {
                    // allow reverts for user calls
                    multicall.add_call(call, true);
                }
            }
            let results = multicall.call_raw().await;

            let responses = match results {
                Ok(results) => results
                    .into_iter()
                    .map(|result| {
                        result.map_err(|e| {
                            Arc::new(MulticallError::ContractError(ContractError::Revert(e)))
                        })
                    })
                    .collect(),
                Err(e) => vec![Err(Arc::new(e)); callbacks.len()],
            };

            for (callback, response) in callbacks.into_iter().zip(responses) {
                // ignore errors, the receiver may have dropped
                let _ = callback.send(response);
            }
        }

        Ok(())
    }
}

impl<M> MulticallMiddleware<M>
where
    M: Middleware,
{
    /// Instantiates the multicall middleware to recognize the given `match_abis`
    /// # Panics
    /// Panics if `max_batch_size` is less than 2
    pub fn new(
        inner: M,
        match_abis: Vec<Abi>,
        max_batch_size: usize,
        multicall_address: Option<Address>,
    ) -> (Self, MulticallProcessor<M>) {
        if max_batch_size < 2 {
            panic!("batches must be at least 2 calls to justify the overhead of multicall");
        }

        let (tx, rx) = mpsc::unbounded_channel();
        let client = Arc::new(inner);

        let contracts = Arc::new(
            match_abis
                .iter()
                .map(|abi| abi.to_owned().into())
                .chain(vec![MULTICALL3_ABI.to_owned().into()])
                .collect(),
        );

        (
            Self { inner: client.clone(), tx, contracts },
            MulticallProcessor { inner: client, rx, multicall_address, max_batch_size },
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

    async fn batch_call(
        &self,
        call: ContractCall<M, Token>,
    ) -> Result<Bytes, MulticallMiddlewareError<M>> {
        let (tx, rx) = oneshot::channel();

        if self.tx.send((call, tx)).is_err() {
            return Err(MulticallMiddlewareError::ProcessorNotRunning);
        };

        match rx.await {
            Err(_) => Err(MulticallMiddlewareError::ProcessorNotRunning),
            Ok(response) => response
                .map(|token| encode(&[token]).into())
                .map_err(MulticallMiddlewareError::MulticallError),
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
            return self.batch_call(call).await;
        }

        return self.inner.call(tx, block).await.map_err(MulticallMiddlewareError::from_err);
    }

    async fn get_block_number(&self) -> Result<ethers_core::types::U64, Self::Error> {
        let data = (GetBlockNumberCall {}).encode();
        let tx = TypedTransaction::Legacy(TransactionRequest::new().data(data.clone()));
        self.call(&tx, None)
            .await
            .map(|b| GetBlockNumberReturn::decode(b.deref()).unwrap().block_number.as_u64().into())
    }

    async fn get_balance<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        address: T,
        block: Option<BlockId>,
    ) -> Result<ethers_core::types::U256, Self::Error> {
        let address_or_name = address.into();
        if address_or_name.as_name().is_some() {
            return self
                .inner
                .get_balance(address_or_name, block)
                .await
                .map_err(MulticallMiddlewareError::from_err);
        }

        let address = *address_or_name.as_address().unwrap();
        let data = (GetEthBalanceCall { addr: address }).encode();
        let tx = TypedTransaction::Legacy(TransactionRequest::new().data(data.clone()));
        self.call(&tx, block).await.map(|b| GetEthBalanceReturn::decode(b.deref()).unwrap().balance)
    }

    // TODO: implement more middleware functions?
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethers_contract::abigen;
    use ethers_providers::{MockProvider, Provider};

    abigen!(Test, r#"[read(string) view returns (bytes4)]"#);

    #[tokio::test]
    #[should_panic(
        expected = "batches must be at least 2 calls to justify the overhead of multicall"
    )]
    async fn needs_min_batch() {
        // will panic if batch size is less than 2
        let _ = MulticallMiddleware::new(Provider::new(MockProvider::new()), vec![], 1, None);
    }

    #[tokio::test]
    async fn needs_processor() {
        let (provider, _) =
            MulticallMiddleware::new(Provider::new(MockProvider::new()), vec![], 2, None);
        let e = provider.get_block_number().await.unwrap_err();
        assert!(matches!(e, MulticallMiddlewareError::ProcessorNotRunning));
    }

    #[tokio::test]
    async fn matches_multicall_signatures() {
        let (provider1, mock1) = Provider::mocked();
        let (provider2, mock2) = Provider::mocked();
        let mock_multicall = Address::random();

        let (provider, processor) =
            MulticallMiddleware::new(provider1.clone(), vec![], 2, Some(mock_multicall));

        let mut multicall = Multicall::new(provider2.clone(), Some(mock_multicall)).await.unwrap();

        tokio::spawn(async move {
            let _ = processor.run().await;
        });

        let address = Address::zero();

        let _ = tokio::join!(provider.get_block_number(), provider.get_balance(address, None));

        let _ =
            multicall.add_get_block_number().add_get_eth_balance(address, false).call_raw().await;

        assert!(mock1.requests_match(&mock2));
    }

    #[tokio::test]
    async fn uses_batch_size() {
        let (provider1, mock1) = Provider::mocked();
        let (provider2, mock2) = Provider::mocked();
        let mock_multicall = Address::random();

        let (provider, processor) =
            MulticallMiddleware::new(provider1.clone(), vec![], 2, Some(mock_multicall));

        let mut multicall = Multicall::new(provider2.clone(), Some(mock_multicall)).await.unwrap();

        tokio::spawn(async move {
            let _ = processor.run().await;
        });

        let _ = tokio::join!(
            provider.get_block_number(),
            provider.get_block_number(),
            provider.get_block_number()
        );

        let _ = multicall.add_get_block_number().add_get_block_number().call_raw().await;
        multicall.clear_calls();

        let _ = multicall.add_get_block_number().call_raw().await;

        assert!(mock1.requests_match(&mock2));
    }

    #[tokio::test]
    async fn matches_provided_signatures() {
        let (provider1, mock1) = Provider::mocked();
        let (provider2, mock2) = Provider::mocked();
        let mock_multicall = Address::random();

        let (provider, processor) = MulticallMiddleware::new(
            provider1.clone(),
            vec![TEST_ABI.clone()],
            2,
            Some(mock_multicall),
        );

        let mut multicall = Multicall::new(provider2.clone(), Some(mock_multicall)).await.unwrap();

        let mock_test = Address::random();
        let test1 = Test::new(mock_test, Arc::new(provider.clone()));
        let test2 = Test::new(mock_test, Arc::new(provider2.clone()));

        tokio::spawn(async move {
            let _ = processor.run().await;
        });

        let call1 = test1.read("call1".to_string());
        let call2 = test1.read("call2".to_string());

        let _ = tokio::join!(call1.call(), call2.call());

        let _ = multicall
            .add_call(test2.read("call1".to_string()), true)
            .add_call(test2.read("call2".to_string()), true)
            .call_raw()
            .await;

        assert!(mock1.requests_match(&mock2));
    }
}
