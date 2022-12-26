use crate::{provider::ProviderError, JsonRpcClient};
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::{atomic::AtomicU64, Arc};
use thiserror::Error;
// use tokio::net::windows::net_pipe;

#[derive(Debug)]
pub struct NamedPipe {
    id: Arc<AtomicU64>,
}

impl NamedPipe {
    pub async fn connect() -> Self {
        let id = Arc::new(AtomicU64::new(1));
        Self { id }
    }
}

#[async_trait]
impl JsonRpcClient for NamedPipe {
    type Error = ClientError;

    async fn request<T: Serialize + Send + Sync, R: DeserializeOwned>(
        &self,
        method: &str,
        params: T,
    ) -> Result<R, ClientError> {
        todo!()
    }
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("test")]
    Test,
}

impl From<ClientError> for ProviderError {
    fn from(value: ClientError) -> Self {
        ProviderError::JsonRpcClientError(Box::new(value))
    }
}
