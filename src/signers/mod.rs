//! Sign and broadcast transactions
mod networks;
pub use networks::instantiated::*;
use networks::Network;

mod wallet;
pub use wallet::Wallet;

mod client;
pub(crate) use client::Client;

use crate::types::{Signature, Transaction, UnsignedTransaction};

/// Trait for signing transactions and messages
///
/// Implement this trait to support different signing modes, e.g. Ledger, hosted etc.
pub trait Signer {
    /// Signs the hash of the provided message after prefixing it
    fn sign_message<S: AsRef<[u8]>>(&self, message: S) -> Signature;

    /// Signs the transaction
    fn sign_transaction(&self, message: UnsignedTransaction) -> Transaction;
}
