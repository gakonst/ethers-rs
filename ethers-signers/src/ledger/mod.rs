pub mod app;
pub mod types;

use crate::Signer;
use app::LedgerEthereum;
use async_trait::async_trait;
use ethers_core::types::{
    transaction::eip2718::TypedTransaction,
    transaction::eip712::{EIP712Domain, Eip712},
    Address, Signature,
};
use types::{LedgerError, INS};

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
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

    async fn sign_typed_data<T: Eip712 + Send + Sync>(
        &self,
        payload: T,
    ) -> Result<Signature, Self::Error> {
        let hash = payload
            .encode_eip712()
            .map_err(|e| Self::Error::Eip712Error(e.to_string()))?;

        let sig = self.sign_message(hash).await?;

        Ok(sig)
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
