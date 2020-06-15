use crate::{JsonRpcClient, Provider, ProviderError};
use ethers_core::types::{TransactionReceipt, TxHash};
use std::{
    future::Future,
    ops::Deref,
    pin::Pin,
    task::{Context, Poll},
};

/// A pending transaction is a transaction which has been submitted but is not yet mined.
/// `await`'ing on a pending transaction will resolve to a transaction receipt
/// once the transaction has enough `confirmations`. The default number of confirmations
/// is 1, but may be adjusted with the `confirmations` method. If the transaction does not
/// have enough confirmations or is not mined, the future will stay in the pending state.
#[derive(Clone, Debug)]
pub struct PendingTransaction<'a, P> {
    tx_hash: TxHash,
    confirmations: usize,
    provider: &'a Provider<P>,
}

impl<'a, P> PartialEq for PendingTransaction<'a, P> {
    fn eq(&self, other: &Self) -> bool {
        self.tx_hash == other.tx_hash
    }
}

impl<'a, P> PartialEq<TxHash> for PendingTransaction<'a, P> {
    fn eq(&self, other: &TxHash) -> bool {
        &self.tx_hash == other
    }
}

impl<'a, P> Eq for PendingTransaction<'a, P> {}

impl<'a, P> Deref for PendingTransaction<'a, P> {
    type Target = TxHash;

    fn deref(&self) -> &Self::Target {
        &self.tx_hash
    }
}

impl<'a, P: JsonRpcClient> PendingTransaction<'a, P> {
    /// Creates a new pending transaction poller from a hash and a provider
    pub fn new(tx_hash: TxHash, provider: &'a Provider<P>) -> Self {
        Self {
            tx_hash,
            confirmations: 1,
            provider,
        }
    }

    /// Sets the number of confirmations for the pending transaction to resolve
    /// to a receipt
    pub fn confirmations(mut self, confs: usize) -> Self {
        self.confirmations = confs;
        self
    }
}

impl<'a, P: JsonRpcClient> Future for PendingTransaction<'a, P> {
    type Output = Result<TransactionReceipt, ProviderError>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output> {
        // TODO: This hangs on the reqwest HTTP request. Why?
        let mut fut = Box::pin(self.provider.get_transaction_receipt(self.tx_hash));
        // TODO: How should we handle errors that happen due to connection issues
        // vs ones that happen due to the tx not being mined? Should we return an Option?
        if let Ok(receipt) = futures_util::ready!(fut.as_mut().poll(ctx)) {
            let inclusion_block = receipt
                .block_number
                .expect("Receipt did not have a block number. This should never happen");

            // If we requested more than 1 confirmation, we need to compare the receipt's
            // block number and the current block
            if self.confirmations > 1 {
                let mut fut = Box::pin(self.provider.get_block_number());
                let current_block = futures_util::ready!(fut.as_mut().poll(ctx))?;
                if current_block >= inclusion_block + self.confirmations {
                    return Poll::Ready(Ok(receipt));
                }
            } else {
                return Poll::Ready(Ok(receipt));
            }
        }

        // If none of the above cases were hit, just wait for the next poll
        Poll::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Http;
    use ethers_core::{types::TransactionRequest, utils::Ganache};
    use std::convert::TryFrom;

    #[tokio::test]
    async fn test_pending_tx() {
        let _ganache = Ganache::new().spawn();
        let provider = Provider::<Http>::try_from("http://localhost:8545").unwrap();
        let accounts = provider.get_accounts().await.unwrap();
        let tx = TransactionRequest::pay(accounts[0], 1000).from(accounts[0]);

        let pending_tx = provider.send_transaction(tx).await.unwrap();

        let receipt = provider.get_transaction_receipt(pending_tx.tx_hash).await.unwrap();

        // the pending tx resolves to the same receipt
        let tx_receipt = pending_tx.await.unwrap();
        assert_eq!(receipt, tx_receipt);
    }
}
