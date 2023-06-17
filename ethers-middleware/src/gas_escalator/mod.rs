mod geometric;
pub use geometric::GeometricGasPrice;

mod linear;
pub use linear::LinearGasPrice;

use async_trait::async_trait;

use futures_channel::oneshot;
use futures_util::{lock::Mutex, select_biased};
use instant::Instant;
use std::{pin::Pin, sync::Arc};
use thiserror::Error;
use tracing_futures::Instrument;

use ethers_core::types::{
    transaction::eip2718::TypedTransaction, BlockId, TransactionRequest, TxHash, U256,
};
use ethers_providers::{interval, Middleware, MiddlewareError, PendingTransaction, StreamExt};

#[cfg(not(target_arch = "wasm32"))]
use tokio::spawn;

type ToEscalate = Arc<Mutex<Vec<(TxHash, TransactionRequest, Instant, Option<BlockId>)>>>;

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

/// Error thrown when the GasEscalator interacts with the blockchain
#[derive(Debug, Error)]
pub enum GasEscalatorError<M: Middleware> {
    #[error("{0}")]
    /// Thrown when an internal middleware errors
    MiddlewareError(M::Error),

    #[error("Gas escalation is only supported for EIP2930 or Legacy transactions")]
    UnsupportedTxType,
}

// Boilerplate
impl<M: Middleware> MiddlewareError for GasEscalatorError<M> {
    type Inner = M::Error;

    fn from_err(src: M::Error) -> GasEscalatorError<M> {
        GasEscalatorError::MiddlewareError(src)
    }

    fn as_inner(&self) -> Option<&Self::Inner> {
        match self {
            GasEscalatorError::MiddlewareError(e) => Some(e),
            _ => None,
        }
    }
}

/// The frequency at which transactions will be bumped
#[derive(Debug, Clone, Copy)]
pub enum Frequency {
    /// On a per block basis using the eth_newBlock filter
    PerBlock,
    /// On a duration basis (in milliseconds)
    Duration(u64),
}

#[derive(Debug)]
pub(crate) struct GasEscalatorMiddlewareInternal<M> {
    pub(crate) inner: Arc<M>,
    /// The transactions which are currently being monitored for escalation
    #[allow(clippy::type_complexity)]
    pub txs: ToEscalate,
    _background: oneshot::Sender<()>,
}

/// A Gas escalator allows bumping transactions' gas price to avoid getting them
/// stuck in the memory pool.
///
/// GasEscalator runs a background task which monitors the blockchain for tx
/// confirmation, and bumps fees over time if txns do not occur. This task
/// periodically loops over a stored history of sent transactions, and checks
/// if any require fee bumps. If so, it will resend the same transaction with a
/// higher fee.
///
/// Using [`GasEscalatorMiddleware::new`] will create a new instance of the
/// background task. Using [`GasEscalatorMiddleware::clone`] will crate a new
/// instance of the middleware, but will not create a new background task. The
/// background task is shared among all clones.
///
/// ## Footgun
///
/// If you drop the middleware, the background task will be dropped as well,
/// and any transactions you have sent will stop escalating. We recommend
/// holding an instance of the middleware throughout your application's
/// lifecycle, or leaking an `Arc` of it so that it is never dropped.
///
/// ## Outstanding issue
///
/// This task is fallible, and will stop if the provider's connection is lost.
/// If this happens, the middleware will become unable to properly escalate gas
/// prices. Transactions will still be dispatched, but no fee-bumping will
/// happen. This will also cause a memory leak, as the middleware will keep
/// appending to the list of transactions to escalate (and nothing will ever
/// clear that list).
///
/// We intend to fix this issue in a future release.
///
/// ## Example
///
/// ```no_run
/// use ethers_providers::{Provider, Http};
/// use ethers_middleware::{
///     gas_escalator::{GeometricGasPrice, Frequency, GasEscalatorMiddleware},
///     gas_oracle::{GasNow, GasCategory, GasOracleMiddleware},
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
/// let gas_oracle = GasNow::new().category(GasCategory::SafeLow);
/// let provider = GasOracleMiddleware::new(provider, gas_oracle);
/// ```
#[derive(Debug, Clone)]
pub struct GasEscalatorMiddleware<M> {
    pub(crate) inner: Arc<GasEscalatorMiddlewareInternal<M>>,
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<M> Middleware for GasEscalatorMiddleware<M>
where
    M: Middleware,
{
    type Error = GasEscalatorError<M>;
    type Provider = M::Provider;
    type Inner = M;

    fn inner(&self) -> &Self::Inner {
        &self.inner.inner
    }

    async fn send_transaction<T: Into<TypedTransaction> + Send + Sync>(
        &self,
        tx: T,
        block: Option<BlockId>,
    ) -> Result<PendingTransaction<'_, Self::Provider>, Self::Error> {
        self.inner.send_transaction(tx, block).await
    }
}

impl<M> GasEscalatorMiddlewareInternal<M>
where
    M: Middleware,
{
    async fn send_transaction<T: Into<TypedTransaction> + Send + Sync>(
        &self,
        tx: T,
        block: Option<BlockId>,
    ) -> Result<PendingTransaction<'_, M::Provider>, GasEscalatorError<M>> {
        let tx = tx.into();

        let pending_tx = self
            .inner
            .send_transaction(tx.clone(), block)
            .await
            .map_err(MiddlewareError::from_err)?;

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

impl<M> GasEscalatorMiddleware<M>
where
    M: Middleware,
{
    /// Initializes the middleware with the provided gas escalator and the chosen
    /// escalation frequency (per block or per second)
    #[allow(clippy::let_and_return)]
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new<E>(inner: M, escalator: E, frequency: Frequency) -> Self
    where
        E: GasEscalator + 'static,
        M: 'static,
    {
        let (tx, rx) = oneshot::channel();
        let inner = Arc::new(inner);

        let txs: ToEscalate = Default::default();

        let this = Arc::new(GasEscalatorMiddlewareInternal {
            inner: inner.clone(),
            txs: txs.clone(),
            _background: tx,
        });

        let esc = EscalationTask { inner, escalator, frequency, txs, shutdown: rx };

        {
            spawn(esc.escalate().instrument(tracing::trace_span!("gas-escalation")));
        }

        Self { inner: this }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
pub struct EscalationTask<M, E> {
    inner: M,
    escalator: E,
    frequency: Frequency,
    txs: ToEscalate,
    shutdown: oneshot::Receiver<()>,
}

#[cfg(not(target_arch = "wasm32"))]
impl<M, E> EscalationTask<M, E> {
    pub fn new(
        inner: M,
        escalator: E,
        frequency: Frequency,
        txs: ToEscalate,
        shutdown: oneshot::Receiver<()>,
    ) -> Self {
        Self { inner, escalator, frequency, txs, shutdown }
    }

    async fn escalate(mut self) -> Result<(), GasEscalatorError<M>>
    where
        M: Middleware,
        E: GasEscalator,
    {
        // the escalation frequency is either on a per-block basis, or on a duration basis
        let watcher: WatcherFuture = match self.frequency {
            Frequency::PerBlock => Box::pin(
                self.inner.watch_blocks().await.map_err(MiddlewareError::from_err)?.map(|_| ()),
            ),
            Frequency::Duration(ms) => Box::pin(interval(std::time::Duration::from_millis(ms))),
        };

        let mut watcher = watcher.fuse();

        loop {
            select_biased! {
            _ = &mut self.shutdown => {
                tracing::debug!("Shutting down escalation task, middleware has gone away");
                return Ok(())
            }
            opt = watcher.next() => {
                if opt.is_none() {
                    tracing::error!("timing future has gone away");
                    return Ok(());
                }
                let now = Instant::now();

                // We take the contents of the mutex, and then add them back in
                // later.
                let mut txs: Vec<_> = {
                    let mut txs = self.txs.lock().await;
                    std::mem::take(&mut (*txs))
                    // Lock scope ends
                };

                let len = txs.len();
                // Pop all transactions and re-insert those that have not been included yet
                for _ in 0..len {
                    // this must never panic as we're explicitly within bounds
                    let (tx_hash, mut replacement_tx, time, priority) =
                        txs.pop().expect("should have element in vector");

                    let receipt = self
                        .inner
                        .get_transaction_receipt(tx_hash)
                        .await
                        .map_err(MiddlewareError::from_err)?;

                    tracing::trace!(tx_hash = ?tx_hash, "checking if exists");

                    if receipt.is_none() {
                        let old_gas_price = replacement_tx.gas_price.expect("gas price must be set");
                        // Get the new gas price based on how much time passed since the
                        // tx was last broadcast
                        let new_gas_price = self
                            .escalator
                            .get_gas_price(old_gas_price, now.duration_since(time).as_secs());

                        let new_txhash = if new_gas_price == old_gas_price {
                             tx_hash
                        } else {
                            // bump the gas price
                            replacement_tx.gas_price = Some(new_gas_price);

                            // the tx hash will be different so we need to update it
                            match self.inner.send_transaction(replacement_tx.clone(), priority).await {
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
                                        tracing::error!(
                                            err = %err,
                                            "Killing escalator backend"
                                        );
                                        return Err(GasEscalatorError::MiddlewareError(err))
                                    }
                                }
                            }
                        };
                        txs.push((new_txhash, replacement_tx, time, priority));
                    }
                }
                // after this big ugly loop, we dump everything back in
                // we don't replace here, as the vec in the mutex may contain
                // items!
                self.txs.lock().await.extend(txs);
            }}
        }
    }
}
