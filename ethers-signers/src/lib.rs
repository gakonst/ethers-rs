//! Provides a unified interface for locally signing transactions.
//!
//! You can implement the `Signer` trait to extend functionality to other signers
//! such as Hardware Security Modules, KMS etc.
//!
//! The exposed interfaces return a recoverable signature. In order to convert the signature
//! and the [`TransactionRequest`] to a [`Transaction`], look at the signing middleware.
//!
//! Supported signers:
//! - Private key
//! - Ledger
//!
//! ```no_run
//! # use ethers::{
//!     signers::{LocalWallet, Signer},
//!     core::{k256::ecdsa::SigningKey, types::TransactionRequest},
//! };
//! # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
//! // instantiate the wallet
//! let wallet = "dcf2cbdd171a21c480aa7f53d77f31bb102282b3ff099c78e3118b37348c72f7"
//!     .parse::<LocalWallet>()?;
//!
//! // create a transaction
//! let tx = TransactionRequest::new()
//!     .to("vitalik.eth") // this will use ENS
//!     .value(10000);
//!
//! // sign it
//! let signature = wallet.sign_transaction(&tx).await?;
//!
//! // can also sign a message
//! let signature = wallet.sign_message("hello world").await?;
//! signature.verify("hello world", wallet.address()).unwrap();
//! # Ok(())
//! # }
// mod wallet;
// pub use wallet::Wallet;
mod wallet;
pub use wallet::Wallet;

/// A wallet instantiated with a locally stored private key
pub type LocalWallet = Wallet<ethers_core::k256::ecdsa::SigningKey>;

#[cfg(feature = "yubi")]
/// A wallet instantiated with a YubiHSM
pub type YubiWallet = Wallet<yubihsm::ecdsa::Signer<ethers_core::k256::Secp256k1>>;

#[cfg(feature = "ledger")]
mod ledger;
#[cfg(feature = "ledger")]
pub use ledger::{
    app::LedgerEthereum as Ledger,
    types::{DerivationType as HDPath, LedgerError},
};

#[cfg(feature = "yubi")]
pub use yubihsm;

use async_trait::async_trait;
use ethers_core::types::{Address, Signature, TransactionRequest};
use std::error::Error;

/// Applies [EIP155](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-155.md)
pub fn to_eip155_v<T: Into<u8>>(recovery_id: T, chain_id: Option<u64>) -> u64 {
    let standard_v: u8 = recovery_id.into();
    if let Some(chain_id) = chain_id {
        // When signing with a chain ID, add chain replay protection.
        (standard_v as u64) + 35 + chain_id * 2
    } else {
        // Otherwise, convert to 'Electrum' notation.
        (standard_v as u64) + 27
    }
}

/// Trait for signing transactions and messages
///
/// Implement this trait to support different signing modes, e.g. Ledger, hosted etc.
#[async_trait]
pub trait Signer: std::fmt::Debug + Send + Sync {
    type Error: Error + Send + Sync;
    /// Signs the hash of the provided message after prefixing it
    async fn sign_message<S: Send + Sync + AsRef<[u8]>>(
        &self,
        message: S,
    ) -> Result<Signature, Self::Error>;

    /// Signs the transaction
    async fn sign_transaction(
        &self,
        message: &TransactionRequest,
    ) -> Result<Signature, Self::Error>;

    /// Returns the signer's Ethereum Address
    fn address(&self) -> Address;
}
