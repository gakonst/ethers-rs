use std::time::Duration;

use ethers_core::types::H256;
use tokio::time::{self, Interval};

use crate::{types::TransactionReceipt, Connection, Provider, ProviderError};

/// A pending transaction that can be polled until the reference transaction has
/// either been dropped or included in a block.
pub struct PendingTransaction<C> {
    /// The hash of the submitted pending transaction.
    pub txn_hash: H256,
    provider: Provider<C>,
}

impl<C: Connection> PendingTransaction<C> {
    /// Wraps the given `txn_hash` and `provider` that will be used to poll the
    /// transaction's receipt.
    pub fn new(txn_hash: H256, provider: Provider<C>) -> Self {
        Self { txn_hash, provider }
    }

    /// Polls the [`TransactionReceipt`] for the pending transaction.
    ///
    /// The number of `confirmations`
    pub async fn poll_receipt(
        self,
        confirmations: usize,
        interval: Option<Duration>,
    ) -> Result<Option<TransactionReceipt>, Box<ProviderError>> {
        let mut interval = time::interval(interval.unwrap_or(Duration::from_secs(17)));
        loop {
            // first tick resolves immediately
            interval.tick().await;

            // return `None`, if the txn was dropped from the mempool
            let txn = match self.provider.get_transaction_by_hash(&self.txn_hash).await? {
                Some(txn) => txn,
                None => return Ok(None),
            };

            // if the transaction has not yet been confirmed, poll again later
            let number = match txn.block_number {
                Some(number) => number.low_u64(),
                None => continue,
            };

            let receipt = self.provider.get_transaction_receipt(&self.txn_hash).await?;
            if confirmations == 0 {
                return Ok(receipt);
            } else {
                let wanted_block_number = number + confirmations as u64;
                return self.poll_block_confirmations(interval, wanted_block_number).await;
            }
        }
    }

    async fn poll_block_confirmations(
        self,
        mut interval: Interval,
        wanted_block_number: u64,
    ) -> Result<Option<TransactionReceipt>, Box<ProviderError>> {
        loop {
            interval.tick().await;

            let latest_block = self.provider.get_block_number().await?;
            if latest_block < wanted_block_number {
                continue;
            }

            return self.provider.get_transaction_receipt(&self.txn_hash).await;
        }
    }
}
