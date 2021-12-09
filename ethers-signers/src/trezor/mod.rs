pub mod app;
pub mod types;

use crate::Signer;
use app::TrezorEthereum;
use async_trait::async_trait;
use ethers_core::types::{
    transaction::{eip2718::TypedTransaction, eip712::Eip712},
    Address, Signature,
};
use types::TrezorError;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Signer for TrezorEthereum {
    type Error = TrezorError;

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

    /// Signs a EIP712 derived struct
    async fn sign_typed_data<T: Eip712 + Send + Sync>(
        &self,
        payload: &T,
    ) -> Result<Signature, Self::Error> {
        self.sign_typed_struct(payload).await
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
