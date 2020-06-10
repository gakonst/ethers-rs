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
/// via the same API. Sending transactions also supports using ENS as a receiver. If you will
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
/// let mut client: Client<_, _> = Provider::<Http>::try_from("http://localhost:8545")
///     .expect("could not instantiate HTTP Provider").into();
///
/// // since it derefs to `Provider`, we can just call any of the JSON-RPC API methods
/// let block = client.get_block(100u64).await?;
///
/// // calling `sign_message` and `send_transaction` will use the unlocked accounts
/// // on the node.
/// let signed_msg = client.sign_message(b"hello".to_vec()).await?;
///
/// let tx = TransactionRequest::pay("vitalik.eth", 100);
/// let tx_hash = client.send_transaction(tx, None).await?;
///
/// // if we set a signer, signing of messages and transactions will be done locally
/// // (transactions will be broadcast via the eth_sendRawTransaction API)
/// let wallet: Wallet = "380eb0f3d505f087e438eca80bc4df9a7faa24f868e69fc0440261a0fc0567dc"
///     .parse()
///     .unwrap();
///
/// let client = client.with_signer(wallet);
///
/// let signed_msg2 = client.sign_message(b"hello".to_vec()).await?;
///
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
/// [`Provider`](../ethers_providers/struct.Provider.html)
pub struct Client<P, S> {
    pub(crate) provider: Provider<P>,
    pub(crate) signer: Option<S>,
    pub(crate) address: Address,
}

impl<P, S> From<Provider<P>> for Client<P, S> {
    fn from(provider: Provider<P>) -> Self {
        Client {
            provider,
            signer: None,
            address: Address::zero(),
        }
    }
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
    /// Signs a message with the internal signer, or if none is present it will make a call to
    /// the connected node's `eth_call` API.
    pub async fn sign_message<T: Into<Bytes>>(&self, msg: T) -> Result<Signature, ClientError> {
        let msg = msg.into();
        Ok(if let Some(ref signer) = self.signer {
            signer.sign_message(msg)
        } else {
            self.provider.sign(&msg, &self.address).await?
        })
    }

    /// Signs and broadcasts the transaction
    pub async fn send_transaction(
        &self,
        mut tx: TransactionRequest,
        block: Option<BlockNumber>,
    ) -> Result<TxHash, ClientError> {
        if let Some(ref to) = tx.to {
            if let NameOrAddress::Name(ens_name) = to {
                let addr = self
                    .resolve_name(&ens_name)
                    .await?
                    .ok_or_else(|| ClientError::EnsError(ens_name.to_owned()))?;
                tx.to = Some(addr.into())
            }
        }

        // if there is no local signer, then the transaction should use the
        // node's signer which should already be unlocked
        let signer = if let Some(ref signer) = self.signer {
            signer
        } else {
            return Ok(self.provider.send_transaction(tx).await?);
        };

        // fill any missing fields
        self.fill_transaction(&mut tx, block).await?;

        // sign the transaction with the network
        let signed_tx = signer.sign_transaction(tx).map_err(Into::into)?;

        // broadcast it
        self.provider.send_raw_transaction(&signed_tx).await?;

        Ok(signed_tx.hash)
    }

    async fn fill_transaction(
        &self,
        tx: &mut TransactionRequest,
        block: Option<BlockNumber>,
    ) -> Result<(), ClientError> {
        tx.from = Some(self.address());

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

    /// Returns the client's address (or `address(0)` if no signer is set)
    pub fn address(&self) -> Address {
        self.signer
            .as_ref()
            .map(|s| s.address())
            .unwrap_or_default()
    }

    /// Returns a reference to the client's provider
    pub fn provider(&self) -> &Provider<P> {
        &self.provider
    }

    /// Returns a reference to the client's signer
    ///
    /// # Panics
    ///
    /// If `self.signer` is `None`
    pub fn signer_unchecked(&self) -> &S {
        self.signer.as_ref().expect("no signer is configured")
    }

    /// Sets the signer
    pub fn with_signer(&mut self, signer: S) -> &mut Self {
        self.signer = Some(signer);
        self
    }

    /// Sets the provider
    pub fn with_provider(&mut self, provider: Provider<P>) -> &mut Self {
        self.provider = provider;
        self
    }

    /// Sets the default account to be used with the `eth_sign` API calls
    pub fn from(&mut self, address: Address) -> &mut Self {
        self.address = address;
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
