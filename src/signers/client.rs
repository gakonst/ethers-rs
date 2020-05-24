use crate::{
    providers::{JsonRpcClient, Provider},
    signers::Signer,
    types::{Address, Transaction, TransactionRequest},
};

use std::ops::Deref;

#[derive(Clone, Debug)]
pub struct Client<'a, S, P> {
    pub(crate) provider: &'a Provider<P>,
    pub(crate) signer: S,
}

impl<'a, S: Signer, P: JsonRpcClient> Client<'a, S, P> {
    /// Signs the transaction and then broadcasts its RLP encoding via the `eth_sendRawTransaction`
    /// API
    pub async fn sign_and_send_transaction(
        &self,
        tx: TransactionRequest,
    ) -> Result<Transaction, P::Error> {
        // sign the transaction
        let signed_tx = self.signer.sign_transaction(tx).unwrap(); // TODO

        // broadcast it
        self.provider.send_raw_transaction(&signed_tx).await?;

        Ok(signed_tx)
    }

    pub fn address(&self) -> Address {
        self.signer.address()
    }
}

// Abuse Deref to use the Provider's methods without re-writing everything.
// This is an anti-pattern and should not be encouraged, but this improves the UX while
// keeping the LoC low
impl<'a, S, P> Deref for Client<'a, S, P> {
    type Target = &'a Provider<P>;

    fn deref(&self) -> &Self::Target {
        &self.provider
    }
}
