use crate::{
    providers::{JsonRpcClient, Provider},
    signers::Signer,
    types::{Address, BlockNumber, Transaction, TransactionRequest},
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
        mut tx: TransactionRequest,
        block: Option<BlockNumber>,
    ) -> Result<Transaction, P::Error> {
        // TODO: Convert to join'ed futures
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
