use ethers_core::{
    abi::{decode, Detokenize, Function, ParamType, Token},
    types::{Address, BlockNumber, Bytes, Chain, NameOrAddress, TxHash, H160, U256},
};
use ethers_providers::{Middleware, ProviderError};

use std::{convert::TryFrom, sync::Arc};

use crate::{
    call::{ContractCall, ContractError},
    Lazy,
};

mod multicall_contract;
use multicall_contract::multicall_3::{
    Call as Multicall1Call, Call3 as Multicall3Call, Call3Value as Multicall3CallValue,
    Result as MulticallResult,
};

// Export the contract interface
pub use multicall_contract::multicall_3::Multicall3 as MulticallContract;

/// The Multicall3 contract address that is deployed in [`MULTICALL_SUPPORTED_CHAIN_IDS`].
pub const MULTICALL_ADDRESS: Address = H160([
    0xca, 0x11, 0xbd, 0xe0, 0x59, 0x77, 0xb3, 0x63, 0x11, 0x67, 0x02, 0x88, 0x62, 0xbe, 0x2a, 0x17,
    0x39, 0x76, 0xca, 0x11,
]);

/// The chain IDs that [`MULTICALL_ADDRESS`] has been deployed to.
pub static MULTICALL_SUPPORTED_CHAIN_IDS: Lazy<[U256; 47]> = Lazy::new(|| {
    use Chain::*;
    // from: https://github.com/mds1/multicall#multicall3-contract-addresses
    [
        U256::from(Mainnet),                  // Mainnet
        U256::from(Kovan),                    // Kovan
        U256::from(Rinkeby),                  // Rinkeby
        U256::from(Goerli),                   // Goerli
        U256::from(Ropsten),                  // Ropsten
        U256::from(Sepolia),                  // Sepolia
        U256::from(Optimism),                 // Optimism
        U256::from(OptimismKovan),            // OptimismKovan
        U256::from(420),                      // OptimismGoerli
        U256::from(Arbitrum),                 // Arbitrum
        U256::from(421613),                   // ArbitrumGoerli,
        U256::from(ArbitrumTestnet),          // Arbitrum Rinkeby
        U256::from(Polygon),                  // Polygon
        U256::from(PolygonMumbai),            // PolygonMumbai
        U256::from(XDai),                     // XDai
        U256::from(Avalanche),                // Avalanche
        U256::from(AvalancheFuji),            // AvalancheFuji
        U256::from(FantomTestnet),            // FantomTestnet
        U256::from(Fantom),                   // Fantom
        U256::from(BinanceSmartChain),        // BinanceSmartChain
        U256::from(BinanceSmartChainTestnet), // BinanceSmartChainTestnet
        U256::from(Moonbeam),                 // Moonbeam
        U256::from(Moonriver),                // Moonriver
        U256::from(Moonbase),                 // Moonbase
        U256::from(1666600000),               // Harmony0
        U256::from(1666600001),               // Harmony1
        U256::from(1666600002),               // Harmony2
        U256::from(1666600003),               // Harmony3
        U256::from(Cronos),                   // Cronos
        U256::from(122),                      // Fuse
        U256::from(19),                       // Songbird
        U256::from(16),                       // CostonTestnet
        U256::from(288),                      // Boba
        U256::from(Aurora),                   // Aurora
        U256::from(592),                      // Astar
        U256::from(66),                       // OKC
        U256::from(128),                      // Heco
        U256::from(1088),                     // Metis
        U256::from(Rsk),                      // Rsk
        U256::from(31),                       // RskTestnet
        U256::from(Evmos),                    // Evmos
        U256::from(EvmosTestnet),             // EvmosTestnet
        U256::from(71402),                    // Godwoken
        U256::from(71401),                    // GodwokenTestnet
        U256::from(8217),                     // Klaytn
        U256::from(2001),                     // Milkomeda
        U256::from(321),                      // KCC
    ]
});

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum MulticallVersion {
    Multicall = 1,
    Multicall2 = 2,
    #[default]
    Multicall3 = 3,
}

impl From<MulticallVersion> for u8 {
    fn from(v: MulticallVersion) -> Self {
        v as u8
    }
}

impl TryFrom<u8> for MulticallVersion {
    type Error = String;
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            1 => Ok(MulticallVersion::Multicall),
            2 => Ok(MulticallVersion::Multicall2),
            3 => Ok(MulticallVersion::Multicall3),
            _ => Err(format!("Invalid Multicall version: {}", v)),
        }
    }
}

/// A Multicall is an abstraction for sending batched calls/transactions to the Ethereum blockchain.
/// It stores an instance of the [`Multicall` smart contract](https://etherscan.io/address/0xcA11bde05977b3631167028862bE2a173976CA11#code)
/// and the user provided list of transactions to be made.
///
/// `Multicall` can instantiate the Multicall contract instance from the chain ID of the client
/// supplied to [`new`]. All the supported chains are available [`here`](https://github.com/mds1/multicall#multicall3-contract-addresses).
///
/// Additionally, the `block` number can be provided for the call by using the [`block`] method.
/// Build on the `Multicall` instance by adding calls using the [`add_call`] method.
///
/// # Example
///
/// ```no_run
/// use ethers_core::{
///     abi::Abi,
///     types::{Address, H256, U256},
/// };
/// use ethers_contract::{Contract, Multicall};
/// use ethers_providers::{Middleware, Http, Provider, PendingTransaction};
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
///     .add_call(first_call, false)
///     .add_call(second_call, false);
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
///     .add_call(first_broadcast, false)
///     .add_call(second_broadcast, false);
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
///     .eth_balance_of(address_1, false)
///     .eth_balance_of(address_2, false);
/// let _balances: (U256, U256) = multicall.call().await?;
/// # Ok(())
/// # }
/// ```
///
/// [`new`]: method@crate::Multicall::new
/// [`block`]: method@crate::Multicall::block
/// [`add_call`]: method@crate::Multicall::add_call
pub struct Multicall<M> {
    version: MulticallVersion,
    legacy: bool,
    block: Option<BlockNumber>,
    calls: Vec<Call>,
    contract: MulticallContract<M>,
}

impl<M> Clone for Multicall<M> {
    fn clone(&self) -> Self {
        Multicall {
            calls: self.calls.clone(),
            block: self.block,
            contract: self.contract.clone(),
            legacy: self.legacy,
            version: self.version,
        }
    }
}

impl<M: Middleware> std::fmt::Debug for Multicall<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Multicall")
            .field("version", &self.version)
            .field("legacy", &self.legacy)
            .field("block", &self.block)
            .field("calls", &self.calls)
            .field("contract", &self.contract)
            .finish()
    }
}

/// Helper struct for managing calls to be made to the `function` in smart contract `target`
/// with `data`.
#[derive(Clone, Debug)]
pub struct Call {
    target: Address,
    data: Bytes,
    value: U256,
    allow_failure: bool,
    function: Function,
}

impl<M: Middleware> Multicall<M> {
    /// Creates a new Multicall instance from the provided client. If provided with an `address`,
    /// it instantiates the Multicall contract with that address, otherwise it defaults to
    /// [`MULTICALL_ADDRESS`].
    ///
    /// # Panics
    /// If a `None` address is provided and the provided client's network does not belong to one of
    /// the [supported networks](MULTICALL_SUPPORTED_CHAIN_IDS).
    pub async fn new(
        client: impl Into<Arc<M>>,
        address: Option<Address>,
    ) -> Result<Self, ContractError<M>> {
        let client = client.into();

        // Fetch chain id and the corresponding address of Multicall contract
        // preference is given to Multicall contract's address if provided
        // otherwise check the supported chain IDs for the client's chain ID
        let address: Address = match address {
            Some(addr) => addr,
            None => {
                let chain_id =
                    client.get_chainid().await.map_err(ContractError::MiddlewareError)?;
                if !MULTICALL_SUPPORTED_CHAIN_IDS.contains(&chain_id) {
                    panic!(
                            "Chain ID {} is currently not supported by Multicall. Provide an address instead.", chain_id
                        )
                }
                MULTICALL_ADDRESS
            }
        };

        // Instantiate the multicall contract
        let contract = MulticallContract::new(address, client);

        Ok(Self {
            version: MulticallVersion::Multicall3,
            legacy: false,
            block: None,
            calls: vec![],
            contract,
        })
    }

    /// Creates a new Multicall instance synchronously from the provided client and address or chain
    /// ID. Uses the [default multicall address](MULTICALL_ADDRESS) if no address is provided.
    ///
    /// # Panics
    /// If neither an address or chain_id are provided. Since this is not an async function, it will
    /// not be able to fetch chain_id from the provider to check if it is supported by the default
    /// multicall address. Use new(client, None).await instead.
    ///
    /// If the provided chain_id is not in the [supported networks](MULTICALL_SUPPORTED_CHAIN_IDS).
    pub fn new_with_chain_id(
        client: impl Into<Arc<M>>,
        address: Option<Address>,
        chain_id: Option<impl Into<U256>>,
    ) -> Self {
        // If no address is provided, check if chain_id is supported and use the default multicall
        // address.
        let address: Address = match address {
            Some(addr) => addr,
            None => {
                // Can't fetch chain_id from provider so we panic instead.
                let chain_id =
                    chain_id.expect("Must provide at least one of: address or chain ID.").into();
                if !MULTICALL_SUPPORTED_CHAIN_IDS.contains(&chain_id) {
                    panic!("Chain ID {} is currently not supported by Multicall. Provide an address instead.", chain_id)
                }
                MULTICALL_ADDRESS
            }
        };

        // Instantiate the multicall contract
        let contract = MulticallContract::new(address, client.into());

        Self {
            version: MulticallVersion::Multicall3,
            legacy: false,
            block: None,
            calls: vec![],
            contract,
        }
    }

    /// Changes which functions to use when making the contract call. The default is 3. Version
    /// differences (adapted from [here](https://github.com/mds1/multicall#multicall---)):
    ///
    /// Multicall (v1): The original contract containing an aggregate method to batch calls. Each
    /// call returns only the return data and none are allowed to fail.
    ///
    /// Multicall2 (v2): The same as Multicall, but provides additional functions that allow calls
    /// within the batch to fail. Useful for situations where a call may fail depending on the state
    /// of the contract.
    ///
    /// Multicall3 (v3): This is the recommended version. It's cheaper to use (so you can fit more
    /// calls into a single request), and it adds an aggregate3 method so you can specify whether
    /// calls are allowed to fail on a per-call basis. Additionally, it's deployed on every network
    /// at the same address.
    pub fn version(mut self, version: MulticallVersion) -> Self {
        self.version = version;
        self
    }

    /// Makes a legacy transaction instead of an EIP-1559 one.
    #[must_use]
    pub fn legacy(mut self) -> Self {
        self.legacy = true;
        self
    }

    /// Sets the `block` field for the multicall aggregate call.
    #[must_use]
    pub fn block(mut self, block: impl Into<BlockNumber>) -> Self {
        self.block = Some(block.into());
        self
    }

    /// Appends a `call` to the list of calls for the Multicall instance.
    /// `allow_revert` specifies whether or not this call is allowed to revert in the multicall
    /// (requires version >= 2).
    /// Sending transactions with value is only available for version 3.
    pub fn add_call<D: Detokenize>(
        &mut self,
        call: ContractCall<M, D>,
        allow_failure: bool,
    ) -> &mut Self {
        match (call.tx.to(), call.tx.data()) {
            (Some(NameOrAddress::Address(target)), Some(data)) => {
                let call = Call {
                    target: *target,
                    data: data.clone(),
                    value: call.tx.value().cloned().unwrap_or(U256::zero()),
                    allow_failure,
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
    pub fn eth_balance_of(&mut self, addr: Address, allow_revert: bool) -> &mut Self {
        let call = self.contract.get_eth_balance(addr);
        self.add_call(call, allow_revert)
    }

    /// Clear the batch of calls from the Multicall instance. Re-use the already instantiated
    /// Multicall, to send a different batch of transactions or do another aggregate query
    ///
    /// ```no_run
    /// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
    /// # use ethers_core::{abi::Abi, types::{Address, H256}};
    /// # use ethers_providers::{Provider, Http};
    /// # use ethers_contract::{Multicall, Contract};
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
    ///     .add_call(broadcast_1, false)
    ///     .add_call(broadcast_2, false);
    ///
    /// let _tx_hash = multicall.send().await?;
    ///
    /// # let call_1 = contract.method::<_, String>("getValue", ())?;
    /// # let call_2 = contract.method::<_, Address>("lastSender", ())?;
    /// multicall
    ///     .clear_calls()
    ///     .add_call(call_1, false)
    ///     .add_call(call_2, false);
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
    /// # use ethers_core::types::{U256, Address};
    /// # use ethers_providers::{Provider, Http};
    /// # use ethers_contract::Multicall;
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
    /// # Panics
    ///
    /// If more than the maximum number of supported calls are added. The maximum
    /// limits is constrained due to tokenization/detokenization support for tuples
    ///
    /// Note: this method _does not_ send a transaction from your account
    ///
    /// [`ContractError<M>`]: crate::ContractError<M>
    pub async fn call<D: Detokenize>(&self) -> Result<D, ContractError<M>> {
        assert!(self.calls.len() < 16, "Cannot decode more than 16 calls");
        let tokens = self.call_raw().await?;
        let tokens = vec![Token::Tuple(tokens)];
        let data = D::from_tokens(tokens)?;
        Ok(data)
    }

    /// Queries the Ethereum blockchain via an `eth_call`, but via the Multicall contract and
    /// without detokenization.
    ///
    /// It returns a [`ContractError<M>`] if there is any error in the RPC call.
    ///
    /// ```no_run
    /// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
    /// # use ethers_core::types::{U256, Address};
    /// # use ethers_providers::{Provider, Http};
    /// # use ethers_contract::Multicall;
    /// # use std::convert::TryFrom;
    /// #
    /// # let client = Provider::<Http>::try_from("http://localhost:8545")?;
    /// #
    /// # let multicall = Multicall::new(client, None).await?;
    /// // The consumer of the API is responsible for detokenizing the results
    /// // as the results will be a Vec<Token>
    /// let tokens = multicall.call_raw().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Note: this method _does not_ send a transaction from your account
    ///
    /// [`ContractError<M>`]: crate::ContractError<M>
    pub async fn call_raw(&self) -> Result<Vec<Token>, ContractError<M>> {
        // Different call result types based on version
        let tokens: Vec<Token> = match self.version {
            MulticallVersion::Multicall => {
                let call = self.as_aggregate();
                let (_, return_data) = call.call().await?;
                self.calls
                    .iter()
                    .zip(&return_data)
                    .map(|(call, bytes)| {
                        let mut tokens: Vec<Token> = call.function.decode_output(bytes.as_ref())?;
                        Ok(match tokens.len() {
                            0 => Token::Tuple(vec![]),
                            1 => tokens.remove(0),
                            _ => Token::Tuple(tokens),
                        })
                    })
                    .collect::<Result<Vec<Token>, ContractError<M>>>()?
            }
            // Same result type (`MulticallResult`)
            v @ (MulticallVersion::Multicall2 | MulticallVersion::Multicall3) => {
                let is_v2 = v as u8 == 2;
                let call = if is_v2 { self.as_try_aggregate() } else { self.as_aggregate_3() };
                let return_data = call.call().await?;
                self.calls
                    .iter()
                    .zip(&return_data)
                    .map(|(call, res)| {
                        let ret = &res.return_data;
                        let res_token: Token = if res.success {
                            // Decode using call.function
                            let mut res_tokens = call.function.decode_output(ret)?;
                            match res_tokens.len() {
                                0 => Token::Tuple(vec![]),
                                1 => res_tokens.remove(0),
                                _ => Token::Tuple(res_tokens),
                            }
                        } else {
                            // Call reverted

                            // In v2 (`tryAggregate`) a call might revert even if it was not allowed
                            // by `call.allow_failure`, because in the contract this is not checked
                            // on a per-call basis, but on a per-transaction basis, which we set to
                            // true if *any* `allow_failure` is true in the calls vector in
                            // `as_try_aggregate`.
                            if !call.allow_failure {
                                return Err(ContractError::ProviderError(
                                    ProviderError::CustomError(format!(
                                        "Illegal revert.\n{:?}\n{:?}",
                                        call, res
                                    )),
                                ))
                            }

                            // "Error(string)" (0x08c379a0)
                            if ret.len() >= 4 && ret[..4] == [0x08, 0xc3, 0x79, 0xa0] {
                                decode(&[ParamType::String], &ret[4..])?.remove(0)
                            } else if ret.is_empty() {
                                Token::String(String::new())
                            } else {
                                Token::Bytes(ret.to_vec())
                            }
                        };
                        // (bool, (...))
                        Ok(Token::Tuple(vec![Token::Bool(res.success), res_token]))
                    })
                    .collect::<Result<Vec<Token>, ContractError<M>>>()?
            }
        };

        Ok(tokens)
    }

    /// Signs and broadcasts a batch of transactions by using the Multicall contract as proxy.
    ///
    /// ```no_run
    /// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
    /// # use ethers_providers::{Provider, Http};
    /// # use ethers_contract::Multicall;
    /// # use std::convert::TryFrom;
    /// # let client = Provider::<Http>::try_from("http://localhost:8545")?;
    /// # let multicall = Multicall::new(client, None).await?;
    /// let tx_hash = multicall.send().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Note: this method sends a transaction from your account, and will return an error
    /// if you do not have sufficient funds for gas or value.
    pub async fn send(&self) -> Result<TxHash, ContractError<M>> {
        // Broadcast transaction and return the transaction hash
        // TODO: Can we make this return a PendingTransaction directly instead?
        // Seems hard due to `returns a value referencing data owned by the current function`

        // running clippy --fix on this throws E0597
        #[allow(clippy::let_and_return)]
        let tx_hash = match self.version {
            MulticallVersion::Multicall => {
                let call = self.as_aggregate();
                let hash = *call.send().await?;
                hash
            }
            MulticallVersion::Multicall2 => {
                let call = self.as_try_aggregate();
                let hash = *call.send().await?;
                hash
            }
            MulticallVersion::Multicall3 => {
                let call = self.as_aggregate_3_value();
                let hash = *call.send().await?;
                hash
            }
        };

        Ok(tx_hash)
    }

    /// Multicall1
    fn as_aggregate(&self) -> ContractCall<M, (U256, Vec<Bytes>)> {
        // Map the calls vector into appropriate types for `aggregate` function
        let calls: Vec<Multicall1Call> = self
            .calls
            .iter()
            .map(|call| Multicall1Call { target: call.target, call_data: call.data.clone() })
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

    /// Multicall2
    fn as_try_aggregate(&self) -> ContractCall<M, Vec<MulticallResult>> {
        let mut allow_failure = false;
        // Map the calls vector into appropriate types for `try_aggregate` function
        let calls: Vec<Multicall1Call> = self
            .calls
            .iter()
            .map(|call| {
                // Allow entire call failure if at least one call is allowed to fail.
                // To avoid iterating multiple times, equivalent of:
                // self.calls.iter().any(|call| call.allow_failure)
                allow_failure = allow_failure || call.allow_failure;
                Multicall1Call { target: call.target, call_data: call.data.clone() }
            })
            .collect();

        // Construct the ContractCall for `try_aggregate` function to broadcast the transaction
        let mut contract_call = self.contract.try_aggregate(!allow_failure, calls);

        if let Some(block) = self.block {
            contract_call = contract_call.block(block)
        };

        if self.legacy {
            contract_call = contract_call.legacy();
        };

        contract_call
    }

    /// Multicall3
    fn as_aggregate_3(&self) -> ContractCall<M, Vec<MulticallResult>> {
        // Map the calls vector into appropriate types for `aggregate_3` function
        let calls: Vec<Multicall3Call> = self
            .calls
            .iter()
            .map(|call| Multicall3Call {
                target: call.target,
                call_data: call.data.clone(),
                allow_failure: call.allow_failure,
            })
            .collect();

        // Construct the ContractCall for `aggregate_3` function to broadcast the transaction
        let mut contract_call = self.contract.aggregate_3(calls);

        if let Some(block) = self.block {
            contract_call = contract_call.block(block)
        };

        if self.legacy {
            contract_call = contract_call.legacy();
        };

        contract_call
    }

    /// Multicall3 + values (only .send())
    fn as_aggregate_3_value(&self) -> ContractCall<M, Vec<MulticallResult>> {
        // Map the calls vector into appropriate types for `aggregate_3_value` function
        let mut total_value = U256::zero();
        let calls: Vec<Multicall3CallValue> = self
            .calls
            .iter()
            .map(|call| {
                total_value += call.value;
                Multicall3CallValue {
                    target: call.target,
                    call_data: call.data.clone(),
                    allow_failure: call.allow_failure,
                    value: call.value,
                }
            })
            .collect();

        if total_value.is_zero() {
            // No value is being sent
            self.as_aggregate_3()
        } else {
            // Construct the ContractCall for `aggregate_3_value` function to broadcast the
            // transaction
            let mut contract_call = self.contract.aggregate_3_value(calls);

            if let Some(block) = self.block {
                contract_call = contract_call.block(block)
            };

            if self.legacy {
                contract_call = contract_call.legacy();
            };

            contract_call.value(total_value)
        }
    }
}
