mod wallet;
pub use wallet::Wallet;

mod client;
pub use client::{Client, ClientError};

use ethers_core::types::{Address, Signature, Transaction, TransactionRequest};
use ethers_providers::http::Provider;
use std::error::Error;

/// Trait for signing transactions and messages
///
/// Implement this trait to support different signing modes, e.g. Ledger, hosted etc.
// TODO: We might need a `SignerAsync` trait for HSM use cases?
pub trait Signer {
    type Error: Error;
    /// Signs the hash of the provided message after prefixing it
    fn sign_message<S: AsRef<[u8]>>(&self, message: S) -> Signature;

    /// Signs the transaction
    fn sign_transaction(&self, message: TransactionRequest) -> Result<Transaction, Self::Error>;

    /// Returns the signer's Ethereum Address
    fn address(&self) -> Address;
}

/// An HTTP client configured to work with ANY blockchain without replay protection
pub type HttpClient = Client<Provider, Wallet>;
