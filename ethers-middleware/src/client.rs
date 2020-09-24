use ethers_signers::Signer;

use ethers_core::types::{
    Address, BlockNumber, Bytes, NameOrAddress, Signature, TransactionRequest, TxHash,
};
use ethers_providers::Middleware;

use async_trait::async_trait;
use futures_util::{future::ok, join};
use std::future::Future;
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
pub struct Client<M, S> {
    pub(crate) inner: M,
    pub(crate) signer: S,
    pub(crate) address: Address,
}

use ethers_providers::FromErr;

impl<M: Middleware, S: Signer> FromErr<M::Error> for ClientError<M, S> {
    fn from(src: M::Error) -> ClientError<M, S> {
        ClientError::MiddlewareError(src)
    }
}

#[derive(Error, Debug)]
/// Error thrown when the client interacts with the blockchain
pub enum ClientError<M: Middleware, S: Signer> {
    #[error("{0}")]
    /// Thrown when the internal call to the signer fails
    SignerError(S::Error),

    #[error("{0}")]
    MiddlewareError(M::Error),
}

// Helper functions for locally signing transactions
impl<M, S> Client<M, S>
where
    M: Middleware,
    S: Signer,
{
    /// Creates a new client from the provider and signer.
    pub async fn new(inner: M, signer: S) -> Result<Self, ClientError<M, S>> {
        let address = signer.address().await.map_err(ClientError::SignerError)?;
        Ok(Client {
            inner,
            signer,
            address,
        })
    }

    async fn submit_transaction(
        &self,
        tx: TransactionRequest,
    ) -> Result<TxHash, ClientError<M, S>> {
        let signed_tx = self
            .signer
            .sign_transaction(tx)
            .await
            .map_err(ClientError::SignerError)?;
        self.inner
            .send_raw_transaction(&signed_tx)
            .await
            .map_err(ClientError::MiddlewareError)
    }

    async fn fill_transaction(
        &self,
        tx: &mut TransactionRequest,
        block: Option<BlockNumber>,
    ) -> Result<(), ClientError<M, S>> {
        // set the `from` field
        if tx.from.is_none() {
            tx.from = Some(self.address());
        }

        // will poll and await the futures concurrently
        let (gas_price, gas, nonce) = join!(
            maybe(tx.gas_price, self.inner.get_gas_price()),
            maybe(tx.gas, self.inner.estimate_gas(&tx)),
            maybe(
                tx.nonce,
                self.inner.get_transaction_count(self.address(), block)
            ),
        );
        tx.gas_price = Some(gas_price.map_err(ClientError::MiddlewareError)?);
        tx.gas = Some(gas.map_err(ClientError::MiddlewareError)?);
        tx.nonce = Some(nonce.map_err(ClientError::MiddlewareError)?);

        Ok(())
    }

    /// Returns the client's address
    pub fn address(&self) -> Address {
        self.address
    }

    /// Returns a reference to the client's signer
    pub fn signer(&self) -> &S {
        &self.signer
    }
}

#[async_trait]
impl<M, S> Middleware for Client<M, S>
where
    M: Middleware,
    S: Signer,
{
    type Error = ClientError<M, S>;
    type Provider = M::Provider;
    type Inner = M;

    fn inner(&self) -> &M {
        &self.inner
    }

    /// Signs and broadcasts the transaction. The optional parameter `block` can be passed so that
    /// gas cost and nonce calculations take it into account. For simple transactions this can be
    /// left to `None`.
    async fn send_transaction(
        &self,
        mut tx: TransactionRequest,
        block: Option<BlockNumber>,
    ) -> Result<TxHash, Self::Error> {
        if let Some(ref to) = tx.to {
            if let NameOrAddress::Name(ens_name) = to {
                let addr = self
                    .inner
                    .resolve_name(&ens_name)
                    .await
                    .map_err(ClientError::MiddlewareError)?;
                tx.to = Some(addr.into())
            }
        }

        // fill any missing fields
        self.fill_transaction(&mut tx, block).await?;

        // if we have a nonce manager set, we should try handling the result in
        // case there was a nonce mismatch
        let tx_hash = self.submit_transaction(tx).await?;

        Ok(tx_hash)
    }

    /// Signs a message with the internal signer, or if none is present it will make a call to
    /// the connected node's `eth_call` API.
    async fn sign<T: Into<Bytes> + Send + Sync>(
        &self,
        data: T,
        _: &Address,
    ) -> Result<Signature, Self::Error> {
        Ok(self.signer.sign_message(data.into()).await.unwrap())
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
