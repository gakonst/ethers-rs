pub mod app;
pub mod types;

use crate::Signer;
use app::LedgerEthereum;
use async_trait::async_trait;
use ethers_core::types::{Address, Signature, TransactionRequest};
use types::LedgerError;

#[async_trait(?Send)]
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
    async fn sign_transaction(
        &self,
        message: &TransactionRequest,
    ) -> Result<Signature, Self::Error> {
        self.sign_tx(message, self.chain_id).await
    }

    /// Returns the signer's Ethereum Address
    fn address(&self) -> Address {
        self.address
    }
}
