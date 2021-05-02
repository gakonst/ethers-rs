use ethers_core::{
    types::{
        Address, BlockId, Bytes, NameOrAddress, Signature, Transaction, TransactionRequest, U256,
    },
    utils::keccak256,
};
use ethers_providers::{FromErr, Middleware, PendingTransaction};
use ethers_signers::Signer;

use async_trait::async_trait;
use futures_util::{future::ok, join};
use std::future::Future;
use thiserror::Error;

#[derive(Clone, Debug)]
/// Middleware used for locally signing transactions, compatible with any implementer
/// of the [`Signer`] trait.
///
/// # Example
///
/// ```no_run
/// use ethers::{
///     providers::{Middleware, Provider, Http},
///     signers::LocalWallet,
///     middleware::SignerMiddleware,
///     types::{Address, TransactionRequest},
/// };
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
///
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
        SignerMiddleware {
            inner,
            signer,
            address,
        }
    }

    async fn sign_transaction(
        &self,
        tx: TransactionRequest,
    ) -> Result<Transaction, SignerMiddlewareError<M, S>> {
        // The nonce, gas and gasprice fields must already be populated
        let nonce = tx.nonce.ok_or(SignerMiddlewareError::NonceMissing)?;
        let gas_price = tx.gas_price.ok_or(SignerMiddlewareError::GasPriceMissing)?;
        let gas = tx.gas.ok_or(SignerMiddlewareError::GasMissing)?;

        let signature = self
            .signer
            .sign_transaction(&tx)
            .await
            .map_err(SignerMiddlewareError::SignerError)?;

        // Get the actual transaction hash
        let rlp = tx.rlp_signed(&signature);
        let hash = keccak256(&rlp.as_ref());

        // This function should not be called with ENS names
        let to = tx.to.map(|to| match to {
            NameOrAddress::Address(inner) => inner,
            NameOrAddress::Name(_) => {
                panic!("Expected `to` to be an Ethereum Address, not an ENS name")
            }
        });

        Ok(Transaction {
            hash: hash.into(),
            nonce,
            from: self.address(),
            to,
            value: tx.value.unwrap_or_default(),
            gas_price,
            gas,
            input: tx.data.unwrap_or_default(),
            v: signature.v.into(),
            r: U256::from_big_endian(signature.r.as_bytes()),
            s: U256::from_big_endian(signature.s.as_bytes()),

            // Leave these empty as they're only used for included transactions
            block_hash: None,
            block_number: None,
            transaction_index: None,

            // Celo support
            #[cfg(feature = "celo")]
            fee_currency: tx.fee_currency,
            #[cfg(feature = "celo")]
            gateway_fee: tx.gateway_fee,
            #[cfg(feature = "celo")]
            gateway_fee_recipient: tx.gateway_fee_recipient,
        })
    }

    async fn fill_transaction(
        &self,
        tx: &mut TransactionRequest,
        block: Option<BlockId>,
    ) -> Result<(), SignerMiddlewareError<M, S>> {
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
        tx.gas_price = Some(gas_price.map_err(SignerMiddlewareError::MiddlewareError)?);
        tx.gas = Some(gas.map_err(SignerMiddlewareError::MiddlewareError)?);
        tx.nonce = Some(nonce.map_err(SignerMiddlewareError::MiddlewareError)?);

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

#[async_trait]
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

    /// `SignerMiddleware` is instantiated with a signer.
    async fn is_signer(&self) -> bool {
        true
    }

    /// Signs and broadcasts the transaction. The optional parameter `block` can be passed so that
    /// gas cost and nonce calculations take it into account. For simple transactions this can be
    /// left to `None`.
    async fn send_transaction(
        &self,
        mut tx: TransactionRequest,
        block: Option<BlockId>,
    ) -> Result<PendingTransaction<'_, Self::Provider>, Self::Error> {
        if let Some(NameOrAddress::Name(ens_name)) = tx.to {
            let addr = self
                .inner
                .resolve_name(&ens_name)
                .await
                .map_err(SignerMiddlewareError::MiddlewareError)?;
            tx.to = Some(addr.into())
        }

        // fill any missing fields
        self.fill_transaction(&mut tx, block).await?;

        // if we have a nonce manager set, we should try handling the result in
        // case there was a nonce mismatch
        let signed_tx = self.sign_transaction(tx).await?;

        // Submit the raw transaction
        self.inner
            .send_raw_transaction(&signed_tx)
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
        self.signer
            .sign_message(data.into())
            .await
            .map_err(SignerMiddlewareError::SignerError)
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

#[cfg(all(test, not(feature = "celo")))]
mod tests {
    use super::*;
    use ethers::{providers::Provider, signers::LocalWallet};
    use std::convert::TryFrom;

    #[tokio::test]
    async fn signs_tx() {
        // retrieved test vector from:
        // https://web3js.readthedocs.io/en/v1.2.0/web3-eth-accounts.html#eth-accounts-signtransaction
        let tx = TransactionRequest {
            from: None,
            to: Some(
                "F0109fC8DF283027b6285cc889F5aA624EaC1F55"
                    .parse::<Address>()
                    .unwrap()
                    .into(),
            ),
            value: Some(1_000_000_000.into()),
            gas: Some(2_000_000.into()),
            nonce: Some(0.into()),
            gas_price: Some(21_000_000_000u128.into()),
            data: None,
        };
        let chain_id = 1u64;

        let provider = Provider::try_from("http://localhost:8545").unwrap();
        let key = "4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318"
            .parse::<LocalWallet>()
            .unwrap()
            .with_chain_id(chain_id);
        let client = SignerMiddleware::new(provider, key);

        let tx = client.sign_transaction(tx).await.unwrap();

        assert_eq!(
            tx.hash,
            "de8db924885b0803d2edc335f745b2b8750c8848744905684c20b987443a9593"
                .parse()
                .unwrap()
        );

        let expected_rlp = Bytes::from(hex::decode("f869808504e3b29200831e848094f0109fc8df283027b6285cc889f5aa624eac1f55843b9aca008025a0c9cf86333bcb065d140032ecaab5d9281bde80f21b9687b3e94161de42d51895a0727a108a0b8d101465414033c3f705a9c7b826e596766046ee1183dbc8aeaa68").unwrap());
        assert_eq!(tx.rlp(), expected_rlp);
    }
}
