use crate::{ens, http::Provider as HttpProvider, JsonRpcClient};

use ethers_core::{
    abi::{self, Detokenize, ParamType},
    types::{
        Address, Block, BlockId, BlockNumber, Bytes, Filter, Log, NameOrAddress, Selector,
        Transaction, TransactionReceipt, TransactionRequest, TxHash, U256,
    },
    utils,
};

use serde::Deserialize;
use thiserror::Error;
use url::{ParseError, Url};

use std::{convert::TryFrom, fmt::Debug};

/// An abstract provider for interacting with the [Ethereum JSON RPC
/// API](https://github.com/ethereum/wiki/wiki/JSON-RPC). Must be instantiated
/// with a [`Network`](networks/trait.Network.html) and a data transport
/// (e.g. HTTP, Websockets etc.)
#[derive(Clone, Debug)]
pub struct Provider<P>(P, Option<Address>);

#[derive(Debug, Error)]
/// An error thrown when making a call to the provider
pub enum ProviderError {
    #[error(transparent)]
    // dyn dispatch the error to avoid generic return types
    JsonRpcClientError(#[from] Box<dyn std::error::Error>),
    #[error("ens name not found: {0}")]
    EnsError(String),
}

// JSON RPC bindings
impl<P: JsonRpcClient> Provider<P> {
    ////// Blockchain Status
    //
    // Functions for querying the state of the blockchain

    /// Gets the latest block number via the `eth_BlockNumber` API
    pub async fn get_block_number(&self) -> Result<U256, ProviderError> {
        Ok(self
            .0
            .request("eth_blockNumber", None::<()>)
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
                let args = vec![hash, include_txs];
                self.0
                    .request("eth_getBlockByHash", Some(args))
                    .await
                    .map_err(Into::into)?
            }
            BlockId::Number(num) => {
                let num = utils::serialize(&num);
                let args = vec![num, include_txs];
                self.0
                    .request("eth_getBlockByNumber", Some(args))
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
            .request("eth_getTransactionByHash", Some(hash))
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
            .request("eth_getTransactionReceipt", Some(hash))
            .await
            .map_err(Into::into)?)
    }

    /// Gets the current gas price as estimated by the node
    pub async fn get_gas_price(&self) -> Result<U256, ProviderError> {
        Ok(self
            .0
            .request("eth_gasPrice", None::<()>)
            .await
            .map_err(Into::into)?)
    }

    /// Gets the accounts on the node
    pub async fn get_accounts(&self) -> Result<Vec<Address>, ProviderError> {
        Ok(self
            .0
            .request("eth_accounts", None::<()>)
            .await
            .map_err(Into::into)?)
    }

    /// Returns the nonce of the address
    pub async fn get_transaction_count(
        &self,
        from: Address,
        block: Option<BlockNumber>,
    ) -> Result<U256, ProviderError> {
        let from = utils::serialize(&from);
        let block = utils::serialize(&block.unwrap_or(BlockNumber::Latest));
        Ok(self
            .0
            .request("eth_getTransactionCount", Some(&[from, block]))
            .await
            .map_err(Into::into)?)
    }

    /// Returns the account's balance
    pub async fn get_balance(
        &self,
        from: Address,
        block: Option<BlockNumber>,
    ) -> Result<U256, ProviderError> {
        let from = utils::serialize(&from);
        let block = utils::serialize(&block.unwrap_or(BlockNumber::Latest));
        Ok(self
            .0
            .request("eth_getBalance", Some(&[from, block]))
            .await
            .map_err(Into::into)?)
    }

    /// Returns the currently configured chain id, a value used in replay-protected
    /// transaction signing as introduced by EIP-155.
    pub async fn get_chainid(&self) -> Result<U256, ProviderError> {
        Ok(self
            .0
            .request("eth_chainId", None::<()>)
            .await
            .map_err(Into::into)?)
    }

    ////// Contract Execution
    //
    // These are relatively low-level calls. The Contracts API should usually be used instead.

    /// Send the read-only (constant) transaction to a single Ethereum node and return the result (as bytes) of executing it.
    /// This is free, since it does not change any state on the blockchain.
    pub async fn call(
        &self,
        tx: TransactionRequest,
        block: Option<BlockNumber>,
    ) -> Result<Bytes, ProviderError> {
        let tx = utils::serialize(&tx);
        let block = utils::serialize(&block.unwrap_or(BlockNumber::Latest));
        Ok(self
            .0
            .request("eth_call", Some(vec![tx, block]))
            .await
            .map_err(Into::into)?)
    }

    /// Send a transaction to a single Ethereum node and return the estimated amount of gas required (as a U256) to send it
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
            .request("eth_estimateGas", Some(args))
            .await
            .map_err(Into::into)?)
    }

    /// Send the transaction to the entire Ethereum network and returns the transaction's hash
    /// This will consume gas from the account that signed the transaction.
    pub async fn send_transaction(
        &self,
        mut tx: TransactionRequest,
    ) -> Result<TxHash, ProviderError> {
        if let Some(ref to) = tx.to {
            if let NameOrAddress::Name(ens_name) = to {
                // resolve to an address
                let addr = self
                    .resolve_name(&ens_name)
                    .await?
                    .ok_or_else(|| ProviderError::EnsError(ens_name.to_owned()))?;

                // set the value
                tx.to = Some(addr.into())
            }
        }

        Ok(self
            .0
            .request("eth_sendTransaction", Some(tx))
            .await
            .map_err(Into::into)?)
    }

    /// Send the raw RLP encoded transaction to the entire Ethereum network and returns the transaction's hash
    /// This will consume gas from the account that signed the transaction.
    pub async fn send_raw_transaction(&self, tx: &Transaction) -> Result<TxHash, ProviderError> {
        let rlp = utils::serialize(&tx.rlp());
        Ok(self
            .0
            .request("eth_sendRawTransaction", Some(rlp))
            .await
            .map_err(Into::into)?)
    }

    ////// Contract state

    /// Returns an array (possibly empty) of logs that match the filter
    pub async fn get_logs(&self, filter: &Filter) -> Result<Vec<Log>, ProviderError> {
        Ok(self
            .0
            .request("eth_getLogs", Some(filter))
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
    pub async fn resolve_name(&self, ens_name: &str) -> Result<Option<Address>, ProviderError> {
        self.query_resolver(ParamType::Address, ens_name, ens::ADDR_SELECTOR)
            .await
    }

    /// Returns the ENS name the `address` resolves to (or None if not configured).
    /// # Panics
    ///
    /// If the bytes returned from the ENS registrar/resolver cannot be interpreted as
    /// a string. This should theoretically never happen.
    pub async fn lookup_address(&self, address: Address) -> Result<Option<String>, ProviderError> {
        let ens_name = ens::reverse_address(address);
        self.query_resolver(ParamType::String, &ens_name, ens::NAME_SELECTOR)
            .await
    }

    async fn query_resolver<T: Detokenize>(
        &self,
        param: ParamType,
        ens_name: &str,
        selector: Selector,
    ) -> Result<Option<T>, ProviderError> {
        // Get the ENS address, prioritize the local override variable
        let ens_addr = self.1.unwrap_or(ens::ENS_ADDRESS);

        // first get the resolver responsible for this name
        // the call will return a Bytes array which we convert to an address
        let data = self
            .call(ens::get_resolver(ens_addr, ens_name), None)
            .await?;

        let resolver_address: Address = decode_bytes(ParamType::Address, data);
        if resolver_address == Address::zero() {
            return Ok(None);
        }

        // resolve
        let data = self
            .call(ens::resolve(resolver_address, selector, ens_name), None)
            .await?;

        Ok(Some(decode_bytes(param, data)))
    }

    /// Overrides the default ENS address set by the provider's `Network` type.
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

    #[tokio::test]
    // Test vector from: https://docs.ethers.io/ethers.js/v5-beta/api-providers.html#id2
    async fn mainnet_resolve_name() {
        let provider = Provider::<HttpProvider>::try_from(
            "https://mainnet.infura.io/v3/9408f47dedf04716a03ef994182cf150",
        )
        .unwrap();

        let addr = provider
            .resolve_name("registrar.firefly.eth")
            .await
            .unwrap();
        assert_eq!(
            addr.unwrap(),
            "6fC21092DA55B392b045eD78F4732bff3C580e2c".parse().unwrap()
        );

        // registrar not found
        let addr = provider.resolve_name("asdfasdffads").await.unwrap();
        assert!(addr.is_none());

        // name not found
        let addr = provider
            .resolve_name("asdfasdf.registrar.firefly.eth")
            .await
            .unwrap();
        assert!(addr.is_none());
    }

    #[tokio::test]
    // Test vector from: https://docs.ethers.io/ethers.js/v5-beta/api-providers.html#id2
    async fn mainnet_lookup_address() {
        let provider = Provider::<HttpProvider>::try_from(
            "https://mainnet.infura.io/v3/9408f47dedf04716a03ef994182cf150",
        )
        .unwrap();

        let name = provider
            .lookup_address("6fC21092DA55B392b045eD78F4732bff3C580e2c".parse().unwrap())
            .await
            .unwrap();

        assert_eq!(name.unwrap(), "registrar.firefly.eth");

        let name = provider
            .lookup_address("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".parse().unwrap())
            .await
            .unwrap();

        assert!(name.is_none());
    }
}
