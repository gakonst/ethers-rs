use crate::{
    ens,
    http::Provider as HttpProvider,
    stream::{FilterStream, FilterWatcher},
    JsonRpcClient, PendingTransaction,
};

use ethers_core::{
    abi::{self, Detokenize, ParamType},
    types::{
        Address, Block, BlockId, BlockNumber, Bytes, Filter, Log, NameOrAddress, Selector,
        Signature, Transaction, TransactionReceipt, TransactionRequest, TxHash, H256, U256, U64,
    },
    utils,
};

use serde::Deserialize;
use thiserror::Error;
use url::{ParseError, Url};

use std::{convert::TryFrom, fmt::Debug};

/// An abstract provider for interacting with the [Ethereum JSON RPC
/// API](https://github.com/ethereum/wiki/wiki/JSON-RPC). Must be instantiated
/// with a data transport which implements the [`JsonRpcClient`](trait.JsonRpcClient.html) trait
/// (e.g. [HTTP](struct.Http.html), Websockets etc.)
///
/// # Example
///
/// ```no_run
/// # use ethers_providers::JsonRpcClient;
/// use ethers_providers::{Provider, Http};
/// use std::convert::TryFrom;
///
/// let provider = Provider::<Http>::try_from(
///     "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27"
/// ).expect("could not instantiate HTTP Provider");
///
/// # async fn foo<P: JsonRpcClient>(provider: &Provider<P>) -> Result<(), Box<dyn std::error::Error>> {
/// let block = provider.get_block(100u64).await?;
/// println!("Got block: {}", serde_json::to_string(&block)?);
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct Provider<P>(P, Option<Address>);

#[derive(Debug, Error)]
/// An error thrown when making a call to the provider
pub enum ProviderError {
    /// An internal error in the JSON RPC Client
    #[error(transparent)]
    JsonRpcClientError(#[from] Box<dyn std::error::Error + Send + Sync>),

    /// An error during ENS name resolution
    #[error("ens name not found: {0}")]
    EnsError(String),
}

/// Types of filters supported by the JSON-RPC.
#[derive(Clone, Debug)]
pub enum FilterKind<'a> {
    /// `eth_newBlockFilter`
    Logs(&'a Filter),

    /// `eth_newBlockFilter` filter
    NewBlocks,

    /// `eth_newPendingTransactionFilter` filter
    PendingTransactions,
}

// JSON RPC bindings
impl<P: JsonRpcClient> Provider<P> {
    /// Instantiate a new provider with a backend.
    pub fn new(provider: P) -> Self {
        Self(provider, None)
    }

    ////// Blockchain Status
    //
    // Functions for querying the state of the blockchain

    /// Gets the latest block number via the `eth_BlockNumber` API
    pub async fn get_block_number(&self) -> Result<U64, ProviderError> {
        Ok(self
            .0
            .request("eth_blockNumber", ())
            .await
            .map_err(Into::into)?)
    }

    /// Gets the block at `block_hash_or_number` (transaction hashes only)
    pub async fn get_block(
        &self,
        block_hash_or_number: impl Into<BlockId>,
    ) -> Result<Block<TxHash>, ProviderError> {
        Ok(self
            .get_block_gen(block_hash_or_number.into(), false)
            .await?)
    }

    /// Gets the block at `block_hash_or_number` (full transactions included)
    pub async fn get_block_with_txs(
        &self,
        block_hash_or_number: impl Into<BlockId>,
    ) -> Result<Block<Transaction>, ProviderError> {
        Ok(self
            .get_block_gen(block_hash_or_number.into(), true)
            .await?)
    }

    async fn get_block_gen<Tx: for<'a> Deserialize<'a>>(
        &self,
        id: BlockId,
        include_txs: bool,
    ) -> Result<Block<Tx>, ProviderError> {
        let include_txs = utils::serialize(&include_txs);

        Ok(match id {
            BlockId::Hash(hash) => {
                let hash = utils::serialize(&hash);
                self.0
                    .request("eth_getBlockByHash", [hash, include_txs])
                    .await
                    .map_err(Into::into)?
            }
            BlockId::Number(num) => {
                let num = utils::serialize(&num);
                self.0
                    .request("eth_getBlockByNumber", [num, include_txs])
                    .await
                    .map_err(Into::into)?
            }
        })
    }

    /// Gets the transaction with `transaction_hash`
    pub async fn get_transaction<T: Send + Sync + Into<TxHash>>(
        &self,
        transaction_hash: T,
    ) -> Result<Transaction, ProviderError> {
        let hash = transaction_hash.into();
        Ok(self
            .0
            .request("eth_getTransactionByHash", [hash])
            .await
            .map_err(Into::into)?)
    }

    /// Gets the transaction receipt with `transaction_hash`
    pub async fn get_transaction_receipt<T: Send + Sync + Into<TxHash>>(
        &self,
        transaction_hash: T,
    ) -> Result<TransactionReceipt, ProviderError> {
        let hash = transaction_hash.into();
        Ok(self
            .0
            .request("eth_getTransactionReceipt", [hash])
            .await
            .map_err(Into::into)?)
    }

    /// Gets the current gas price as estimated by the node
    pub async fn get_gas_price(&self) -> Result<U256, ProviderError> {
        Ok(self
            .0
            .request("eth_gasPrice", ())
            .await
            .map_err(Into::into)?)
    }

    /// Gets the accounts on the node
    pub async fn get_accounts(&self) -> Result<Vec<Address>, ProviderError> {
        Ok(self
            .0
            .request("eth_accounts", ())
            .await
            .map_err(Into::into)?)
    }

    /// Returns the nonce of the address
    pub async fn get_transaction_count(
        &self,
        from: impl Into<NameOrAddress>,
        block: Option<BlockNumber>,
    ) -> Result<U256, ProviderError> {
        let from = match from.into() {
            NameOrAddress::Name(ens_name) => self.resolve_name(&ens_name).await?,
            NameOrAddress::Address(addr) => addr,
        };

        let from = utils::serialize(&from);
        let block = utils::serialize(&block.unwrap_or(BlockNumber::Latest));
        Ok(self
            .0
            .request("eth_getTransactionCount", [from, block])
            .await
            .map_err(Into::into)?)
    }

    /// Returns the account's balance
    pub async fn get_balance(
        &self,
        from: impl Into<NameOrAddress>,
        block: Option<BlockNumber>,
    ) -> Result<U256, ProviderError> {
        let from = match from.into() {
            NameOrAddress::Name(ens_name) => self.resolve_name(&ens_name).await?,
            NameOrAddress::Address(addr) => addr,
        };

        let from = utils::serialize(&from);
        let block = utils::serialize(&block.unwrap_or(BlockNumber::Latest));
        Ok(self
            .0
            .request("eth_getBalance", [from, block])
            .await
            .map_err(Into::into)?)
    }

    /// Returns the currently configured chain id, a value used in replay-protected
    /// transaction signing as introduced by EIP-155.
    pub async fn get_chainid(&self) -> Result<U256, ProviderError> {
        Ok(self
            .0
            .request("eth_chainId", ())
            .await
            .map_err(Into::into)?)
    }

    ////// Contract Execution
    //
    // These are relatively low-level calls. The Contracts API should usually be used instead.

    /// Sends the read-only (constant) transaction to a single Ethereum node and return the result (as bytes) of executing it.
    /// This is free, since it does not change any state on the blockchain.
    pub async fn call(
        &self,
        tx: &TransactionRequest,
        block: Option<BlockNumber>,
    ) -> Result<Bytes, ProviderError> {
        let tx = utils::serialize(tx);
        let block = utils::serialize(&block.unwrap_or(BlockNumber::Latest));
        Ok(self
            .0
            .request("eth_call", [tx, block])
            .await
            .map_err(Into::into)?)
    }

    /// Sends a transaction to a single Ethereum node and return the estimated amount of gas required (as a U256) to send it
    /// This is free, but only an estimate. Providing too little gas will result in a transaction being rejected
    /// (while still consuming all provided gas).
    pub async fn estimate_gas(
        &self,
        tx: &TransactionRequest,
        block: Option<BlockNumber>,
    ) -> Result<U256, ProviderError> {
        let tx = utils::serialize(tx);

        let args = match block {
            Some(block) => vec![tx, utils::serialize(&block)],
            None => vec![tx],
        };

        Ok(self
            .0
            .request("eth_estimateGas", args)
            .await
            .map_err(Into::into)?)
    }

    /// Sends the transaction to the entire Ethereum network and returns the transaction's hash
    /// This will consume gas from the account that signed the transaction.
    pub async fn send_transaction(
        &self,
        mut tx: TransactionRequest,
    ) -> Result<PendingTransaction<'_, P>, ProviderError> {
        if let Some(ref to) = tx.to {
            if let NameOrAddress::Name(ens_name) = to {
                // resolve to an address
                let addr = self.resolve_name(&ens_name).await?;

                // set the value
                tx.to = Some(addr.into())
            }
        }

        let tx_hash = self
            .0
            .request("eth_sendTransaction", [tx])
            .await
            .map_err(Into::into)?;
        Ok(PendingTransaction::new(tx_hash, self))
    }

    /// Send the raw RLP encoded transaction to the entire Ethereum network and returns the transaction's hash
    /// This will consume gas from the account that signed the transaction.
    pub async fn send_raw_transaction(
        &self,
        tx: &Transaction,
    ) -> Result<PendingTransaction<'_, P>, ProviderError> {
        let rlp = utils::serialize(&tx.rlp());
        let tx_hash = self
            .0
            .request("eth_sendRawTransaction", [rlp])
            .await
            .map_err(Into::into)?;
        Ok(PendingTransaction::new(tx_hash, self))
    }

    /// Signs data using a specific account. This account needs to be unlocked.
    pub async fn sign<T: Into<Bytes>>(
        &self,
        data: T,
        from: &Address,
    ) -> Result<Signature, ProviderError> {
        let data = utils::serialize(&data.into());
        let from = utils::serialize(from);
        Ok(self
            .0
            .request("eth_sign", [from, data])
            .await
            .map_err(Into::into)?)
    }

    ////// Contract state

    /// Returns an array (possibly empty) of logs that match the filter
    pub async fn get_logs(&self, filter: &Filter) -> Result<Vec<Log>, ProviderError> {
        Ok(self
            .0
            .request("eth_getLogs", [filter])
            .await
            .map_err(Into::into)?)
    }

    /// Streams matching filter logs
    pub async fn watch(
        &self,
        filter: &Filter,
    ) -> Result<impl FilterStream<Log> + '_, ProviderError> {
        let id = self.new_filter(FilterKind::Logs(filter)).await?;
        let fut = move || Box::pin(self.get_filter_changes(id));
        Ok(FilterWatcher::new(id, fut))
    }

    /// Streams new block hashes
    pub async fn watch_blocks(&self) -> Result<impl FilterStream<H256> + '_, ProviderError> {
        let id = self.new_filter(FilterKind::NewBlocks).await?;
        let fut = move || Box::pin(self.get_filter_changes(id));
        Ok(FilterWatcher::new(id, fut))
    }

    /// Streams pending transactions
    pub async fn watch_pending_transactions(
        &self,
    ) -> Result<impl FilterStream<H256> + '_, ProviderError> {
        let id = self.new_filter(FilterKind::PendingTransactions).await?;
        let fut = move || Box::pin(self.get_filter_changes(id));
        Ok(FilterWatcher::new(id, fut))
    }

    /// Creates a filter object, based on filter options, to notify when the state changes (logs).
    /// To check if the state has changed, call `get_filter_changes` with the filter id.
    pub async fn new_filter(&self, filter: FilterKind<'_>) -> Result<U256, ProviderError> {
        let (method, args) = match filter {
            FilterKind::NewBlocks => ("eth_newBlockFilter", utils::serialize(&())),
            FilterKind::PendingTransactions => {
                ("eth_newPendingTransactionFilter", utils::serialize(&()))
            }
            FilterKind::Logs(filter) => ("eth_newFilter", utils::serialize(&filter)),
        };

        Ok(self.0.request(method, [args]).await.map_err(Into::into)?)
    }

    /// Uninstalls a filter
    pub async fn uninstall_filter<T: Into<U256>>(&self, id: T) -> Result<bool, ProviderError> {
        let id = utils::serialize(&id.into());
        Ok(self
            .0
            .request("eth_uninstallFilter", [id])
            .await
            .map_err(Into::into)?)
    }

    /// Polling method for a filter, which returns an array of logs which occurred since last poll.
    ///
    /// This method must be called with one of the following return types, depending on the filter
    /// type:
    /// - `eth_newBlockFilter`: `H256`, returns block hashes
    /// - `eth_newPendingTransactionFilter`: `H256`, returns transaction hashes
    /// - `eth_newFilter`: `Log`, returns raw logs
    ///
    /// If one of these types is not used, decoding will fail and the method will
    /// return an error.
    pub async fn get_filter_changes<T, R>(&self, id: T) -> Result<Vec<R>, ProviderError>
    where
        T: Into<U256>,
        R: for<'a> Deserialize<'a>,
    {
        let id = utils::serialize(&id.into());
        Ok(self
            .0
            .request("eth_getFilterChanges", [id])
            .await
            .map_err(Into::into)?)
    }

    // TODO: get_code, get_storage_at

    ////// Ethereum Naming Service
    // The Ethereum Naming Service (ENS) allows easy to remember and use names to
    // be assigned to Ethereum addresses. Any provider operation which takes an address
    // may also take an ENS name.
    //
    // ENS also provides the ability for a reverse lookup, which determines the name for an address if it has been configured.

    /// Returns the address that the `ens_name` resolves to (or None if not configured).
    ///
    /// # Panics
    ///
    /// If the bytes returned from the ENS registrar/resolver cannot be interpreted as
    /// an address. This should theoretically never happen.
    pub async fn resolve_name(&self, ens_name: &str) -> Result<Address, ProviderError> {
        self.query_resolver(ParamType::Address, ens_name, ens::ADDR_SELECTOR)
            .await
    }

    /// Returns the ENS name the `address` resolves to (or None if not configured).
    /// # Panics
    ///
    /// If the bytes returned from the ENS registrar/resolver cannot be interpreted as
    /// a string. This should theoretically never happen.
    pub async fn lookup_address(&self, address: Address) -> Result<String, ProviderError> {
        let ens_name = ens::reverse_address(address);
        self.query_resolver(ParamType::String, &ens_name, ens::NAME_SELECTOR)
            .await
    }

    async fn query_resolver<T: Detokenize>(
        &self,
        param: ParamType,
        ens_name: &str,
        selector: Selector,
    ) -> Result<T, ProviderError> {
        // Get the ENS address, prioritize the local override variable
        let ens_addr = self.1.unwrap_or(ens::ENS_ADDRESS);

        // first get the resolver responsible for this name
        // the call will return a Bytes array which we convert to an address
        let data = self
            .call(&ens::get_resolver(ens_addr, ens_name), None)
            .await?;

        let resolver_address: Address = decode_bytes(ParamType::Address, data);
        if resolver_address == Address::zero() {
            return Err(ProviderError::EnsError(ens_name.to_owned()));
        }

        // resolve
        let data = self
            .call(&ens::resolve(resolver_address, selector, ens_name), None)
            .await?;

        Ok(decode_bytes(param, data))
    }

    #[cfg(test)]
    /// ganache-only function for mining empty blocks
    pub async fn mine(&self, num_blocks: usize) -> Result<(), ProviderError> {
        for _ in 0..num_blocks {
            self.0
                .request::<_, U256>("evm_mine", None::<()>)
                .await
                .map_err(Into::into)?;
        }
        Ok(())
    }

    /// Sets the ENS Address (default: mainnet)
    pub fn ens<T: Into<Address>>(mut self, ens: T) -> Self {
        self.1 = Some(ens.into());
        self
    }
}

/// infallbile conversion of Bytes to Address/String
///
/// # Panics
///
/// If the provided bytes were not an interpretation of an address
fn decode_bytes<T: Detokenize>(param: ParamType, bytes: Bytes) -> T {
    let tokens =
        abi::decode(&[param], &bytes.0).expect("could not abi-decode bytes to address tokens");
    T::from_tokens(tokens).expect("could not parse tokens as address")
}

impl TryFrom<&str> for Provider<HttpProvider> {
    type Error = ParseError;

    fn try_from(src: &str) -> Result<Self, Self::Error> {
        Ok(Provider(HttpProvider::new(Url::parse(src)?), None))
    }
}

#[cfg(test)]
mod ens_tests {
    use super::*;

    const INFURA: &str = "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27";

    #[tokio::test]
    // Test vector from: https://docs.ethers.io/ethers.js/v5-beta/api-providers.html#id2
    async fn mainnet_resolve_name() {
        let provider = Provider::<HttpProvider>::try_from(INFURA).unwrap();

        let addr = provider
            .resolve_name("registrar.firefly.eth")
            .await
            .unwrap();
        assert_eq!(
            addr,
            "6fC21092DA55B392b045eD78F4732bff3C580e2c".parse().unwrap()
        );

        // registrar not found
        provider.resolve_name("asdfasdffads").await.unwrap_err();

        // name not found
        provider
            .resolve_name("asdfasdf.registrar.firefly.eth")
            .await
            .unwrap_err();
    }

    #[tokio::test]
    // Test vector from: https://docs.ethers.io/ethers.js/v5-beta/api-providers.html#id2
    async fn mainnet_lookup_address() {
        let provider = Provider::<HttpProvider>::try_from(INFURA).unwrap();

        let name = provider
            .lookup_address("6fC21092DA55B392b045eD78F4732bff3C580e2c".parse().unwrap())
            .await
            .unwrap();

        assert_eq!(name, "registrar.firefly.eth");

        provider
            .lookup_address("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".parse().unwrap())
            .await
            .unwrap_err();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethers_core::types::H256;
    use futures_util::StreamExt;

    #[tokio::test]
    #[ignore]
    // Ganache new block filters are super buggy! This test must be run with
    // geth or parity running e.g. `geth --dev --rpc --dev.period 1`
    async fn test_new_block_filter() {
        let num_blocks = 3;

        let provider = Provider::<HttpProvider>::try_from("http://localhost:8545").unwrap();
        let start_block = provider.get_block_number().await.unwrap();

        let stream = provider
            .watch_blocks()
            .await
            .unwrap()
            .interval(1000u64)
            .stream();

        let hashes: Vec<H256> = stream.take(num_blocks).collect::<Vec<H256>>().await;
        for (i, hash) in hashes.iter().enumerate() {
            let block = provider
                .get_block(start_block + i as u64 + 1)
                .await
                .unwrap();
            assert_eq!(*hash, block.hash.unwrap());
        }
    }

    // this must be run with geth or parity since ganache-core still does not support
    // eth_pendingTransactions, https://github.com/trufflesuite/ganache-core/issues/405
    // example command: `geth --dev --rpc --dev.period 1`
    #[tokio::test]
    #[ignore]
    async fn test_new_pending_txs_filter() {
        let num_txs = 5;

        let provider = Provider::<HttpProvider>::try_from("http://localhost:8545").unwrap();
        let accounts = provider.get_accounts().await.unwrap();

        let stream = provider
            .watch_pending_transactions()
            .await
            .unwrap()
            .interval(1000u64)
            .stream();

        let mut tx_hashes = Vec::new();
        let tx = TransactionRequest::new()
            .from(accounts[0])
            .to(accounts[0])
            .value(1e18 as u64);

        for _ in 0..num_txs {
            tx_hashes.push(provider.send_transaction(tx.clone()).await.unwrap());
        }

        let hashes: Vec<H256> = stream.take(num_txs).collect::<Vec<H256>>().await;
        assert_eq!(tx_hashes, hashes);
    }
}
