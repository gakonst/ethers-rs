use ethers_core::{
    abi::{Detokenize, Error as AbiError, Function, InvalidOutputType},
    types::{Address, BlockNumber, TransactionRequest, TxHash, U256},
};
use ethers_providers::{JsonRpcClient, ProviderError};
use ethers_signers::{Client, ClientError, Signer};

use std::{fmt::Debug, marker::PhantomData, sync::Arc};

use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
/// An Error which is thrown when interacting with a smart contract
pub enum ContractError {
    /// Thrown when the ABI decoding fails
    #[error(transparent)]
    DecodingError(#[from] AbiError),

    /// Thrown when detokenizing an argument
    #[error(transparent)]
    DetokenizationError(#[from] InvalidOutputType),

    /// Thrown when a client call fails
    #[error(transparent)]
    ClientError(#[from] ClientError),

    /// Thrown when a provider call fails
    #[error(transparent)]
    ProviderError(#[from] ProviderError),

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
pub struct ContractCall<P, S, D> {
    /// The raw transaction object
    pub tx: TransactionRequest,
    /// The ABI of the function being called
    pub function: Function,
    /// Optional block number to be used when calculating the transaction's gas and nonce
    pub block: Option<BlockNumber>,
    pub(crate) client: Arc<Client<P, S>>,
    pub(crate) datatype: PhantomData<D>,
}

impl<P, S, D: Detokenize> ContractCall<P, S, D> {
    /// Sets the `from` field in the transaction to the provided value
    pub fn from<T: Into<Address>>(mut self, from: T) -> Self {
        self.tx.from = Some(from.into());
        self
    }

    /// Sets the `gas` field in the transaction to the provided value
    pub fn gas<T: Into<U256>>(mut self, gas: T) -> Self {
        self.tx.gas = Some(gas.into());
        self
    }

    /// Sets the `gas_price` field in the transaction to the provided value
    pub fn gas_price<T: Into<U256>>(mut self, gas_price: T) -> Self {
        self.tx.gas_price = Some(gas_price.into());
        self
    }

    /// Sets the `value` field in the transaction to the provided value
    pub fn value<T: Into<U256>>(mut self, value: T) -> Self {
        self.tx.value = Some(value.into());
        self
    }

    /// Sets the `block` field for sending the tx to the chain
    pub fn block<T: Into<BlockNumber>>(mut self, block: T) -> Self {
        self.block = Some(block.into());
        self
    }
}

impl<P, S, D> ContractCall<P, S, D>
where
    S: Signer,
    P: JsonRpcClient,
    D: Detokenize,
{
    /// Queries the blockchain via an `eth_call` for the provided transaction.
    ///
    /// If executed on a non-state mutating smart contract function (i.e. `view`, `pure`)
    /// then it will return the raw data from the chain.
    ///
    /// If executed on a mutating smart contract function, it will do a "dry run" of the call
    /// and return the return type of the transaction without mutating the state
    ///
    /// Note: this function _does not_ send a transaction from your account
    pub async fn call(&self) -> Result<D, ContractError> {
        let bytes = self.client.call(&self.tx, self.block).await?;

        let tokens = self.function.decode_output(&bytes.0)?;

        let data = D::from_tokens(tokens)?;

        Ok(data)
    }

    /// Signs and broadcasts the provided transaction
    pub async fn send(self) -> Result<TxHash, ContractError> {
        Ok(self.client.send_transaction(self.tx, self.block).await?)
    }
}
