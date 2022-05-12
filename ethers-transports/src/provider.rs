#[cfg(feature = "ipc")]
use std::path::Path;
use std::{borrow::Cow, error, fmt, sync::Arc};

use ethers_core::types::{Address, BlockNumber, Bytes, U256};
use serde::{Deserialize, Serialize};

#[cfg(all(unix, feature = "ipc"))]
use crate::connections::ipc::{Ipc, IpcError};
use crate::{
    connections,
    err::TransportError,
    jsonrpc::{JsonRpcError, Request},
    types::SyncStatus,
    Connection, ConnectionExt, DuplexConnection, SubscriptionStream,
};

/// A provider for Ethereum JSON-RPC API calls.
///
/// This type provides type-safe bindings to all RPC calls defined in the
/// [JSON-RPC API specification](https://eth.wiki/json-rpc/API).
#[derive(Clone, Copy)]
pub struct Provider<C> {
    connection: C,
}

impl<C> Provider<C> {
    /// Returns a new [`Provider`].
    pub fn new(connection: C) -> Self {
        Self { connection }
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
        let transport = Ipc::connect(path).await?;
        Ok(Self { connection: transport })
    }
}

impl Provider<Arc<dyn Connection>> {
    /// Attempts to connect to any of the available connections based on the
    /// given `path`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ethers_transports::Provider;
    ///
    /// # async fn connect_any() -> Result<(), Box<dyn std::error::Error>> {
    /// // connects via HTTP
    /// let provider = Provider::connect("http://localhost:8545").await?;
    /// // connect via websocket
    /// let provider = Provider::connect("ws://localhost:8546").await?;
    /// // connects to a local IPC socket
    /// let provider = Provider::connect("ipc:///home/user/.ethereum/geth.ipc").await?;
    /// let provider = Provider::connect("/home/user/.ethereum/geth.ipc").await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///  
    /// Fails, if the selected connection can not be established.
    ///
    /// # Panics
    ///
    /// Panics, if a connection is selected which has not been feature-enabled
    /// at compile time, e.g., if a HTTP url is given but the `http` cargo
    /// feature is not enabled.
    pub async fn connect(path: &str) -> Result<Self, Box<TransportError>> {
        let connection: Arc<dyn Connection> = if path.starts_with("http") {
            #[cfg(feature = "http")]
            {
                let http = connections::http::Http::new(path)
                    .map_err(|err| TransportError::transport(err))?;
                Arc::new(http)
            }
            #[cfg(not(feature = "http"))]
            {
                panic!("path starts with http/https, but `http` cargo feature is not enabled");
            }
        } else if path.starts_with("ws") {
            #[cfg(feature = "ws")]
            {
                todo!("...")
            }
            #[cfg(not(feature = "ws"))]
            {
                panic!("path starts with ws/wss, but `ws` cargo feature is not enabled");
            }
        } else {
            #[cfg(feature = "ipc")]
            {
                // the path is allowed start with "ipc://"
                let ipc = connections::ipc::Ipc::connect(path.trim_start_matches("ipc://"))
                    .await
                    .map_err(|err| TransportError::transport(err))?;
                Arc::new(ipc)
            }
            #[cfg(not(feature = "ipc"))]
            {
                todo!("ipc path detected, but `ipc` cargo feature is not enabled");
            }
        };

        Ok(Self { connection })
    }
}

impl<C: Connection + 'static> Provider<C> {
    /// Borrows the underlying [`Connection`] and returns a new provider that
    /// can be cheaply cloned and copied.
    pub fn borrow(&self) -> Provider<&'_ C> {
        let connection = &self.connection;
        Provider { connection }
    }
}

impl<C: Connection> Provider<C> {
    /// Returns the current ethereum protocol version.
    pub async fn get_protocol_version(&self) -> Result<String, Box<ProviderError>> {
        self.send_request("eth_protocolVersion", ()).await
    }

    /// Returns data about the sync status or `None`, if the client is fully
    /// synced.
    pub async fn syncing(&self) -> Result<Option<SyncStatus>, Box<ProviderError>> {
        #[derive(Deserialize)]
        struct Helper(
            #[serde(deserialize_with = "crate::types::deserialize_sync_status")] Option<SyncStatus>,
        );

        let Helper(status) = self.send_request("eth_syncing", ()).await?;
        Ok(status)
    }

    /// Returns the client coinbase address.
    pub async fn get_coinbase(&self) -> Result<Address, Box<ProviderError>> {
        self.send_request("eth_coinbase", ()).await
    }

    /// Returns `true` if the client is actively mining new blocks.
    pub async fn get_mining(&self) -> Result<bool, Box<ProviderError>> {
        self.send_request("eth_mining", ()).await
    }

    /// Returns the number of hashes per second that the node is mining with.
    pub async fn get_hashrate(&self) -> Result<U256, Box<ProviderError>> {
        self.send_request("eth_hashrate", ()).await
    }

    /// Returns the current price per gas in wei.
    pub async fn get_gas_price(&self) -> Result<U256, Box<ProviderError>> {
        self.send_request("eth_gasPrice", ()).await
    }

    /// Returns a list of addresses owned by client.
    pub async fn get_accounts(&self) -> Result<Vec<Address>, Box<ProviderError>> {
        self.send_request("eth_getAccounts", ()).await
    }

    /// Returns the number of most recent block.
    pub async fn get_block_number(&self) -> Result<u64, Box<ProviderError>> {
        self.send_request("eth_blockNumber", ()).await
    }

    /// Returns the balance of the account of given address.
    pub async fn get_balance(
        &self,
        address: &Address,
        block: &BlockNumber,
    ) -> Result<U256, Box<ProviderError>> {
        self.send_request("eth_getBalance", (address, block)).await
    }

    /// Returns the value from a storage position at a given address.
    pub async fn get_storage_at(
        &self,
        address: &Address,
        pos: &U256,
        block: Option<&BlockNumber>,
    ) -> Result<U256, Box<ProviderError>> {
        self.send_request("eth_getStorageAt", (address, pos, block)).await
    }

    /// Returns the number of transactions sent from an address.
    pub async fn get_transaction_count(
        &self,
        address: &Address,
    ) -> Result<U256, Box<ProviderError>> {
        self.send_request("eth_getTransactionCount", [address]).await
    }

    /// Returns code at a given address.
    pub async fn get_code(
        &self,
        address: &Address,
        block: Option<&BlockNumber>,
    ) -> Result<Bytes, Box<ProviderError>> {
        self.send_request("eth_getCode", (address, block)).await
    }

    /// Signs the given `message` using the account at `address`.
    ///
    /// The sign method calculates an Ethereum specific signature with:
    /// `sign(keccak256("\x19Ethereum Signed Message:\n" + len(message) + message)))`.
    ///
    /// By adding a prefix to the message makes the calculated signature
    /// recognisable as an Ethereum specific signature.
    /// This prevents misuse where a malicious DApp can sign arbitrary data
    /// (e.g. transaction) and use the signature to impersonate the victim.
    ///
    /// **Note** the address to sign with must be unlocked.
    pub async fn sign(
        &self,
        address: &Address,
        message: &Bytes,
    ) -> Result<Bytes, Box<ProviderError>> {
        self.send_request("eth_sign", (address, message)).await
    }

    async fn send_request<P, R>(&self, method: &str, params: P) -> Result<R, Box<ProviderError>>
    where
        P: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        // send the request & await its (raw) response
        let raw = self.connection.send_request(method, params).await.map_err(|err| {
            err.to_provider_err()
                .with_ctx(format!("failed RPC call to `{method}` (rpc request failed)"))
        })?;

        // decode the response to the expected result type
        let decoded = serde_json::from_str(raw.get()).map_err(|err| {
            ProviderError::json(err).with_ctx(format!(
                "failed RPC call to `{method}` (response deserialization failed)"
            ))
        })?;

        Ok(decoded)
    }
}

impl<C: DuplexConnection + Clone> Provider<C> {
    pub async fn subscribe_new_heads(
        &self,
    ) -> Result<SubscriptionStream<(), C>, Box<ProviderError>> {
        let connection = self.connection.clone();

        let id = connection.request_id();
        let request = Request { id, method: "eth_subscribe", params: ("newHeads") }.to_json();

        let (id, rx) =
            connection.subscribe(id, request).await.map_err(|err| err.to_provider_err())?;

        Ok(SubscriptionStream::new(id, connection, rx))
    }
}

// TODO: Transport(Box<TransportError>), Json(serde_json::Error)
// + context (string)
#[derive(Debug)]
pub struct ProviderError {
    pub kind: ErrorKind,
    pub(crate) context: Cow<'static, str>,
}

impl ProviderError {
    pub fn context(&self) -> Option<&str> {
        if self.context.is_empty() {
            None
        } else {
            Some(self.context.as_ref())
        }
    }

    pub fn as_jsonrpc(&self) -> Option<&JsonRpcError> {
        match &self.kind {
            ErrorKind::Transport(err) => match err.as_ref() {
                TransportError::JsonRpc(err) => Some(err),
                _ => None,
            },
            _ => None,
        }
    }

    fn json(err: serde_json::Error) -> Box<Self> {
        Box::new(Self { kind: ErrorKind::Json(err), context: "".into() })
    }

    fn with_ctx(mut self: Box<Self>, context: impl Into<Cow<'static, str>>) -> Box<Self> {
        self.context = context.into();
        self
    }
}

impl error::Error for ProviderError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match &self.kind {
            ErrorKind::Json(err) => Some(err),
            ErrorKind::Transport(err) => Some(&*err),
        }
    }
}

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = &self.kind;
        match self.context() {
            Some(ctx) => write!(f, "{ctx}: {kind}"),
            None => write!(f, "{kind}"),
        }
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    /// The error returned when parsing the raw response into the expected type
    /// fails.
    Json(serde_json::Error),
    Transport(Box<TransportError>),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(err) => write!(f, "failed to parse JSON response to expected type: {err}"),
            Self::Transport(err) => write!(f, "{err}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{future::Future, sync::Arc};

    use tokio::runtime::Builder;

    use crate::{connections::noop, Connection, Provider};

    fn block_on(future: impl Future<Output = ()>) {
        Builder::new_current_thread().enable_all().build().unwrap().block_on(future);
    }

    #[test]
    fn object_safety() {
        block_on(async move {
            let provider = Provider::new(noop::Noop);
            let res = provider.get_block_number().await;
            assert!(res.is_err());

            let provider: Provider<Arc<dyn Connection>> = Provider::new(Arc::new(noop::Noop));
            let res = provider.get_block_number().await;
            assert!(res.is_err());
        });
    }
}
