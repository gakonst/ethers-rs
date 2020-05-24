use crate::{
    providers::{JsonRpcClient, Provider},
    signers::Signer,
    types::{Transaction, TxHash, UnsignedTransaction},
};

#[derive(Clone, Debug)]
pub struct Client<'a, S, P> {
    pub provider: &'a Provider<P>,
    pub signer: S,
}

impl<'a, S: Signer, P: JsonRpcClient> Client<'a, S, P> {
    pub async fn send_transaction(&self, tx: UnsignedTransaction) -> Result<Transaction, P::Error> {
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
    ) -> Result<Transaction, P::Error> {
        self.provider.get_transaction(hash).await
    }
}
