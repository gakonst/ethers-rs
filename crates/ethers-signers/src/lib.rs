//! # ethers-signers
//!
//! Provides a unified interface for locally signing transactions and interacting
//! with the Ethereum JSON-RPC. You can implement the `Signer` trait to extend
//! functionality to other signers such as Hardware Security Modules, KMS etc.
//!
//! ```ignore
//! # use anyhow::Result;
//! # use ethers::{providers::HttpProvider, signers::MainnetWallet, types::TransactionRequest};
//! # use std::convert::TryFrom;
//! # async fn main() -> Result<()> {
//! // connect to the network
//! let provider = HttpProvider::try_from("http://localhost:8545")?;
//!
//! // instantiate the wallet and connect it to the provider to get a client
//! let client = "dcf2cbdd171a21c480aa7f53d77f31bb102282b3ff099c78e3118b37348c72f7"
//!     .parse::<MainnetWallet>()?
//!     .connect(&provider);
//!
//! // create a transaction
//! let tx = TransactionRequest::new()
//!     .to("vitalik.eth") // this will use ENS
//!     .value(10000);
//!
//! // send it! (this will resolve the ENS name to an address under the hood)
//! let hash = client.send_transaction(tx, None).await?;
//!
//! // get the mined tx
//! let tx = client.get_transaction(hash).await?;
//!
//! // get the receipt
//! let receipt = client.get_transaction_receipt(tx.hash).await?;
//!
//! println!("{}", serde_json::to_string(&tx)?);
//! println!("{}", serde_json::to_string(&receipt)?);
//!
//! # Ok(())
//! # }
// TODO: We might need a `SignerAsync` trait for HSM use cases?

mod wallet;
pub use wallet::Wallet;

mod client;
pub use client::Client;

use ethers_core::types::{Address, Signature, Transaction, TransactionRequest};
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
