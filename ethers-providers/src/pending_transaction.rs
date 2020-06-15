use crate::{JsonRpcClient, Provider, ProviderError};
use ethers_core::types::{TransactionReceipt, TxHash, U64};
use pin_project::pin_project;
use std::{
    fmt,
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
#[pin_project]
pub struct PendingTransaction<'a, P> {
    tx_hash: TxHash,
    confirmations: usize,
    provider: &'a Provider<P>,
    state: PendingTxState<'a>,
}

impl<'a, P: JsonRpcClient> PendingTransaction<'a, P> {
    /// Creates a new pending transaction poller from a hash and a provider
    pub fn new(tx_hash: TxHash, provider: &'a Provider<P>) -> Self {
        let fut = Box::pin(provider.get_transaction_receipt(tx_hash));
        Self {
            tx_hash,
            confirmations: 1,
            provider,
            state: PendingTxState::GettingReceipt(fut),
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
        let this = self.project();

        match this.state {
            PendingTxState::GettingReceipt(fut) => {
                let receipt = futures_util::ready!(fut.as_mut().poll(ctx))?;
                *this.state = PendingTxState::CheckingReceipt(Box::new(receipt))
            }
            PendingTxState::CheckingReceipt(receipt) => {
                // If we requested more than 1 confirmation, we need to compare the receipt's
                // block number and the current block
                if *this.confirmations > 1 {
                    let fut = Box::pin(this.provider.get_block_number());
                    *this.state =
                        PendingTxState::GettingBlockNumber(fut, Box::new(*receipt.clone()))
                } else {
                    let receipt = *receipt.clone();
                    *this.state = PendingTxState::Completed;
                    return Poll::Ready(Ok(receipt));
                }
            }
            PendingTxState::GettingBlockNumber(fut, receipt) => {
                let inclusion_block = receipt
                    .block_number
                    .expect("Receipt did not have a block number. This should never happen");

                let current_block = futures_util::ready!(fut.as_mut().poll(ctx))?;

                // if the transaction has at least K confirmations, return the receipt
                // (subtract 1 since the tx already has 1 conf when it's mined)
                if current_block >= inclusion_block + *this.confirmations - 1 {
                    let receipt = *receipt.clone();
                    *this.state = PendingTxState::Completed;
                    return Poll::Ready(Ok(receipt));
                } else {
                    // we need to re-instantiate the get_block_number future so that
                    // we poll again
                    let fut = Box::pin(this.provider.get_block_number());
                    *this.state = PendingTxState::GettingBlockNumber(fut, receipt.clone());
                    return Poll::Pending;
                }
            }
            PendingTxState::Completed => {
                panic!("polled pending transaction future after completion")
            }
        };

        Poll::Pending
    }
}

impl<'a, P> fmt::Debug for PendingTransaction<'a, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PendingTransaction")
            .field("tx_hash", &self.tx_hash)
            .field("confirmations", &self.confirmations)
            .field("state", &self.state)
            .finish()
    }
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

// Helper type alias
type PinBoxFut<'a, T> = Pin<Box<dyn Future<Output = Result<T, ProviderError>> + 'a>>;

// We box the TransactionReceipts to keep the enum small.
enum PendingTxState<'a> {
    /// Polling the blockchain for the receipt
    GettingReceipt(PinBoxFut<'a, TransactionReceipt>),

    /// Polling the blockchain for the current block number
    GettingBlockNumber(PinBoxFut<'a, U64>, Box<TransactionReceipt>),

    /// If the pending tx required only 1 conf, it will return early. Otherwise it will
    /// proceed to the next state which will poll the block number until there have been
    /// enough confirmations
    CheckingReceipt(Box<TransactionReceipt>),

    /// Future has completed and should panic if polled again
    Completed,
}

impl<'a> fmt::Debug for PendingTxState<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = match self {
            PendingTxState::GettingReceipt(_) => "GettingReceipt",
            PendingTxState::GettingBlockNumber(_, _) => "GettingBlockNumber",
            PendingTxState::CheckingReceipt(_) => "CheckingReceipt",
            PendingTxState::Completed => "Completed",
        };

        f.debug_struct("PendingTxState")
            .field("state", &state)
            .finish()
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

        let receipt = provider
            .get_transaction_receipt(pending_tx.tx_hash)
            .await
            .unwrap();

        // the pending tx resolves to the same receipt
        let tx_receipt = pending_tx.confirmations(1).await.unwrap();
        assert_eq!(receipt, tx_receipt);
    }
}
