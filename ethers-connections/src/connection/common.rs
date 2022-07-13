use std::hash::BuildHasherDefault;

use hashers::fx_hash::FxHasher64;
use serde_json::value::RawValue;
use tokio::sync::{mpsc, oneshot};

use ethers_core::types::U256;

use crate::{BatchResponsePayload, NotificationReceiver, ResponsePayload, ResponseReceiver};

pub(super) type FxHashMap<K, V> = std::collections::HashMap<K, V, BuildHasherDefault<FxHasher64>>;

/// A subscription consists of a sender instance and a receiver to be picked up
/// by the subscribing callsite.
pub(super) type Subscription = (mpsc::UnboundedSender<Box<RawValue>>, Option<NotificationReceiver>);

/// The shared state for a request server task.
pub(super) struct Shared {
    /// The map of pending requests.
    pub(super) pending: FxHashMap<u64, ResponseReceiver>,
    /// The set of pending batch requests.
    pub(super) pending_batches: FxHashMap<Box<[u64]>, PendingBatchCall>,
    /// The map of registered subscriptions.
    pub(super) subs: FxHashMap<U256, Subscription>,
}

impl Default for Shared {
    fn default() -> Self {
        Self {
            pending: FxHashMap::with_capacity_and_hasher(64, Default::default()),
            pending_batches: FxHashMap::with_capacity_and_hasher(64, Default::default()),
            subs: FxHashMap::with_capacity_and_hasher(64, Default::default()),
        }
    }
}

/// A request to a server task.
pub(super) enum Request {
    Call { id: u64, tx: oneshot::Sender<ResponsePayload>, request: Box<RawValue> },
    BatchCall { ids: Box<[u64]>, tx: oneshot::Sender<BatchResponsePayload>, request: Box<RawValue> },
    Subscribe { id: U256, tx: oneshot::Sender<Option<NotificationReceiver>> },
    Unsubscribe { id: U256 },
}

pub(super) struct PendingBatchCall {
    pub ids: Box<[u64]>,
    pub tx: oneshot::Sender<BatchResponsePayload>,
}
