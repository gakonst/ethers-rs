//! A middleware supporting development-specific JSON RPC methods
//!
//! # Example
//!
//! ```no_run
//! use ethers_providers::{Provider, Http, Middleware, DevRpcMiddleware};
//! use ethers_core::types::TransactionRequest;
//! use ethers_core::utils::Anvil;
//!
//! # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
//! let anvil = Anvil::new().spawn();
//! let provider = Provider::<Http>::try_from(anvil.endpoint())?;
//! let client = DevRpcMiddleware::new(provider);
//!
//! // snapshot the initial state
//! let block0 = client.get_block_number().await?;
//! let snap_id = client.snapshot().await?;
//!
//! // send a transaction
//! let accounts = client.get_accounts().await?;
//! let from = accounts[0];
//! let to = accounts[1];
//! let balance_before = client.get_balance(to, None).await?;
//! let tx = TransactionRequest::new().to(to).value(1000).from(from);
//! client.send_transaction(tx, None).await?.await?;
//! let balance_after = client.get_balance(to, None).await?;
//! assert_eq!(balance_after, balance_before + 1000);
//!
//! // revert to snapshot
//! client.revert_to_snapshot(snap_id).await?;
//! let balance_after_revert = client.get_balance(to, None).await?;
//! assert_eq!(balance_after_revert, balance_before);
//! # Ok(()) }
//! ```

use crate::{Middleware, MiddlewareError, ProviderError};
use async_trait::async_trait;
use ethers_core::types::U256;
use thiserror::Error;

use std::fmt::Debug;

/// `DevRpcMiddleware`
#[derive(Clone, Debug)]
pub struct DevRpcMiddleware<M>(M);

/// DevRpcMiddleware Errors
#[derive(Error, Debug)]
pub enum DevRpcMiddlewareError<M: Middleware> {
    /// Internal Middleware error
    #[error("{0}")]
    MiddlewareError(M::Error),

    /// Internal Provider error
    #[error("{0}")]
    ProviderError(ProviderError),

    /// Attempted to revert to unavailable snapshot
    #[error("Could not revert to snapshot")]
    NoSnapshot,
}

#[async_trait]
impl<M: Middleware> Middleware for DevRpcMiddleware<M> {
    type Error = DevRpcMiddlewareError<M>;
    type Provider = M::Provider;
    type Inner = M;

    fn inner(&self) -> &M {
        &self.0
    }
}

impl<M: Middleware> MiddlewareError for DevRpcMiddlewareError<M> {
    type Inner = M::Error;

    fn from_err(src: M::Error) -> DevRpcMiddlewareError<M> {
        DevRpcMiddlewareError::MiddlewareError(src)
    }

    fn as_inner(&self) -> Option<&Self::Inner> {
        match self {
            DevRpcMiddlewareError::MiddlewareError(e) => Some(e),
            _ => None,
        }
    }
}

impl<M> From<ProviderError> for DevRpcMiddlewareError<M>
where
    M: Middleware,
{
    fn from(src: ProviderError) -> Self {
        Self::ProviderError(src)
    }
}

impl<M: Middleware> DevRpcMiddleware<M> {
    /// Instantiate a new `DevRpcMiddleware`
    pub fn new(inner: M) -> Self {
        Self(inner)
    }

    /// Create a new snapshot on the DevRpc node. Return the Snapshot ID
    ///
    /// ### Note
    ///
    /// Ganache, Hardhat and Anvil increment snapshot ID even if no state has changed
    pub async fn snapshot(&self) -> Result<U256, DevRpcMiddlewareError<M>> {
        self.provider().request::<(), U256>("evm_snapshot", ()).await.map_err(From::from)
    }

    /// Revert the state of the DevRpc node to the Snapshot, specified by its ID
    pub async fn revert_to_snapshot(&self, id: U256) -> Result<(), DevRpcMiddlewareError<M>> {
        let ok = self
            .provider()
            .request::<[U256; 1], bool>("evm_revert", [id])
            .await
            .map_err(DevRpcMiddlewareError::ProviderError)?;
        if ok {
            Ok(())
        } else {
            Err(DevRpcMiddlewareError::NoSnapshot)
        }
    }
}

#[cfg(test)]
// Celo blocks can not get parsed when used with Ganache
#[cfg(not(feature = "celo"))]
mod tests {
    use super::*;
    use crate::{Http, Provider};
    use ethers_core::utils::Anvil;
    use std::convert::TryFrom;

    #[tokio::test]
    async fn test_snapshot() {
        let anvil = Anvil::new().spawn();
        let provider = Provider::<Http>::try_from(anvil.endpoint()).unwrap();
        let client = DevRpcMiddleware::new(provider);

        // snapshot initial state
        let block0 = client.get_block_number().await.unwrap();
        let time0 = client.get_block(block0).await.unwrap().unwrap().timestamp;
        let snap_id0 = client.snapshot().await.unwrap();

        // mine a new block
        client.provider().mine(1).await.unwrap();

        // snapshot state
        let block1 = client.get_block_number().await.unwrap();
        let time1 = client.get_block(block1).await.unwrap().unwrap().timestamp;
        let snap_id1 = client.snapshot().await.unwrap();

        // mine some blocks
        client.provider().mine(5).await.unwrap();

        // snapshot state
        let block2 = client.get_block_number().await.unwrap();
        let time2 = client.get_block(block2).await.unwrap().unwrap().timestamp;
        let snap_id2 = client.snapshot().await.unwrap();

        // mine some blocks
        client.provider().mine(5).await.unwrap();

        // revert_to_snapshot should reset state to snap id
        client.revert_to_snapshot(snap_id2).await.unwrap();
        let block = client.get_block_number().await.unwrap();
        let time = client.get_block(block).await.unwrap().unwrap().timestamp;
        assert_eq!(block, block2);
        assert_eq!(time, time2);

        client.revert_to_snapshot(snap_id1).await.unwrap();
        let block = client.get_block_number().await.unwrap();
        let time = client.get_block(block).await.unwrap().unwrap().timestamp;
        assert_eq!(block, block1);
        assert_eq!(time, time1);

        // revert_to_snapshot should throw given non-existent or
        // previously used snapshot
        let result = client.revert_to_snapshot(snap_id1).await;
        assert!(result.is_err());

        client.revert_to_snapshot(snap_id0).await.unwrap();
        let block = client.get_block_number().await.unwrap();
        let time = client.get_block(block).await.unwrap().unwrap().timestamp;
        assert_eq!(block, block0);
        assert_eq!(time, time0);
    }
}
