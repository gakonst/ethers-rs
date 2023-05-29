use crate::call::{ContractCall, ContractError};
use ethers_core::{
    abi::{Detokenize, Function, Token, Tokenizable},
    types::{
        transaction::eip2718::TypedTransaction, Address, BlockNumber, Bytes, NameOrAddress, U256,
    },
};
use ethers_providers::{Middleware, PendingTransaction};
use std::{convert::TryFrom, fmt, result::Result as StdResult, sync::Arc};

/// The Multicall contract bindings. Auto-generated with `abigen`.
pub mod contract;
pub use contract::Multicall3 as MulticallContract;
use contract::{
    Call as Multicall1Call, Call3 as Multicall3Call, Call3Value as Multicall3CallValue,
    Result as MulticallResult,
};

pub mod constants;

/// Type alias for `Result<T, MulticallError<M>>`
pub type Result<T, M> = StdResult<T, error::MulticallError<M>>;

/// MultiCall error type
pub mod error;

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

/// The version of the [`Multicall`](super::Multicall).
/// Used to determine which methods of the Multicall smart contract to use:
/// - [`Multicall`] : `aggregate((address,bytes)[])`
/// - [`Multicall2`] : `try_aggregate(bool, (address,bytes)[])`
/// - [`Multicall3`] : `aggregate3((address,bool,bytes)[])` or
///   `aggregate3Value((address,bool,uint256,bytes)[])`
///
/// [`Multicall`]: #variant.Multicall
/// [`Multicall2`]: #variant.Multicall2
/// [`Multicall3`]: #variant.Multicall3
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum MulticallVersion {
    /// V1
    Multicall = 1,
    /// V2
    Multicall2 = 2,
    /// V3
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
    fn try_from(v: u8) -> StdResult<Self, Self::Error> {
        match v {
            1 => Ok(MulticallVersion::Multicall),
            2 => Ok(MulticallVersion::Multicall2),
            3 => Ok(MulticallVersion::Multicall3),
            _ => Err(format!("Invalid Multicall version: {v}. Accepted values: 1, 2, 3.")),
        }
    }
}

impl MulticallVersion {
    /// True if call is v1
    #[inline]
    pub fn is_v1(&self) -> bool {
        matches!(self, Self::Multicall)
    }

    /// True if call is v2
    #[inline]
    pub fn is_v2(&self) -> bool {
        matches!(self, Self::Multicall2)
    }

    /// True if call is v3
    #[inline]
    pub fn is_v3(&self) -> bool {
        matches!(self, Self::Multicall3)
    }
}

/// A Multicall is an abstraction for sending batched calls/transactions to the Ethereum blockchain.
/// It stores an instance of the [`Multicall` smart contract](https://etherscan.io/address/0xcA11bde05977b3631167028862bE2a173976CA11#code)
/// and the user provided list of transactions to be called or executed on chain.
///
/// `Multicall` can be instantiated asynchronously from the chain ID of the provided client using
/// [`new`] or synchronously by providing a chain ID in [`new_with_chain`]. This, by default, uses
/// [`constants::MULTICALL_ADDRESS`], but can be overridden by providing `Some(address)`.
/// A list of all the supported chains is available [`here`](https://github.com/mds1/multicall#multicall3-contract-addresses).
///
/// Set the contract's version by using [`version`].
///
/// The `block` number can be provided for the call by using [`block`].
///
/// Transactions default to `EIP1559`. This can be changed by using [`legacy`].
///
/// Build on the `Multicall` instance by adding calls using [`add_call`] and call or broadcast them
/// all at once by using [`call`] and [`send`] respectively.
///
/// # Example
///
/// Using Multicall (version 1):
///
/// ```no_run
/// use ethers_core::{
///     abi::Abi,
///     types::{Address, H256, U256},
/// };
/// use ethers_contract::{Contract, Multicall, MulticallVersion};
/// use ethers_providers::{Middleware, Http, Provider, PendingTransaction};
/// use std::{convert::TryFrom, sync::Arc};
///
/// # async fn bar() -> Result<(), Box<dyn std::error::Error>> {
/// // this is a dummy address used for illustration purposes
/// let address = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".parse::<Address>()?;
///
/// // (ugly way to write the ABI inline, you can otherwise read it from a file)
/// let abi: Abi = serde_json::from_str(r#"[{"inputs":[{"internalType":"string","name":"value","type":"string"}],"stateMutability":"nonpayable","type":"constructor"},{"anonymous":false,"inputs":[{"indexed":true,"internalType":"address","name":"author","type":"address"},{"indexed":true,"internalType":"address","name":"oldAuthor","type":"address"},{"indexed":false,"internalType":"string","name":"oldValue","type":"string"},{"indexed":false,"internalType":"string","name":"newValue","type":"string"}],"name":"ValueChanged","type":"event"},{"inputs":[],"name":"getValue","outputs":[{"internalType":"string","name":"","type":"string"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"lastSender","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"string","name":"value","type":"string"}],"name":"setValue","outputs":[],"stateMutability":"nonpayable","type":"function"}]"#)?;
///
/// // connect to the network
/// let client = Provider::<Http>::try_from("http://localhost:8545")?;
///
/// // create the contract object. This will be used to construct the calls for multicall
/// let client = Arc::new(client);
/// let contract = Contract::<Provider<Http>>::new(address, abi, client.clone());
///
/// // note that these [`ContractCall`]s are futures, and need to be `.await`ed to resolve.
/// // But we will let `Multicall` to take care of that for us
/// let first_call = contract.method::<_, String>("getValue", ())?;
/// let second_call = contract.method::<_, Address>("lastSender", ())?;
///
/// // Since this example connects to a known chain, we need not provide an address for
/// // the Multicall contract and we set that to `None`. If you wish to provide the address
/// // for the Multicall contract, you can pass the `Some(multicall_addr)` argument.
/// // Construction of the `Multicall` instance follows the builder pattern:
/// let mut multicall = Multicall::new(client.clone(), None).await?;
/// multicall
///     .add_call(first_call, false)
///     .add_call(second_call, false);
///
/// // `await`ing on the `call` method lets us fetch the return values of both the above calls
/// // in one single RPC call
/// let return_data: (String, Address) = multicall.call().await?;
///
/// // the same `Multicall` instance can be re-used to do a different batch of transactions.
/// // Say we wish to broadcast (send) a couple of transactions via the Multicall contract.
/// let first_broadcast = contract.method::<_, H256>("setValue", "some value".to_owned())?;
/// let second_broadcast = contract.method::<_, H256>("setValue", "new value".to_owned())?;
/// multicall
///     .clear_calls()
///     .add_call(first_broadcast, false)
///     .add_call(second_broadcast, false);
///
/// // `await`ing the `send` method waits for the transaction to be broadcast, which also
/// // returns the transaction hash
/// let tx_receipt = multicall.send().await?.await.expect("tx dropped");
///
/// // you can also query ETH balances of multiple addresses
/// let address_1 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".parse::<Address>()?;
/// let address_2 = "ffffffffffffffffffffffffffffffffffffffff".parse::<Address>()?;
///
/// multicall
///     .clear_calls()
///     .add_get_eth_balance(address_1, false)
///     .add_get_eth_balance(address_2, false);
/// let balances: (U256, U256) = multicall.call().await?;
///
/// # Ok(())
/// # }
/// ```
///
/// [`new`]: #method.new
/// [`new_with_chain`]: #method.new_with_chain
/// [`version`]: #method.version
/// [`block`]: #method.block
/// [`legacy`]: #method.legacy
/// [`add_call`]: #method.add_call
/// [`call`]: #method.call
/// [`send`]: #method.send
#[must_use = "Multicall does nothing unless you use `call` or `send`"]
pub struct Multicall<M> {
    /// The Multicall contract interface.
    pub contract: MulticallContract<M>,

    /// The version of which methods to use when making the contract call.
    pub version: MulticallVersion,

    /// Whether to use a legacy or a EIP-1559 transaction.
    pub legacy: bool,

    /// The `block` field of the Multicall aggregate call.
    pub block: Option<BlockNumber>,

    /// The internal call vector.
    calls: Vec<Call>,
}

// Manually implement Clone and Debug to avoid trait bounds.
impl<M> Clone for Multicall<M> {
    fn clone(&self) -> Self {
        Self {
            contract: self.contract.clone(),
            version: self.version,
            legacy: self.legacy,
            block: self.block,
            calls: self.calls.clone(),
        }
    }
}

impl<M> fmt::Debug for Multicall<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Multicall")
            .field("address", &self.contract.address())
            .field("version", &self.version)
            .field("legacy", &self.legacy)
            .field("block", &self.block)
            .field("calls", &self.calls)
            .finish()
    }
}

impl<M: Middleware> Multicall<M> {
    /// Creates a new Multicall instance from the provided client. If provided with an `address`,
    /// it instantiates the Multicall contract with that address, otherwise it defaults to
    /// [`constants::MULTICALL_ADDRESS`].
    ///
    /// # Errors
    ///
    /// Returns a [`error::MulticallError`] if the provider returns an error while getting
    /// `network_version`.
    ///
    /// # Panics
    ///
    /// If a `None` address is provided and the client's network is
    /// [not supported](constants::MULTICALL_SUPPORTED_CHAIN_IDS).
    pub async fn new(client: impl Into<Arc<M>>, address: Option<Address>) -> Result<Self, M> {
        let client = client.into();

        // Fetch chain id and the corresponding address of Multicall contract
        // preference is given to Multicall contract's address if provided
        // otherwise check the supported chain IDs for the client's chain ID
        let address: Address = match address {
            Some(addr) => addr,
            None => {
                let chain_id = client
                    .get_chainid()
                    .await
                    .map_err(ContractError::from_middleware_error)?
                    .as_u64();
                if !constants::MULTICALL_SUPPORTED_CHAIN_IDS.contains(&chain_id) {
                    return Err(error::MulticallError::InvalidChainId(chain_id))
                }
                constants::MULTICALL_ADDRESS
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
    /// ID. Uses the [default multicall address](constants::MULTICALL_ADDRESS) if no address is
    /// provided.
    ///
    /// # Errors
    ///
    /// Returns a [`error::MulticallError`] if the provided chain_id is not in the
    /// [supported networks](constants::MULTICALL_SUPPORTED_CHAIN_IDS).
    ///
    /// # Panics
    ///
    /// If neither an address or chain_id are provided. Since this is not an async function, it will
    /// not be able to query `net_version` to check if it is supported by the default multicall
    /// address. Use new(client, None).await instead.
    pub fn new_with_chain_id(
        client: impl Into<Arc<M>>,
        address: Option<Address>,
        chain_id: Option<impl Into<u64>>,
    ) -> Result<Self, M> {
        // If no address is provided, check if chain_id is supported and use the default multicall
        // address.
        let address: Address = match (address, chain_id) {
            (Some(addr), _) => addr,
            (_, Some(chain_id)) => {
                let chain_id = chain_id.into();
                if !constants::MULTICALL_SUPPORTED_CHAIN_IDS.contains(&chain_id) {
                    return Err(error::MulticallError::InvalidChainId(chain_id))
                }
                constants::MULTICALL_ADDRESS
            }
            _ => {
                // Can't fetch chain_id from provider since we're not in an async function so we
                // panic instead.
                panic!("Must provide at least one of: address or chain ID.")
            }
        };

        // Instantiate the multicall contract
        let contract = MulticallContract::new(address, client.into());

        Ok(Self {
            version: MulticallVersion::Multicall3,
            legacy: false,
            block: None,
            calls: vec![],
            contract,
        })
    }

    /// Changes which functions to use when making the contract call. The default is 3. Version
    /// differences (adapted from [here](https://github.com/mds1/multicall#multicall---)):
    ///
    /// - Multicall (v1): This is the recommended version for simple calls. The original contract
    /// containing an aggregate method to batch calls. Each call returns only the return data and
    /// none are allowed to fail.
    ///
    /// - Multicall2 (v2): The same as Multicall, but provides additional methods that allow either
    /// all or no calls within the batch to fail. Included for backward compatibility. Use v3 to
    /// allow failure on a per-call basis.
    ///
    /// - Multicall3 (v3): This is the recommended version for allowing failing calls. It's cheaper
    /// to use (so you can fit more calls into a single request), and it adds an aggregate3 method
    /// so you can specify whether calls are allowed to fail on a per-call basis.
    ///
    /// Note: all these versions are available in the same contract address
    /// ([`constants::MULTICALL_ADDRESS`]) so changing version just changes the methods used,
    /// not the contract address.
    pub fn version(mut self, version: MulticallVersion) -> Self {
        self.version = version;
        self
    }

    /// Makes a legacy transaction instead of an EIP-1559 one.
    pub fn legacy(mut self) -> Self {
        self.legacy = true;
        self
    }

    /// Sets the `block` field of the Multicall aggregate call.
    pub fn block(mut self, block: impl Into<BlockNumber>) -> Self {
        self.block = Some(block.into());
        self
    }

    /// Appends a `call` to the list of calls of the Multicall instance.
    ///
    /// Version specific details:
    /// - `1`: `allow_failure` is ignored.
    /// - `>=2`: `allow_failure` specifies whether or not this call is allowed to revert in the
    ///   multicall.
    /// - `3`: Transaction values are used when broadcasting transactions with [`send`], otherwise
    ///   they are always ignored.
    ///
    /// [`send`]: #method.send
    pub fn add_call<D: Detokenize>(
        &mut self,
        call: ContractCall<M, D>,
        allow_failure: bool,
    ) -> &mut Self {
        let (to, data, value) = match call.tx {
            TypedTransaction::Legacy(tx) => (tx.to, tx.data, tx.value),
            TypedTransaction::Eip2930(tx) => (tx.tx.to, tx.tx.data, tx.tx.value),
            TypedTransaction::Eip1559(tx) => (tx.to, tx.data, tx.value),
            #[cfg(feature = "optimism")]
            TypedTransaction::OptimismDeposited(tx) => (tx.tx.to, tx.tx.data, tx.tx.value),
        };
        if data.is_none() && !call.function.outputs.is_empty() {
            return self
        }
        if let Some(NameOrAddress::Address(target)) = to {
            let call = Call {
                target,
                data: data.unwrap_or_default(),
                value: value.unwrap_or_default(),
                allow_failure,
                function: call.function,
            };
            self.calls.push(call);
        }
        self
    }

    /// Appends multiple `call`s to the list of calls of the Multicall instance.
    ///
    /// See [`add_call`] for more details.
    ///
    /// [`add_call`]: #method.add_call
    pub fn add_calls<D: Detokenize>(
        &mut self,
        allow_failure: bool,
        calls: impl IntoIterator<Item = ContractCall<M, D>>,
    ) -> &mut Self {
        for call in calls {
            self.add_call(call, allow_failure);
        }
        self
    }

    /// Appends a `call` to the list of calls of the Multicall instance for querying the block hash
    /// of a given block number.
    ///
    /// Note: this call will return 0 if `block_number` is not one of the most recent 256 blocks.
    /// ([Reference](https://docs.soliditylang.org/en/latest/units-and-global-variables.html?highlight=blockhash#block-and-transaction-properties))
    pub fn add_get_block_hash(&mut self, block_number: impl Into<U256>) -> &mut Self {
        let call = self.contract.get_block_hash(block_number.into());
        self.add_call(call, false)
    }

    /// Appends a `call` to the list of calls of the Multicall instance for querying the current
    /// block number.
    pub fn add_get_block_number(&mut self) -> &mut Self {
        let call = self.contract.get_block_number();
        self.add_call(call, false)
    }

    /// Appends a `call` to the list of calls of the Multicall instance for querying the current
    /// block coinbase address.
    pub fn add_get_current_block_coinbase(&mut self) -> &mut Self {
        let call = self.contract.get_current_block_coinbase();
        self.add_call(call, false)
    }

    /// Appends a `call` to the list of calls of the Multicall instance for querying the current
    /// block difficulty.
    ///
    /// Note: in a post-merge environment, the return value of this call will be the output of the
    /// randomness beacon provided by the beacon chain.
    /// ([Reference](https://eips.ethereum.org/EIPS/eip-4399#abstract))
    pub fn add_get_current_block_difficulty(&mut self) -> &mut Self {
        let call = self.contract.get_current_block_difficulty();
        self.add_call(call, false)
    }

    /// Appends a `call` to the list of calls of the Multicall instance for querying the current
    /// block gas limit.
    pub fn add_get_current_block_gas_limit(&mut self) -> &mut Self {
        let call = self.contract.get_current_block_gas_limit();
        self.add_call(call, false)
    }

    /// Appends a `call` to the list of calls of the Multicall instance for querying the current
    /// block timestamp.
    pub fn add_get_current_block_timestamp(&mut self) -> &mut Self {
        let call = self.contract.get_current_block_timestamp();
        self.add_call(call, false)
    }

    /// Appends a `call` to the list of calls of the Multicall instance for querying the ETH
    /// balance of an address.
    pub fn add_get_eth_balance(
        &mut self,
        address: impl Into<Address>,
        allow_failure: bool,
    ) -> &mut Self {
        let call = self.contract.get_eth_balance(address.into());
        self.add_call(call, allow_failure)
    }

    /// Appends a `call` to the list of calls of the Multicall instance for querying the last
    /// block hash.
    pub fn add_get_last_block_hash(&mut self) -> &mut Self {
        let call = self.contract.get_last_block_hash();
        self.add_call(call, false)
    }

    /// Appends a `call` to the list of calls of the Multicall instance for querying the current
    /// block base fee.
    ///
    /// Note: this call will fail if the chain that it is called on does not implement the
    /// [BASEFEE opcode](https://eips.ethereum.org/EIPS/eip-3198).
    pub fn add_get_basefee(&mut self, allow_failure: bool) -> &mut Self {
        let call = self.contract.get_basefee();
        self.add_call(call, allow_failure)
    }

    /// Appends a `call` to the list of calls of the Multicall instance for querying the chain id.
    pub fn add_get_chain_id(&mut self) -> &mut Self {
        let call = self.contract.get_chain_id();
        self.add_call(call, false)
    }

    /// Clears the batch of calls from the Multicall instance.
    /// Re-use the already instantiated Multicall to send a different batch of transactions or do
    /// another aggregate query.
    ///
    /// # Examples
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
    /// let _tx_receipt = multicall.send().await?.await.expect("tx dropped");
    ///
    /// # let call_1 = contract.method::<_, String>("getValue", ())?;
    /// # let call_2 = contract.method::<_, Address>("lastSender", ())?;
    /// multicall
    ///     .clear_calls()
    ///     .add_call(call_1, false)
    ///     .add_call(call_2, false);
    /// // Version 1:
    /// let return_data: (String, Address) = multicall.call().await?;
    /// // Version 2 and above (each call returns also the success status as the first element):
    /// let return_data: ((bool, String), (bool, Address)) = multicall.call().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn clear_calls(&mut self) -> &mut Self {
        self.calls.clear();
        self
    }

    /// Queries the Ethereum blockchain using `eth_call`, but via the Multicall contract.
    ///
    /// For handling calls that have the same result type, see [`call_array`].
    ///
    /// For handling each call's result individually, see [`call_raw`].
    ///
    /// [`call_raw`]: #method.call_raw
    /// [`call_array`]: #method.call_array
    ///
    /// # Errors
    ///
    /// Returns a [`error::MulticallError`] if there are any errors in the RPC call or while
    /// detokenizing the tokens back to the expected return type.
    ///
    /// Returns an error if any call failed, even if `allow_failure` was set, or if the return data
    /// was empty.
    ///
    /// # Examples
    ///
    /// The return type must be annotated as a tuple when calling this method:
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
    /// // or using the turbofish syntax:
    /// let result = multicall.call::<(U256, (String, Address), bool)>().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn call<T: Tokenizable>(&self) -> Result<T, M> {
        let results = self.call_raw().await?;
        let tokens = results
            .into_iter()
            .map(|res| {
                res.map_err(|data| {
                    error::MulticallError::ContractError(ContractError::Revert(data))
                })
            })
            .collect::<Result<_, _>>()?;
        T::from_token(Token::Tuple(tokens)).map_err(Into::into)
    }

    /// Queries the Ethereum blockchain using `eth_call`, but via the Multicall contract, assuming
    /// that every call returns same type.
    ///
    /// # Errors
    ///
    /// Returns a [`error::MulticallError`] if there are any errors in the RPC call or while
    /// detokenizing the tokens back to the expected return type.
    ///
    /// Returns an error if any call failed, even if `allow_failure` was set, or if the return data
    /// was empty.
    ///
    /// # Examples
    ///
    /// The return type must be annotated while calling this method:
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
    /// // If the all Solidity function calls `returns (uint256)`:
    /// let result: Vec<U256> = multicall.call_array().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn call_array<T: Tokenizable>(&self) -> Result<Vec<T>, M> {
        self.call_raw()
            .await?
            .into_iter()
            .map(|res| {
                res.map_err(|data| {
                    error::MulticallError::ContractError(ContractError::Revert(data))
                })
                .and_then(|token| T::from_token(token).map_err(Into::into))
            })
            .collect()
    }

    /// Queries the Ethereum blockchain using `eth_call`, but via the Multicall contract.
    ///
    /// Returns a vector of `Result<Token, Bytes>` for each call added to the Multicall:
    /// `Err(Bytes)` if the individual call failed while allowed or the return data was empty,
    /// `Ok(Token)` otherwise.
    ///
    /// If the Multicall version is 1, this will always be a vector of `Ok`.
    ///
    /// # Errors
    ///
    /// Returns a [`error::MulticallError`] if there are any errors in the RPC call.
    ///
    /// # Examples
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
    /// let tokens = multicall.call_raw().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn call_raw(&self) -> Result<Vec<StdResult<Token, Bytes>>, M> {
        // Different call result types based on version
        match self.version {
            // Wrap the return data with `success: true` since version 1 reverts if any call failed
            MulticallVersion::Multicall => {
                let call = self.as_aggregate();
                let (_, bytes) = ContractCall::call(&call).await?;
                self.parse_call_result(
                    bytes
                        .into_iter()
                        .map(|return_data| MulticallResult { success: true, return_data }),
                )
            }
            // Same result type (`MulticallResult`)
            MulticallVersion::Multicall2 | MulticallVersion::Multicall3 => {
                let call = if self.version.is_v2() {
                    self.as_try_aggregate()
                } else {
                    self.as_aggregate_3()
                };
                let results = ContractCall::call(&call).await?;
                self.parse_call_result(results.into_iter())
            }
        }
    }

    /// For each call and its `return_data`: if `success` is true, parses `return_data` with the
    /// call's function outputs, otherwise returns the bytes in `Err`.
    fn parse_call_result(
        &self,
        return_data: impl Iterator<Item = MulticallResult>,
    ) -> Result<Vec<StdResult<Token, Bytes>>, M> {
        let mut results = Vec::with_capacity(self.calls.len());
        for (call, MulticallResult { success, return_data }) in self.calls.iter().zip(return_data) {
            let result = if !success || return_data.is_empty() {
                // v2: In the function call to `tryAggregate`, the `allow_failure` check
                // is done on a per-transaction basis, and we set this transaction-wide
                // check to true when *any* call is allowed to fail. If this is true
                // then a call that is not allowed to revert (`call.allow_failure`) may
                // still do so because of other calls that are in the same multicall
                // aggregate.
                if !success && !call.allow_failure {
                    return Err(error::MulticallError::IllegalRevert)
                }

                Err(return_data)
            } else {
                let mut res_tokens = call.function.decode_output(return_data.as_ref())?;
                Ok(if res_tokens.len() == 1 {
                    res_tokens.pop().unwrap()
                } else {
                    Token::Tuple(res_tokens)
                })
            };
            results.push(result);
        }
        Ok(results)
    }

    /// Signs and broadcasts a batch of transactions by using the Multicall contract as proxy,
    /// returning the pending transaction.
    ///
    /// Note: this method will broadcast a transaction from an account, meaning it must have
    /// sufficient funds for gas and transaction value.
    ///
    /// # Errors
    ///
    /// Returns a [`error::MulticallError`] if there are any errors in the RPC call.
    ///
    /// # Examples
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
    pub async fn send(&self) -> Result<PendingTransaction<'_, M::Provider>, M> {
        let tx = match self.version {
            MulticallVersion::Multicall => self.as_aggregate().tx,
            MulticallVersion::Multicall2 => self.as_try_aggregate().tx,
            MulticallVersion::Multicall3 => self.as_aggregate_3_value().tx,
        };
        let client: &M = self.contract.client_ref();
        client.send_transaction(tx, self.block.map(Into::into)).await.map_err(|e| {
            error::MulticallError::ContractError(ContractError::from_middleware_error(e))
        })
    }

    /// v1
    #[inline]
    fn as_aggregate(&self) -> ContractCall<M, (U256, Vec<Bytes>)> {
        // Map the calls vector into appropriate types for `aggregate` function
        let calls: Vec<Multicall1Call> = self
            .calls
            .clone()
            .into_iter()
            .map(|call| Multicall1Call { target: call.target, call_data: call.data })
            .collect();

        // Construct the ContractCall for `aggregate` function to broadcast the transaction
        let contract_call = self.contract.aggregate(calls);

        self.set_call_flags(contract_call)
    }

    /// v2
    #[inline]
    fn as_try_aggregate(&self) -> ContractCall<M, Vec<MulticallResult>> {
        let mut allow_failure = false;
        // Map the calls vector into appropriate types for `try_aggregate` function
        let calls: Vec<Multicall1Call> = self
            .calls
            .clone()
            .into_iter()
            .map(|call| {
                // Allow entire call failure if at least one call is allowed to fail.
                // To avoid iterating multiple times, equivalent of:
                // self.calls.iter().any(|call| call.allow_failure)
                allow_failure |= call.allow_failure;
                Multicall1Call { target: call.target, call_data: call.data }
            })
            .collect();

        // Construct the ContractCall for `try_aggregate` function to broadcast the transaction
        let contract_call = self.contract.try_aggregate(!allow_failure, calls);

        self.set_call_flags(contract_call)
    }

    /// v3
    #[inline]
    fn as_aggregate_3(&self) -> ContractCall<M, Vec<MulticallResult>> {
        // Map the calls vector into appropriate types for `aggregate_3` function
        let calls: Vec<Multicall3Call> = self
            .calls
            .clone()
            .into_iter()
            .map(|call| Multicall3Call {
                target: call.target,
                call_data: call.data,
                allow_failure: call.allow_failure,
            })
            .collect();

        // Construct the ContractCall for `aggregate_3` function to broadcast the transaction
        let contract_call = self.contract.aggregate_3(calls);

        self.set_call_flags(contract_call)
    }

    /// v3 + values (only .send())
    #[inline]
    fn as_aggregate_3_value(&self) -> ContractCall<M, Vec<MulticallResult>> {
        // Map the calls vector into appropriate types for `aggregate_3_value` function
        let mut total_value = U256::zero();
        let calls: Vec<Multicall3CallValue> = self
            .calls
            .clone()
            .into_iter()
            .map(|call| {
                total_value += call.value;
                Multicall3CallValue {
                    target: call.target,
                    call_data: call.data,
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
            let contract_call = self.contract.aggregate_3_value(calls);

            self.set_call_flags(contract_call).value(total_value)
        }
    }

    /// Sets the block and legacy flags on a [ContractCall] if they were set on Multicall.
    fn set_call_flags<D: Detokenize>(&self, mut call: ContractCall<M, D>) -> ContractCall<M, D> {
        if let Some(block) = self.block {
            call.block = Some(block.into());
        }

        if self.legacy {
            call.legacy()
        } else {
            call
        }
    }
}
