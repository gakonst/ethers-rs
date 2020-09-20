use crate::{NonceManager, Signer};

use ethers_core::types::{
    Address, BlockNumber, Bytes, NameOrAddress, Signature, TransactionRequest, TxHash, U256,
};
use ethers_providers::{
    gas_oracle::{GasOracle, GasOracleError},
    JsonRpcClient, Provider, ProviderError,
};

use futures_util::{future::ok, join};
use std::{future::Future, ops::Deref, sync::atomic::Ordering, time::Duration};

use thiserror::Error;

#[derive(Debug)]
/// A client provides an interface for signing and broadcasting locally signed transactions
/// It Derefs to [`Provider`], which allows interacting with the Ethereum JSON-RPC provider
/// via the same API. Sending transactions also supports using [ENS](https://ens.domains/) as a receiver. If you will
/// not be using a local signer, it is recommended to use a [`Provider`] instead.
///
/// # Example
///
/// ```no_run
/// use ethers_providers::{Provider, Http};
/// use ethers_signers::{Client, ClientError, Wallet};
/// use ethers_core::types::{Address, TransactionRequest};
/// use std::convert::TryFrom;
///
/// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
/// let provider = Provider::<Http>::try_from("http://localhost:8545")
///     .expect("could not instantiate HTTP Provider");
///
/// // By default, signing of messages and transactions is done locally
/// // (transactions will be broadcast via the eth_sendRawTransaction API)
/// let wallet: Wallet = "380eb0f3d505f087e438eca80bc4df9a7faa24f868e69fc0440261a0fc0567dc"
///     .parse()?;
///
/// let mut client = Client::new(provider, wallet).await?;
///
/// // since it derefs to `Provider`, we can just call any of the JSON-RPC API methods
/// let block = client.get_block(100u64).await?;
///
/// // You can use the node's `eth_sign` and `eth_sendTransaction` calls by calling the
/// // internal provider's method.
/// let signed_msg = client.provider().sign(b"hello".to_vec(), &client.address()).await?;
///
/// let tx = TransactionRequest::pay("vitalik.eth", 100);
/// let tx_hash = client.send_transaction(tx, None).await?;
///
/// // You can `await` on the pending transaction to get the receipt with a pre-specified
/// // number of confirmations
/// let receipt = client.pending_transaction(tx_hash).confirmations(6).await?;
///
/// // You can connect with other wallets at runtime via the `with_signer` function
/// let wallet2: Wallet = "cd8c407233c0560f6de24bb2dc60a8b02335c959a1a17f749ce6c1ccf63d74a7"
///     .parse()?;
///
/// let signed_msg2 = client.with_signer(wallet2).sign_message(b"hello".to_vec()).await?;
///
/// // This call will be made with `wallet2` since `with_signer` takes a mutable reference.
/// let tx2 = TransactionRequest::new()
///     .to("0xd8da6bf26964af9d7eed9e03e53415d37aa96045".parse::<Address>()?)
///     .value(200);
/// let tx_hash2 = client.send_transaction(tx2, None).await?;
///
/// # Ok(())
/// # }
///
/// ```
///
/// [`Provider`]: ethers_providers::Provider
pub struct Client<P, S> {
    pub(crate) provider: Provider<P>,
    pub(crate) signer: Option<S>,
    pub(crate) address: Address,
    pub(crate) gas_oracle: Option<Box<dyn GasOracle>>,
    pub(crate) nonce_manager: Option<NonceManager>,
}

#[derive(Debug, Error)]
/// Error thrown when the client interacts with the blockchain
pub enum ClientError {
    #[error(transparent)]
    /// Throw when the call to the provider fails
    ProviderError(#[from] ProviderError),

    #[error(transparent)]
    /// Throw when a call to the gas oracle fails
    GasOracleError(#[from] GasOracleError),

    #[error(transparent)]
    /// Thrown when the internal call to the signer fails
    SignerError(#[from] Box<dyn std::error::Error + Send + Sync>),

    #[error("ens name not found: {0}")]
    /// Thrown when an ENS name is not found
    EnsError(String),
}

// Helper functions for locally signing transactions
impl<P, S> Client<P, S>
where
    P: JsonRpcClient,
    S: Signer,
{
    /// Creates a new client from the provider and signer.
    pub async fn new(provider: Provider<P>, signer: S) -> Result<Self, ClientError> {
        let address = signer.address().await.map_err(Into::into)?;
        Ok(Client {
            provider,
            signer: Some(signer),
            address,
            gas_oracle: None,
            nonce_manager: None,
        })
    }

    /// Signs a message with the internal signer, or if none is present it will make a call to
    /// the connected node's `eth_call` API.
    pub async fn sign_message<T: Into<Bytes>>(&self, msg: T) -> Result<Signature, ClientError> {
        Ok(if let Some(ref signer) = self.signer {
            signer.sign_message(msg.into()).await.map_err(Into::into)?
        } else {
            self.provider.sign(msg, &self.address()).await?
        })
    }

    /// Signs and broadcasts the transaction. The optional parameter `block` can be passed so that
    /// gas cost and nonce calculations take it into account. For simple transactions this can be
    /// left to `None`.
    pub async fn send_transaction(
        &self,
        mut tx: TransactionRequest,
        block: Option<BlockNumber>,
    ) -> Result<TxHash, ClientError> {
        if let Some(ref to) = tx.to {
            if let NameOrAddress::Name(ens_name) = to {
                let addr = self.resolve_name(&ens_name).await?;
                tx.to = Some(addr.into())
            }
        }

        // fill any missing fields
        self.fill_transaction(&mut tx, block).await?;

        // if we have a nonce manager set, we should try handling the result in
        // case there was a nonce mismatch
        let tx_hash = if let Some(ref nonce_manager) = self.nonce_manager {
            let mut tx_clone = tx.clone();
            match self.submit_transaction(tx).await {
                Ok(tx_hash) => tx_hash,
                Err(err) => {
                    let nonce = self.get_transaction_count(block).await?;
                    if nonce != nonce_manager.nonce.load(Ordering::SeqCst).into() {
                        // try re-submitting the transaction with the correct nonce if there
                        // was a nonce mismatch
                        nonce_manager.nonce.store(nonce.as_u64(), Ordering::SeqCst);
                        tx_clone.nonce = Some(nonce);
                        self.submit_transaction(tx_clone).await?
                    } else {
                        // propagate the error otherwise
                        return Err(err);
                    }
                }
            }
        } else {
            self.submit_transaction(tx).await?
        };

        Ok(tx_hash)
    }

    async fn submit_transaction(&self, tx: TransactionRequest) -> Result<TxHash, ClientError> {
        Ok(if let Some(ref signer) = self.signer {
            let signed_tx = signer.sign_transaction(tx).await.map_err(Into::into)?;
            self.provider.send_raw_transaction(&signed_tx).await?
        } else {
            self.provider.send_transaction(tx).await?
        })
    }

    async fn fill_transaction(
        &self,
        tx: &mut TransactionRequest,
        block: Option<BlockNumber>,
    ) -> Result<(), ClientError> {
        // set the `from` field
        if tx.from.is_none() {
            tx.from = Some(self.address());
        }

        // assign gas price if a gas oracle has been provided
        if let Some(gas_oracle) = &self.gas_oracle {
            if let Ok(gas_price) = gas_oracle.fetch().await {
                tx.gas_price = Some(gas_price);
            }
        }

        // will poll and await the futures concurrently
        let (gas_price, gas, nonce) = join!(
            maybe(tx.gas_price, self.provider.get_gas_price()),
            maybe(tx.gas, self.provider.estimate_gas(&tx)),
            maybe(tx.nonce, self.get_transaction_count_with_manager(block)),
        );
        tx.gas_price = Some(gas_price?);
        tx.gas = Some(gas?);
        tx.nonce = Some(nonce?);

        Ok(())
    }

    async fn get_transaction_count_with_manager(
        &self,
        block: Option<BlockNumber>,
    ) -> Result<U256, ClientError> {
        // If there's a nonce manager set, short circuit by just returning the next nonce
        if let Some(ref nonce_manager) = self.nonce_manager {
            // initialize the nonce the first time the manager is called
            if !nonce_manager.initialized.load(Ordering::SeqCst) {
                let nonce = self
                    .provider
                    .get_transaction_count(self.address(), block)
                    .await?;
                nonce_manager.nonce.store(nonce.as_u64(), Ordering::SeqCst);
                nonce_manager.initialized.store(true, Ordering::SeqCst);
            }

            return Ok(nonce_manager.next());
        }

        self.get_transaction_count(block).await
    }

    pub async fn get_transaction_count(
        &self,
        block: Option<BlockNumber>,
    ) -> Result<U256, ClientError> {
        Ok(self
            .provider
            .get_transaction_count(self.address(), block)
            .await?)
    }

    /// Returns the client's address
    pub fn address(&self) -> Address {
        self.address
    }

    /// Returns a reference to the client's provider
    pub fn provider(&self) -> &Provider<P> {
        &self.provider
    }

    /// Returns a reference to the client's signer
    pub fn signer(&self) -> Option<&S> {
        self.signer.as_ref()
    }

    /// Sets the signer and returns a mutable reference to self so that it can be used in chained
    /// calls.
    pub fn with_signer(&mut self, signer: S) -> &Self {
        self.signer = Some(signer);
        self
    }

    /// Sets the provider and returns a mutable reference to self so that it can be used in chained
    /// calls.
    pub fn with_provider(&mut self, provider: Provider<P>) -> &Self {
        self.provider = provider;
        self
    }

    /// Sets the address which will be used for interacting with the blockchain.
    /// Useful if no signer is set and you want to specify a default sender for
    /// your transactions or if you have changed the signer manually.
    pub fn with_sender<T: Into<Address>>(mut self, address: T) -> Self {
        self.address = address.into();
        self
    }

    /// Sets the default polling interval for event filters and pending transactions
    pub fn interval<T: Into<Duration>>(mut self, interval: T) -> Self {
        let provider = self.provider.interval(interval.into());
        self.provider = provider;
        self
    }

    /// Sets the gas oracle to query for gas estimates while broadcasting transactions
    pub fn gas_oracle(mut self, gas_oracle: Box<dyn GasOracle>) -> Self {
        self.gas_oracle = Some(gas_oracle);
        self
    }

    pub fn with_nonce_manager(mut self) -> Self {
        self.nonce_manager = Some(NonceManager::new());
        self
    }
}

/// Calls the future if `item` is None, otherwise returns a `futures::ok`
async fn maybe<F, T, E>(item: Option<T>, f: F) -> Result<T, E>
where
    F: Future<Output = Result<T, E>>,
{
    if let Some(item) = item {
        ok(item).await
    } else {
        f.await
    }
}

// Abuse Deref to use the Provider's methods without re-writing everything.
// This is an anti-pattern and should not be encouraged, but this improves the UX while
// keeping the LoC low
impl<P, S> Deref for Client<P, S> {
    type Target = Provider<P>;

    fn deref(&self) -> &Self::Target {
        &self.provider
    }
}

impl<P: JsonRpcClient, S> From<Provider<P>> for Client<P, S> {
    fn from(provider: Provider<P>) -> Self {
        Self {
            provider,
            signer: None,
            address: Address::zero(),
            gas_oracle: None,
            nonce_manager: None,
        }
    }
}
