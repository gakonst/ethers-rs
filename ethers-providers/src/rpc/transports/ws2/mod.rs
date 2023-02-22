#![allow(missing_docs)]

pub(crate) mod macros;

mod backend;
mod manager;

mod types;

pub(self) use types::*;

mod error;
pub use error::*;

use futures_channel::{mpsc, oneshot};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::value::RawValue;

use crate::{JsonRpcClient, PubsubClient};
use async_trait::async_trait;
use ethers_core::types::U256;
use manager::RequestManager;

#[derive(Debug)]
pub struct WsClient {
    instructions: mpsc::UnboundedSender<Instruction>,
}

impl WsClient {
    pub async fn connect(conn: ConnectionDetails) -> Result<Self, WsClientError> {
        let (man, this) = RequestManager::connect(conn).await?;
        man.spawn();
        Ok(this)
    }

    async fn request<R>(&self, method: &str, params: Box<RawValue>) -> Result<R, WsClientError>
    where
        R: DeserializeOwned,
    {
        let (tx, rx) = oneshot::channel();
        let instruction = Instruction::Request { method: method.to_owned(), params, sender: tx };
        self.instructions
            .unbounded_send(instruction)
            .map_err(|_| WsClientError::UnexpectedClose)?;

        let res = rx.await.map_err(|_| WsClientError::UnexpectedClose)??;

        Ok(serde_json::from_str(res.get())?)
    }

    async fn request_sub<R>(&self, params: Box<RawValue>) -> Result<R, WsClientError>
    where
        R: DeserializeOwned,
    {
        let (tx, rx) = mpsc::unbounded();
        let (req_tx, req_rx) = oneshot::channel();

        let instruction = Instruction::SubscriptionRequest { params, channel: tx, sender: req_tx };

        self.instructions
            .unbounded_send(instruction)
            .map_err(|_| WsClientError::UnexpectedClose)?;

        let res = req_rx.await.map_err(|_| WsClientError::UnexpectedClose)??;

        Ok(serde_json::from_str(res.get())?)
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl JsonRpcClient for WsClient {
    type Error = WsClientError;

    async fn request<T, R>(&self, method: &str, params: T) -> Result<R, WsClientError>
    where
        T: Serialize + Send + Sync,
        R: DeserializeOwned,
    {
        let params = serde_json::to_string(&params)?;
        let params = RawValue::from_string(params)?;

        if method == "eth_subscribe" {
            self.request_sub(params).await
        } else {
            self.request(method, params).await
        }
    }
}

impl PubsubClient for WsClient {
    type NotificationStream = mpsc::UnboundedReceiver<Box<RawValue>>;

    fn subscribe<T: Into<U256>>(&self, id: T) -> Result<Self::NotificationStream, WsClientError> {
        let (tx, rx) = oneshot::channel();

        rx.await
    }

    fn unsubscribe<T: Into<U256>>(&self, id: T) -> Result<(), WsClientError> {
        self.instructions
            .unbounded_send(Instruction::Unsubscribe { id: id.into() })
            .map_err(|_| WsClientError::UnexpectedClose)
    }
}
