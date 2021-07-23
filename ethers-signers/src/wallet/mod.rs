mod hash;

mod mnemonic;
pub use mnemonic::{MnemonicBuilder, MnemonicBuilderError};

mod private_key;
pub use private_key::WalletError;

#[cfg(feature = "yubihsm")]
mod yubi;

use crate::{to_eip155_v, Signer};
use ethers_core::{
    k256::{
        ecdsa::{recoverable::Signature as RecoverableSignature, signature::DigestSigner},
        elliptic_curve::FieldBytes,
        Secp256k1,
    },
    types::{Address, Signature, TransactionRequest, H256},
    utils::hash_message,
};
use hash::Sha256Proxy;

use async_trait::async_trait;
use std::fmt;

/// An Ethereum private-public key pair which can be used for signing messages.
///
/// # Examples
///
/// ## Signing and Verifying a message
///
/// The wallet can be used to produce ECDSA [`Signature`] objects, which can be
/// then verified. Note that this uses [`hash_message`] under the hood which will
/// prefix the message being hashed with the `Ethereum Signed Message` domain separator.
///
/// ```
/// use ethers_core::rand::thread_rng;
/// use ethers_signers::{LocalWallet, Signer};
///
/// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
/// let wallet = LocalWallet::new(&mut thread_rng());
///
/// // Optionally, the wallet's chain id can be set, in order to use EIP-155
/// // replay protection with different chains
/// let wallet = wallet.set_chain_id(1337u64);
///
/// // The wallet can be used to sign messages
/// let message = b"hello";
/// let signature = wallet.sign_message(message).await?;
/// assert_eq!(signature.recover(&message[..]).unwrap(), wallet.address());
/// # Ok(())
/// # }
/// ```
///
/// [`Signature`]: ethers_core::types::Signature
/// [`hash_message`]: fn@ethers_core::utils::hash_message
pub struct Wallet<D: DigestSigner<Sha256Proxy, RecoverableSignature>> {
    /// The Wallet's private Key
    pub(crate) signer: D,
    /// The wallet's address
    pub(crate) address: Address,
    /// The wallet's chain id (for EIP-155), signs w/o replay protection if left unset
    pub(crate) chain_id: Option<u64>,
}

#[async_trait]
impl<D: Sync + Send + DigestSigner<Sha256Proxy, RecoverableSignature>> Signer for Wallet<D> {
    type Error = std::convert::Infallible;

    async fn sign_message<S: Send + Sync + AsRef<[u8]>>(
        &self,
        message: S,
    ) -> Result<Signature, Self::Error> {
        let message = message.as_ref();
        let message_hash = hash_message(message);

        Ok(self.sign_hash_with_eip155(message_hash, None))
    }

    async fn sign_transaction(&self, tx: &TransactionRequest) -> Result<Signature, Self::Error> {
        let sighash = tx.sighash(self.chain_id);
        Ok(self.sign_hash_with_eip155(sighash, self.chain_id))
    }

    fn address(&self) -> Address {
        self.address
    }
}

impl<D: DigestSigner<Sha256Proxy, RecoverableSignature>> Wallet<D> {
    fn sign_hash_with_eip155(&self, hash: H256, chain_id: Option<u64>) -> Signature {
        let recoverable_sig: RecoverableSignature =
            self.signer.sign_digest(Sha256Proxy::from(hash));

        let v = to_eip155_v(recoverable_sig.recovery_id(), chain_id);

        let r_bytes: FieldBytes<Secp256k1> = recoverable_sig.r().into();
        let s_bytes: FieldBytes<Secp256k1> = recoverable_sig.s().into();
        let r = H256::from_slice(r_bytes.as_slice());
        let s = H256::from_slice(s_bytes.as_slice());

        Signature { r, s, v }
    }

    /// Sets the wallet's chain_id, used in conjunction with EIP-155 signing
    pub fn set_chain_id<T: Into<u64>>(mut self, chain_id: T) -> Self {
        self.chain_id = Some(chain_id.into());
        self
    }

    /// Gets the wallet's signer
    pub fn signer(&self) -> &D {
        &self.signer
    }

    /// Gets the wallet's chain id
    pub fn chain_id(&self) -> Option<u64> {
        self.chain_id
    }

    /// Returns the wallet's address
    pub fn address(&self) -> Address {
        self.address
    }
}

// do not log the signer
impl<D: DigestSigner<Sha256Proxy, RecoverableSignature>> fmt::Debug for Wallet<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Wallet")
            .field("address", &self.address)
            .field("chain_Id", &self.chain_id)
            .finish()
    }
}
