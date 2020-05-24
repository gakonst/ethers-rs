use crate::{
    jsonrpc::ClientError,
    providers::{Provider, ProviderTrait},
    types::{Transaction, TxHash, UnsignedTransaction},
    wallet::Signer,
};

use thiserror::Error;

#[derive(Clone, Debug)]
pub struct Client<'a, S> {
    pub(super) provider: &'a Provider,
    pub signer: S,
}

#[derive(Error, Debug)]
pub enum SignerError {
    #[error(transparent)]
    ClientError(#[from] ClientError),
    #[error("no provider was found")]
    NoProvider,
}

impl<'a, S: Signer> Client<'a, S> {
    pub async fn send_transaction(
        &self,
        tx: UnsignedTransaction,
    ) -> Result<Transaction, SignerError> {
        // sign the transaction
        let signed_tx = self.signer.sign_transaction(tx.clone());

        // broadcast it
        self.provider.send_raw_transaction(&signed_tx.rlp()).await?;

        Ok(signed_tx)
    }

    // TODO: Forward all other calls to the provider
    pub async fn get_transaction<T: Send + Sync + Into<TxHash>>(
        &self,
        hash: T,
    ) -> Result<Transaction, ClientError> {
        self.provider.get_transaction(hash).await
    }
}
