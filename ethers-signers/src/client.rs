use crate::Signer;

use ethers_core::types::{
    Address, BlockNumber, Bytes, NameOrAddress, Signature, TransactionRequest, TxHash,
};
use ethers_providers::{JsonRpcClient, Provider, ProviderError};

use futures_util::{future::ok, join};
use std::{future::Future, ops::Deref};
use thiserror::Error;

#[derive(Clone, Debug)]
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
///     .parse()
///     .unwrap();
///
/// let mut client = Client::new(provider, wallet);
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
/// // You can connect with other wallets at runtime via the `with_signer` function
/// let wallet2: Wallet = "cd8c407233c0560f6de24bb2dc60a8b02335c959a1a17f749ce6c1ccf63d74a7"
///     .parse()
///     .unwrap();
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
/// [`Provider`]: ../ethers_providers/struct.Provider.html
pub struct Client<P, S> {
    pub(crate) provider: Provider<P>,
    pub(crate) signer: S,
    pub(crate) address: Address,
}

#[derive(Debug, Error)]
/// Error thrown when the client interacts with the blockchain
pub enum ClientError {
    #[error(transparent)]
    /// Throw when the call to the provider fails
    ProviderError(#[from] ProviderError),

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
    S: Signer,
    P: JsonRpcClient,
{
    /// Creates a new client from the provider and signer.
    pub fn new(provider: Provider<P>, signer: S) -> Self {
        let address = signer.address();
        Client {
            provider,
            signer,
            address,
        }
    }

    /// Signs a message with the internal signer, or if none is present it will make a call to
    /// the connected node's `eth_call` API.
    pub async fn sign_message<T: Into<Bytes>>(&self, msg: T) -> Result<Signature, ClientError> {
        Ok(self.signer.sign_message(msg.into()))
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
                let addr = self
                    .resolve_name(&ens_name)
                    .await?;
                tx.to = Some(addr.into())
            }
        }

        // fill any missing fields
        self.fill_transaction(&mut tx, block).await?;

        // sign the transaction with the network
        let signed_tx = self.signer.sign_transaction(tx).map_err(Into::into)?;

        // broadcast it
        self.provider.send_raw_transaction(&signed_tx).await?;

        Ok(signed_tx.hash)
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

        // will poll and await the futures concurrently
        let (gas_price, gas, nonce) = join!(
            maybe(tx.gas_price, self.provider.get_gas_price()),
            maybe(tx.gas, self.provider.estimate_gas(&tx, block)),
            maybe(
                tx.nonce,
                self.provider.get_transaction_count(self.address(), block)
            ),
        );
        tx.gas_price = Some(gas_price?);
        tx.gas = Some(gas?);
        tx.nonce = Some(nonce?);

        Ok(())
    }

    /// Returns the client's address
    pub fn address(&self) -> Address {
        self.signer.address()
    }

    /// Returns a reference to the client's provider
    pub fn provider(&self) -> &Provider<P> {
        &self.provider
    }

    /// Returns a reference to the client's signer
    pub fn signer(&self) -> &S {
        &self.signer
    }

    /// Sets the signer and returns a mutable reference to self so that it can be used in chained
    /// calls.
    pub fn with_signer(&mut self, signer: S) -> &mut Self {
        self.signer = signer;
        self
    }

    /// Sets the provider and returns a mutable reference to self so that it can be used in chained
    /// calls.
    pub fn with_provider(&mut self, provider: Provider<P>) -> &mut Self {
        self.provider = provider;
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
