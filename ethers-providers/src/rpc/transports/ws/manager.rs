use std::{
    collections::{BTreeMap, HashMap},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};

use ethers_core::types::U256;
use futures_channel::{mpsc, oneshot};
use futures_util::{select_biased, StreamExt};
use serde_json::value::RawValue;

use crate::JsonRpcError;

use super::{
    backend::{BackendDriver, WsBackend},
    ActiveSub, ConnectionDetails, InFlight, Instruction, Notification, PubSubItem, Response, SubId,
    WsClient, WsClientError,
};

pub type SharedChannelMap = Arc<Mutex<HashMap<U256, mpsc::UnboundedReceiver<Box<RawValue>>>>>;

pub const DEFAULT_RECONNECTS: usize = 5;

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

    fn count(&self) -> usize {
        self.subs.len()
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

    #[tracing::instrument(skip(self))]
    fn end_subscription(&mut self, id: u64) -> Option<Box<RawValue>> {
        if let Some(sub) = self.subs.remove(&id) {
            if let Some(server_id) = sub.current_server_id {
                tracing::debug!(server_id = format!("0x{server_id:x}"), "Ending subscription");
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
            tracing::trace!("No current server id");
        }
        tracing::trace!("Cannot end unknown subscription");
        None
    }

    #[tracing::instrument(skip_all, fields(server_id = ?notification.subscription))]
    fn handle_notification(&mut self, notification: Notification) {
        let server_id = notification.subscription;

        // If no alias, just return
        let id_opt = self.aliases.get(&server_id).copied();
        if id_opt.is_none() {
            tracing::debug!(
                server_id = format!("0x{server_id:x}"),
                "No aliased subscription found"
            );
            return
        }
        let id = id_opt.unwrap();

        // alias exists, or should be dropped from alias table
        let sub_opt = self.subs.get(&id);
        if sub_opt.is_none() {
            tracing::trace!(id, "Aliased subscription found, but not active");
            self.aliases.remove(&server_id);
        }
        let active = sub_opt.unwrap();

        tracing::debug!(id, "Forwarding notification to listener");
        // send the notification over the channel
        let send_res = active.channel.unbounded_send(notification.result);

        // receiver has dropped, so we drop the sub
        if send_res.is_err() {
            tracing::debug!(id, "Listener dropped. Dropping alias and subs");
            // TODO: end subcription here?
            self.aliases.remove(&server_id);
            self.subs.remove(&id);
        }
    }

    fn req_success(&mut self, id: u64, result: Box<RawValue>) -> Box<RawValue> {
        if let Ok(server_id) = serde_json::from_str::<SubId>(result.get()) {
            tracing::debug!(id, server_id = %server_id.0, "Registering new sub alias");
            self.add_alias(server_id.0, id);
            let result = U256::from(id);
            RawValue::from_string(format!("\"0x{result:x}\"")).unwrap()
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
        // So we make it before the request is even dispatched :)
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

    reconnects: usize,
}

impl RequestManager {
    fn next_id(&mut self) -> u64 {
        self.id.fetch_add(1, Ordering::Relaxed)
    }

    pub async fn connect(conn: ConnectionDetails) -> Result<(Self, WsClient), WsClientError> {
        Self::connect_with_reconnects(conn, DEFAULT_RECONNECTS).await
    }

    pub async fn connect_with_reconnects(
        conn: ConnectionDetails,
        reconnects: usize,
    ) -> Result<(Self, WsClient), WsClientError> {
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
                reconnects,
            },
            WsClient { instructions: instructions_tx, channel_map },
        ))
    }

    async fn reconnect(&mut self) -> Result<(), WsClientError> {
        if self.reconnects == 0 {
            return Err(WsClientError::TooManyReconnects)
        }
        self.reconnects -= 1;

        tracing::info!(remaining = self.reconnects, url = self.conn.url, "Reconnecting to backend");
        // create the new backend
        let (s, mut backend) = WsBackend::connect(self.conn.clone()).await?;

        // spawn the new backend
        s.spawn();

        // swap out the backend
        std::mem::swap(&mut self.backend, &mut backend);

        // rename for clarity
        let mut old_backend = backend;

        // Drain anything in the backend
        tracing::debug!("Draining old backend to_handle channel");
        while let Some(to_handle) = old_backend.to_handle.next().await {
            self.handle(to_handle);
        }

        // issue a shutdown command (even though it's likely gone)
        old_backend.shutdown();

        tracing::debug!(count = self.subs.count(), "Re-starting active subscriptions");

        // reissue subscriptionps
        for (id, sub) in self.subs.to_reissue() {
            self.backend
                .dispatcher
                .unbounded_send(sub.serialize_raw(*id)?)
                .map_err(|_| WsClientError::DeadChannel)?;
        }

        tracing::debug!(count = self.reqs.len(), "Re-issuing pending requests");
        // reissue requests. We filter these to prevent in-flight requests for
        // subscriptions to be re-issued twice (once in above loop, once in this loop).
        for (id, req) in self.reqs.iter().filter(|(id, _)| !self.subs.has(**id)) {
            self.backend
                .dispatcher
                .unbounded_send(req.serialize_raw(*id)?)
                .map_err(|_| WsClientError::DeadChannel)?;
        }
        tracing::info!(subs = self.subs.count(), reqs = self.reqs.len(), "Re-connection complete");

        Ok(())
    }

    #[tracing::instrument(skip(self, result))]
    fn req_success(&mut self, id: u64, result: Box<RawValue>) {
        // pending fut is missing, this is fine
        tracing::trace!(%result, "Success response received");
        if let Some(req) = self.reqs.remove(&id) {
            tracing::debug!("Sending result to request listener");
            // Allow subscription manager to rewrite the result if the request
            // corresponds to a known ID
            let result = if self.subs.has(id) { self.subs.req_success(id, result) } else { result };
            let _ = req.channel.send(Ok(result));
        } else {
            tracing::trace!("No InFlight found");
        }
    }

    fn req_fail(&mut self, id: u64, error: JsonRpcError) {
        // pending fut is missing, this is fine
        if let Some(req) = self.reqs.remove(&id) {
            // pending fut has been dropped, this is fine
            let _ = req.channel.send(Err(error));
        }
    }

    fn handle(&mut self, item: PubSubItem) {
        match item {
            PubSubItem::Success { id, result } => self.req_success(id, result),
            PubSubItem::Error { id, error } => self.req_fail(id, error),
            PubSubItem::Notification { params } => self.subs.handle_notification(params),
        }
    }

    #[tracing::instrument(skip(self, params, sender))]
    fn service_request(
        &mut self,
        id: u64,
        method: String,
        params: Box<RawValue>,
        sender: oneshot::Sender<Response>,
    ) -> Result<(), WsClientError> {
        let in_flight = InFlight { method, params, channel: sender };
        let req = in_flight.serialize_raw(id)?;

        // Ordering matters here. We want this block above the unbounded send,
        // and after the serialization
        if in_flight.method == "eth_subscribe" {
            self.subs.service_subscription_request(id, in_flight.params.clone())?;
        }

        // Must come after self.subs.service_subscription_request. Do not re-order
        tracing::debug!("Dispatching request to backend");
        self.backend.dispatcher.unbounded_send(req).map_err(|_| WsClientError::DeadChannel)?;

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
                // We bias the loop so that we always handle messages before
                // reconnecting, and always reconnect before
                select_biased! {
                    item_opt = self.backend.to_handle.next() => {
                        match item_opt {
                            Some(item) => self.handle(item),
                            // Backend is gone, so reconnect
                            None => if let Err(e) = self.reconnect().await {
                                break Err(e);
                            }
                        }
                    },
                    _ = &mut self.backend.error => {
                        if let Err(e) = self.reconnect().await {
                            break Err(e);
                        }
                    },
                    inst_opt = self.instructions.next() => {
                        match inst_opt {
                            Some(instruction) => if let Err(e) = self.service_instruction(instruction) { break Err(e)},
                            // User-facing side is gone, so just exit
                            None => break Ok(()),
                        }
                    }
                }
            };
            if let Err(err) = result {
                tracing::error!(%err, "Error during reconnection");
            }
            // Issue the shutdown command. we don't care if it is received
            self.backend.shutdown();
        };

        #[cfg(target_arch = "wasm32")]
        super::spawn_local(fut);

        #[cfg(not(target_arch = "wasm32"))]
        tokio::spawn(fut);
    }
}
