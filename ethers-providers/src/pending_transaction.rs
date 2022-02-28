use crate::{
    stream::{interval, DEFAULT_POLL_INTERVAL},
    JsonRpcClient, Middleware, PinBoxFut, Provider, ProviderError,
};
use ethers_core::types::{Transaction, TransactionReceipt, TxHash, U64};
use futures_core::stream::Stream;
use futures_util::stream::StreamExt;
use pin_project::pin_project;
use std::{
    fmt,
    future::Future,
    ops::Deref,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

#[cfg(not(target_arch = "wasm32"))]
use futures_timer::Delay;
#[cfg(target_arch = "wasm32")]
use wasm_timer::Delay;

/// A pending transaction is a transaction which has been submitted but is not yet mined.
/// `await`'ing on a pending transaction will resolve to a transaction receipt
/// once the transaction has enough `confirmations`. The default number of confirmations
/// is 1, but may be adjusted with the `confirmations` method. If the transaction does not
/// have enough confirmations or is not mined, the future will stay in the pending state.
///
/// # Example
///
///```
/// # use ethers_providers::{Provider, Http};
/// # use ethers_core::utils::Ganache;
/// # use std::convert::TryFrom;
/// use ethers_providers::Middleware;
/// use ethers_core::types::TransactionRequest;
///
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let ganache = Ganache::new().spawn();
/// # let client = Provider::<Http>::try_from(ganache.endpoint()).unwrap();
/// # let accounts = client.get_accounts().await?;
/// # let from = accounts[0];
/// # let to = accounts[1];
/// # let balance_before = client.get_balance(to, None).await?;
/// let tx = TransactionRequest::new().to(to).value(1000).from(from);
/// let receipt = client
///     .send_transaction(tx, None)
///     .await?                           // PendingTransaction<_>
///     .log_msg("Pending transfer hash") // print pending tx hash with message
///     .await?;                          // Result<Option<TransactionReceipt>, _>
/// # let _ = receipt;
/// # let balance_after = client.get_balance(to, None).await?;
/// # assert_eq!(balance_after, balance_before + 1000);
/// # Ok(())
/// # }
/// ```
#[pin_project]
pub struct PendingTransaction<'a, P> {
    tx_hash: TxHash,
    confirmations: usize,
    provider: &'a Provider<P>,
    state: PendingTxState<'a>,
    interval: Box<dyn Stream<Item = ()> + Send + Unpin>,
}

impl<'a, P: JsonRpcClient> PendingTransaction<'a, P> {
    /// Creates a new pending transaction poller from a hash and a provider
    pub fn new(tx_hash: TxHash, provider: &'a Provider<P>) -> Self {
        let delay = Box::pin(Delay::new(DEFAULT_POLL_INTERVAL));
        Self {
            tx_hash,
            confirmations: 1,
            provider,
            state: PendingTxState::InitialDelay(delay),
            interval: Box::new(interval(DEFAULT_POLL_INTERVAL)),
        }
    }

    /// Returns the Provider associated with the pending transaction
    pub fn provider(&self) -> Provider<P>
    where
        P: Clone,
    {
        self.provider.clone()
    }

    /// Returns the transaction hash of the pending transaction
    pub fn tx_hash(&self) -> TxHash {
        self.tx_hash
    }

    /// Sets the number of confirmations for the pending transaction to resolve
    /// to a receipt
    #[must_use]
    pub fn confirmations(mut self, confs: usize) -> Self {
        self.confirmations = confs;
        self
    }

    /// Sets the polling interval
    #[must_use]
    pub fn interval<T: Into<Duration>>(mut self, duration: T) -> Self {
        let duration = duration.into();

        self.interval = Box::new(interval(duration));

        if matches!(self.state, PendingTxState::InitialDelay(_)) {
            self.state = PendingTxState::InitialDelay(Box::pin(Delay::new(duration)))
        }

        self
    }
}

impl<'a, P> PendingTransaction<'a, P> {
    /// Allows inspecting the content of a pending transaction in a builder-like way to avoid
    /// more verbose calls, e.g.:
    /// `let mined = token.transfer(recipient, amt).send().await?.inspect(|tx| println!(".{}",
    /// *tx)).await?;`
    pub fn inspect<F>(self, mut f: F) -> Self
    where
        F: FnMut(&Self),
    {
        f(&self);
        self
    }

    /// Logs the pending transaction hash along with a custom message before it.
    pub fn log_msg<S: std::fmt::Display>(self, msg: S) -> Self {
        self.inspect(|s| println!("{}: {:?}", msg, **s))
    }

    /// Logs the pending transaction's hash
    pub fn log(self) -> Self {
        self.inspect(|s| println!("Pending hash: {:?}", **s))
    }
}

macro_rules! rewake_with_new_state {
    ($ctx:ident, $this:ident, $new_state:expr) => {
        *$this.state = $new_state;
        $ctx.waker().wake_by_ref();
        return Poll::Pending
    };
}

macro_rules! rewake_with_new_state_if {
    ($condition:expr, $ctx:ident, $this:ident, $new_state:expr) => {
        if $condition {
            rewake_with_new_state!($ctx, $this, $new_state);
        }
    };
}

impl<'a, P: JsonRpcClient> Future for PendingTransaction<'a, P> {
    type Output = Result<Option<TransactionReceipt>, ProviderError>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output> {
        let this = self.project();

        match this.state {
            PendingTxState::InitialDelay(fut) => {
                let _ready = futures_util::ready!(fut.as_mut().poll(ctx));
                tracing::debug!("Starting to poll pending tx {:?}", *this.tx_hash);
                let fut = Box::pin(this.provider.get_transaction(*this.tx_hash));
                rewake_with_new_state!(ctx, this, PendingTxState::GettingTx(fut));
            }
            PendingTxState::PausedGettingTx => {
                // Wait the polling period so that we do not spam the chain when no
                // new block has been mined
                let _ready = futures_util::ready!(this.interval.poll_next_unpin(ctx));
                let fut = Box::pin(this.provider.get_transaction(*this.tx_hash));
                *this.state = PendingTxState::GettingTx(fut);
                ctx.waker().wake_by_ref();
            }
            PendingTxState::GettingTx(fut) => {
                let tx_res = futures_util::ready!(fut.as_mut().poll(ctx));
                // If the provider errors, just try again after the interval.
                // nbd.
                rewake_with_new_state_if!(
                    tx_res.is_err(),
                    ctx,
                    this,
                    PendingTxState::PausedGettingTx
                );

                let tx_opt = tx_res.unwrap();
                // If the tx is no longer in the mempool, return Ok(None)
                if tx_opt.is_none() {
                    tracing::debug!("Dropped from mempool, pending tx {:?}", *this.tx_hash);
                    *this.state = PendingTxState::Completed;
                    return Poll::Ready(Ok(None))
                }

                // If it hasn't confirmed yet, poll again later
                let tx = tx_opt.unwrap();
                rewake_with_new_state_if!(
                    tx.block_number.is_none(),
                    ctx,
                    this,
                    PendingTxState::PausedGettingTx
                );

                // Start polling for the receipt now
                tracing::debug!("Getting receipt for pending tx {:?}", *this.tx_hash);
                let fut = Box::pin(this.provider.get_transaction_receipt(*this.tx_hash));
                rewake_with_new_state!(ctx, this, PendingTxState::GettingReceipt(fut));
            }
            PendingTxState::PausedGettingReceipt => {
                // Wait the polling period so that we do not spam the chain when no
                // new block has been mined
                let _ready = futures_util::ready!(this.interval.poll_next_unpin(ctx));
                let fut = Box::pin(this.provider.get_transaction_receipt(*this.tx_hash));
                *this.state = PendingTxState::GettingReceipt(fut);
                ctx.waker().wake_by_ref();
            }
            PendingTxState::GettingReceipt(fut) => {
                if let Ok(receipt) = futures_util::ready!(fut.as_mut().poll(ctx)) {
                    tracing::debug!("Checking receipt for pending tx {:?}", *this.tx_hash);
                    *this.state = PendingTxState::CheckingReceipt(receipt)
                } else {
                    *this.state = PendingTxState::PausedGettingReceipt
                }
                ctx.waker().wake_by_ref();
            }
            PendingTxState::CheckingReceipt(receipt) => {
                rewake_with_new_state_if!(
                    receipt.is_none(),
                    ctx,
                    this,
                    PendingTxState::PausedGettingReceipt
                );

                // If we requested more than 1 confirmation, we need to compare the receipt's
                // block number and the current block
                if *this.confirmations > 1 {
                    tracing::debug!("Waiting on confirmations for pending tx {:?}", *this.tx_hash);

                    let fut = Box::pin(this.provider.get_block_number());
                    *this.state = PendingTxState::GettingBlockNumber(fut, receipt.take());

                    // Schedule the waker to poll again
                    ctx.waker().wake_by_ref();
                } else {
                    let receipt = receipt.take();
                    *this.state = PendingTxState::Completed;
                    return Poll::Ready(Ok(receipt))
                }
            }
            PendingTxState::PausedGettingBlockNumber(receipt) => {
                // Wait the polling period so that we do not spam the chain when no
                // new block has been mined
                let _ready = futures_util::ready!(this.interval.poll_next_unpin(ctx));

                // we need to re-instantiate the get_block_number future so that
                // we poll again
                let fut = Box::pin(this.provider.get_block_number());
                *this.state = PendingTxState::GettingBlockNumber(fut, receipt.take());
                ctx.waker().wake_by_ref();
            }
            PendingTxState::GettingBlockNumber(fut, receipt) => {
                let current_block = futures_util::ready!(fut.as_mut().poll(ctx))?;

                // This is safe so long as we only enter the `GettingBlock`
                // loop from `CheckingReceipt`, which contains an explicit
                // `is_none` check
                let receipt = receipt.take().expect("GettingBlockNumber without receipt");

                // Wait for the interval
                let inclusion_block = receipt
                    .block_number
                    .expect("Receipt did not have a block number. This should never happen");
                // if the transaction has at least K confirmations, return the receipt
                // (subtract 1 since the tx already has 1 conf when it's mined)
                if current_block > inclusion_block + *this.confirmations - 1 {
                    let receipt = Some(receipt);
                    *this.state = PendingTxState::Completed;
                    return Poll::Ready(Ok(receipt))
                } else {
                    tracing::trace!(tx_hash = ?this.tx_hash, "confirmations {}/{}", current_block - inclusion_block + 1, this.confirmations);
                    *this.state = PendingTxState::PausedGettingBlockNumber(Some(receipt));
                    ctx.waker().wake_by_ref();
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

// We box the TransactionReceipts to keep the enum small.
enum PendingTxState<'a> {
    /// Initial delay to ensure the GettingTx loop doesn't immediately fail
    InitialDelay(Pin<Box<Delay>>),

    /// Waiting for interval to elapse before calling API again
    PausedGettingTx,

    /// Polling The blockchain to see if the Tx has confirmed or dropped
    GettingTx(PinBoxFut<'a, Option<Transaction>>),

    /// Waiting for interval to elapse before calling API again
    PausedGettingReceipt,

    /// Polling the blockchain for the receipt
    GettingReceipt(PinBoxFut<'a, Option<TransactionReceipt>>),

    /// If the pending tx required only 1 conf, it will return early. Otherwise it will
    /// proceed to the next state which will poll the block number until there have been
    /// enough confirmations
    CheckingReceipt(Option<TransactionReceipt>),

    /// Waiting for interval to elapse before calling API again
    PausedGettingBlockNumber(Option<TransactionReceipt>),

    /// Polling the blockchain for the current block number
    GettingBlockNumber(PinBoxFut<'a, U64>, Option<TransactionReceipt>),

    /// Future has completed and should panic if polled again
    Completed,
}

impl<'a> fmt::Debug for PendingTxState<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = match self {
            PendingTxState::InitialDelay(_) => "InitialDelay",
            PendingTxState::PausedGettingTx => "PausedGettingTx",
            PendingTxState::GettingTx(_) => "GettingTx",
            PendingTxState::PausedGettingReceipt => "PausedGettingReceipt",
            PendingTxState::GettingReceipt(_) => "GettingReceipt",
            PendingTxState::GettingBlockNumber(_, _) => "GettingBlockNumber",
            PendingTxState::PausedGettingBlockNumber(_) => "PausedGettingBlockNumber",
            PendingTxState::CheckingReceipt(_) => "CheckingReceipt",
            PendingTxState::Completed => "Completed",
        };

        f.debug_struct("PendingTxState").field("state", &state).finish()
    }
}
