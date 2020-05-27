//! Ethereum compatible signers
//!
//! Currently supported:
//! - [x] Private Key
//! - [ ] Encrypted Json
//! - [ ] Ledger
//! - [ ] Trezor
//!
//! Implement the `Signer` trait to add support for new signers, e.g. with Ledger.
//!
//! TODO: We might need a `SignerAsync` trait for HSM use cases?

mod wallet;
pub use wallet::Wallet;

mod client;
pub use client::Client;

use ethers_types::{Address, Signature, Transaction, TransactionRequest};
use std::error::Error;

/// Trait for signing transactions and messages
///
/// Implement this trait to support different signing modes, e.g. Ledger, hosted etc.
pub trait Signer {
    type Error: Error;
    /// Signs the hash of the provided message after prefixing it
    fn sign_message<S: AsRef<[u8]>>(&self, message: S) -> Signature;

    /// Signs the transaction
    fn sign_transaction(&self, message: TransactionRequest) -> Result<Transaction, Self::Error>;

    /// Returns the signer's Ethereum Address
    fn address(&self) -> Address;
}

use ethers_providers::networks::{Any, Mainnet};

/// A Wallet instantiated with chain_id = 1 for Ethereum Mainnet.
pub type MainnetWallet = Wallet<Mainnet>;

/// A wallet which does not use EIP-155 and does not take the chain id into account
/// when creating transactions
pub type AnyWallet = Wallet<Any>;
