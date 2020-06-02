use crate::Signer;

use ethers_core::types::{
    Address, BlockNumber, Bytes, NameOrAddress, Signature, TransactionRequest, TxHash,
};
use ethers_providers::{JsonRpcClient, Provider, ProviderError};

use std::ops::Deref;
use thiserror::Error;

#[derive(Clone, Debug)]
/// A client provides an interface for signing and broadcasting locally signed transactions
/// It Derefs to `Provider`, which allows interacting with the Ethereum JSON-RPC provider
/// via the same API.
pub struct Client<P, S> {
    pub(crate) provider: Provider<P>,
    pub(crate) signer: Option<S>,
    pub(crate) address: Address,
}

impl<P, S> From<Provider<P>> for Client<P, S> {
    fn from(provider: Provider<P>) -> Self {
        Client {
            provider,
            signer: None,
            address: Address::zero(),
        }
    }
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error(transparent)]
    ProviderError(#[from] ProviderError),

    #[error(transparent)]
    SignerError(#[from] Box<dyn std::error::Error + Send + Sync>),

    #[error("ens name not found: {0}")]
    EnsError(String),
}

// Helper functions for locally signing transactions
impl<P, S> Client<P, S>
where
    S: Signer,
    P: JsonRpcClient,
{
    /// Signs a message with the internal signer, or if none is present it will make a call to
    /// the connected node's `eth_call` API.
    pub async fn sign_message<T: Into<Bytes>>(&self, msg: T) -> Result<Signature, ClientError> {
        let msg = msg.into();
        Ok(if let Some(ref signer) = self.signer {
            signer.sign_message(msg)
        } else {
            self.provider.sign(&msg, &self.address).await?
        })
    }

    /// Signs and broadcasts the transaction
    pub async fn send_transaction(
        &self,
        mut tx: TransactionRequest,
        block: Option<BlockNumber>,
    ) -> Result<TxHash, ClientError> {
        if let Some(ref to) = tx.to {
            if let NameOrAddress::Name(ens_name) = to {
                let addr = self
                    .resolve_name(&ens_name)
                    .await?
                    .ok_or_else(|| ClientError::EnsError(ens_name.to_owned()))?;
                tx.to = Some(addr.into())
            }
        }

        // if there is no local signer, then the transaction should use the
        // node's signer which should already be unlocked
        let signer = if let Some(ref signer) = self.signer {
            signer
        } else {
            return Ok(self.provider.send_transaction(tx).await?);
        };

        // fill any missing fields
        self.fill_transaction(&mut tx, block).await?;

        // sign the transaction with the network
        let signed_tx = signer.sign_transaction(tx).map_err(Into::into)?;

        // broadcast it
        self.provider.send_raw_transaction(&signed_tx).await?;

        Ok(signed_tx.hash)
    }

    // TODO: Convert to join'ed futures
    async fn fill_transaction(
        &self,
        tx: &mut TransactionRequest,
        block: Option<BlockNumber>,
    ) -> Result<(), ClientError> {
        // get the gas price
        if tx.gas_price.is_none() {
            tx.gas_price = Some(self.provider.get_gas_price().await?);
        }

        // estimate the gas
        if tx.gas.is_none() {
            tx.from = Some(self.address());
            tx.gas = Some(self.provider.estimate_gas(&tx, block).await?);
        }

        // set our nonce
        if tx.nonce.is_none() {
            tx.nonce = Some(
                self.provider
                    .get_transaction_count(self.address(), block)
                    .await?,
            );
        }

        Ok(())
    }

    /// Returns the client's address
    pub fn address(&self) -> Address {
        self.signer
            .as_ref()
            .map(|s| s.address())
            .unwrap_or_default()
    }

    /// Returns a reference to the client's provider
    pub fn provider(&self) -> &Provider<P> {
        &self.provider
    }

    /// Returns a reference to the client's signer, will panic if no signer is set
    pub fn signer_unchecked(&self) -> &S {
        self.signer.as_ref().expect("no signer is configured")
    }

    /// Sets the signer
    pub fn with_signer(&mut self, signer: S) -> &mut Self {
        self.signer = Some(signer);
        self
    }

    /// Sets the provider
    pub fn with_provider(&mut self, provider: Provider<P>) -> &mut Self {
        self.provider = provider;
        self
    }

    /// Sets the account to be used with the `eth_sign` API calls
    pub fn from(&mut self, address: Address) -> &mut Self {
        self.address = address;
        self
    }
}

// Abuse Deref to use the Provider's methods without re-writing everything.
// This is an anti-pattern and should not be encouraged, but this improves the UX while
// keeping the LoC low
impl<P, S> Deref for Client<P, S> {
    type Target = Provider<P>;

    fn deref(&self) -> &Self::Target {
        &self.provider
    }
}
