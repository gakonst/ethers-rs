use ethers_core::{
    abi::{Detokenize, Function, Token},
    types::{Address, BlockNumber, Bytes, NameOrAddress, TxHash, U256},
};
use ethers_providers::Middleware;

use std::{collections::HashMap, str::FromStr, sync::Arc};

use crate::{
    call::{ContractCall, ContractError},
    Lazy,
};

mod multicall_contract;
use multicall_contract::MulticallContract;

/// A lazily computed hash map with the Ethereum network IDs as keys and the corresponding
/// Multicall smart contract addresses as values
pub static ADDRESS_BOOK: Lazy<HashMap<U256, Address>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // mainnet
    let addr =
        Address::from_str("eefba1e63905ef1d7acba5a8513c70307c1ce441").expect("Decoding failed");
    m.insert(U256::from(1u8), addr);

    // rinkeby
    let addr =
        Address::from_str("42ad527de7d4e9d9d011ac45b31d8551f8fe9821").expect("Decoding failed");
    m.insert(U256::from(4u8), addr);

    // goerli
    let addr =
        Address::from_str("77dca2c955b15e9de4dbbcf1246b4b85b651e50e").expect("Decoding failed");
    m.insert(U256::from(5u8), addr);

    // kovan
    let addr =
        Address::from_str("2cc8688c5f75e365aaeeb4ea8d6a480405a48d2a").expect("Decoding failed");
    m.insert(U256::from(42u8), addr);

    // xdai
    let addr =
        Address::from_str("b5b692a88bdfc81ca69dcb1d924f59f0413a602a").expect("Decoding failed");
    m.insert(U256::from(100u8), addr);

    m
});

/// A Multicall is an abstraction for sending batched calls/transactions to the Ethereum blockchain.
/// It stores an instance of the [`Multicall` smart contract](https://etherscan.io/address/0xeefba1e63905ef1d7acba5a8513c70307c1ce441#code)
/// and the user provided list of transactions to be made.
///
/// `Multicall` can instantiate the Multicall contract instance from the chain ID of the client
/// supplied to [`new`]. It supports the Ethereum mainnet, as well as testnets
/// [Rinkeby](https://rinkeby.etherscan.io/address/0x42ad527de7d4e9d9d011ac45b31d8551f8fe9821#code),
/// [Goerli](https://goerli.etherscan.io/address/0x77dca2c955b15e9de4dbbcf1246b4b85b651e50e) and
/// [Kovan](https://kovan.etherscan.io/address/0x2cc8688c5f75e365aaeeb4ea8d6a480405a48d2a#code).
///
/// Additionally, the `block` number can be provided for the call by using the [`block`] method.
/// Build on the `Multicall` instance by adding calls using the [`add_call`] method.
///
/// # Example
///
/// ```no_run
/// use ethers::{
///     abi::Abi,
///     contract::{Contract, Multicall},
///     providers::{Middleware, Http, Provider, PendingTransaction},
///     types::{Address, H256, U256},
/// };
/// use std::{convert::TryFrom, sync::Arc};
///
/// # async fn bar() -> Result<(), Box<dyn std::error::Error>> {
/// // this is a dummy address used for illustration purpose
/// let address = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".parse::<Address>()?;
///
/// // (ugly way to write the ABI inline, you can otherwise read it from a file)
/// let abi: Abi = serde_json::from_str(r#"[{"inputs":[{"internalType":"string","name":"value","type":"string"}],"stateMutability":"nonpayable","type":"constructor"},{"anonymous":false,"inputs":[{"indexed":true,"internalType":"address","name":"author","type":"address"},{"indexed":true,"internalType":"address","name":"oldAuthor","type":"address"},{"indexed":false,"internalType":"string","name":"oldValue","type":"string"},{"indexed":false,"internalType":"string","name":"newValue","type":"string"}],"name":"ValueChanged","type":"event"},{"inputs":[],"name":"getValue","outputs":[{"internalType":"string","name":"","type":"string"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"lastSender","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"string","name":"value","type":"string"}],"name":"setValue","outputs":[],"stateMutability":"nonpayable","type":"function"}]"#)?;
///
/// // connect to the network
/// let client = Provider::<Http>::try_from("https://kovan.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27")?;
///
/// // create the contract object. This will be used to construct the calls for multicall
/// let client = Arc::new(client);
/// let contract = Contract::<Provider<Http>>::new(address, abi, Arc::clone(&client));
///
/// // note that these [`ContractCall`]s are futures, and need to be `.await`ed to resolve.
/// // But we will let `Multicall` to take care of that for us
/// let first_call = contract.method::<_, String>("getValue", ())?;
/// let second_call = contract.method::<_, Address>("lastSender", ())?;
///
/// // since this example connects to the Kovan testnet, we need not provide an address for
/// // the Multicall contract and we set that to `None`. If you wish to provide the address
/// // for the Multicall contract, you can pass the `Some(multicall_addr)` argument.
/// // Construction of the `Multicall` instance follows the builder pattern
/// let mut multicall = Multicall::new(Arc::clone(&client), None).await?;
/// multicall
///     .add_call(first_call)
///     .add_call(second_call);
///
/// // `await`ing on the `call` method lets us fetch the return values of both the above calls
/// // in one single RPC call
/// let _return_data: (String, Address) = multicall.call().await?;
///
/// // the same `Multicall` instance can be re-used to do a different batch of transactions.
/// // Say we wish to broadcast (send) a couple of transactions via the Multicall contract.
/// let first_broadcast = contract.method::<_, H256>("setValue", "some value".to_owned())?;
/// let second_broadcast = contract.method::<_, H256>("setValue", "new value".to_owned())?;
/// let multicall = multicall
///     .clear_calls()
///     .add_call(first_broadcast)
///     .add_call(second_broadcast);
///
/// // `await`ing the `send` method waits for the transaction to be broadcast, which also
/// // returns the transaction hash
/// let tx_hash = multicall.send().await?;
/// let _tx_receipt = PendingTransaction::new(tx_hash, &client).await?;
///
/// // you can also query ETH balances of multiple addresses
/// let address_1 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".parse::<Address>()?;
/// let address_2 = "ffffffffffffffffffffffffffffffffffffffff".parse::<Address>()?;
/// let multicall = multicall
///     .clear_calls()
///     .eth_balance_of(address_1)
///     .eth_balance_of(address_2);
/// let _balances: (U256, U256) = multicall.call().await?;
/// # Ok(())
/// # }
/// ```
///
/// [`new`]: method@crate::Multicall::new
/// [`block`]: method@crate::Multicall::block
/// [`add_call`]: methond@crate::Multicall::add_call
#[derive(Clone)]
pub struct Multicall<M> {
    calls: Vec<Call>,
    block: Option<BlockNumber>,
    contract: MulticallContract<M>,
    legacy: bool,
}

#[derive(Clone)]
/// Helper struct for managing calls to be made to the `function` in smart contract `target`
/// with `data`
pub struct Call {
    target: Address,
    data: Bytes,
    function: Function,
}

impl<M: Middleware> Multicall<M> {
    /// Creates a new Multicall instance from the provided client. If provided with an `address`,
    /// it instantiates the Multicall contract with that address. Otherwise it fetches the address
    /// from the address book.
    ///
    /// # Panics
    /// If a `None` address is provided, and the provided client also does not belong to one of
    /// the supported network IDs (mainnet, kovan, rinkeby and goerli)
    pub async fn new<C: Into<Arc<M>>>(
        client: C,
        address: Option<Address>,
    ) -> Result<Self, ContractError<M>> {
        let client = client.into();

        // Fetch chain id and the corresponding address of Multicall contract
        // preference is given to Multicall contract's address if provided
        // otherwise check the address book for the client's chain ID
        let address: Address = match address {
            Some(addr) => addr,
            None => {
                let chain_id = client
                    .get_chainid()
                    .await
                    .map_err(ContractError::MiddlewareError)?;
                match ADDRESS_BOOK.get(&chain_id) {
                    Some(addr) => *addr,
                    None => panic!(
                        "Must either be a supported Network ID or provide Multicall contract address"
                    ),
                }
            }
        };

        // Instantiate the multicall contract
        let contract = MulticallContract::new(address, client);

        Ok(Self {
            calls: vec![],
            block: None,
            contract,
            legacy: false,
        })
    }

    /// Makes a legacy transaction instead of an EIP-1559 one
    pub fn legacy(mut self) -> Self {
        self.legacy = true;
        self
    }

    /// Sets the `block` field for the multicall aggregate call
    pub fn block<T: Into<BlockNumber>>(mut self, block: T) -> Self {
        self.block = Some(block.into());
        self
    }

    /// Appends a `call` to the list of calls for the Multicall instance
    ///
    /// # Panics
    ///
    /// If more than the maximum number of supported calls are added. The maximum
    /// limits is constrained due to tokenization/detokenization support for tuples
    pub fn add_call<D: Detokenize>(&mut self, call: ContractCall<M, D>) -> &mut Self {
        if self.calls.len() >= 16 {
            panic!("Cannot support more than {} calls", 16);
        }

        match (call.tx.to(), call.tx.data()) {
            (Some(NameOrAddress::Address(target)), Some(data)) => {
                let call = Call {
                    target: *target,
                    data: data.clone(),
                    function: call.function,
                };
                self.calls.push(call);
                self
            }
            _ => self,
        }
    }

    /// Appends a `call` to the list of calls for the Multicall instance for querying
    /// the ETH balance of an address
    ///
    /// # Panics
    ///
    /// If more than the maximum number of supported calls are added. The maximum
    /// limits is constrained due to tokenization/detokenization support for tuples
    pub fn eth_balance_of(&mut self, addr: Address) -> &mut Self {
        let call = self.contract.get_eth_balance(addr);
        self.add_call(call)
    }

    /// Clear the batch of calls from the Multicall instance. Re-use the already instantiated
    /// Multicall, to send a different batch of transactions or do another aggregate query
    ///
    /// ```no_run
    /// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
    /// # use ethers::{abi::Abi, prelude::*};
    /// # use std::{sync::Arc, convert::TryFrom};
    /// #
    /// # let client = Provider::<Http>::try_from("http://localhost:8545")?;
    /// # let client = Arc::new(client);
    /// #
    /// # let abi: Abi = serde_json::from_str("")?;
    /// # let address = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".parse::<Address>()?;
    /// # let contract = Contract::<Provider<Http>>::new(address, abi, client.clone());
    /// #
    /// # let broadcast_1 = contract.method::<_, H256>("setValue", "some value".to_owned())?;
    /// # let broadcast_2 = contract.method::<_, H256>("setValue", "new value".to_owned())?;
    /// #
    /// let mut multicall = Multicall::new(client, None).await?;
    /// multicall
    ///     .add_call(broadcast_1)
    ///     .add_call(broadcast_2);
    ///
    /// let _tx_hash = multicall.send().await?;
    ///
    /// # let call_1 = contract.method::<_, String>("getValue", ())?;
    /// # let call_2 = contract.method::<_, Address>("lastSender", ())?;
    /// multicall
    ///     .clear_calls()
    ///     .add_call(call_1)
    ///     .add_call(call_2);
    /// let return_data: (String, Address) = multicall.call().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn clear_calls(&mut self) -> &mut Self {
        self.calls.clear();
        self
    }

    /// Queries the Ethereum blockchain via an `eth_call`, but via the Multicall contract.
    ///
    /// It returns a [`ContractError<M>`] if there is any error in the RPC call or while
    /// detokenizing the tokens back to the expected return type. The return type must be
    /// annonated while calling this method.
    ///
    /// ```no_run
    /// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
    /// # use ethers::prelude::*;
    /// # use std::convert::TryFrom;
    /// #
    /// # let client = Provider::<Http>::try_from("http://localhost:8545")?;
    /// #
    /// # let multicall = Multicall::new(client, None).await?;
    /// // If the Solidity function calls has the following return types:
    /// // 1. `returns (uint256)`
    /// // 2. `returns (string, address)`
    /// // 3. `returns (bool)`
    /// let result: (U256, (String, Address), bool) = multicall.call().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Note: this method _does not_ send a transaction from your account
    ///
    /// [`ContractError<M>`]: crate::ContractError<M>
    pub async fn call<D: Detokenize>(&self) -> Result<D, ContractError<M>> {
        let contract_call = self.as_contract_call();

        // Fetch response from the Multicall contract
        let (_block_number, return_data) = contract_call.call().await?;

        // Decode return data into ABI tokens
        let tokens = self
            .calls
            .iter()
            .zip(&return_data)
            .map(|(call, bytes)| {
                let mut tokens: Vec<Token> = call.function.decode_output(bytes)?;

                Ok(match tokens.len() {
                    0 => Token::Tuple(vec![]),
                    1 => tokens.remove(0),
                    _ => Token::Tuple(tokens),
                })
            })
            .collect::<Result<Vec<Token>, ContractError<M>>>()?;

        // Form tokens that represent tuples
        let tokens = vec![Token::Tuple(tokens)];

        // Detokenize from the tokens into the provided tuple D
        let data = D::from_tokens(tokens)?;

        Ok(data)
    }

    /// Signs and broadcasts a batch of transactions by using the Multicall contract as proxy.
    ///
    /// ```no_run
    /// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
    /// # use ethers::prelude::*;
    /// # use std::convert::TryFrom;
    /// # let client = Provider::<Http>::try_from("http://localhost:8545")?;
    /// # let multicall = Multicall::new(client, None).await?;
    /// let tx_hash = multicall.send().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Note: this method sends a transaction from your account, and will return an error
    /// if you do not have sufficient funds to pay for gas
    pub async fn send(&self) -> Result<TxHash, ContractError<M>> {
        let contract_call = self.as_contract_call();

        // Broadcast transaction and return the transaction hash
        // TODO: Can we make this return a PendingTransaction directly instead?
        // Seems hard due to `returns a value referencing data owned by the current function`
        let tx_hash = *contract_call.send().await?;

        Ok(tx_hash)
    }

    fn as_contract_call(&self) -> ContractCall<M, (U256, Vec<Vec<u8>>)> {
        // Map the Multicall struct into appropriate types for `aggregate` function
        let calls: Vec<(Address, Vec<u8>)> = self
            .calls
            .iter()
            .map(|call| (call.target, call.data.to_vec()))
            .collect();

        // Construct the ContractCall for `aggregate` function to broadcast the transaction
        let mut contract_call = self.contract.aggregate(calls);
        if let Some(block) = self.block {
            contract_call = contract_call.block(block)
        };

        if self.legacy {
            contract_call = contract_call.legacy();
        };

        contract_call
    }
}
