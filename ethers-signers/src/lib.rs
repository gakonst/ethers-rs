#![doc = include_str!("../README.md")]
#![deny(unsafe_code, rustdoc::broken_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod wallet;
pub use wallet::{MnemonicBuilder, Wallet, WalletError};

/// Re-export the BIP-32 crate so that wordlists can be accessed conveniently.
pub use coins_bip39;

/// A wallet instantiated with a locally stored private key
pub type LocalWallet = Wallet<ethers_core::k256::ecdsa::SigningKey>;

#[cfg(all(feature = "yubihsm", not(target_arch = "wasm32")))]
/// A wallet instantiated with a YubiHSM
pub type YubiWallet = Wallet<yubihsm::ecdsa::Signer<ethers_core::k256::Secp256k1>>;

#[cfg(all(feature = "ledger", not(target_arch = "wasm32")))]
mod ledger;
#[cfg(all(feature = "ledger", not(target_arch = "wasm32")))]
pub use ledger::{
    app::LedgerEthereum as Ledger,
    types::{DerivationType as HDPath, LedgerError},
};

#[cfg(all(feature = "trezor", not(target_arch = "wasm32")))]
mod trezor;
#[cfg(all(feature = "trezor", not(target_arch = "wasm32")))]
pub use trezor::{
    app::TrezorEthereum as Trezor,
    types::{DerivationType as TrezorHDPath, TrezorError},
};

#[cfg(all(feature = "yubihsm", not(target_arch = "wasm32")))]
pub use yubihsm;

#[cfg(feature = "aws")]
mod aws;
#[cfg(feature = "aws")]
pub use aws::{AwsSigner, AwsSignerError};

use async_trait::async_trait;
use ethers_core::types::{
    transaction::{eip2718::TypedTransaction, eip712::Eip712},
    Address, Signature,
};
use std::error::Error;

/// Applies [EIP155](https://github.com/ethereum/EIPs/blob/master/EIPS/eip-155.md)
pub fn to_eip155_v<T: Into<u8>>(recovery_id: T, chain_id: u64) -> u64 {
    (recovery_id.into() as u64) + 35 + chain_id * 2
}

/// Trait for signing transactions and messages
///
/// Implement this trait to support different signing modes, e.g. Ledger, hosted etc.
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait Signer: std::fmt::Debug + Send + Sync {
    type Error: Error + Send + Sync;
    /// Signs the hash of the provided message after prefixing it
    async fn sign_message<S: Send + Sync + AsRef<[u8]>>(
        &self,
        message: S,
    ) -> Result<Signature, Self::Error>;

    /// Signs the transaction
    async fn sign_transaction(&self, message: &TypedTransaction) -> Result<Signature, Self::Error>;

    /// Encodes and signs the typed data according EIP-712.
    /// Payload must implement Eip712 trait.
    async fn sign_typed_data<T: Eip712 + Send + Sync>(
        &self,
        payload: &T,
    ) -> Result<Signature, Self::Error>;

    /// Returns the signer's Ethereum Address
    fn address(&self) -> Address;

    /// Returns the signer's chain id
    fn chain_id(&self) -> u64;

    /// Sets the signer's chain id
    #[must_use]
    fn with_chain_id<T: Into<u64>>(self, chain_id: T) -> Self;
}
