extern crate hex;

use ethers_core::{
    abi::{Detokenize, Function, Token},
    types::{Address, BlockNumber, NameOrAddress, TransactionRequest, U256},
};
use ethers_providers::JsonRpcClient;
use ethers_signers::{Client, Signer};
use hex::FromHex;
use lazy_static::lazy_static;

use std::{collections::HashMap, sync::Arc};

use crate::call::{ContractCall, ContractError};
use crate::multicall_contract::MulticallContract;

lazy_static! {
    static ref ADDRESS_BOOK: HashMap<U256, Address> = {
        let mut m = HashMap::new();

        // mainnet
        let addr = <[u8; 20]>::from_hex("eefba1e63905ef1d7acba5a8513c70307c1ce441").expect("Decoding failed");
        m.insert(U256::from(1u8), Address::from(addr));

        // rinkeby
        let addr = <[u8; 20]>::from_hex("42ad527de7d4e9d9d011ac45b31d8551f8fe9821").expect("Decoding failed");
        m.insert(U256::from(4u8), Address::from(addr));

        // goerli
        let addr = <[u8; 20]>::from_hex("77dca2c955b15e9de4dbbcf1246b4b85b651e50e").expect("Decoding failed");
        m.insert(U256::from(5u8), Address::from(addr));

        // kovan
        let addr = <[u8; 20]>::from_hex("2cc8688c5f75e365aaeeb4ea8d6a480405a48d2a").expect("Decoding failed");
        m.insert(U256::from(42u8), Address::from(addr));

        m
    };
}

#[derive(Clone)]
struct Call {
    target: Address,
    data: Vec<u8>,
    function: Function,
}

pub struct Multicall<P, S> {
    address: Option<Address>,
    calls: Vec<Call>,
    block: Option<BlockNumber>,
    client: Arc<Client<P, S>>,
}

impl<P, S> Multicall<P, S> {
    pub fn new(
        address: Option<Address>,
        block: Option<BlockNumber>,
        client: impl Into<Arc<Client<P, S>>>,
    ) -> Self {
        Self {
            address,
            calls: vec![],
            block,
            client: client.into(),
        }
    }

    pub fn add_one(mut self, tx: TransactionRequest, function: Function) -> Self {
        match (tx.to, tx.data) {
            (Some(NameOrAddress::Address(target)), Some(data)) => {
                let call = Call {
                    target,
                    data: data.0,
                    function,
                };
                self.calls.push(call);
                self
            }
            _ => self,
        }
    }

    pub fn add_many(mut self, txs: Vec<TransactionRequest>, functions: Vec<Function>) -> Self {
        let calls: Vec<Call> = txs
            .into_iter()
            .zip(functions.into_iter())
            .filter_map(|(tx, function)| match (tx.to, tx.data) {
                (Some(NameOrAddress::Address(target)), Some(data)) => Some(Call {
                    target,
                    data: data.0,
                    function,
                }),
                _ => None,
            })
            .collect();

        self.calls.extend(calls);
        self
    }

    pub fn add_call<D: Detokenize>(mut self, call: ContractCall<P, S, D>) -> Self {
        match (call.tx.to, call.tx.data) {
            (Some(NameOrAddress::Address(target)), Some(data)) => {
                let call = Call {
                    target,
                    data: data.0,
                    function: call.function,
                };
                self.calls.push(call);
                self
            }
            _ => self,
        }
    }

    pub fn add_calls<D: Detokenize>(mut self, calls: Vec<ContractCall<P, S, D>>) -> Self {
        let calls: Vec<Call> = calls
            .into_iter()
            .filter_map(|call| match (call.tx.to, call.tx.data) {
                (Some(NameOrAddress::Address(target)), Some(data)) => Some(Call {
                    target,
                    data: data.0,
                    function: call.function,
                }),
                _ => None,
            })
            .collect();

        self.calls.extend(calls);
        self
    }
}

impl<P, S> Multicall<P, S>
where
    P: JsonRpcClient,
    S: Signer,
{
    pub async fn call<D: Detokenize>(&self) -> Result<(U256, D), ContractError> {
        // 1. Fetch chain id and the corresponding address of Multicall contract
        // preference is given to Multicall contract's address if provided
        // otherwise check the address book for the client's chain ID
        let address = match self.address {
            Some(address) => address,
            None => {
                let chain_id = self.client.get_chainid().await?;
                match ADDRESS_BOOK.get(&chain_id) {
                    Some(address) => address.clone(),
                    None => return Err(ContractError::ConstructorError),
                }
            }
        };

        // 2. Instantiate the multicall contract
        let multicall = MulticallContract::new(address, Arc::clone(&self.client));

        // 3. Map the Multicall struct into appropriate types for `aggregate` function
        let calls: Vec<(Address, Vec<u8>)> = self
            .calls
            .clone()
            .into_iter()
            .map(|call| (call.target, call.data))
            .collect();

        // 4. Call the `aggregate` function and get return data
        let contract_call = multicall.aggregate(calls);
        let contract_call = {
            if let Some(block) = self.block {
                contract_call.block(block)
            } else {
                contract_call
            }
        };

        let (block_number, vec_bytes) = contract_call.call().await?;

        // 5. Decode return data into ABI tokens
        let tokens: Vec<Token> = self
            .calls
            .clone()
            .into_iter()
            .zip(vec_bytes.into_iter())
            .map(|(call, bytes)| {
                let tokens: Vec<Token> = call.function.decode_output(&bytes).unwrap();

                // NOTE: post processing
                match tokens.len() {
                    0 => Token::Tuple(vec![]),
                    1 => tokens[0].clone(),
                    _ => Token::Tuple(tokens),
                }
            })
            .collect();

        // 6. Form tokens that represent tuples
        let tokens = vec![Token::Tuple(tokens)];

        // 7. Detokenize from the tokens into the provided tuple D
        let data = D::from_tokens(tokens)?;

        Ok((block_number, data))
    }
}
