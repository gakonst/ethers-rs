use crate::{Contract, ContractError};

use ethers_core::{
    abi::{Abi, Tokenize},
    types::{Bytes, TransactionRequest},
};
use ethers_providers::JsonRpcClient;
use ethers_signers::{Client, Signer};

#[derive(Debug, Clone)]
/// Helper which manages the deployment transaction of a smart contract
pub struct Deployer<'a, P, S> {
    /// The deployer's transaction, exposed for overriding the defaults
    pub tx: TransactionRequest,
    abi: Abi,
    client: &'a Client<P, S>,
    confs: usize,
}

impl<'a, P, S> Deployer<'a, P, S>
where
    S: Signer,
    P: JsonRpcClient,
{
    /// Sets the number of confirmations to wait for the contract deployment transaction
    pub fn confirmations<T: Into<usize>>(mut self, confirmations: T) -> Self {
        self.confs = confirmations.into();
        self
    }

    /// Broadcasts the contract deployment transaction and after waiting for it to
    /// be sufficiently confirmed (default: 1), it returns a [`Contract`](crate::Contract)
    /// struct at the deployed contract's address.
    pub async fn send(self) -> Result<Contract<'a, P, S>, ContractError> {
        let pending_tx = self.client.send_transaction(self.tx, None).await?;

        let receipt = pending_tx.confirmations(self.confs).await?;

        let address = receipt
            .contract_address
            .ok_or(ContractError::ContractNotDeployed)?;

        let contract = Contract::new(address, self.abi.clone(), self.client);
        Ok(contract)
    }

    /// Returns a reference to the deployer's ABI
    pub fn abi(&self) -> &Abi {
        &self.abi
    }

    /// Returns a reference to the deployer's client
    pub fn client(&self) -> &Client<P, S> {
        &self.client
    }
}

#[derive(Debug, Clone)]
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
/// use ethers::{
///     utils::Solc,
///     contract::ContractFactory,
///     providers::{Provider, Http},
///     signers::Wallet
/// };
/// use std::convert::TryFrom;
///
/// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
/// // first we'll compile the contract (you can alternatively compile it yourself
/// // and pass the ABI/Bytecode
/// let compiled = Solc::new("./tests/contract.sol").build().unwrap();
/// let contract = compiled
///     .get("SimpleStorage")
///     .expect("could not find contract");
///
/// // connect to the network
/// let provider = Provider::<Http>::try_from("http://localhost:8545").unwrap();
/// let client = "380eb0f3d505f087e438eca80bc4df9a7faa24f868e69fc0440261a0fc0567dc"
///     .parse::<Wallet>()?.connect(provider);
///
/// // create a factory which will be used to deploy instances of the contract
/// let factory = ContractFactory::new(contract.abi.clone(), contract.bytecode.clone(), &client);
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
pub struct ContractFactory<'a, P, S> {
    client: &'a Client<P, S>,
    abi: Abi,
    bytecode: Bytes,
}

impl<'a, P, S> ContractFactory<'a, P, S>
where
    S: Signer,
    P: JsonRpcClient,
{
    /// Creates a factory for deployment of the Contract with bytecode, and the
    /// constructor defined in the abi. The client will be used to send any deployment
    /// transaction.
    pub fn new(abi: Abi, bytecode: Bytes, client: &'a Client<P, S>) -> Self {
        Self {
            client,
            abi,
            bytecode,
        }
    }

    /// Constructs the deployment transaction based on the provided constructor
    /// arguments and returns a `Deployer` instance. You must call `send()` in order
    /// to actually deploy the contract.
    ///
    /// Notes:
    /// 1. If there are no constructor arguments, you should pass `()` as the argument.
    /// 1. The default poll duration is 7 seconds.
    /// 1. The default number of confirmations is 1 block.
    pub fn deploy<T: Tokenize>(
        self,
        constructor_args: T,
    ) -> Result<Deployer<'a, P, S>, ContractError> {
        // Encode the constructor args & concatenate with the bytecode if necessary
        let params = constructor_args.into_tokens();
        let data: Bytes = match (self.abi.constructor(), params.is_empty()) {
            (None, false) => {
                return Err(ContractError::ConstructorError);
            }
            (None, true) => self.bytecode.clone(),
            (Some(constructor), _) => {
                Bytes(constructor.encode_input(self.bytecode.0.clone(), &params)?)
            }
        };

        // create the tx object. Since we're deploying a contract, `to` is `None`
        let tx = TransactionRequest {
            to: None,
            data: Some(data),
            ..Default::default()
        };

        Ok(Deployer {
            client: self.client,
            abi: self.abi,
            tx,
            confs: 1,
        })
    }
}
