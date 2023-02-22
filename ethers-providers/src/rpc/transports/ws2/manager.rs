use std::{
    collections::{BTreeMap, HashMap},
    sync::atomic::{AtomicU64, Ordering},
};

use ethers_core::types::U256;
use futures_channel::{mpsc, oneshot};
use futures_util::{select, StreamExt};
use serde_json::value::RawValue;

use crate::JsonRpcError;

use super::{
    backend::{Backend, WsBackend},
    ActiveSub, ConnectionDetails, InFlight, Instruction, Notification, Response, SubId, WsClient,
    WsClientError, WsItem,
};

#[derive(Default)]
pub struct SubscriptionManager {
    subs: BTreeMap<u64, ActiveSub>,
    aliases: HashMap<U256, u64>,
    sub_channel_holding: HashMap<u64, mpsc::UnboundedReceiver<Notification>>,
}

impl SubscriptionManager {
    fn add_alias(&mut self, sub: U256, id: u64) {
        if let Some(entry) = self.subs.get_mut(&id) {
            entry.aliases.insert(sub);
        }
        self.aliases.insert(sub, id);
    }

    fn remove_alias(&mut self, sub: U256) {
        if let Some(entry) = self.subs.get_mut(&sub) {
            entry.aliases.remove(&sub);
        }
        self.aliases.remove(&sub);
    }

    fn remove_subscription(&mut self, id: u64) -> Box<RawValue> {
        if let Some(sub) = self.subs.remove(&id) {
            sub.aliases.iter().for_each(|id| self.aliases.remove_alias(id));
        }
    }

    fn get_subscription(
        &mut self,
        id: U256,
        sender: oneshot::Sender<mpsc::UnboundedReceiver<Notification>>,
    ) {
        if let Some(channel) = self.sub_channel_holding.remove(&id.low_u64()) {
            let _ = sender.send(channel);
        }
    }

    fn handle_notification(&mut self, params: Notification) {
        let sub_id = params.subscription;

        if let Some(id) = self.aliases.get(&sub_id) {
            if let Some(active) = self.subs.get(id) {
                // send the notification over the channel
                let send_res = active.channel.unbounded_send(params);
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
            self.add_alias(sub_id.subscription, id);
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
        // we make both a pending req and an active sub here
        let active_sub = ActiveSub { params, channel: tx };
        let req = active_sub.to_request(id);

        self.sub_channel_holding.insert(id, rx);
        self.subs.insert(id, active_sub);

        Ok(RawValue::from_string(serde_json::to_string(&req)?)?)
    }
}

pub struct RequestManager {
    id: AtomicU64,
    subs: SubscriptionManager,
    reqs: BTreeMap<u64, InFlight>,
    backend: Backend,
    conn: ConnectionDetails,
    instructions: mpsc::UnboundedReceiver<Instruction>, // TODO
}

impl RequestManager {
    fn next_id(&mut self) -> u64 {
        self.id.fetch_add(1, Ordering::Relaxed)
    }

    pub async fn connect(conn: ConnectionDetails) -> Result<(Self, WsClient), WsClientError> {
        let (ws, backend) = WsBackend::connect(conn.clone()).await?;

        let (tx, rx) = mpsc::unbounded();

        ws.spawn();

        Ok((
            Self {
                id: Default::default(),
                subs: Default::default(),
                reqs: Default::default(),
                backend,
                conn,
                instructions: rx,
            },
            WsClient { instructions: tx },
        ))
    }

    async fn reconnect(&mut self) -> Result<(), WsClientError> {
        // create the new backend
        let (s, mut backend) = WsBackend::connect(self.conn.clone()).await?;

        // spawn the new backend
        s.spawn();

        // swap the out our backend
        std::mem::swap(&mut self.backend, &mut backend);

        // rename for clarity
        let old_backend = backend;

        // don't care if it errored
        let _ = old_backend.shutdown.send(());

        // reissue subscriptionps
        for (id, sub) in self.subs.to_reissue() {
            self.backend
                .dispatcher
                .unbounded_send(sub.serialize_raw(*id))
                .map_err(|_| WsClientError::DeadChannel)?;
        }

        // reissue requests
        for (id, req) in self.reqs.iter() {
            self.backend
                .dispatcher
                .unbounded_send(req.serialize_raw(*id))
                .map_err(|_| WsClientError::DeadChannel)?;
        }

        Ok(())
    }

    fn req_success(&mut self, id: u64, result: Box<RawValue>) {
        // pending fut is missing, this is fine
        if let Some(req) = self.reqs.remove(&id) {
            if self.subs.has(id) {
                let result = self.subs.req_success(id, result);
                let _ = req.channel.send(Ok(result));
            } else {
                // if error, pending fut has been dropped, this is fine
                let _ = req.channel.send(Ok(result));
            }
        }
    }

    fn req_fail(&mut self, id: u64, error: JsonRpcError) {
        // pending fut is missing, this is fine
        if let Some(mut req) = self.reqs.remove(&id) {
            // pending fut has been dropped, this is fine
            let _ = req.channel.send(Err(error));
        }
    }

    fn handle_notification(&mut self, params: Notification) {
        self.subs.handle_notification(params)
    }

    fn handle(&mut self, item: WsItem) {
        match item {
            WsItem::Success { id, result } => self.req_success(id, result),
            WsItem::Error { id, error } => self.req_fail(id, error),
            WsItem::Notification { params } => self.handle_notification(params),
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
        self.reqs.insert(id, in_flight);
        Ok(())
    }

    fn service_instruction(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Request { method, params, sender } => {
                let id = self.next_id();
                self.service_request(id, method, params, sender);
                if method == "eth_subscribe" {
                    self.subs.service_subscription_request(id, params);
                }
            }
            Instruction::GetSubscription { id, sender } => self.subs.get_subscription(id, sender),
            Instruction::Unsubscribe { id } => {
                let req = self.subs.unsub(id);
                let _ = self.backend.dispatcher.unbounded_send(req);
            }
        }
    }

    pub fn spawn(mut self) {
        let fut = async move {
            loop {
                select! {
                    _ = &mut self.backend.error => {
                        self.reconnect().await.unwrap();
                    },
                    item_opt = self.backend.to_handle.next() => {
                        match item_opt {
                            Some(item) => self.handle(item),
                            None => self.reconnect().await.unwrap()
                        }
                    },
                    inst_opt = self.instructions.next() => {
                        match inst_opt {
                            Some(instruction) => self.service_instruction(instruction),
                            None => {
                                let _ = self.backend.shutdown.send(());
                                break;
                            },
                        }
                    }
                }
            }
        };

        #[cfg(target_arch = "wasm32")]
        super::spawn_local(fut);

        #[cfg(not(target_arch = "wasm32"))]
        tokio::spawn(fut);
    }
}
