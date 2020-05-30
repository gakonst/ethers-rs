use crate::Contract;

use ethers_providers::{networks::Network, JsonRpcClient};
use ethers_signers::{Client, Signer};
use ethers_types::{
    abi::{Abi, Tokenize},
    Bytes,
};

#[derive(Debug, Clone)]
pub struct ContractFactory<'a, P, N, S> {
    client: &'a Client<'a, P, N, S>,
    abi: &'a Abi,
    bytecode: &'a Bytes,
}

impl<'a, P: JsonRpcClient, N: Network, S: Signer> ContractFactory<'a, P, N, S> {
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
    pub async fn deploy<T: Tokenize>(
        constructor_args: T,
    ) -> Result<Contract<'a, P, N, S>, P::Error> {
        // 1. Encode the constructor args
        //
        // 2. Create the runtime bytecode by concatenating the bytecode with the constructor
        // arguments (?)
        //
        // 3. Call `client.send_transaction()` to deploy
        //
        // 4. Get the address of the contract from the transaction receipt
        //
        // 5. Instantiate & return the contract object
    }
}
