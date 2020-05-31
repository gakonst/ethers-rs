use crate::{Contract, ContractError};

use ethers_core::{
    abi::{Abi, Tokenize},
    types::{Bytes, TransactionRequest},
};
use ethers_providers::{networks::Network, JsonRpcClient};
use ethers_signers::{Client, Signer};

use std::time::Duration;
use tokio::time;

/// Poll for tx confirmation once every 7 seconds.
/// TODO: Can this be improved by replacing polling with an "on new block" subscription?
const POLL_INTERVAL: u64 = 7000;

#[derive(Debug, Clone)]
pub struct Deployer<'a, P, N, S> {
    client: &'a Client<'a, P, N, S>,
    abi: &'a Abi,
    tx: TransactionRequest,
    confs: usize,
    poll_interval: Duration,
}

impl<'a, P, N, S> Deployer<'a, P, N, S>
where
    S: Signer,
    P: JsonRpcClient,
    P::Error: 'static,
    N: Network,
{
    pub fn poll_interval<T: Into<Duration>>(mut self, interval: T) -> Self {
        self.poll_interval = interval.into();
        self
    }

    pub fn confirmations<T: Into<usize>>(mut self, confirmations: T) -> Self {
        self.confs = confirmations.into();
        self
    }

    pub async fn send(self) -> Result<Contract<'a, P, N, S>, ContractError<P>> {
        let tx_hash = self
            .client
            .send_transaction(self.tx, None)
            .await
            .map_err(ContractError::CallError)?;

        // poll for the receipt
        let address;
        loop {
            if let Ok(receipt) = self.client.get_transaction_receipt(tx_hash).await {
                address = receipt
                    .contract_address
                    .ok_or(ContractError::ContractNotDeployed)?;
                break;
            }

            time::delay_for(Duration::from_millis(POLL_INTERVAL)).await;
        }

        let contract = Contract::new(self.client, self.abi, address);
        Ok(contract)
    }
}

#[derive(Debug, Clone)]
pub struct ContractFactory<'a, P, N, S> {
    client: &'a Client<'a, P, N, S>,
    abi: &'a Abi,
    bytecode: &'a Bytes,
}

impl<'a, P, N, S> ContractFactory<'a, P, N, S>
where
    S: Signer,
    P: JsonRpcClient,
    P::Error: 'static,
    N: Network,
{
    /// Instantiate a new contract factory
    pub fn new(client: &'a Client<'a, P, N, S>, abi: &'a Abi, bytecode: &'a Bytes) -> Self {
        Self {
            client,
            abi,
            bytecode,
        }
    }

    /// Deploys an instance of the contract with the provider constructor arguments
    /// and returns the contract's instance
    pub fn deploy<T: Tokenize>(
        &self,
        constructor_args: T,
    ) -> Result<Deployer<'a, P, N, S>, ContractError<P>> {
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
            poll_interval: Duration::from_millis(POLL_INTERVAL),
        })
    }
}
