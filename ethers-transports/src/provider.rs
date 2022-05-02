#[cfg(feature = "ipc")]
use std::path::Path;
use std::{borrow::Cow, future::Future};

use ethers_core::types::{Address, U256};

#[cfg(all(unix, feature = "ipc"))]
use crate::transports::ipc::{Ipc, IpcError};
use crate::types::SyncStatus;
use crate::{err::TransportError, BidiTransport, ResponsePayload, Transport, TransportExt};

/// A provider for Ethereum JSON-RPC API calls.
pub struct Provider<T> {
    transport: T,
}

impl<T> Provider<T> {
    pub fn new(transport: T) -> Self {
        Self { transport }
    }
}

#[cfg(all(unix, feature = "ipc"))]
impl Provider<Ipc> {
    /// Attempts to establish a connection with the IPC socket at the given
    /// `path`.
    ///
    /// # Errors
    ///
    /// This fails, if the file at `path` is not a valid IPC socket.
    pub async fn connect(path: impl AsRef<Path>) -> Result<Self, IpcError> {
        todo!()
    }
}

impl<T: Transport> Provider<T> {
    /// Returns the current ethereum protocol version.
    pub async fn get_protocol_version(&self) -> Result<String, ProviderError> {
        todo!()
    }

    /// Returns data about the sync status or `None`, if the client is fully
    /// synced.
    pub async fn get_syncing(&self) -> Result<Option<SyncStatus>, ProviderError> {
        #[derive(Deserialize)]
        struct Helper(
            #[serde(deserialize_with = "crate::types::deserialize_sync_status")] Option<SyncStatus>,
        );

        const METHOD: &str = "eth_syncing";
        let request = self.transport.send_request(METHOD, ());
        let Helper(status) = Self::decode_response(METHOD, request).await?;
        Ok(status)
    }

    /// Returns the client coinbase address.
    pub async fn get_coinbase(&self) -> Result<Address, ProviderError> {
        todo!()
    }

    ///Returns `true` if the client is actively mining new blocks.
    pub async fn get_mining(&self) -> Result<bool, ProviderError> {
        todo!()
    }

    pub async fn get_accounts(&self) -> Result<Vec<Address>, ProviderError> {
        const METHOD: &str = "eth_getAccounts";
        let request = self.transport.send_request(METHOD, ());
        Self::decode_response(METHOD, request).await
    }

    pub async fn get_balance(&self, address: &Address) -> Result<U256, Box<ProviderError>> {
        const METHOD: &str = "eth_getBalance";
        let request = self.transport.send_request(METHOD, ());
        Self::decode_response(METHOD, request).await
    }

    pub async fn get_storage_at(
        &self,
        address: &Address,
        pos: &U256,
        block: Option<&U256>,
    ) -> Result<U256, ProviderError> {
        todo!()
    }

    pub async fn get_block_number(&self) -> Result<u64, ProviderError> {
        const METHOD: &str = "eth_blockNumber";
        let request = self.transport.send_request(METHOD, ());
        Self::decode_response(METHOD, request).await
    }

    pub async fn get_transaction_count(
        &self,
        address: &Address,
    ) -> Result<U256, Box<ProviderError>> {
        const METHOD: &str = "eth_getTransactionCount";
        let request = self.transport.send_request(METHOD, ());
        Self::decode_response(METHOD, request).await
    }

    async fn decode_response<R: Deserialize>(
        method: &str,
        request: Box<dyn Future<Item = ResponsePayload>>,
    ) -> Result<R, ProviderError> {
        let raw = request.await.map_err(|err| {
            transport_err(err)
                .with_context(format!("failed RPC call to `{method}` (rpc request failed)"))
        })?;

        let decoded = serde_json::from_str(raw.get()).map_err(|err| {
            ProviderError::json(err).with_ctx(format!(
                "failed RPC call to `{method}` (response deserialization failed)"
            ))
        })?;

        Ok(decoded)
    }
}

impl<T: BidiTransport> Provider<T> {}

// TODO: Transport(Box<TransportError>), Json(serde_json::Error)
// + context (string)
pub struct ProviderError {
    pub kind: ErrorKind,
    context: Cow<'static, str>,
}

impl ProviderError {
    fn transport(err: Box<TransportError>) -> Box<Self> {
        todo!()
    }

    fn json(err: serde_json::Error) -> Box<Self> {
        todo!()
    }

    fn with_ctx(mut self: Box<Self>, context: impl Into<Cow<'static, str>>) -> Box<Self> {
        self.context = context.into();
        self
    }
}

pub enum ErrorKind {
    Transport(Box<TransportError>),
}

fn transport_err(err: Box<TransportError>) -> Box<Self> {
    todo!()
}
