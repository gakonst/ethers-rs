pub mod app;
pub mod types;

use crate::Signer;
use app::LedgerEthereum;
use async_trait::async_trait;
use ethers_core::types::{
    transaction::eip2718::TypedTransaction, transaction::eip712::Eip712, Address, Signature,
};
use types::LedgerError;

const EIP712_MIN_VERSION: &str = ">=1.6.0";

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

    /// Signs a EIP712 derived struct
    async fn sign_typed_data<T: Eip712 + Send + Sync>(
        &self,
        payload: T,
    ) -> Result<Signature, Self::Error> {
        // See comment for v1.6.0 requirement 
        // https://github.com/LedgerHQ/app-ethereum/issues/105#issuecomment-765316999
        let req = semver::VersionReq::parse(EIP712_MIN_VERSION)?;
        let version = semver::Version::parse(&self.version().await?)?;

        // Enforce app version is greater than EIP712_MIN_VERSION
        if !req.matches(&version) {
            return Err(Self::Error::UnsupportedAppVersion(EIP712_MIN_VERSION.to_string()));
        }

        let domain = payload.domain_separator()
            .map_err(|e| Self::Error::Eip712Error(e.to_string()))?;
        let struct_hash = payload.struct_hash()
            .map_err(|e| Self::Error::Eip712Error(e.to_string()))?;

        let sig = self.sign_typed_struct(domain, struct_hash).await?;

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
