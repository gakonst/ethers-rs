mod geometric;
use ethers_core::types::transaction::eip2718::TypedTransaction;
pub use geometric::GeometricGasPrice;

mod linear;
pub use linear::LinearGasPrice;

use async_trait::async_trait;
use ethers_core::types::{BlockId, TransactionRequest, TxHash, U256};
use ethers_providers::{interval, FromErr, Middleware, PendingTransaction, StreamExt};
use futures_util::lock::Mutex;
use instant::Instant;
use std::{pin::Pin, sync::Arc};
use thiserror::Error;

#[cfg(not(target_arch = "wasm32"))]
use tokio::spawn;

#[cfg(target_arch = "wasm32")]
type WatcherFuture<'a> = Pin<Box<dyn futures_util::stream::Stream<Item = ()> + 'a>>;
#[cfg(not(target_arch = "wasm32"))]
type WatcherFuture<'a> = Pin<Box<dyn futures_util::stream::Stream<Item = ()> + Send + 'a>>;

/// Trait for fetching updated gas prices after a transaction has been first
/// broadcast
pub trait GasEscalator: Send + Sync + std::fmt::Debug {
    /// Given the initial gas price and the time elapsed since the transaction's
    /// first broadcast, it returns the new gas price
    fn get_gas_price(&self, initial_price: U256, time_elapsed: u64) -> U256;
}

#[derive(Debug, Clone)]
/// The frequency at which transactions will be bumped
pub enum Frequency {
    /// On a per block basis using the eth_newBlock filter
    PerBlock,
    /// On a duration basis (in milliseconds)
    Duration(u64),
}

#[derive(Debug)]
/// A Gas escalator allows bumping transactions' gas price to avoid getting them
/// stuck in the memory pool.
///
/// ```no_run
/// use ethers_providers::{Provider, Http};
/// use ethers_middleware::{
///     gas_escalator::{GeometricGasPrice, Frequency, GasEscalatorMiddleware},
///     gas_oracle::{EthGasStation, GasCategory, GasOracleMiddleware},
/// };
/// use std::{convert::TryFrom, time::Duration, sync::Arc};
///
/// let provider = Provider::try_from("http://localhost:8545")
///     .unwrap()
///     .interval(Duration::from_millis(2000u64));
///
/// let provider = {
///     let escalator = GeometricGasPrice::new(5.0, 10u64, None::<u64>);
///     GasEscalatorMiddleware::new(provider, escalator, Frequency::PerBlock)
/// };
///
/// // ... proceed to wrap it in other middleware
/// let gas_oracle = EthGasStation::new(None).category(GasCategory::SafeLow);
/// let provider = GasOracleMiddleware::new(provider, gas_oracle);
/// ```
pub struct GasEscalatorMiddleware<M, E> {
    pub(crate) inner: Arc<M>,
    pub(crate) escalator: E,
    /// The transactions which are currently being monitored for escalation
    #[allow(clippy::type_complexity)]
    pub txs: Arc<Mutex<Vec<(TxHash, TransactionRequest, Instant, Option<BlockId>)>>>,
    frequency: Frequency,
}

impl<M, E: Clone> Clone for GasEscalatorMiddleware<M, E> {
    fn clone(&self) -> Self {
        GasEscalatorMiddleware {
            inner: self.inner.clone(),
            escalator: self.escalator.clone(),
            txs: self.txs.clone(),
            frequency: self.frequency.clone(),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<M, E> Middleware for GasEscalatorMiddleware<M, E>
where
    M: Middleware,
    E: GasEscalator,
{
    type Error = GasEscalatorError<M>;
    type Provider = M::Provider;
    type Inner = M;

    fn inner(&self) -> &M {
        &self.inner
    }

    async fn send_transaction<T: Into<TypedTransaction> + Send + Sync>(
        &self,
        tx: T,
        block: Option<BlockId>,
    ) -> Result<PendingTransaction<'_, Self::Provider>, Self::Error> {
        let tx = tx.into();

        let pending_tx = self
            .inner()
            .send_transaction(tx.clone(), block)
            .await
            .map_err(GasEscalatorError::MiddlewareError)?;

        let tx = match tx {
            TypedTransaction::Legacy(inner) => inner,
            TypedTransaction::Eip2930(inner) => inner.tx,
            _ => return Err(GasEscalatorError::UnsupportedTxType),
        };

        // insert the tx in the pending txs
        let mut lock = self.txs.lock().await;
        lock.push((*pending_tx, tx, Instant::now(), block));

        Ok(pending_tx)
    }
}

impl<M, E> GasEscalatorMiddleware<M, E>
where
    M: Middleware,
    E: GasEscalator,
{
    /// Initializes the middleware with the provided gas escalator and the chosen
    /// escalation frequency (per block or per second)
    #[allow(clippy::let_and_return)]
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(inner: M, escalator: E, frequency: Frequency) -> Self
    where
        E: Clone + 'static,
        M: Clone + 'static,
    {
        use tracing_futures::Instrument;

        let this = Self {
            inner: Arc::new(inner),
            escalator,
            frequency,
            txs: Arc::new(Mutex::new(Vec::new())),
        };

        {
            let this2 = this.clone();
            spawn(async move {
                this2.escalate().instrument(tracing::trace_span!("gas-escalation")).await.unwrap();
            });
        }

        this
    }

    /// Re-broadcasts pending transactions with a gas price escalator
    pub async fn escalate(&self) -> Result<(), GasEscalatorError<M>> {
        // the escalation frequency is either on a per-block basis, or on a duration basis
        let mut watcher: WatcherFuture = match self.frequency {
            Frequency::PerBlock => Box::pin(
                self.inner
                    .watch_blocks()
                    .await
                    .map_err(GasEscalatorError::MiddlewareError)?
                    .map(|_| ()),
            ),
            Frequency::Duration(ms) => Box::pin(interval(std::time::Duration::from_millis(ms))),
        };

        while watcher.next().await.is_some() {
            let now = Instant::now();
            let mut txs = self.txs.lock().await;
            let len = txs.len();

            // Pop all transactions and re-insert those that have not been included yet
            for _ in 0..len {
                // this must never panic as we're explicitly within bounds
                let (tx_hash, mut replacement_tx, time, priority) =
                    txs.pop().expect("should have element in vector");

                let receipt = self.get_transaction_receipt(tx_hash).await?;
                tracing::trace!(tx_hash = ?tx_hash, "checking if exists");
                if receipt.is_none() {
                    let old_gas_price = replacement_tx.gas_price.expect("gas price must be set");
                    // Get the new gas price based on how much time passed since the
                    // tx was last broadcast
                    let new_gas_price = self
                        .escalator
                        .get_gas_price(old_gas_price, now.duration_since(time).as_secs());

                    let new_txhash = if new_gas_price != old_gas_price {
                        // bump the gas price
                        replacement_tx.gas_price = Some(new_gas_price);

                        // the tx hash will be different so we need to update it
                        match self.inner().send_transaction(replacement_tx.clone(), priority).await
                        {
                            Ok(new_tx_hash) => {
                                let new_tx_hash = *new_tx_hash;
                                tracing::trace!(
                                    old_tx_hash = ?tx_hash,
                                    new_tx_hash = ?new_tx_hash,
                                    old_gas_price = ?old_gas_price,
                                    new_gas_price = ?new_gas_price,
                                    "escalated"
                                );
                                new_tx_hash
                            }
                            Err(err) => {
                                if err.to_string().contains("nonce too low") {
                                    // ignore "nonce too low" errors because they
                                    // may happen if we try to broadcast a higher
                                    // gas price tx when one of the previous ones
                                    // was already mined (meaning we also do not
                                    // push it back to the pending txs vector)
                                    continue
                                } else {
                                    return Err(GasEscalatorError::MiddlewareError(err))
                                }
                            }
                        }
                    } else {
                        tx_hash
                    };

                    txs.push((new_txhash, replacement_tx, time, priority));
                }
            }
        }

        Ok(())
    }
}

// Boilerplate
impl<M: Middleware> FromErr<M::Error> for GasEscalatorError<M> {
    fn from(src: M::Error) -> GasEscalatorError<M> {
        GasEscalatorError::MiddlewareError(src)
    }
}

#[derive(Error, Debug)]
/// Error thrown when the GasEscalator interacts with the blockchain
pub enum GasEscalatorError<M: Middleware> {
    #[error("{0}")]
    /// Thrown when an internal middleware errors
    MiddlewareError(M::Error),

    #[error("Gas escalation is only supported for EIP2930 or Legacy transactions")]
    UnsupportedTxType,
}
