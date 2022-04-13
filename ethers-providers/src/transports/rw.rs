//! A [JsonRpcClient] implementation that serves as a wrapper around two different [JsonRpcClient]
//! and uses a dedicated client for read and the other for write operations

use crate::{provider::ProviderError, JsonRpcClient};

use async_trait::async_trait;

use serde::{de::DeserializeOwned, Serialize};

use thiserror::Error;

/// A client contains two clients.
///
/// One is used for _read_ operations
/// One is used for _write_ operations that consume gas `["eth_sendTransaction",
/// "eth_sendRawTransaction"]`
///
/// **Note**: if the method is unknown this client falls back to the _read_ client
// # Example
#[derive(Debug, Clone)]
pub struct RwClient<Read, Write> {
    /// client used to read
    r: Read,
    /// client used to write
    w: Write,
}

impl<Read, Write> RwClient<Read, Write> {
    /// Creates a new client using two different clients
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use url::Url;
    ///  async fn t(){
    /// use ethers_providers::{Http, RwClient, Ws};
    /// let http = Http::new(Url::parse("http://localhost:8545").unwrap());
    /// let ws = Ws::connect("ws://localhost:8545").await.unwrap();
    /// let rw = RwClient::new(http, ws);
    /// # }
    /// ```
    pub fn new(r: Read, w: Write) -> RwClient<Read, Write> {
        Self { r, w }
    }

    /// Returns the client used for read operations
    pub fn read_client(&self) -> &Read {
        &self.r
    }

    /// Returns the client used for read operations
    pub fn write_client(&self) -> &Write {
        &self.w
    }

    /// Returns a new `RwClient` with transposed clients
    pub fn transpose(self) -> RwClient<Write, Read> {
        let RwClient { r, w } = self;
        RwClient::new(w, r)
    }

    /// Consumes the client and returns the underlying clients
    pub fn split(self) -> (Read, Write) {
        let RwClient { r, w } = self;
        (r, w)
    }
}

#[derive(Error, Debug)]
/// Error thrown when using either read or write client
pub enum RwClientError<Read, Write>
where
    Read: JsonRpcClient,
    <Read as JsonRpcClient>::Error: Sync + Send + 'static,
    Write: JsonRpcClient,
    <Write as JsonRpcClient>::Error: Sync + Send + 'static,
{
    /// Thrown if the _read_ request failed
    #[error(transparent)]
    Read(Read::Error),
    #[error(transparent)]
    /// Thrown if the _write_ request failed
    Write(Write::Error),
}

impl<Read, Write> From<RwClientError<Read, Write>> for ProviderError
where
    Read: JsonRpcClient + 'static,
    <Read as JsonRpcClient>::Error: Sync + Send + 'static,
    Write: JsonRpcClient + 'static,
    <Write as JsonRpcClient>::Error: Sync + Send + 'static,
{
    fn from(src: RwClientError<Read, Write>) -> Self {
        ProviderError::JsonRpcClientError(Box::new(src))
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<Read, Write> JsonRpcClient for RwClient<Read, Write>
where
    Read: JsonRpcClient + 'static,
    <Read as JsonRpcClient>::Error: Sync + Send + 'static,
    Write: JsonRpcClient + 'static,
    <Write as JsonRpcClient>::Error: Sync + Send + 'static,
{
    type Error = RwClientError<Read, Write>;

    /// Sends a POST request with the provided method and the params serialized as JSON
    /// over HTTP
    async fn request<T: Serialize + Send + Sync, R: DeserializeOwned>(
        &self,
        method: &str,
        params: T,
    ) -> Result<R, Self::Error>
    where
        T: std::fmt::Debug + Serialize + Send + Sync,
        R: DeserializeOwned,
    {
        match method {
            "eth_sendTransaction" | "eth_sendRawTransaction" => {
                self.w.request(method, params).await.map_err(RwClientError::Write)
            }
            _ => self.r.request(method, params).await.map_err(RwClientError::Read),
        }
    }
}
