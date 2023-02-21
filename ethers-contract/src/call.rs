#![allow(clippy::return_self_not_must_use)]

use super::base::{decode_function_data, AbiError};
use ethers_core::{
    abi::{AbiDecode, AbiEncode, Detokenize, Function, InvalidOutputType, Tokenizable},
    types::{
        transaction::eip2718::TypedTransaction, Address, BlockId, Bytes, Selector,
        TransactionRequest, U256,
    },
    utils::id,
};
use ethers_providers::{
    call_raw::{CallBuilder, RawCall},
    Middleware, PendingTransaction, ProviderError,
};

use std::{
    borrow::{Borrow, Cow},
    fmt::Debug,
    future::{Future, IntoFuture},
    marker::PhantomData,
    pin::Pin,
};

use thiserror::Error as ThisError;

/// A helper trait for types that represent all call input parameters of a specific function
pub trait EthCall: Tokenizable + AbiDecode + AbiEncode + Send + Sync {
    /// The name of the function
    fn function_name() -> Cow<'static, str>;

    /// Retrieves the ABI signature for the call
    fn abi_signature() -> Cow<'static, str>;

    /// The selector of the function
    fn selector() -> Selector {
        id(Self::abi_signature())
    }
}

#[derive(ThisError, Debug)]
/// An Error which is thrown when interacting with a smart contract
pub enum ContractError<M: Middleware> {
    /// Thrown when the ABI decoding fails
    #[error(transparent)]
    DecodingError(#[from] ethers_core::abi::Error),

    /// Thrown when the internal BaseContract errors
    #[error(transparent)]
    AbiError(#[from] AbiError),

    /// Thrown when detokenizing an argument
    #[error(transparent)]
    DetokenizationError(#[from] InvalidOutputType),

    /// Thrown when a middleware call fails
    #[error("{0}")]
    MiddlewareError(M::Error),

    /// Thrown when a provider call fails
    #[error("{0}")]
    ProviderError(ProviderError),

    /// Thrown during deployment if a constructor argument was passed in the `deploy`
    /// call but a constructor was not present in the ABI
    #[error("constructor is not defined in the ABI")]
    ConstructorError,

    /// Thrown if a contract address is not found in the deployment transaction's
    /// receipt
    #[error("Contract was not deployed")]
    ContractNotDeployed,
}

/// `ContractCall` is a [`FunctionCall`] object with an [`std::sync::Arc`] middleware.
/// This type alias exists to preserve backwards compatibility with
/// less-abstract Contracts.
///
/// For full usage docs, see [`FunctionCall`].
pub type ContractCall<M, D> = FunctionCall<std::sync::Arc<M>, M, D>;

#[derive(Debug)]
#[must_use = "contract calls do nothing unless you `send` or `call` them"]
/// Helper for managing a transaction before submitting it to a node
pub struct FunctionCall<B, M, D> {
    /// The raw transaction object
    pub tx: TypedTransaction,
    /// The ABI of the function being called
    pub function: Function,
    /// Optional block number to be used when calculating the transaction's gas and nonce
    pub block: Option<BlockId>,
    pub(crate) client: B,
    pub(crate) datatype: PhantomData<D>,
    pub(crate) _m: PhantomData<M>,
}

impl<B, M, D> Clone for FunctionCall<B, M, D>
where
    B: Clone,
{
    fn clone(&self) -> Self {
        FunctionCall {
            tx: self.tx.clone(),
            function: self.function.clone(),
            block: self.block,
            client: self.client.clone(),
            datatype: self.datatype,
            _m: self._m,
        }
    }
}

impl<B, M, D> FunctionCall<B, M, D>
where
    B: Borrow<M>,
    D: Detokenize,
{
    /// Sets the `from` field in the transaction to the provided value
    pub fn from<T: Into<Address>>(mut self, from: T) -> Self {
        self.tx.set_from(from.into());
        self
    }

    /// Uses a Legacy transaction instead of an EIP-1559 one to execute the call
    pub fn legacy(mut self) -> Self {
        self.tx = match self.tx {
            TypedTransaction::Eip1559(inner) => {
                let tx: TransactionRequest = inner.into();
                TypedTransaction::Legacy(tx)
            }
            other => other,
        };
        self
    }

    /// Sets the `gas` field in the transaction to the provided value
    pub fn gas<T: Into<U256>>(mut self, gas: T) -> Self {
        self.tx.set_gas(gas);
        self
    }

    /// Sets the `gas_price` field in the transaction to the provided value
    /// If the internal transaction is an EIP-1559 one, then it sets both
    /// `max_fee_per_gas` and `max_priority_fee_per_gas` to the same value
    pub fn gas_price<T: Into<U256>>(mut self, gas_price: T) -> Self {
        self.tx.set_gas_price(gas_price);
        self
    }

    /// Sets the `value` field in the transaction to the provided value
    pub fn value<T: Into<U256>>(mut self, value: T) -> Self {
        self.tx.set_value(value);
        self
    }

    /// Sets the `block` field for sending the tx to the chain
    pub fn block<T: Into<BlockId>>(mut self, block: T) -> Self {
        self.block = Some(block.into());
        self
    }
}

impl<B, M, D> FunctionCall<B, M, D>
where
    B: Borrow<M>,
    M: Middleware,
    D: Detokenize,
{
    /// Returns the underlying transaction's ABI encoded data
    pub fn calldata(&self) -> Option<Bytes> {
        self.tx.data().cloned()
    }

    /// Returns the estimated gas cost for the underlying transaction to be executed
    pub async fn estimate_gas(&self) -> Result<U256, ContractError<M>> {
        self.client
            .borrow()
            .estimate_gas(&self.tx, self.block)
            .await
            .map_err(ContractError::MiddlewareError)
    }

    /// Queries the blockchain via an `eth_call` for the provided transaction.
    ///
    /// If executed on a non-state mutating smart contract function (i.e. `view`, `pure`)
    /// then it will return the raw data from the chain.
    ///
    /// If executed on a mutating smart contract function, it will do a "dry run" of the call
    /// and return the return type of the transaction without mutating the state
    ///
    /// Note: this function _does not_ send a transaction from your account
    pub async fn call(&self) -> Result<D, ContractError<M>> {
        let client: &M = self.client.borrow();
        let bytes =
            client.call(&self.tx, self.block).await.map_err(ContractError::MiddlewareError)?;

        // decode output
        let data = decode_function_data(&self.function, &bytes, false)?;

        Ok(data)
    }

    /// Returns an implementer of [`RawCall`] which can be `.await`d to query the blockchain via
    /// `eth_call`, returning the deoded return data.
    ///
    /// The returned call can also be used to override the input parameters to `eth_call`.
    ///
    /// Note: this function _does not_ send a transaction from your account
    pub fn call_raw(
        &self,
    ) -> impl RawCall<'_> + Future<Output = Result<D, ContractError<M>>> + Debug {
        let call = self.call_raw_bytes();
        call.map(move |res: Result<Bytes, ProviderError>| {
            let bytes = res.map_err(ContractError::ProviderError)?;
            decode_function_data(&self.function, &bytes, false).map_err(From::from)
        })
    }

    /// Returns a [`CallBuilder`] which can be `.await`d to query the blochcain via `eth_call`,
    /// returning the raw bytes from the transaction.
    ///
    /// The returned call can also be used to override the input parameters to `eth_call`.
    ///
    /// Note: this function _does not_ send a transaction from your account
    pub fn call_raw_bytes(&self) -> CallBuilder<'_, M::Provider> {
        let call = self.client.borrow().provider().call_raw(&self.tx);
        if let Some(block) = self.block {
            call.block(block)
        } else {
            call
        }
    }

    /// Signs and broadcasts the provided transaction
    pub async fn send(&self) -> Result<PendingTransaction<'_, M::Provider>, ContractError<M>> {
        self.client
            .borrow()
            .send_transaction(self.tx.clone(), self.block)
            .await
            .map_err(ContractError::MiddlewareError)
    }
}

/// [`FunctionCall`] can be turned into [`Future`] automatically with `.await`.
/// Defaults to calling [`FunctionCall::call`].
impl<B, M, D> IntoFuture for FunctionCall<B, M, D>
where
    Self: 'static,
    B: Borrow<M> + Send + Sync,
    M: Middleware,
    D: Detokenize + Send + Sync,
{
    type Output = Result<D, ContractError<M>>;

    #[cfg(target_arch = "wasm32")]
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output>>>;

    #[cfg(not(target_arch = "wasm32"))]
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move { self.call().await })
    }
}
