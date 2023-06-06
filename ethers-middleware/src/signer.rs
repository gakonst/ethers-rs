use ethers_core::types::{
    transaction::{eip2718::TypedTransaction, eip2930::AccessListWithGasUsed},
    Address, BlockId, Bytes, Chain, Signature, TransactionRequest, U256,
};
use ethers_providers::{maybe, Middleware, MiddlewareError, PendingTransaction};
use ethers_signers::Signer;
use std::convert::TryFrom;

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
/// [`Signer`]: ethers_signers::Signer
pub struct SignerMiddleware<M, S> {
    pub(crate) inner: M,
    pub(crate) signer: S,
    pub(crate) address: Address,
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
    /// Thrown if the signer's chain_id is different than the chain_id of the transaction
    #[error("specified chain_id is different than the signer's chain_id")]
    DifferentChainID,
}

impl<M: Middleware, S: Signer> MiddlewareError for SignerMiddlewareError<M, S> {
    type Inner = M::Error;

    fn from_err(src: M::Error) -> Self {
        SignerMiddlewareError::MiddlewareError(src)
    }

    fn as_inner(&self) -> Option<&Self::Inner> {
        match self {
            SignerMiddlewareError::MiddlewareError(e) => Some(e),
            _ => None,
        }
    }
}

// Helper functions for locally signing transactions
impl<M, S> SignerMiddleware<M, S>
where
    M: Middleware,
    S: Signer,
{
    /// Creates a new client from the provider and signer.
    /// Sets the address of this middleware to the address of the signer.
    /// The chain_id of the signer will not be set to the chain id of the provider. If the signer
    /// passed here is initialized with a different chain id, then the client may throw errors, or
    /// methods like `sign_transaction` may error.
    /// To automatically set the signer's chain id, see `new_with_provider_chain`.
    ///
    /// [`Middleware`] ethers_providers::Middleware
    /// [`Signer`] ethers_signers::Signer
    pub fn new(inner: M, signer: S) -> Self {
        let address = signer.address();
        SignerMiddleware { inner, signer, address }
    }

    /// Signs and returns the RLP encoding of the signed transaction.
    /// If the transaction does not have a chain id set, it sets it to the signer's chain id.
    /// Returns an error if the transaction's existing chain id does not match the signer's chain
    /// id.
    async fn sign_transaction(
        &self,
        mut tx: TypedTransaction,
    ) -> Result<Bytes, SignerMiddlewareError<M, S>> {
        // compare chain_id and use signer's chain_id if the tranasaction's chain_id is None,
        // return an error if they are not consistent
        let chain_id = self.signer.chain_id();
        match tx.chain_id() {
            Some(id) if id.as_u64() != chain_id => {
                return Err(SignerMiddlewareError::DifferentChainID)
            }
            None => {
                tx.set_chain_id(chain_id);
            }
            _ => {}
        }

        let signature =
            self.signer.sign_transaction(&tx).await.map_err(SignerMiddlewareError::SignerError)?;

        // Return the raw rlp-encoded signed transaction
        Ok(tx.rlp_signed(&signature))
    }

    /// Returns the client's address
    pub fn address(&self) -> Address {
        self.address
    }

    /// Returns a reference to the client's signer
    pub fn signer(&self) -> &S {
        &self.signer
    }

    /// Builds a SignerMiddleware with the given Signer.
    #[must_use]
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

    /// Creates a new client from the provider and signer.
    /// Sets the address of this middleware to the address of the signer.
    /// Sets the chain id of the signer to the chain id of the inner [`Middleware`] passed in,
    /// using the [`Signer`]'s implementation of with_chain_id.
    ///
    /// [`Middleware`] ethers_providers::Middleware
    /// [`Signer`] ethers_signers::Signer
    pub async fn new_with_provider_chain(
        inner: M,
        signer: S,
    ) -> Result<Self, SignerMiddlewareError<M, S>> {
        let address = signer.address();
        let chain_id =
            inner.get_chainid().await.map_err(|e| SignerMiddlewareError::MiddlewareError(e))?;
        let signer = signer.with_chain_id(chain_id.as_u64());
        Ok(SignerMiddleware { inner, signer, address })
    }

    fn set_tx_from_if_none(&self, tx: &TypedTransaction) -> TypedTransaction {
        let mut tx = tx.clone();
        if tx.from().is_none() {
            tx.set_from(self.address);
        }
        tx
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

        // get the signer's chain_id if the transaction does not set it
        let chain_id = self.signer.chain_id();
        if tx.chain_id().is_none() {
            tx.set_chain_id(chain_id);
        }

        // If a chain_id is matched to a known chain that doesn't support EIP-1559, automatically
        // change transaction to be Legacy type.
        if let Some(chain_id) = tx.chain_id() {
            let chain = Chain::try_from(chain_id.as_u64());
            if chain.unwrap_or_default().is_legacy() {
                if let TypedTransaction::Eip1559(inner) = tx {
                    let tx_req: TransactionRequest = inner.clone().into();
                    *tx = TypedTransaction::Legacy(tx_req);
                }
            }
        }

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

    async fn estimate_gas(
        &self,
        tx: &TypedTransaction,
        block: Option<BlockId>,
    ) -> Result<U256, Self::Error> {
        let tx = self.set_tx_from_if_none(tx);
        self.inner.estimate_gas(&tx, block).await.map_err(SignerMiddlewareError::MiddlewareError)
    }

    async fn create_access_list(
        &self,
        tx: &TypedTransaction,
        block: Option<BlockId>,
    ) -> Result<AccessListWithGasUsed, Self::Error> {
        let tx = self.set_tx_from_if_none(tx);
        self.inner
            .create_access_list(&tx, block)
            .await
            .map_err(SignerMiddlewareError::MiddlewareError)
    }

    async fn call(
        &self,
        tx: &TypedTransaction,
        block: Option<BlockId>,
    ) -> Result<Bytes, Self::Error> {
        let tx = self.set_tx_from_if_none(tx);
        self.inner().call(&tx, block).await.map_err(SignerMiddlewareError::MiddlewareError)
    }
}

#[cfg(all(test, not(feature = "celo")))]
mod tests {
    use super::*;
    use ethers_core::{
        types::{Eip1559TransactionRequest, TransactionRequest},
        utils::{self, keccak256, Anvil},
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
            chain_id: None,
        }
        .into();
        let chain_id = 1u64;

        // Signer middlewares now rely on a working provider which it can query the chain id from,
        // so we make sure Anvil is started with the chain id that the expected tx was signed
        // with
        let anvil = Anvil::new().args(vec!["--chain-id".to_string(), chain_id.to_string()]).spawn();
        let provider = Provider::try_from(anvil.endpoint()).unwrap();
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
    async fn signs_tx_none_chainid() {
        // retrieved test vector from:
        // https://web3js.readthedocs.io/en/v1.2.0/web3-eth-accounts.html#eth-accounts-signtransaction
        // the signature is different because we're testing signer middleware handling the None
        // case for a non-mainnet chain id
        let tx = TransactionRequest {
            from: None,
            to: Some("F0109fC8DF283027b6285cc889F5aA624EaC1F55".parse::<Address>().unwrap().into()),
            value: Some(1_000_000_000.into()),
            gas: Some(2_000_000.into()),
            nonce: Some(U256::zero()),
            gas_price: Some(21_000_000_000u128.into()),
            data: None,
            chain_id: None,
        }
        .into();
        let chain_id = 1337u64;

        // Signer middlewares now rely on a working provider which it can query the chain id from,
        // so we make sure Anvil is started with the chain id that the expected tx was signed
        // with
        let anvil = Anvil::new().args(vec!["--chain-id".to_string(), chain_id.to_string()]).spawn();
        let provider = Provider::try_from(anvil.endpoint()).unwrap();
        let key = "4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318"
            .parse::<LocalWallet>()
            .unwrap()
            .with_chain_id(chain_id);
        let client = SignerMiddleware::new(provider, key);

        let tx = client.sign_transaction(tx).await.unwrap();

        let expected_rlp = Bytes::from(hex::decode("f86b808504e3b29200831e848094f0109fc8df283027b6285cc889f5aa624eac1f55843b9aca0080820a95a08290324bae25ca0490077e0d1f4098730333088f6a500793fa420243f35c6b23a06aca42876cd28fdf614a4641e64222fee586391bb3f4061ed5dfefac006be850").unwrap());
        assert_eq!(tx, expected_rlp);
    }

    #[tokio::test]
    async fn anvil_consistent_chainid() {
        let anvil = Anvil::new().spawn();
        let provider = Provider::try_from(anvil.endpoint()).unwrap();
        let chain_id = provider.get_chainid().await.unwrap();
        assert_eq!(chain_id, U256::from(31337));

        // Intentionally do not set the chain id here so we ensure that the signer pulls the
        // provider's chain id.
        let key = LocalWallet::new(&mut rand::thread_rng());

        // combine the provider and wallet and test that the chain id is the same for both the
        // signer returned by the middleware and through the middleware itself.
        let client = SignerMiddleware::new_with_provider_chain(provider, key).await.unwrap();
        let middleware_chainid = client.get_chainid().await.unwrap();
        assert_eq!(chain_id, middleware_chainid);

        let signer = client.signer();
        let signer_chainid = signer.chain_id();
        assert_eq!(chain_id.as_u64(), signer_chainid);
    }

    #[tokio::test]
    async fn anvil_consistent_chainid_not_default() {
        let anvil = Anvil::new().args(vec!["--chain-id", "13371337"]).spawn();
        let provider = Provider::try_from(anvil.endpoint()).unwrap();
        let chain_id = provider.get_chainid().await.unwrap();
        assert_eq!(chain_id, U256::from(13371337));

        // Intentionally do not set the chain id here so we ensure that the signer pulls the
        // provider's chain id.
        let key = LocalWallet::new(&mut rand::thread_rng());

        // combine the provider and wallet and test that the chain id is the same for both the
        // signer returned by the middleware and through the middleware itself.
        let client = SignerMiddleware::new_with_provider_chain(provider, key).await.unwrap();
        let middleware_chainid = client.get_chainid().await.unwrap();
        assert_eq!(chain_id, middleware_chainid);

        let signer = client.signer();
        let signer_chainid = signer.chain_id();
        assert_eq!(chain_id.as_u64(), signer_chainid);
    }

    #[tokio::test]
    async fn handles_tx_from_field() {
        let anvil = Anvil::new().spawn();
        let acc = anvil.addresses()[0];
        let provider = Provider::try_from(anvil.endpoint()).unwrap();
        let key = LocalWallet::new(&mut rand::thread_rng()).with_chain_id(1u32);
        provider
            .send_transaction(
                TransactionRequest::pay(key.address(), utils::parse_ether(1u64).unwrap()).from(acc),
                None,
            )
            .await
            .unwrap()
            .await
            .unwrap()
            .unwrap();
        let client = SignerMiddleware::new_with_provider_chain(provider, key).await.unwrap();

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
        // signer should result in the default anvil account being used
        let request_from_other = request.from(acc);
        let hash = *client.send_transaction(request_from_other, None).await.unwrap();
        let tx = client.get_transaction(hash).await.unwrap().unwrap();
        assert_eq!(tx.from, acc);
    }

    #[tokio::test]
    async fn converts_tx_to_legacy_to_match_chain() {
        let eip1559 = Eip1559TransactionRequest {
            from: None,
            to: Some("F0109fC8DF283027b6285cc889F5aA624EaC1F55".parse::<Address>().unwrap().into()),
            value: Some(1_000_000_000.into()),
            gas: Some(2_000_000.into()),
            nonce: Some(U256::zero()),
            access_list: Default::default(),
            max_priority_fee_per_gas: None,
            data: None,
            chain_id: None,
            max_fee_per_gas: None,
        };
        let mut tx = TypedTransaction::Eip1559(eip1559);

        let chain_id = 324u64; // zksync does not support EIP-1559

        // Signer middlewares now rely on a working provider which it can query the chain id from,
        // so we make sure Anvil is started with the chain id that the expected tx was signed
        // with
        let anvil = Anvil::new().args(vec!["--chain-id".to_string(), chain_id.to_string()]).spawn();
        let provider = Provider::try_from(anvil.endpoint()).unwrap();
        let key = "4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318"
            .parse::<LocalWallet>()
            .unwrap()
            .with_chain_id(chain_id);
        let client = SignerMiddleware::new(provider, key);
        client.fill_transaction(&mut tx, None).await.unwrap();

        assert!(tx.as_eip1559_ref().is_none());
        assert_eq!(tx, TypedTransaction::Legacy(tx.as_legacy_ref().unwrap().clone()));
    }

    #[tokio::test]
    async fn does_not_convert_to_legacy_for_eip1559_chain() {
        let eip1559 = Eip1559TransactionRequest {
            from: None,
            to: Some("F0109fC8DF283027b6285cc889F5aA624EaC1F55".parse::<Address>().unwrap().into()),
            value: Some(1_000_000_000.into()),
            gas: Some(2_000_000.into()),
            nonce: Some(U256::zero()),
            access_list: Default::default(),
            max_priority_fee_per_gas: None,
            data: None,
            chain_id: None,
            max_fee_per_gas: None,
        };
        let mut tx = TypedTransaction::Eip1559(eip1559);

        let chain_id = 1u64; // eth main supports EIP-1559

        // Signer middlewares now rely on a working provider which it can query the chain id from,
        // so we make sure Anvil is started with the chain id that the expected tx was signed
        // with
        let anvil = Anvil::new().args(vec!["--chain-id".to_string(), chain_id.to_string()]).spawn();
        let provider = Provider::try_from(anvil.endpoint()).unwrap();
        let key = "4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318"
            .parse::<LocalWallet>()
            .unwrap()
            .with_chain_id(chain_id);
        let client = SignerMiddleware::new(provider, key);
        client.fill_transaction(&mut tx, None).await.unwrap();

        assert!(tx.as_legacy_ref().is_none());
        assert_eq!(tx, TypedTransaction::Eip1559(tx.as_eip1559_ref().unwrap().clone()));
    }
}
