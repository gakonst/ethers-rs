// Code adapted from: https://github.com/althea-net/guac_rs/tree/master/web3/src/jsonrpc
use crate::{provider::ProviderError, JsonRpcClient};

use async_trait::async_trait;
use reqwest::{header::HeaderValue, Client, Error as ReqwestError};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    str::FromStr,
    sync::atomic::{AtomicU64, Ordering},
};
use thiserror::Error;
use url::Url;

use super::common::{Authorization, JsonRpcError, Request, Response};

/// A low-level JSON-RPC Client over HTTP.
///
/// # Example
///
/// ```no_run
/// use ethers_core::types::U64;
/// use ethers_providers::{JsonRpcClient, Http};
/// use std::str::FromStr;
///
/// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
/// let provider = Http::from_str("http://localhost:8545")?;
/// let block_number: U64 = provider.request("eth_blockNumber", ()).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct Provider {
    id: AtomicU64,
    client: Client,
    url: Url,
}

#[derive(Error, Debug)]
/// Error thrown when sending an HTTP request
pub enum ClientError {
    /// Thrown if the request failed
    #[error(transparent)]
    ReqwestError(#[from] ReqwestError),
    #[error(transparent)]
    /// Thrown if the response could not be parsed
    JsonRpcError(#[from] JsonRpcError),

    #[error("Deserialization Error: {err}. Response: {text}")]
    /// Serde JSON Error
    SerdeJson { err: serde_json::Error, text: String },
}

impl From<ClientError> for ProviderError {
    fn from(src: ClientError) -> Self {
        ProviderError::JsonRpcClientError(Box::new(src))
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl JsonRpcClient for Provider {
    type Error = ClientError;

    /// Sends a POST request with the provided method and the params serialized as JSON
    /// over HTTP
    async fn request<T: Serialize + Send + Sync, R: DeserializeOwned>(
        &self,
        method: &str,
        params: T,
    ) -> Result<R, ClientError> {
        let next_id = self.id.fetch_add(1, Ordering::SeqCst);
        let payload = Request::new(next_id, method, params);

        let res = self.client.post(self.url.as_ref()).json(&payload).send().await?;
        let text = res.text().await?;
        let response: Response<'_> = match serde_json::from_str(&text) {
            Ok(response) => response,
            Err(err) => return Err(ClientError::SerdeJson { err, text }),
        };

        let raw = response.as_result().map_err(Clone::clone)?;
        let res = serde_json::from_str(raw.get())
            .map_err(|err| ClientError::SerdeJson { err, text: raw.to_string() })?;

        Ok(res)
    }
}

impl Provider {
    /// Initializes a new HTTP Client
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_providers::Http;
    /// use url::Url;
    ///
    /// let url = Url::parse("http://localhost:8545").unwrap();
    /// let provider = Http::new(url);
    /// ```
    pub fn new(url: impl Into<Url>) -> Self {
        Self::new_with_client(url, Client::new())
    }

    /// Initializes a new HTTP Client with authentication
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_providers::{Authorization, Http};
    /// use url::Url;
    ///
    /// let url = Url::parse("http://localhost:8545").unwrap();
    /// let provider = Http::new_with_auth(url, Authorization::basic("admin", "good_password"));
    /// ```
    pub fn new_with_auth(
        url: impl Into<Url>,
        auth: Authorization,
    ) -> Result<Self, HttpClientError> {
        let mut auth_value = HeaderValue::from_str(&auth.to_string())?;
        auth_value.set_sensitive(true);

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(reqwest::header::AUTHORIZATION, auth_value);

        let client = Client::builder().default_headers(headers).build()?;

        Ok(Self::new_with_client(url, client))
    }

    /// Allows to customize the provider by providing your own http client
    ///
    /// # Example
    ///
    /// ```
    /// use ethers_providers::Http;
    /// use url::Url;
    ///
    /// let url = Url::parse("http://localhost:8545").unwrap();
    /// let client = reqwest::Client::builder().build().unwrap();
    /// let provider = Http::new_with_client(url, client);
    /// ```
    pub fn new_with_client(url: impl Into<Url>, client: reqwest::Client) -> Self {
        Self { id: AtomicU64::new(0), client, url: url.into() }
    }
}

impl FromStr for Provider {
    type Err = url::ParseError;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let url = Url::parse(src)?;
        Ok(Provider::new(url))
    }
}

impl Clone for Provider {
    fn clone(&self) -> Self {
        Self { id: AtomicU64::new(0), client: self.client.clone(), url: self.url.clone() }
    }
}

#[derive(Error, Debug)]
/// Error thrown when dealing with Http clients
pub enum HttpClientError {
    /// Thrown if unable to build headers for client
    #[error(transparent)]
    InvalidHeader(#[from] http::header::InvalidHeaderValue),

    /// Thrown if unable to build client
    #[error(transparent)]
    ClientBuild(#[from] reqwest::Error),
}
