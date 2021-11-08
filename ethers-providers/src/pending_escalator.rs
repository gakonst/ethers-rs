use ethers_core::types::{Bytes, TransactionReceipt, H256};
use pin_project::pin_project;
use std::{
    future::{self, Future},
    pin::Pin,
    task::Poll,
    time::{Duration, Instant},
};
use tokio::time::Sleep;

use crate::{JsonRpcClient, Middleware, PendingTransaction, Provider, ProviderError};

type PinBoxFut<'a, T> = Pin<Box<dyn future::Future<Output = T> + 'a + Send>>;

/// States for the EscalatingPending future
enum PendingStates<'a, P> {
    Initial(PinBoxFut<'a, Result<PendingTransaction<'a, P>, ProviderError>>),
    Sleeping(Pin<Box<Sleep>>),
    BroadcastingNew(PinBoxFut<'a, Result<PendingTransaction<'a, P>, ProviderError>>),
    CheckingReceipts(Vec<PinBoxFut<'a, Result<Option<TransactionReceipt>, ProviderError>>>),
    Completed,
}

/// An EscalatingPending is a pending transaction that handles increasing its
/// own gas price over time, by broadcasting successive versions with higher
/// gas prices
#[pin_project(project = PendingProj)]
pub struct EscalatingPending<'a, P>
where
    P: JsonRpcClient,
{
    provider: &'a Provider<P>,
    broadcast_interval: Duration,
    polling_interval: Duration,
    txns: Vec<Bytes>,
    last: Option<Instant>,
    sent: Vec<H256>,
    state: PendingStates<'a, P>,
}

impl<'a, P> EscalatingPending<'a, P>
where
    P: JsonRpcClient,
{
    /// Instantiate a new EscalatingPending. This should only be called by the
    /// Middleware trait. Callers MUST ensure that transactions are in _reverse_
    /// broadcast order (this just makes writing the code easier, as we
    /// can use `pop()` a lot)
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
            last: None,
            sent: vec![],
            state: PendingStates::Initial(provider.send_raw_transaction(first)),
        }
    }

    pub fn broadcast_interval(mut self, duration: u64) -> Self {
        self.broadcast_interval = Duration::from_secs(duration);
        self
    }

    pub fn polling_interval(mut self, duration: u64) -> Self {
        self.polling_interval = Duration::from_secs(duration);
        self
    }
}

macro_rules! check_all_receipts {
    ($cx:ident, $this:ident) => {
        let futs: Vec<_> = $this
            .sent
            .iter()
            .map(|tx_hash| $this.provider.get_transaction_receipt(*tx_hash))
            .collect();
        *$this.state = CheckingReceipts(futs);
        $cx.waker().wake_by_ref();
        return Poll::Pending;
    };
}

macro_rules! sleep {
    ($cx:ident, $this:ident) => {
        *$this.state =
            PendingStates::Sleeping(Box::pin(tokio::time::sleep(*$this.polling_interval)));
        $cx.waker().wake_by_ref();
        return Poll::Pending;
    };
}

macro_rules! completed {
    ($this:ident, $output:expr) => {
        *$this.state = Completed;
        return Poll::Ready($output);
    };
}

macro_rules! broadcast_checks {
    ($cx:ident, $this:ident, $fut:ident) => {
        match $fut.as_mut().poll($cx) {
            Poll::Ready(Ok(pending)) => {
                $this.sent.push(*pending);
                check_all_receipts!($cx, $this);
            }
            Poll::Ready(Err(e)) => {
                completed!($this, Err(e));
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
        use PendingStates::*;

        let this = self.project();

        match this.state {
            Initial(fut) => {
                broadcast_checks!(cx, this, fut);
            }
            Sleeping(fut) => {
                if fut.as_mut().poll(cx).is_ready() {
                    // if timer has elapsed (or this is the first tx)
                    if this.last.is_none()
                        || this.last.clone().unwrap().elapsed() > *this.broadcast_interval
                    {
                        // then if we have a TX to broadcast, start
                        // broadcasting it
                        if let Some(next_to_broadcast) = this.txns.pop() {
                            let fut = this.provider.send_raw_transaction(next_to_broadcast);
                            *this.state = BroadcastingNew(fut);
                            cx.waker().wake_by_ref();
                            return Poll::Pending;
                        }
                    }

                    check_all_receipts!(cx, this);
                }

                return Poll::Pending;
            }
            BroadcastingNew(fut) => {
                broadcast_checks!(cx, this, fut);
            }
            CheckingReceipts(futs) => {
                // if drained, sleep
                if futs.is_empty() {
                    sleep!(cx, this);
                }

                // otherwise drain one and check if we have a receipt
                let mut pollee = futs.pop().expect("checked");
                match pollee.as_mut().poll(cx) {
                    //
                    Poll::Ready(Ok(Some(receipt))) => {
                        completed!(this, Ok(receipt));
                    }
                    // rewake until drained
                    Poll::Ready(Ok(None)) => cx.waker().wake_by_ref(),
                    // bubble up errors
                    Poll::Ready(Err(e)) => {
                        completed!(this, Err(e));
                    }
                    Poll::Pending => {
                        // stick it pack in the list for polling again later
                        futs.push(pollee);
                        return Poll::Pending;
                    }
                }
            }
            Completed => panic!("polled after completion"),
        }

        Poll::Pending
    }
}
