use ethers_core::types::{Bytes, TransactionReceipt, H256};
use futures_util::{stream::FuturesUnordered, StreamExt};
use pin_project::pin_project;
use std::{
    future::Future,
    pin::Pin,
    task::Poll,
    time::{Duration, Instant},
};

#[cfg(not(target_arch = "wasm32"))]
use futures_timer::Delay;
#[cfg(target_arch = "wasm32")]
use wasm_timer::Delay;

use crate::{JsonRpcClient, Middleware, PendingTransaction, PinBoxFut, Provider, ProviderError};

/// States for the EscalatingPending future
enum EscalatorStates<'a, P> {
    Initial(PinBoxFut<'a, PendingTransaction<'a, P>>),
    Sleeping(Pin<Box<Delay>>),
    BroadcastingNew(PinBoxFut<'a, PendingTransaction<'a, P>>),
    CheckingReceipts(FuturesUnordered<PinBoxFut<'a, Option<TransactionReceipt>>>),
    Completed,
}

/// An EscalatingPending is a pending transaction that increases its own gas
/// price over time, by broadcasting successive versions with higher gas prices.
#[must_use]
#[pin_project(project = PendingProj)]
#[derive(Debug)]
pub struct EscalatingPending<'a, P>
where
    P: JsonRpcClient,
{
    provider: &'a Provider<P>,
    broadcast_interval: Duration,
    polling_interval: Duration,
    txns: Vec<Bytes>,
    last: Instant,
    sent: Vec<H256>,
    state: EscalatorStates<'a, P>,
}

impl<'a, P> EscalatingPending<'a, P>
where
    P: JsonRpcClient,
{
    /// Instantiate a new EscalatingPending. This should only be called by the
    /// Middleware trait.
    ///
    /// Callers MUST ensure that transactions are in _reverse_ broadcast order
    /// (this just makes writing the code easier, as we can use `pop()` a lot).
    ///
    /// TODO: consider deserializing and checking invariants (gas order, etc.)
    pub(crate) fn new(provider: &'a Provider<P>, mut txns: Vec<Bytes>) -> Self {
        if txns.is_empty() {
            panic!("bad args");
        }

        let first = txns.pop().expect("bad args");
        // Sane-feeling default intervals
        Self {
            provider,
            broadcast_interval: Duration::from_millis(150),
            polling_interval: Duration::from_millis(10),
            txns,
            // placeholder value. We set this again after the initial broadcast
            // future resolves
            last: Instant::now(),
            sent: vec![],
            state: EscalatorStates::Initial(Box::pin(provider.send_raw_transaction(first))),
        }
    }

    /// Set the broadcast interval. This controls how often the escalator
    /// broadcasts a new transaction at a higher gas price
    pub fn with_broadcast_interval(mut self, duration: impl Into<Duration>) -> Self {
        self.broadcast_interval = duration.into();
        self
    }

    /// Set the polling interval. This controls how often the escalator checks
    /// transaction receipts for confirmation.
    pub fn with_polling_interval(mut self, duration: impl Into<Duration>) -> Self {
        self.polling_interval = duration.into();
        self
    }

    /// Get the current polling interval.
    pub fn get_polling_interval(&self) -> Duration {
        self.polling_interval
    }

    /// Get the current broadcast interval.
    pub fn get_broadcast_interval(&self) -> Duration {
        self.broadcast_interval
    }
}

macro_rules! check_all_receipts {
    ($cx:ident, $this:ident) => {
        let futs: futures_util::stream::FuturesUnordered<_> = $this
            .sent
            .iter()
            .map(|tx_hash| $this.provider.get_transaction_receipt(*tx_hash))
            .collect();
        *$this.state = CheckingReceipts(futs);
        $cx.waker().wake_by_ref();
        return Poll::Pending
    };
}

macro_rules! sleep {
    ($cx:ident, $this:ident) => {
        *$this.state = EscalatorStates::Sleeping(Box::pin(Delay::new(*$this.polling_interval)));
        $cx.waker().wake_by_ref();
        return Poll::Pending
    };
}

macro_rules! completed {
    ($this:ident, $output:expr) => {
        *$this.state = Completed;
        return Poll::Ready($output)
    };
}

macro_rules! poll_broadcast_fut {
    ($cx:ident, $this:ident, $fut:ident) => {
        match $fut.as_mut().poll($cx) {
            Poll::Ready(Ok(pending)) => {
                *$this.last = Instant::now();
                $this.sent.push(*pending);
                tracing::info!(
                    tx_hash = ?*pending,
                    escalation = $this.sent.len(),
                    "Escalation transaction broadcast complete"
                );
                check_all_receipts!($cx, $this);
            }
            Poll::Ready(Err(e)) => {
                // kludge. Prevents erroring on "nonce too low" which indicates
                // a previous escalation confirmed during this broadcast attempt
                if format!("{:?}", e).contains("nonce too low") {
                    check_all_receipts!($cx, $this);
                } else {
                    tracing::error!(
                        error = ?e,
                        "Error during transaction broadcast"
                    );

                    completed!($this, Err(e));
                }
            }
            Poll::Pending => return Poll::Pending,
        }
    };
}

impl<'a, P> Future for EscalatingPending<'a, P>
where
    P: JsonRpcClient,
{
    type Output = Result<TransactionReceipt, ProviderError>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        use EscalatorStates::*;

        let this = self.project();

        match this.state {
            // In the initial state we're simply waiting on the first
            // transaction braodcast to complete.
            Initial(fut) => {
                poll_broadcast_fut!(cx, this, fut);
            }
            Sleeping(delay) => {
                let _ready = futures_util::ready!(delay.as_mut().poll(cx));
                // if broadcast timer has elapsed and if we have a TX to
                // broadcast, broadcast it
                if this.last.elapsed() > *this.broadcast_interval {
                    if let Some(next_to_broadcast) = this.txns.pop() {
                        let fut = this.provider.send_raw_transaction(next_to_broadcast);
                        *this.state = BroadcastingNew(fut);
                        cx.waker().wake_by_ref();
                        return Poll::Pending
                    }
                }
                check_all_receipts!(cx, this);
            }
            // This state is functionally equivalent to Initial, but we
            // differentiate it for clarity
            BroadcastingNew(fut) => {
                poll_broadcast_fut!(cx, this, fut);
            }
            CheckingReceipts(futs) => {
                // Poll the set of `get_transaction_receipt` futures to check
                // if any previously-broadcast transaction was confirmed.
                // Continue doing this until all are resolved
                match futs.poll_next_unpin(cx) {
                    // We have found a receipt. This means that all other
                    // broadcast txns are now invalid, so we can drop the
                    // futures and complete
                    Poll::Ready(Some(Ok(Some(receipt)))) => {
                        completed!(this, Ok(receipt));
                    }
                    // A `get_transaction_receipt` request resolved, but but we
                    // found no receipt, rewake and check if any other requests
                    // are resolved
                    Poll::Ready(Some(Ok(None))) => cx.waker().wake_by_ref(),
                    // A request errored. We complete the future with the error.
                    Poll::Ready(Some(Err(e))) => {
                        completed!(this, Err(e));
                    }
                    // We have run out of `get_transaction_receipt` requests.
                    // Sleep and then check if we should broadcast again (or
                    // check receipts again)
                    Poll::Ready(None) => {
                        sleep!(cx, this);
                    }
                    // No request has resolved yet. Try again later
                    Poll::Pending => return Poll::Pending,
                }
            }
            Completed => panic!("polled after completion"),
        }

        Poll::Pending
    }
}

impl<'a, P> std::fmt::Debug for EscalatorStates<'a, P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = match self {
            Self::Initial(_) => "Initial",
            Self::Sleeping(_) => "Sleeping",
            Self::BroadcastingNew(_) => "BroadcastingNew",
            Self::CheckingReceipts(_) => "CheckingReceipts",
            Self::Completed => "Completed",
        };
        f.debug_struct("EscalatorStates").field("state", &state).finish()
    }
}
