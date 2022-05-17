use std::time::Duration;

use ethers_core::types::H256;
use tokio::time::Interval;

use crate::{types::TransactionReceipt, Connection, Provider, ProviderError};

pub struct PendingTransaction<C> {
    pub txn_hash: H256,
    provider: Provider<C>,
}

impl<C: Connection> PendingTransaction<C> {
    pub fn new(txn_hash: H256, provider: Provider<C>) -> Self {
        Self { txn_hash, provider }
    }

    pub async fn poll_receipt(
        self,
        confirmations: usize,
        interval: Option<Duration>,
    ) -> Result<Option<TransactionReceipt>, Box<ProviderError>> {
        let mut interval = tokio::time::interval(interval.unwrap_or(Duration::from_secs(17)));
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

            // return `None`, if the receipt stops existing at some point (e.g.,
            // due to a re-org)
            let receipt = match self.provider.get_transaction_receipt(&self.txn_hash).await? {
                Some(receipt) => receipt,
                None => return Ok(None),
            };

            if receipt.block_number.low_u64() >= wanted_block_number {
                return Ok(Some(receipt));
            }
        }
    }
}
