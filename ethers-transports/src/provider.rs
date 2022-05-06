use std::borrow::Cow;
#[cfg(feature = "ipc")]
use std::path::Path;

use ethers_core::types::{Address, BlockNumber, Bytes, U256};
use serde::{Deserialize, Serialize};

#[cfg(all(unix, feature = "ipc"))]
use crate::transports::ipc::{Ipc, IpcError};
use crate::types::SyncStatus;
use crate::{err::TransportError, BidiTransport, Transport, TransportExt};

/// A provider for Ethereum JSON-RPC API calls.
///
/// This type provides type-safe bindings to all RPC calls defined in the
/// [JSON-RPC API specification](https://eth.wiki/json-rpc/API).
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
        let transport = Ipc::connect(path).await?;
        Ok(Self { transport })
    }
}

impl<T: Transport> Provider<T> {
    /// Returns the current ethereum protocol version.
    pub async fn get_protocol_version(&self) -> Result<String, ProviderError> {
        todo!()
    }

    /// Returns data about the sync status or `None`, if the client is fully
    /// synced.
    pub async fn get_syncing(&self) -> Result<Option<SyncStatus>, Box<ProviderError>> {
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
    pub async fn get_balance(&self, address: &Address) -> Result<U256, Box<ProviderError>> {
        self.send_request("eth_getBalance", [address]).await
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
        let raw = self.transport.send_request(method, params).await.map_err(|err| {
            transport_err(err)
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

impl<T: BidiTransport + Clone> Provider<T> {
    pub async fn subscribe_new_heads(&self) -> Result<(), Box<ProviderError>> {
        todo!("-> impl SubscriptionStream<Item = Block>")
    }
}

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

fn transport_err(err: Box<TransportError>) -> Box<ProviderError> {
    todo!()
}
