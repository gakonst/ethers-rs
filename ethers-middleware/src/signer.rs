use ethers_core::types::{
    transaction::eip2718::TypedTransaction, Address, BlockId, Bytes, Signature,
};
use ethers_providers::{maybe, FromErr, Middleware, PendingTransaction};
use ethers_signers::Signer;

use async_trait::async_trait;
use thiserror::Error;

#[derive(Clone, Debug)]
/// Middleware used for locally signing transactions, compatible with any implementer
/// of the [`Signer`] trait.
///
/// # Example
///
/// ```no_run
/// use ethers_providers::{Middleware, Provider, Http};
/// use ethers_signers::LocalWallet;
/// use ethers_middleware::SignerMiddleware;
/// use ethers_core::types::{Address, TransactionRequest};
/// use std::convert::TryFrom;
///
/// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
/// let provider = Provider::<Http>::try_from("http://localhost:8545")
///     .expect("could not instantiate HTTP Provider");
///
/// // Transactions will be signed with the private key below and will be broadcast
/// // via the eth_sendRawTransaction API)
/// let wallet: LocalWallet = "380eb0f3d505f087e438eca80bc4df9a7faa24f868e69fc0440261a0fc0567dc"
///     .parse()?;
///
/// let mut client = SignerMiddleware::new(provider, wallet);
///
/// // You can sign messages with the key
/// let signed_msg = client.sign(b"hello".to_vec(), &client.address()).await?;
///
/// // ...and sign transactions
/// let tx = TransactionRequest::pay("vitalik.eth", 100);
/// let pending_tx = client.send_transaction(tx, None).await?;
///
/// // You can `await` on the pending transaction to get the receipt with a pre-specified
/// // number of confirmations
/// let receipt = pending_tx.confirmations(6).await?;
///
/// // You can connect with other wallets at runtime via the `with_signer` function
/// let wallet2: LocalWallet = "cd8c407233c0560f6de24bb2dc60a8b02335c959a1a17f749ce6c1ccf63d74a7"
///     .parse()?;
///
/// let signed_msg2 = client.with_signer(wallet2).sign(b"hello".to_vec(), &client.address()).await?;
///
/// // This call will be made with `wallet2` since `with_signer` takes a mutable reference.
/// let tx2 = TransactionRequest::new()
///     .to("0xd8da6bf26964af9d7eed9e03e53415d37aa96045".parse::<Address>()?)
///     .value(200);
/// let tx_hash2 = client.send_transaction(tx2, None).await?;
///
/// # Ok(())
/// # }
/// ```
///
/// [`Provider`]: ethers_providers::Provider
pub struct SignerMiddleware<M, S> {
    pub(crate) inner: M,
    pub(crate) signer: S,
    pub(crate) address: Address,
}

impl<M: Middleware, S: Signer> FromErr<M::Error> for SignerMiddlewareError<M, S> {
    fn from(src: M::Error) -> SignerMiddlewareError<M, S> {
        SignerMiddlewareError::MiddlewareError(src)
    }
}

#[derive(Error, Debug)]
/// Error thrown when the client interacts with the blockchain
pub enum SignerMiddlewareError<M: Middleware, S: Signer> {
    #[error("{0}")]
    /// Thrown when the internal call to the signer fails
    SignerError(S::Error),

    #[error("{0}")]
    /// Thrown when an internal middleware errors
    MiddlewareError(M::Error),

    /// Thrown if the `nonce` field is missing
    #[error("no nonce was specified")]
    NonceMissing,
    /// Thrown if the `gas_price` field is missing
    #[error("no gas price was specified")]
    GasPriceMissing,
    /// Thrown if the `gas` field is missing
    #[error("no gas was specified")]
    GasMissing,
    /// Thrown if a signature is requested from a different address
    #[error("specified from address is not signer")]
    WrongSigner,
}

// Helper functions for locally signing transactions
impl<M, S> SignerMiddleware<M, S>
where
    M: Middleware,
    S: Signer,
{
    /// Creates a new client from the provider and signer.
    pub fn new(inner: M, signer: S) -> Self {
        let address = signer.address();
        SignerMiddleware { inner, signer, address }
    }

    /// Signs and returns the RLP encoding of the signed transaction
    async fn sign_transaction(
        &self,
        tx: TypedTransaction,
    ) -> Result<Bytes, SignerMiddlewareError<M, S>> {
        let signature =
            self.signer.sign_transaction(&tx).await.map_err(SignerMiddlewareError::SignerError)?;

        // Return the raw rlp-encoded signed transaction
        Ok(tx.rlp_signed(self.signer.chain_id(), &signature))
    }

    /// Returns the client's address
    pub fn address(&self) -> Address {
        self.address
    }

    /// Returns a reference to the client's signer
    pub fn signer(&self) -> &S {
        &self.signer
    }

    pub fn with_signer(&self, signer: S) -> Self
    where
        S: Clone,
        M: Clone,
    {
        let mut this = self.clone();
        this.address = signer.address();
        this.signer = signer;
        this
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<M, S> Middleware for SignerMiddleware<M, S>
where
    M: Middleware,
    S: Signer,
{
    type Error = SignerMiddlewareError<M, S>;
    type Provider = M::Provider;
    type Inner = M;

    fn inner(&self) -> &M {
        &self.inner
    }

    /// Returns the client's address
    fn default_sender(&self) -> Option<Address> {
        Some(self.address)
    }

    /// `SignerMiddleware` is instantiated with a signer.
    async fn is_signer(&self) -> bool {
        true
    }

    async fn sign_transaction(
        &self,
        tx: &TypedTransaction,
        _: Address,
    ) -> Result<Signature, Self::Error> {
        Ok(self.signer.sign_transaction(tx).await.map_err(SignerMiddlewareError::SignerError)?)
    }

    /// Helper for filling a transaction's nonce using the wallet
    async fn fill_transaction(
        &self,
        tx: &mut TypedTransaction,
        block: Option<BlockId>,
    ) -> Result<(), Self::Error> {
        // get the `from` field's nonce if it's set, else get the signer's nonce
        let from = if tx.from().is_some() && tx.from() != Some(&self.address()) {
            *tx.from().unwrap()
        } else {
            self.address
        };
        tx.set_from(from);

        let nonce = maybe(tx.nonce().cloned(), self.get_transaction_count(from, block)).await?;
        tx.set_nonce(nonce);
        self.inner()
            .fill_transaction(tx, block)
            .await
            .map_err(SignerMiddlewareError::MiddlewareError)?;
        Ok(())
    }

    /// Signs and broadcasts the transaction. The optional parameter `block` can be passed so that
    /// gas cost and nonce calculations take it into account. For simple transactions this can be
    /// left to `None`.
    async fn send_transaction<T: Into<TypedTransaction> + Send + Sync>(
        &self,
        tx: T,
        block: Option<BlockId>,
    ) -> Result<PendingTransaction<'_, Self::Provider>, Self::Error> {
        let mut tx = tx.into();

        // fill any missing fields
        self.fill_transaction(&mut tx, block).await?;

        // If the from address is set and is not our signer, delegate to inner
        if tx.from().is_some() && tx.from() != Some(&self.address()) {
            return self
                .inner
                .send_transaction(tx, block)
                .await
                .map_err(SignerMiddlewareError::MiddlewareError)
        }

        // if we have a nonce manager set, we should try handling the result in
        // case there was a nonce mismatch
        let signed_tx = self.sign_transaction(tx).await?;

        // Submit the raw transaction
        self.inner
            .send_raw_transaction(signed_tx)
            .await
            .map_err(SignerMiddlewareError::MiddlewareError)
    }

    /// Signs a message with the internal signer, or if none is present it will make a call to
    /// the connected node's `eth_call` API.
    async fn sign<T: Into<Bytes> + Send + Sync>(
        &self,
        data: T,
        _: &Address,
    ) -> Result<Signature, Self::Error> {
        self.signer.sign_message(data.into()).await.map_err(SignerMiddlewareError::SignerError)
    }
}

#[cfg(all(test, not(feature = "celo"), not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use ethers_core::{
        types::TransactionRequest,
        utils::{self, keccak256, Ganache},
    };
    use ethers_providers::Provider;
    use ethers_signers::LocalWallet;
    use std::convert::TryFrom;

    #[tokio::test]
    async fn signs_tx() {
        // retrieved test vector from:
        // https://web3js.readthedocs.io/en/v1.2.0/web3-eth-accounts.html#eth-accounts-signtransaction
        let tx = TransactionRequest {
            from: None,
            to: Some("F0109fC8DF283027b6285cc889F5aA624EaC1F55".parse::<Address>().unwrap().into()),
            value: Some(1_000_000_000.into()),
            gas: Some(2_000_000.into()),
            nonce: Some(0.into()),
            gas_price: Some(21_000_000_000u128.into()),
            data: None,
        }
        .into();
        let chain_id = 1u64;

        let provider = Provider::try_from("http://localhost:8545").unwrap();
        let key = "4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318"
            .parse::<LocalWallet>()
            .unwrap()
            .with_chain_id(chain_id);
        let client = SignerMiddleware::new(provider, key);

        let tx = client.sign_transaction(tx).await.unwrap();

        assert_eq!(
            keccak256(&tx)[..],
            hex::decode("de8db924885b0803d2edc335f745b2b8750c8848744905684c20b987443a9593")
                .unwrap()
        );

        let expected_rlp = Bytes::from(hex::decode("f869808504e3b29200831e848094f0109fc8df283027b6285cc889f5aa624eac1f55843b9aca008025a0c9cf86333bcb065d140032ecaab5d9281bde80f21b9687b3e94161de42d51895a0727a108a0b8d101465414033c3f705a9c7b826e596766046ee1183dbc8aeaa68").unwrap());
        assert_eq!(tx, expected_rlp);
    }

    #[tokio::test]
    async fn handles_tx_from_field() {
        let ganache = Ganache::new().spawn();
        let acc = ganache.addresses()[0];
        let provider = Provider::try_from(ganache.endpoint()).unwrap();
        let key = LocalWallet::new(&mut rand::thread_rng()).with_chain_id(1u32);
        provider
            .send_transaction(
                TransactionRequest::pay(key.address(), utils::parse_ether(1u64).unwrap()).from(acc),
                None,
            )
            .await
            .unwrap();
        let client = SignerMiddleware::new(provider, key);

        let request = TransactionRequest::new();

        // signing a TransactionRequest with a from field of None should yield
        // a signed transaction from the signer address
        let request_from_none = request.clone();
        let hash = *client.send_transaction(request_from_none, None).await.unwrap();
        let tx = client.get_transaction(hash).await.unwrap().unwrap();
        assert_eq!(tx.from, client.address());

        // signing a TransactionRequest with the signer as the from address
        // should yield a signed transaction from the signer
        let request_from_signer = request.clone().from(client.address());
        let hash = *client.send_transaction(request_from_signer, None).await.unwrap();
        let tx = client.get_transaction(hash).await.unwrap().unwrap();
        assert_eq!(tx.from, client.address());

        // signing a TransactionRequest with a from address that is not the
        // signer should result in the default ganache account being used
        let request_from_other = request.from(acc);
        let hash = *client.send_transaction(request_from_other, None).await.unwrap();
        let tx = client.get_transaction(hash).await.unwrap().unwrap();
        assert_eq!(tx.from, acc);
    }
}
