pub mod app;
pub mod types;

use crate::Signer;
use app::LedgerEthereum;
use async_trait::async_trait;
use ethers_core::types::{transaction::eip2718::TypedTransaction, Address, Signature};
use types::LedgerError;

#[async_trait]
impl Signer for LedgerEthereum {
    type Error = LedgerError;

    /// Signs the hash of the provided message after prefixing it
    async fn sign_message<S: Send + Sync + AsRef<[u8]>>(
        &self,
        message: S,
    ) -> Result<Signature, Self::Error> {
        self.sign_message(message).await
    }

    /// Signs the transaction
    async fn sign_transaction(&self, message: &TypedTransaction) -> Result<Signature, Self::Error> {
        self.sign_tx(message).await
    }

    /// Returns the signer's Ethereum Address
    fn address(&self) -> Address {
        self.address
    }

    fn with_chain_id<T: Into<u64>>(mut self, chain_id: T) -> Self {
        self.chain_id = chain_id.into();
        self
    }

    fn chain_id(&self) -> u64 {
        self.chain_id
    }
}
