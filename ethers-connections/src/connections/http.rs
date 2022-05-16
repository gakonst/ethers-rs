use std::{
    str::FromStr,
    sync::atomic::{AtomicU64, Ordering},
};

use reqwest::{header::CONTENT_TYPE, Client};
use url::Url;

use crate::{err::TransportError, jsonrpc::Response, Connection, RequestFuture};

/// An HTTP [`Connection`].
pub struct Http {
    next_id: AtomicU64,
    client: Client,
    url: Url,
}

impl Http {
    /// Creates a new HTTP [`Connection`] over the given `url`.
    ///
    /// # Errors
    ///
    /// Fails, if `url` is not a valid URL.
    pub fn new(url: &str) -> Result<Self, InvalidUrl> {
        let url = url.parse()?;
        Ok(Self::from_url(url))
    }

    fn from_url(url: Url) -> Self {
        Self { next_id: AtomicU64::new(0), client: Client::new(), url }
    }
}

impl Connection for Http {
    fn request_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    fn send_raw_request(&self, _: u64, request: String) -> RequestFuture {
        Box::pin(async move {
            let resp = self
                .client
                .post(self.url.as_ref())
                .header(CONTENT_TYPE, "application/json")
                .body(request)
                .send()
                .await
                .map_err(TransportError::transport)?;

            let text = resp.text().await.map_err(|err| TransportError::transport(err))?;
            let raw = serde_json::from_str(&text)
                .map_err(|source| TransportError::json(&text, source))?;

            match raw {
                Response::Success { result, .. } => Ok(result.to_owned()),
                Response::Error { error, .. } => Err(TransportError::jsonrpc(error)),
                Response::Notification { .. } => todo!("return appropriate JSON error"),
            }
        })
    }
}

impl FromStr for Http {
    type Err = InvalidUrl;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url = Url::from_str(s)?;
        Ok(Self::from_url(url))
    }
}

pub type InvalidUrl = url::ParseError;
