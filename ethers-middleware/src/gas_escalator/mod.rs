mod geometric;
pub use geometric::GeometricGasPrice;

use async_trait::async_trait;
use ethers_core::types::{BlockNumber, TransactionRequest, TxHash, U256};
use ethers_providers::{interval, FromErr, Middleware, StreamExt};
use futures_util::lock::Mutex;
use std::sync::Arc;
use std::{pin::Pin, time::Instant};
use thiserror::Error;

pub trait GasEscalator: Send + Sync + std::fmt::Debug {
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

#[derive(Debug, Clone)]
/// A Gas escalator allows bumping transactions' gas price to avoid getting them
/// stuck in the memory pool.
///
/// Users must wrap this struct in an `Arc` and then spawn the `escalate` call
/// before wrapping it in other middleware.
///
/// ```no_run
/// use ethers::{
///     providers::{Provider, Http},
///     middleware::{
///         GasEscalatorMiddleware,
///         GasOracleMiddleware,
///         gas_escalator::{GeometricGasPrice, Frequency},
///         gas_oracle::{GasNow, GasCategory},
///     },
/// };
/// use std::{convert::TryFrom, time::Duration, sync::Arc};
///
/// let provider = Provider::try_from("http://localhost:8545")
///     .unwrap()
///     .interval(Duration::from_millis(2000u64));
/// let provider = Arc::new({
///     let mut escalator = GeometricGasPrice::new();
///     escalator.every_secs = 10;
///     escalator.coefficient = 5.0;
///     GasEscalatorMiddleware::new(provider, escalator, Frequency::PerBlock)
/// });
///
/// // clone the arc
/// let provider_clone = provider.clone();
/// tokio::spawn(async move {
///     provider_clone.escalate().await;
/// });
///
/// // ... proceed to wrap it in other middleware
/// let gas_oracle = GasNow::new().category(GasCategory::SafeLow);
/// let provider = GasOracleMiddleware::new(provider, gas_oracle);
/// ```
pub struct GasEscalatorMiddleware<M, E> {
    pub(crate) inner: Arc<M>,
    pub(crate) escalator: E,
    /// The transactions which are currently being monitored for escalation
    pub txs: Arc<Mutex<Vec<(TxHash, TransactionRequest, Instant, Option<BlockNumber>)>>>,
    frequency: Frequency,
}

#[async_trait]
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

    async fn send_transaction(
        &self,
        tx: TransactionRequest,
        block: Option<BlockNumber>,
    ) -> Result<TxHash, Self::Error> {
        let tx_hash = self
            .inner()
            .send_transaction(tx.clone(), block)
            .await
            .map_err(GasEscalatorError::MiddlewareError)?;

        // insert the tx in the pending txs
        let mut lock = self.txs.lock().await;
        lock.push((tx_hash, tx, Instant::now(), block));

        Ok(tx_hash)
    }
}

impl<M, E> GasEscalatorMiddleware<M, E>
where
    M: Middleware,
    E: GasEscalator,
{
    /// Initializes the middleware with the provided gas escalator and the chosen
    /// escalation frequency (per block or per second)
    pub fn new(inner: M, escalator: E, frequency: Frequency) -> Self {
        Self {
            inner: Arc::new(inner),
            escalator,
            frequency,
            txs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn spawn<S: futures_util::task::SpawnExt>(&self, executor: S)
    where
        E: Clone + 'static,
        M: Clone + 'static,
    {
        let this = self.clone();
        executor
            .spawn(async move {
                this.escalate().await.unwrap();
            })
            .expect("could not spawn async executor");
    }

    pub async fn escalate(&self) -> Result<(), GasEscalatorError<M>> {
        // the escalation frequency is either on a per-block basis, or on a duratoin basis
        let mut watcher: Pin<Box<dyn futures_util::stream::Stream<Item = ()> + Send>> =
            match self.frequency {
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
                if receipt.is_none() {
                    // Get the new gas price based on how much time passed since the
                    // tx was last broadcast
                    let new_gas_price = self.escalator.get_gas_price(
                        replacement_tx.gas_price.expect("gas price must be set"),
                        now.duration_since(time).as_secs(),
                    );

                    let new_txhash = if Some(new_gas_price) != replacement_tx.gas_price {
                        // bump the gas price
                        replacement_tx.gas_price = Some(new_gas_price);

                        // the tx hash will be different so we need to update it
                        match self
                            .inner()
                            .send_transaction(replacement_tx.clone(), priority)
                            .await
                        {
                            Ok(tx_hash) => tx_hash,
                            Err(err) => {
                                if err.to_string().contains("nonce too low") {
                                    // ignore "nonce too low" errors because they
                                    // may happen if we try to broadcast a higher
                                    // gas price tx when one of the previous ones
                                    // was already mined (meaning we also do not
                                    // push it back to the pending txs vector)
                                    continue;
                                } else {
                                    return Err(GasEscalatorError::MiddlewareError(err));
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
}
