use std::{
    collections::{BTreeMap, HashMap},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};

use ethers_core::types::U256;
use futures_channel::{mpsc, oneshot};
use futures_util::{select, StreamExt};
use serde_json::value::RawValue;

use crate::JsonRpcError;

use super::{
    backend::{BackendDriver, WsBackend},
    ActiveSub, ConnectionDetails, InFlight, Instruction, Notification, PubSubItem, Response, SubId,
    WsClient, WsClientError,
};

pub type SharedChannelMap = Arc<Mutex<HashMap<U256, mpsc::UnboundedReceiver<Box<RawValue>>>>>;

pub struct SubscriptionManager {
    subs: BTreeMap<u64, ActiveSub>,
    aliases: HashMap<U256, u64>,
    // used to communicate to the WsClient
    channel_map: SharedChannelMap,
}

impl SubscriptionManager {
    fn new(channel_map: SharedChannelMap) -> Self {
        Self { subs: Default::default(), aliases: Default::default(), channel_map }
    }

    fn add_alias(&mut self, sub: U256, id: u64) {
        if let Some(entry) = self.subs.get_mut(&id) {
            entry.current_server_id = Some(sub);
        }
        self.aliases.insert(sub, id);
    }

    fn remove_alias(&mut self, server_id: U256) {
        if let Some(id) = self.aliases.get(&server_id) {
            if let Some(sub) = self.subs.get_mut(id) {
                sub.current_server_id = None;
            }
        }
        self.aliases.remove(&server_id);
    }

    fn end_subscription(&mut self, id: u64) -> Option<Box<RawValue>> {
        if let Some(sub) = self.subs.remove(&id) {
            if let Some(server_id) = sub.current_server_id {
                self.remove_alias(server_id);
                // drop the receiver as we don't need the result
                let (channel, _) = oneshot::channel();
                // Serialization errors are ignored, and result in the request
                // not being dispatched. This is fine, as worst case it will
                // result in the server sending us notifications we ignore
                let unsub_request = InFlight {
                    method: "eth_unsubscribe".to_string(),
                    params: SubId(server_id).serialize_raw().ok()?,
                    channel,
                };
                // reuse the RPC ID. this is somewhat dirty.
                return unsub_request.serialize_raw(id).ok()
            }
        }
        None
    }

    fn handle_notification(&mut self, notification: Notification) {
        let sub_id = notification.subscription;

        if let Some(id) = self.aliases.get(&sub_id) {
            if let Some(active) = self.subs.get(id) {
                // send the notification over the channel
                let send_res = active.channel.unbounded_send(notification.result);
                // receiver has dropped, so we drop the sub
                // to consider: this leaves aliases to dead subs in the map if
                // reconnection has occurred. however, this seems like a small
                // use of memory
                if send_res.is_err() {
                    self.aliases.remove(&sub_id);
                }
            }
        }
    }

    fn req_success(&mut self, id: u64, result: Box<RawValue>) -> Box<RawValue> {
        if let Ok(sub_id) = serde_json::from_str::<SubId>(result.get()) {
            self.add_alias(sub_id.0, id);
            let result = U256::from(id);
            RawValue::from_string(format!("{result:?}")).unwrap()
        } else {
            result
        }
    }

    fn has(&self, id: u64) -> bool {
        self.subs.contains_key(&id)
    }

    fn to_reissue(&self) -> impl Iterator<Item = (&u64, &ActiveSub)> {
        self.subs.iter()
    }

    fn service_subscription_request(
        &mut self,
        id: u64,
        params: Box<RawValue>,
    ) -> Result<Box<RawValue>, WsClientError> {
        let (tx, rx) = mpsc::unbounded();

        let active_sub = ActiveSub { params, channel: tx, current_server_id: None };
        let req = active_sub.serialize_raw(id)?;

        // Explicit scope for the lock
        // This insertion should be made BEFORE the request returns.
        {
            self.channel_map.lock().unwrap().insert(id.into(), rx);
        }
        self.subs.insert(id, active_sub);

        Ok(req)
    }
}

pub struct RequestManager {
    id: AtomicU64,
    subs: SubscriptionManager,
    reqs: BTreeMap<u64, InFlight>,
    backend: BackendDriver,
    conn: ConnectionDetails,
    instructions: mpsc::UnboundedReceiver<Instruction>,
}

impl RequestManager {
    fn next_id(&mut self) -> u64 {
        self.id.fetch_add(1, Ordering::Relaxed)
    }

    pub async fn connect(conn: ConnectionDetails) -> Result<(Self, WsClient), WsClientError> {
        let (ws, backend) = WsBackend::connect(conn.clone()).await?;

        let (instructions_tx, instructions_rx) = mpsc::unbounded();
        let channel_map: SharedChannelMap = Default::default();

        ws.spawn();

        Ok((
            Self {
                id: Default::default(),
                subs: SubscriptionManager::new(channel_map.clone()),
                reqs: Default::default(),
                backend,
                conn,
                instructions: instructions_rx,
            },
            WsClient { instructions: instructions_tx, channel_map },
        ))
    }

    async fn reconnect(&mut self) -> Result<(), WsClientError> {
        // create the new backend
        let (s, mut backend) = WsBackend::connect(self.conn.clone()).await?;

        // spawn the new backend
        s.spawn();

        // swap out the backend
        std::mem::swap(&mut self.backend, &mut backend);

        // rename for clarity
        let mut old_backend = backend;

        // Drain anything in the backend
        while let Some(to_handle) = old_backend.to_handle.next().await {
            self.handle(to_handle);
        }

        // issue a shutdown command (even though it's likely gone)
        old_backend.shutdown();

        // reissue subscriptionps
        for (id, sub) in self.subs.to_reissue() {
            self.backend
                .dispatcher
                .unbounded_send(sub.serialize_raw(*id)?)
                .map_err(|_| WsClientError::DeadChannel)?;
        }

        // reissue requests
        for (id, req) in self.reqs.iter() {
            self.backend
                .dispatcher
                .unbounded_send(req.serialize_raw(*id)?)
                .map_err(|_| WsClientError::DeadChannel)?;
        }

        Ok(())
    }

    fn req_success(&mut self, id: u64, result: Box<RawValue>) {
        // pending fut is missing, this is fine
        if let Some(req) = self.reqs.remove(&id) {
            // Allow subscription manager to rewrite the result if the request
            // corresponds to a known ID
            let result = if self.subs.has(id) { self.subs.req_success(id, result) } else { result };
            let _ = req.channel.send(Ok(result));
        }
    }

    fn req_fail(&mut self, id: u64, error: JsonRpcError) {
        // pending fut is missing, this is fine
        if let Some(req) = self.reqs.remove(&id) {
            // pending fut has been dropped, this is fine
            let _ = req.channel.send(Err(error));
        }
    }

    fn handle_notification(&mut self, params: Notification) {
        self.subs.handle_notification(params)
    }

    fn handle(&mut self, item: PubSubItem) {
        match item {
            PubSubItem::Success { id, result } => self.req_success(id, result),
            PubSubItem::Error { id, error } => self.req_fail(id, error),
            PubSubItem::Notification { params } => self.handle_notification(params),
        }
    }

    fn service_request(
        &mut self,
        id: u64,
        method: String,
        params: Box<RawValue>,
        sender: oneshot::Sender<Response>,
    ) -> Result<(), WsClientError> {
        let in_flight = InFlight { method, params, channel: sender };
        let req = in_flight.to_request(id);
        let req = RawValue::from_string(serde_json::to_string(&req)?)?;
        self.backend.dispatcher.unbounded_send(req).map_err(|_| WsClientError::DeadChannel)?;

        if in_flight.method == "eth_subscribe" {
            self.subs.service_subscription_request(id, in_flight.params.clone())?;
        }

        self.reqs.insert(id, in_flight);
        Ok(())
    }

    fn service_instruction(&mut self, instruction: Instruction) -> Result<(), WsClientError> {
        match instruction {
            Instruction::Request { method, params, sender } => {
                let id = self.next_id();
                self.service_request(id, method, params, sender)?;
            }
            Instruction::Unsubscribe { id } => {
                if let Some(req) = self.subs.end_subscription(id.low_u64()) {
                    self.backend
                        .dispatcher
                        .unbounded_send(req)
                        .map_err(|_| WsClientError::DeadChannel)?;
                }
            }
        }
        Ok(())
    }

    pub fn spawn(mut self) {
        let fut = async move {
            let result = loop {
                select! {
                    _ = &mut self.backend.error => {
                        self.reconnect().await.unwrap();
                    },
                    item_opt = self.backend.to_handle.next() => {
                        match item_opt {
                            Some(item) => self.handle(item),
                            None => if let Err(e) = self.reconnect().await {
                                break Err(e);
                            }
                        }
                    },
                    inst_opt = self.instructions.next() => {
                        match inst_opt {
                            Some(instruction) => if let Err(e) = self.service_instruction(instruction) { break Err(e)},
                            None => break Ok(()),
                        }
                    }
                }
            };
            let _ = result; // todo: log result
            self.backend.shutdown();
        };

        #[cfg(target_arch = "wasm32")]
        super::spawn_local(fut);

        #[cfg(not(target_arch = "wasm32"))]
        tokio::spawn(fut);
    }
}
