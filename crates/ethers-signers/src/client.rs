use crate::Signer;

use ethers_providers::{networks::Network, JsonRpcClient, Provider};
use ethers_types::{Address, BlockNumber, TransactionRequest, TxHash};

use std::ops::Deref;

#[derive(Clone, Debug)]
pub struct Client<'a, P, N, S> {
    pub(crate) provider: &'a Provider<P, N>,
    pub(crate) signer: Option<S>,
}

impl<'a, P, N, S> From<&'a Provider<P, N>> for Client<'a, P, N, S> {
    fn from(provider: &'a Provider<P, N>) -> Self {
        Client {
            provider,
            signer: None,
        }
    }
}

impl<'a, P, N, S> Client<'a, P, N, S>
where
    S: Signer,
    P: JsonRpcClient,
    N: Network,
{
    /// Signs the transaction and then broadcasts its RLP encoding via the `eth_sendRawTransaction`
    /// API
    pub async fn send_transaction(
        &self,
        mut tx: TransactionRequest,
        block: Option<BlockNumber>,
    ) -> Result<TxHash, P::Error> {
        // if there is no local signer, then the transaction should use the
        // node's signer which should already be unlocked
        let signer = if let Some(ref signer) = self.signer {
            signer
        } else {
            return self.provider.send_transaction(tx).await;
        };

        // fill any missing fields
        self.fill_transaction(&mut tx, block).await?;

        // sign the transaction
        let signed_tx = signer.sign_transaction(tx).unwrap(); // TODO

        // broadcast it
        self.provider.send_raw_transaction(&signed_tx).await?;

        Ok(signed_tx.hash)
    }

    // TODO: Convert to join'ed futures
    async fn fill_transaction(
        &self,
        tx: &mut TransactionRequest,
        block: Option<BlockNumber>,
    ) -> Result<(), P::Error> {
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

    pub fn address(&self) -> Address {
        self.signer
            .as_ref()
            .map(|s| s.address())
            .unwrap_or_default()
    }

    pub fn provider(&self) -> &Provider<P, N> {
        self.provider
    }
}

// Abuse Deref to use the Provider's methods without re-writing everything.
// This is an anti-pattern and should not be encouraged, but this improves the UX while
// keeping the LoC low
impl<'a, P, N, S> Deref for Client<'a, P, N, S>
where
    N: 'a,
{
    type Target = &'a Provider<P, N>;

    fn deref(&self) -> &Self::Target {
        &self.provider
    }
}
