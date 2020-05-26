//! Sign and broadcast transactions
//!
//! Implement the `Signer` trait to add support for new signers, e.g. with Ledger.
//!
//! TODO: We might need a `SignerAsync` trait for HSM use cases?
mod networks;
pub use networks::instantiated::*;
use networks::Network;

mod wallet;
pub use wallet::Wallet;

mod client;
pub(crate) use client::Client;

use crate::types::{Address, Signature, Transaction, TransactionRequest};
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
