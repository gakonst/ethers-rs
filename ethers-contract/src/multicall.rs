use ethers_core::{
    abi::{Detokenize, Function, Token},
    types::{Address, BlockNumber, NameOrAddress, TxHash, U256},
};
use ethers_providers::JsonRpcClient;
use ethers_signers::{Client, Signer};

use std::{collections::HashMap, str::FromStr, sync::Arc};

use crate::{
    call::{ContractCall, ContractError},
    multicall_contract::MulticallContract,
    Lazy,
};

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
///     providers::{Http, Provider},
///     signers::{Client, Wallet},
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
/// let provider = Provider::<Http>::try_from("https://kovan.infura.io")?;
/// let client = "380eb0f3d505f087e438eca80bc4df9a7faa24f868e69fc0440261a0fc0567dc"
///     .parse::<Wallet>()?.connect(provider);
///
/// // create the contract object. This will be used to construct the calls for multicall
/// let contract = Contract::new(address, abi, client.clone());
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
/// let multicall = Multicall::new(client.clone(), None)
///     .await?
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
/// let _tx_receipt = client.provider().pending_transaction(tx_hash).await?;
/// # Ok(())
/// # }
/// ```
///
/// [`new`]: method@crate::Multicall::new
/// [`block`]: method@crate::Multicall::block
/// [`add_call`]: methond@crate::Multicall::add_call
pub struct Multicall<P, S> {
    calls: Vec<Call>,
    block: Option<BlockNumber>,
    contract: MulticallContract<P, S>,
}

#[derive(Clone)]
/// Helper struct for managing calls to be made to the `function` in smart contract `target`
/// with `data`
pub struct Call {
    target: Address,
    data: Vec<u8>,
    function: Function,
}

impl<P, S> Multicall<P, S>
where
    P: JsonRpcClient,
    S: Signer,
{
    /// Creates a new Multicall instance from the provided client. If provided with an `address`,
    /// it instantiates the Multicall contract with that address. Otherwise it fetches the address
    /// from the address book.
    /// # Panics
    /// If a `None` address is provided, and the provided client also does not belong to one of
    /// the supported network IDs (mainnet, kovan, rinkeby and goerli)
    pub async fn new<C: Into<Arc<Client<P, S>>>>(
        client: C,
        address: Option<Address>,
    ) -> Result<Self, ContractError> {
        // Clone the client
        let client = Arc::clone(&client.into());

        // Fetch chain id and the corresponding address of Multicall contract
        // preference is given to Multicall contract's address if provided
        // otherwise check the address book for the client's chain ID
        let address: Address = match address {
            Some(addr) => addr,
            None => {
                let chain_id = client.get_chainid().await?;
                match ADDRESS_BOOK.get(&chain_id) {
                    Some(addr) => addr.clone(),
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
        })
    }

    /// Sets the `block` field for the multicall aggregate call
    pub fn block<T: Into<BlockNumber>>(mut self, block: T) -> Self {
        self.block = Some(block.into());
        self
    }

    /// Appends a `call` to the list of calls for the Multicall instance
    /// # Panics
    /// If more than the maximum number of supported calls are added. The maximum
    /// limits is constrained due to tokenization/detokenization support for tuples
    pub fn add_call<D: Detokenize>(mut self, call: ContractCall<P, S, D>) -> Self {
        if self.calls.len() >= 16 {
            panic!("Cannot support more than {} calls", 16);
        }

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

    /// Clear the batch of calls from the Multicall instance. Re-use the already instantiated
    /// Multicall, to send a different batch of transactions or do another aggregate query
    ///
    /// ```ignore
    /// let multicall = Multicall::new(client, None)
    ///     .await?
    ///     .add_call(broadcast_1)
    ///     .add_call(broadcast_2);
    /// let _tx_hash = multicall.send().await?;
    ///
    /// let multicall = multicall
    ///     .clear_calls()
    ///     .add_call(call_1)
    ///     .add_call(call_2);
    /// let return_data: (String, Address) = multicall.call().await?;
    /// ```
    pub fn clear_calls(mut self) -> Self {
        self.calls.clear();
        self
    }
}

impl<P, S> Multicall<P, S>
where
    P: JsonRpcClient,
    S: Signer,
{
    /// Queries the Ethereum blockchain via an `eth_call`, but to the Multicall contract.
    ///
    /// It returns a [`ContractError`] if there is any error in the RPC call or while
    /// detokenizing the tokens back to the expected return type. The return type must be
    /// annonated while calling this method.
    ///
    /// ```ignore
    /// // If the solidity function calls has the following return types:
    /// // 1. returns (uint256)
    /// // 2. returns (string, address)
    /// // 3. returns (bool)
    /// let result: (U256, (String, Address), bool) = multicall.call().await?;
    /// ```
    ///
    /// Note: this method _does not_ send a transaction from your account
    pub async fn call<D: Detokenize>(&self) -> Result<D, ContractError> {
        // Map the Multicall struct into appropriate types for `aggregate` function
        let calls: Vec<(Address, Vec<u8>)> = self
            .calls
            .clone()
            .into_iter()
            .map(|call| (call.target, call.data))
            .collect();

        // Call the `aggregate` function and get return data
        let contract_call = self.contract.aggregate(calls);
        let contract_call = {
            if let Some(block) = self.block {
                contract_call.block(block)
            } else {
                contract_call
            }
        };

        // Fetch respons from the Multicall contract
        let (_block_number, vec_bytes) = contract_call.call().await?;

        // Decode return data into ABI tokens
        let tokens: Vec<Token> = self
            .calls
            .clone()
            .into_iter()
            .zip(vec_bytes.into_iter())
            .map(|(call, bytes)| {
                let tokens: Vec<Token> = call.function.decode_output(&bytes).unwrap();

                match tokens.len() {
                    0 => Token::Tuple(vec![]),
                    1 => tokens[0].clone(),
                    _ => Token::Tuple(tokens),
                }
            })
            .collect();

        // Form tokens that represent tuples
        let tokens = vec![Token::Tuple(tokens)];

        // Detokenize from the tokens into the provided tuple D
        let data = D::from_tokens(tokens)?;

        Ok(data)
    }

    /// Signs and broadcasts a batch of transactions by using the Multicall contract as proxy.
    ///
    /// ```ignore
    /// let tx_hash = multicall.send().await?;
    /// ```
    ///
    /// Note: this method sends a transaction from your account, and will return an error
    /// if you do not have sufficient funds to pay for gas
    pub async fn send(&self) -> Result<TxHash, ContractError> {
        // Map the Multicall struct into appropriate types for `aggregate` function
        let calls: Vec<(Address, Vec<u8>)> = self
            .calls
            .clone()
            .into_iter()
            .map(|call| (call.target, call.data))
            .collect();

        // Construct the ContractCall for `aggregate` function to broadcast the transaction
        let contract_call = self.contract.aggregate(calls);
        let contract_call = {
            if let Some(block) = self.block {
                contract_call.block(block)
            } else {
                contract_call
            }
        };

        // Broadcast transaction and return the transaction hash
        let tx_hash = contract_call.send().await?;

        Ok(tx_hash)
    }
}
