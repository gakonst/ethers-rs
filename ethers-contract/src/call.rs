use super::base::{decode_function_data, AbiError};
use ethers_core::{
    abi::{AbiDecode, AbiEncode, Detokenize, Function, InvalidOutputType, Tokenizable},
    types::{
        transaction::eip2718::TypedTransaction, Address, BlockId, Bytes, Selector,
        TransactionRequest, U256,
    },
    utils::id,
};
use ethers_providers::{Middleware, PendingTransaction, ProviderError};

use std::{borrow::Cow, fmt::Debug, marker::PhantomData, sync::Arc};

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

#[derive(Debug, Clone)]
#[must_use = "contract calls do nothing unless you `send` or `call` them"]
/// Helper for managing a transaction before submitting it to a node
pub struct ContractCall<M, D> {
    /// The raw transaction object
    pub tx: TypedTransaction,
    /// The ABI of the function being called
    pub function: Function,
    /// Optional block number to be used when calculating the transaction's gas and nonce
    pub block: Option<BlockId>,
    pub(crate) client: Arc<M>,
    pub(crate) datatype: PhantomData<D>,
}

impl<M, D: Detokenize> ContractCall<M, D> {
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

impl<M, D> ContractCall<M, D>
where
    M: Middleware,
    D: Detokenize,
{
    /// Returns the underlying transaction's ABI encoded data
    pub fn calldata(&self) -> Option<Bytes> {
        self.tx.data().cloned()
    }

    /// Returns the estimated gas cost for the underlying transaction to be executed
    pub async fn estimate_gas(&self) -> Result<U256, ContractError<M>> {
        self.client.estimate_gas(&self.tx).await.map_err(ContractError::MiddlewareError)
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
        let bytes =
            self.client.call(&self.tx, self.block).await.map_err(ContractError::MiddlewareError)?;

        // decode output
        let data = decode_function_data(&self.function, &bytes, false)?;

        Ok(data)
    }

    /// Signs and broadcasts the provided transaction
    pub async fn send(&self) -> Result<PendingTransaction<'_, M::Provider>, ContractError<M>> {
        self.client
            .send_transaction(self.tx.clone(), self.block)
            .await
            .map_err(ContractError::MiddlewareError)
    }
}
