use crate::{Contract, ContractError};

use ethers_core::{
    abi::{Abi, Token, Tokenize},
    types::{
        transaction::eip2718::TypedTransaction, Address, BlockNumber, Bytes, NameOrAddress,
        TransactionReceipt, TransactionRequest, U256, U64,
    },
};
use ethers_providers::{
    call_raw::{CallBuilder, RawCall},
    Middleware,
};

#[cfg(not(feature = "legacy"))]
use ethers_core::types::Eip1559TransactionRequest;

use std::{marker::PhantomData, sync::Arc};

/// Helper which manages the deployment transaction of a smart contract.
///
/// This is just a wrapper type for [Deployer] with an additional type to convert the [Contract]
/// that the deployer returns when sending the transaction.
#[derive(Debug)]
#[must_use = "Deployer does nothing unless you `send` it"]
pub struct ContractDeployer<M, C> {
    /// the actual deployer, exposed for overriding the defaults
    pub deployer: Deployer<M>,
    /// marker for the `Contract` type to create afterwards
    ///
    /// this type will be used to construct it via `From::from(Contract)`
    _contract: PhantomData<C>,
}

impl<M, C> Clone for ContractDeployer<M, C> {
    fn clone(&self) -> Self {
        ContractDeployer { deployer: self.deployer.clone(), _contract: self._contract }
    }
}

impl<M: Middleware, C: From<Contract<M>>> ContractDeployer<M, C> {
    /// Create a new instance of this [ContractDeployer]
    pub fn new(deployer: Deployer<M>) -> Self {
        Self { deployer, _contract: Default::default() }
    }

    /// Sets the number of confirmations to wait for the contract deployment transaction
    pub fn confirmations<T: Into<usize>>(mut self, confirmations: T) -> Self {
        self.deployer.confs = confirmations.into();
        self
    }

    pub fn block<T: Into<BlockNumber>>(mut self, block: T) -> Self {
        self.deployer.block = block.into();
        self
    }

    /// Uses a Legacy transaction instead of an EIP-1559 one to do the deployment
    pub fn legacy(mut self) -> Self {
        self.deployer = self.deployer.legacy();
        self
    }

    /// Sets the `from` field in the deploy transaction to the provided value
    pub fn from<T: Into<Address>>(mut self, from: T) -> Self {
        self.deployer.tx.set_from(from.into());
        self
    }

    /// Sets the `to` field in the deploy transaction to the provided value
    pub fn to<T: Into<NameOrAddress>>(mut self, to: T) -> Self {
        self.deployer.tx.set_to(to.into());
        self
    }

    /// Sets the `gas` field in the deploy transaction to the provided value
    pub fn gas<T: Into<U256>>(mut self, gas: T) -> Self {
        self.deployer.tx.set_gas(gas.into());
        self
    }

    /// Sets the `gas_price` field in the deploy transaction to the provided value
    pub fn gas_price<T: Into<U256>>(mut self, gas_price: T) -> Self {
        self.deployer.tx.set_gas_price(gas_price.into());
        self
    }

    /// Sets the `value` field in the deploy transaction to the provided value
    pub fn value<T: Into<U256>>(mut self, value: T) -> Self {
        self.deployer.tx.set_value(value.into());
        self
    }

    /// Sets the `data` field in the deploy transaction to the provided value
    pub fn data<T: Into<Bytes>>(mut self, data: T) -> Self {
        self.deployer.tx.set_data(data.into());
        self
    }

    /// Sets the `nonce` field in the deploy transaction to the provided value
    pub fn nonce<T: Into<U256>>(mut self, nonce: T) -> Self {
        self.deployer.tx.set_nonce(nonce.into());
        self
    }

    /// Sets the `chain_id` field in the deploy transaction to the provided value
    pub fn chain_id<T: Into<U64>>(mut self, chain_id: T) -> Self {
        self.deployer.tx.set_chain_id(chain_id.into());
        self
    }

    /// Dry runs the deployment of the contract
    ///
    /// Note: this function _does not_ send a transaction from your account
    pub async fn call(&self) -> Result<(), ContractError<M>> {
        self.deployer.call().await
    }

    /// Returns a CallBuilder, which when awaited executes the deployment of this contract via
    /// `eth_call`. This call resolves to the returned data which would have been stored at the
    /// destination address had the deploy transaction been executed via `send()`.
    ///
    /// Note: this function _does not_ send a transaction from your account
    pub fn call_raw(&self) -> CallBuilder<'_, M::Provider> {
        self.deployer.call_raw()
    }

    /// Broadcasts the contract deployment transaction and after waiting for it to
    /// be sufficiently confirmed (default: 1), it returns a new instance of the contract type at
    /// the deployed contract's address.
    pub async fn send(self) -> Result<C, ContractError<M>> {
        let contract = self.deployer.send().await?;
        Ok(C::from(contract))
    }

    /// Broadcasts the contract deployment transaction and after waiting for it to
    /// be sufficiently confirmed (default: 1), it returns a new instance of the contract type at
    /// the deployed contract's address and the corresponding
    /// [`TransactionReceipt`](ethers_core::types::TransactionReceipt).
    pub async fn send_with_receipt(self) -> Result<(C, TransactionReceipt), ContractError<M>> {
        let (contract, receipt) = self.deployer.send_with_receipt().await?;
        Ok((C::from(contract), receipt))
    }

    /// Returns a reference to the deployer's ABI
    pub fn abi(&self) -> &Abi {
        self.deployer.abi()
    }

    /// Returns a pointer to the deployer's client
    pub fn client(&self) -> Arc<M> {
        self.deployer.client()
    }
}

/// Helper which manages the deployment transaction of a smart contract
#[derive(Debug)]
#[must_use = "Deployer does nothing unless you `send` it"]
pub struct Deployer<M> {
    /// The deployer's transaction, exposed for overriding the defaults
    pub tx: TypedTransaction,
    abi: Abi,
    client: Arc<M>,
    confs: usize,
    block: BlockNumber,
}

impl<M> Clone for Deployer<M> {
    fn clone(&self) -> Self {
        Deployer {
            tx: self.tx.clone(),
            abi: self.abi.clone(),
            client: self.client.clone(),
            confs: self.confs,
            block: self.block,
        }
    }
}

impl<M: Middleware> Deployer<M> {
    /// Sets the number of confirmations to wait for the contract deployment transaction
    pub fn confirmations<T: Into<usize>>(mut self, confirmations: T) -> Self {
        self.confs = confirmations.into();
        self
    }

    pub fn block<T: Into<BlockNumber>>(mut self, block: T) -> Self {
        self.block = block.into();
        self
    }

    /// Uses a Legacy transaction instead of an EIP-1559 one to do the deployment
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

    /// Dry runs the deployment of the contract
    ///
    /// Note: this function _does not_ send a transaction from your account
    pub async fn call(&self) -> Result<(), ContractError<M>> {
        self.client
            .call(&self.tx, Some(self.block.into()))
            .await
            .map_err(ContractError::MiddlewareError)?;

        // TODO: It would be nice to handle reverts in a structured way.
        Ok(())
    }

    /// Returns a CallBuilder, which when awaited executes the deployment of this contract via
    /// `eth_call`. This call resolves to the returned data which would have been stored at the
    /// destination address had the deploy transaction been executed via `send()`.
    ///
    /// Note: this function _does not_ send a transaction from your account
    pub fn call_raw(&self) -> CallBuilder<'_, M::Provider> {
        self.client.provider().call_raw(&self.tx).block(self.block.into())
    }

    /// Broadcasts the contract deployment transaction and after waiting for it to
    /// be sufficiently confirmed (default: 1), it returns a [`Contract`](crate::Contract)
    /// struct at the deployed contract's address.
    pub async fn send(self) -> Result<Contract<M>, ContractError<M>> {
        let (contract, _) = self.send_with_receipt().await?;
        Ok(contract)
    }

    /// Broadcasts the contract deployment transaction and after waiting for it to
    /// be sufficiently confirmed (default: 1), it returns a tuple with
    /// the [`Contract`](crate::Contract) struct at the deployed contract's address
    /// and the corresponding [`TransactionReceipt`](ethers_core::types::TransactionReceipt).
    pub async fn send_with_receipt(
        self,
    ) -> Result<(Contract<M>, TransactionReceipt), ContractError<M>> {
        let pending_tx = self
            .client
            .send_transaction(self.tx, Some(self.block.into()))
            .await
            .map_err(ContractError::MiddlewareError)?;

        // TODO: Should this be calculated "optimistically" by address/nonce?
        let receipt = pending_tx
            .confirmations(self.confs)
            .await
            .map_err(|_| ContractError::ContractNotDeployed)?
            .ok_or(ContractError::ContractNotDeployed)?;
        let address = receipt.contract_address.ok_or(ContractError::ContractNotDeployed)?;

        let contract = Contract::new(address, self.abi.clone(), self.client);
        Ok((contract, receipt))
    }

    /// Returns a reference to the deployer's ABI
    pub fn abi(&self) -> &Abi {
        &self.abi
    }

    /// Returns a pointer to the deployer's client
    pub fn client(&self) -> Arc<M> {
        self.client.clone()
    }
}

/// To deploy a contract to the Ethereum network, a `ContractFactory` can be
/// created which manages the Contract bytecode and Application Binary Interface
/// (ABI), usually generated from the Solidity compiler.
///
/// Once the factory's deployment transaction is mined with sufficient confirmations,
/// the [`Contract`](crate::Contract) object is returned.
///
/// # Example
///
/// ```no_run
/// use ethers_solc::Solc;
/// use ethers_contract::ContractFactory;
/// use ethers_providers::{Provider, Http};
/// use ethers_signers::Wallet;
/// use std::convert::TryFrom;
///
/// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
/// // first we'll compile the contract (you can alternatively compile it yourself
/// // and pass the ABI/Bytecode
/// let compiled = Solc::default().compile_source("./tests/contract.sol").unwrap();
/// let contract = compiled
///     .get("./tests/contract.sol", "SimpleStorage")
///     .expect("could not find contract");
///
/// // connect to the network
/// let client = Provider::<Http>::try_from("http://localhost:8545").unwrap();
/// let client = std::sync::Arc::new(client);
///
/// // create a factory which will be used to deploy instances of the contract
/// let factory = ContractFactory::new(contract.abi.unwrap().clone(), contract.bytecode().unwrap().clone(), client);
///
/// // The deployer created by the `deploy` call exposes a builder which gets consumed
/// // by the async `send` call
/// let contract = factory
///     .deploy("initial value".to_string())?
///     .confirmations(0usize)
///     .send()
///     .await?;
/// println!("{}", contract.address());
/// # Ok(())
/// # }
#[derive(Debug)]
pub struct ContractFactory<M> {
    client: Arc<M>,
    abi: Abi,
    bytecode: Bytes,
}

impl<M> Clone for ContractFactory<M> {
    fn clone(&self) -> Self {
        ContractFactory {
            client: self.client.clone(),
            abi: self.abi.clone(),
            bytecode: self.bytecode.clone(),
        }
    }
}

impl<M: Middleware> ContractFactory<M> {
    /// Creates a factory for deployment of the Contract with bytecode, and the
    /// constructor defined in the abi. The client will be used to send any deployment
    /// transaction.
    pub fn new(abi: Abi, bytecode: Bytes, client: Arc<M>) -> Self {
        Self { client, abi, bytecode }
    }

    pub fn deploy_tokens(self, params: Vec<Token>) -> Result<Deployer<M>, ContractError<M>> {
        // Encode the constructor args & concatenate with the bytecode if necessary
        let data: Bytes = match (self.abi.constructor(), params.is_empty()) {
            (None, false) => return Err(ContractError::ConstructorError),
            (None, true) => self.bytecode.clone(),
            (Some(constructor), _) => {
                constructor.encode_input(self.bytecode.to_vec(), &params)?.into()
            }
        };

        // create the tx object. Since we're deploying a contract, `to` is `None`
        // We default to EIP-1559 transactions, but the sender can convert it back
        // to a legacy one
        #[cfg(feature = "legacy")]
        let tx = TransactionRequest { to: None, data: Some(data), ..Default::default() };
        #[cfg(not(feature = "legacy"))]
        let tx = Eip1559TransactionRequest { to: None, data: Some(data), ..Default::default() };
        let tx = tx.into();

        Ok(Deployer {
            client: Arc::clone(&self.client), // cheap clone behind the arc
            abi: self.abi,
            tx,
            confs: 1,
            block: BlockNumber::Latest,
        })
    }

    /// Constructs the deployment transaction based on the provided constructor
    /// arguments and returns a `Deployer` instance. You must call `send()` in order
    /// to actually deploy the contract.
    ///
    /// Notes:
    /// 1. If there are no constructor arguments, you should pass `()` as the argument.
    /// 1. The default poll duration is 7 seconds.
    /// 1. The default number of confirmations is 1 block.
    pub fn deploy<T: Tokenize>(self, constructor_args: T) -> Result<Deployer<M>, ContractError<M>> {
        self.deploy_tokens(constructor_args.into_tokens())
    }
}
