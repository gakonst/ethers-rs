use ethers_core::{
    abi::{Detokenize, Error as AbiError, Function, InvalidOutputType},
    types::{Address, BlockNumber, TransactionRequest, H256, U256},
};
use ethers_providers::{networks::Network, JsonRpcClient};
use ethers_signers::{Client, Signer};

use std::{fmt::Debug, marker::PhantomData};

use thiserror::Error as ThisError;

pub struct ContractCall<'a, P, N, S, D> {
    pub(crate) tx: TransactionRequest,
    pub(crate) function: Function,
    pub(crate) client: &'a Client<'a, P, N, S>,
    pub(crate) block: Option<BlockNumber>,
    pub(crate) datatype: PhantomData<D>,
}

impl<'a, P, N, S, D: Detokenize> ContractCall<'a, S, P, N, D> {
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
}

#[derive(ThisError, Debug)]
// TODO: Can we get rid of this static?
pub enum ContractError<P: JsonRpcClient>
where
    P::Error: 'static,
{
    #[error(transparent)]
    DecodingError(#[from] AbiError),
    #[error(transparent)]
    DetokenizationError(#[from] InvalidOutputType),
    #[error(transparent)]
    CallError(P::Error),
    #[error("constructor is not defined in the ABI")]
    ConstructorError,
    #[error("Contract was not deployed")]
    ContractNotDeployed,
}

impl<'a, P, N, S, D> ContractCall<'a, P, N, S, D>
where
    S: Signer,
    P: JsonRpcClient,
    P::Error: 'static,
    N: Network,
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
    pub async fn call(self) -> Result<D, ContractError<P>> {
        let bytes = self
            .client
            .call(self.tx, self.block)
            .await
            .map_err(ContractError::CallError)?;

        let tokens = self.function.decode_output(&bytes.0)?;

        let data = D::from_tokens(tokens)?;

        Ok(data)
    }

    /// Signs and broadcasts the provided transaction
    pub async fn send(self) -> Result<H256, P::Error> {
        self.client.send_transaction(self.tx, self.block).await
    }
}
