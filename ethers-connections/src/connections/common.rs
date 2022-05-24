use std::{cell::RefCell, hash::BuildHasherDefault};

use ethers_core::types::U256;
use hashers::fx_hash::FxHasher64;
use serde_json::value::RawValue;
use tokio::sync::{mpsc, oneshot};

use crate::{NotificationReceiver, ResponsePayload};

pub(super) type FxHashMap<K, V> = std::collections::HashMap<K, V, BuildHasherDefault<FxHasher64>>;
pub(super) type PendingRequest = oneshot::Sender<ResponsePayload>;
pub(super) type Subscription = (mpsc::UnboundedSender<Box<RawValue>>, Option<NotificationReceiver>);

pub(super) enum Request {
    Call { id: u64, tx: PendingRequest, request: String },
    Subscribe { id: U256, tx: oneshot::Sender<Option<NotificationReceiver>> },
    Unsubscribe { id: U256 },
}

pub(super) struct Shared {
    pub(super) pending: RefCell<FxHashMap<u64, PendingRequest>>,
    pub(super) subs: RefCell<FxHashMap<U256, Subscription>>,
}

impl Shared {
    pub(super) fn new() -> Self {
        todo!()
    }
}
