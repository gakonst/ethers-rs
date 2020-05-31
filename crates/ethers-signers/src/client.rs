use crate::Signer;

use ethers_core::types::{Address, BlockNumber, NameOrAddress, TransactionRequest, TxHash};
use ethers_providers::{networks::Network, JsonRpcClient, Provider};

use std::ops::Deref;

#[derive(Clone, Debug)]
/// A client provides an interface for signing and broadcasting locally signed transactions
/// It Derefs to `Provider`, which allows interacting with the Ethereum JSON-RPC provider
/// via the same API.
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
    /// Signs and broadcasts the transaction
    pub async fn send_transaction(
        &self,
        mut tx: TransactionRequest,
        block: Option<BlockNumber>,
    ) -> Result<TxHash, P::Error> {
        if let Some(ref to) = tx.to {
            if let NameOrAddress::Name(ens_name) = to {
                let addr = self
                    .resolve_name(&ens_name)
                    .await?
                    .expect("TODO: Handle ENS name not found");
                tx.to = Some(addr.into())
            }
        }

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

    /// Returns the client's address
    pub fn address(&self) -> Address {
        self.signer
            .as_ref()
            .map(|s| s.address())
            .unwrap_or_default()
    }

    /// Returns a reference to the client's provider
    pub fn provider(&self) -> &Provider<P, N> {
        self.provider
    }

    /// Returns a reference to the client's signer, will panic if no signer is set
    pub fn signer_unchecked(&self) -> &S {
        self.signer.as_ref().expect("no signer is configured")
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
