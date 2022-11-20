use ethers_core::{
    abi::{AbiDecode, Detokenize, Function, Token},
    types::{Address, BlockNumber, Bytes, Chain, NameOrAddress, TxHash, H160, U256},
};
use ethers_providers::Middleware;

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

/// The Multicall3 contract address that is deployed in [`MULTICALL_SUPPORTED_CHAIN_IDS`]:
/// [`0xcA11bde05977b3631167028862bE2a173976CA11`](https://etherscan.io/address/0xcA11bde05977b3631167028862bE2a173976CA11)
pub const MULTICALL_ADDRESS: Address = H160([
    0xca, 0x11, 0xbd, 0xe0, 0x59, 0x77, 0xb3, 0x63, 0x11, 0x67, 0x02, 0x88, 0x62, 0xbe, 0x2a, 0x17,
    0x39, 0x76, 0xca, 0x11,
]);

/// The chain IDs that [`MULTICALL_ADDRESS`] has been deployed to.
/// Taken from: <https://github.com/mds1/multicall#multicall3-contract-addresses>
pub static MULTICALL_SUPPORTED_CHAIN_IDS: Lazy<[U256; 48]> = Lazy::new(|| {
    use Chain::*;
    [
        U256::from(Mainnet),                  // Mainnet
        U256::from(Kovan),                    // Kovan
        U256::from(Rinkeby),                  // Rinkeby
        U256::from(Goerli),                   // Goerli
        U256::from(Ropsten),                  // Ropsten
        U256::from(Sepolia),                  // Sepolia
        U256::from(Optimism),                 // Optimism
        U256::from(OptimismGoerli),           // OptimismGoerli
        U256::from(OptimismKovan),            // OptimismKovan
        U256::from(Arbitrum),                 // Arbitrum
        U256::from(ArbitrumGoerli),           // ArbitrumGoerli,
        U256::from(ArbitrumTestnet),          // Arbitrum Rinkeby
        U256::from(Polygon),                  // Polygon
        U256::from(PolygonMumbai),            // PolygonMumbai
        U256::from(XDai),                     // XDai
        U256::from(Chiado),                   // ChiadoTestnet
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

#[derive(Debug, thiserror::Error)]
pub enum MulticallError<M: Middleware> {
    #[error(transparent)]
    ContractError(#[from] ContractError<M>),

    #[error("Chain ID {0} is currently not supported by Multicall. Provide an address instead.")]
    InvalidChainId(U256),

    #[error("Illegal revert: Multicall2 call reverted when it wasn't allowed to.")]
    IllegalRevert,
}

pub type Result<T, M> = std::result::Result<T, MulticallError<M>>;

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
    fn try_from(v: u8) -> std::result::Result<Self, Self::Error> {
        match v {
            1 => Ok(MulticallVersion::Multicall),
            2 => Ok(MulticallVersion::Multicall2),
            3 => Ok(MulticallVersion::Multicall3),
            _ => Err(format!("Invalid Multicall version: {v}. Accepted values: 1, 2, 3.")),
        }
    }
}

/// A Multicall is an abstraction for sending batched calls/transactions to the Ethereum blockchain.
/// It stores an instance of the [`Multicall` smart contract](https://etherscan.io/address/0xcA11bde05977b3631167028862bE2a173976CA11#code)
/// and the user provided list of transactions to be called or executed on chain.
///
/// `Multicall` can be instantiated asynchronously from the chain ID of the provided client using
/// [`new`] or synchronously by providing a chain ID in [`new_with_chain`]. This, by default, uses
/// [`MULTICALL_ADDRESS`], but can be overridden by providing `Some(address)`.
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
/// let client = Provider::<Http>::try_from("https://kovan.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27")?;
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
/// // Since this example connects to the Kovan testnet, we need not provide an address for
/// // the Multicall contract and we set that to `None`. If you wish to provide the address
/// // for the Multicall contract, you can pass the `Some(multicall_addr)` argument.
/// // Construction of the `Multicall` instance follows the builder pattern:
/// let mut multicall = Multicall::new(client.clone(), None).await?.version(MulticallVersion::Multicall);
/// multicall
///     .add_call(first_call, false)
///     .add_call(second_call, false);
///
/// // `await`ing on the `call` method lets us fetch the return values of both the above calls
/// // in one single RPC call
/// let _return_data: (String, Address) = multicall.call().await?;
///
/// // using Multicall2 (version 2) or Multicall3 (version 3) differs when parsing `.call()` results
/// multicall = multicall.version(MulticallVersion::Multicall3);
///
/// // each call returns the results in a tuple, with the success status as the first element
/// let _return_data: ((bool, String), (bool, Address)) = multicall.call().await?;
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
/// let tx_hash = multicall.send().await?;
/// let _tx_receipt = PendingTransaction::new(tx_hash, &client).await?;
///
/// // you can also query ETH balances of multiple addresses
/// let address_1 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".parse::<Address>()?;
/// let address_2 = "ffffffffffffffffffffffffffffffffffffffff".parse::<Address>()?;
///
/// // using version 1
/// multicall = multicall.version(MulticallVersion::Multicall);
/// multicall
///     .clear_calls()
///     .add_get_eth_balance(address_1, false)
///     .add_get_eth_balance(address_2, false);
/// let _balances: (U256, U256) = multicall.call().await?;
///
/// // or with version 2 and above
/// multicall = multicall.version(MulticallVersion::Multicall3);
/// multicall
///     .clear_calls()
///     .add_get_eth_balance(address_1, false)
///     .add_get_eth_balance(address_2, false);
/// let _balances: ((bool, U256), (bool, U256)) = multicall.call().await?;
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
#[derive(Clone)]
#[must_use = "Multicall does nothing unless you use `call` or `send`"]
pub struct Multicall<M> {
    version: MulticallVersion,
    legacy: bool,
    block: Option<BlockNumber>,
    calls: Vec<Call>,
    contract: MulticallContract<M>,
}

// Manually implement Debug due to Middleware trait bounds.
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

impl<M: Middleware> Multicall<M> {
    /// Creates a new Multicall instance from the provided client. If provided with an `address`,
    /// it instantiates the Multicall contract with that address, otherwise it defaults to
    /// [`MULTICALL_ADDRESS`].
    ///
    /// # Errors
    ///
    /// Returns a [`MulticallError`] if the provider returns an error while getting
    /// `network_version`.
    ///
    /// # Panics
    ///
    /// If a `None` address is provided and the client's network is
    /// [not supported](MULTICALL_SUPPORTED_CHAIN_IDS).
    pub async fn new(client: impl Into<Arc<M>>, address: Option<Address>) -> Result<Self, M> {
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
                    return Err(MulticallError::InvalidChainId(chain_id))
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
    /// # Errors
    ///
    /// Returns a [`MulticallError`] if the provided chain_id is not in the
    /// [supported networks](MULTICALL_SUPPORTED_CHAIN_IDS).
    ///
    /// # Panics
    ///
    /// If neither an address or chain_id are provided. Since this is not an async function, it will
    /// not be able to query `net_version` to check if it is supported by the default multicall
    /// address. Use new(client, None).await instead.
    pub fn new_with_chain_id(
        client: impl Into<Arc<M>>,
        address: Option<Address>,
        chain_id: Option<impl Into<U256>>,
    ) -> Result<Self, M> {
        // If no address is provided, check if chain_id is supported and use the default multicall
        // address.
        let address: Address = match address {
            Some(addr) => addr,
            None => {
                // Can't fetch chain_id from provider since we're not in an async function so we
                // panic instead.
                let chain_id =
                    chain_id.expect("Must provide at least one of: address or chain ID.").into();
                if !MULTICALL_SUPPORTED_CHAIN_IDS.contains(&chain_id) {
                    return Err(MulticallError::InvalidChainId(chain_id))
                }
                MULTICALL_ADDRESS
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
    /// Note: all these versions are available in the same contract address ([`MULTICALL_ADDRESS`])
    /// so changing version just changes the methods used, not the contract address.
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
    /// - 1: `allow_failure` is ignored.
    /// - >=2: `allow_failure` specifies whether or not this call is allowed to revert in the
    ///   multicall.
    /// - 3: Transaction values are used when broadcasting transactions with [`send`], otherwise
    ///   they are always ignored.
    ///
    /// [`send`]: #method.send
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
                    value: call.tx.value().cloned().unwrap_or_default(),
                    allow_failure,
                    function: call.function,
                };
                self.calls.push(call);
                self
            }
            _ => self,
        }
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
    /// let _tx_hash = multicall.send().await?;
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
    /// Note: this method _does not_ send a transaction from your account.
    ///
    /// # Errors
    ///
    /// Returns a [`MulticallError`] if there are any errors in the RPC call or while detokenizing
    /// the tokens back to the expected return type.
    ///
    /// # Panics
    ///
    /// If more than the maximum number of supported calls are added (16). The maximum limit is
    /// constrained due to tokenization/detokenization support for tuples.
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
    /// // If the Solidity function calls has the following return types:
    /// // 1. `returns (uint256)`
    /// // 2. `returns (string, address)`
    /// // 3. `returns (bool)`
    /// // Version 1:
    /// let result: (U256, (String, Address), bool) = multicall.call().await?;
    /// // Version 2 and above (each call returns also the success status as the first element):
    /// let result: ((bool, U256), (bool, (String, Address)), (bool, bool)) = multicall.call().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn call<D: Detokenize>(&self) -> Result<D, M> {
        assert!(self.calls.len() < 16, "Cannot decode more than 16 calls");
        let tokens = self.call_raw().await?;
        let tokens = vec![Token::Tuple(tokens)];
        let data = D::from_tokens(tokens).map_err(ContractError::DetokenizationError)?;
        Ok(data)
    }

    /// Queries the Ethereum blockchain using `eth_call`, but via the Multicall contract and
    /// without detokenization.
    ///
    /// # Errors
    ///
    /// Returns a [`MulticallError`] if there are any errors in the RPC call.
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
    /// // as the results will be a Vec<Token>
    /// let tokens = multicall.call_raw().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Note: this method _does not_ send a transaction from your account
    ///
    /// [`ContractError<M>`]: crate::ContractError<M>
    pub async fn call_raw(&self) -> Result<Vec<Token>, M> {
        // Different call result types based on version
        let tokens: Vec<Token> = match self.version {
            MulticallVersion::Multicall => {
                let call = self.as_aggregate();
                let (_, return_data) = call.call().await?;
                self.calls
                    .iter()
                    .zip(&return_data)
                    .map(|(call, bytes)| {
                        let mut tokens: Vec<Token> = call
                            .function
                            .decode_output(bytes.as_ref())
                            .map_err(ContractError::DecodingError)?;
                        Ok(match tokens.len() {
                            0 => Token::Tuple(vec![]),
                            1 => tokens.remove(0),
                            _ => Token::Tuple(tokens),
                        })
                    })
                    .collect::<Result<Vec<Token>, M>>()?
            }
            // Same result type (`MulticallResult`)
            v @ (MulticallVersion::Multicall2 | MulticallVersion::Multicall3) => {
                let is_v2 = v == MulticallVersion::Multicall2;
                let call = if is_v2 { self.as_try_aggregate() } else { self.as_aggregate_3() };
                let return_data = call.call().await?;
                self.calls
                    .iter()
                    .zip(&return_data)
                    .map(|(call, res)| {
                        let ret = &res.return_data;
                        let res_token: Token = if res.success {
                            // Decode using call.function
                            let mut res_tokens = call
                                .function
                                .decode_output(ret)
                                .map_err(ContractError::DecodingError)?;
                            match res_tokens.len() {
                                0 => Token::Tuple(vec![]),
                                1 => res_tokens.remove(0),
                                _ => Token::Tuple(res_tokens),
                            }
                        } else {
                            // Call reverted

                            // v2: In the function call to `tryAggregate`, the `allow_failure` check
                            // is done on a per-transaction basis, and we set this transaction-wide
                            // check to true when *any* call is allowed to fail. If this is true
                            // then a call that is not allowed to revert (`call.allow_failure`) may
                            // still do so because of other calls that are in the same multicall
                            // aggregate.
                            if !call.allow_failure {
                                return Err(MulticallError::IllegalRevert)
                            }

                            // Decode with "Error(string)" (0x08c379a0)
                            if ret.len() >= 4 && ret[..4] == [0x08, 0xc3, 0x79, 0xa0] {
                                Token::String(
                                    String::decode(&ret[4..]).map_err(ContractError::AbiError)?,
                                )
                            } else if ret.is_empty() {
                                Token::String(String::new())
                            } else {
                                Token::Bytes(ret.to_vec())
                            }
                        };
                        // (bool, (...))
                        Ok(Token::Tuple(vec![Token::Bool(res.success), res_token]))
                    })
                    .collect::<Result<Vec<Token>, M>>()?
            }
        };

        Ok(tokens)
    }

    /// Signs and broadcasts a batch of transactions by using the Multicall contract as proxy,
    /// returning the transaction hash once the transaction confirms.
    ///
    /// Note: this method will broadcast a transaction from an account, meaning it must have
    /// sufficient funds for gas and transaction value.
    ///
    /// # Errors
    ///
    /// Returns a [`MulticallError`] if there are any errors in the RPC call.
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
    pub async fn send(&self) -> Result<TxHash, M> {
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

    /// v1
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

    /// v2
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

    /// v3
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

    /// v3 + values (only .send())
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
