use std::{
    future::Future,
    str::FromStr,
    sync::atomic::{AtomicU64, Ordering},
};

use reqwest::{header::CONTENT_TYPE, Client};
use serde::de;
use serde_json::value::RawValue;
use url::Url;

use crate::{batch::BatchError, jsonrpc as rpc, BatchResponseFuture, Connection, ResponseFuture};

use super::ConnectionError;

/// An HTTP [`Connection`].
#[derive(Debug)]
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
        Self { next_id: AtomicU64::new(1), client: Client::new(), url }
    }

    fn http_request(
        &self,
        request: Box<RawValue>,
    ) -> impl Future<Output = Result<reqwest::Response, reqwest::Error>> + 'static {
        self.client
            .post(self.url.as_ref())
            .header(CONTENT_TYPE, "application/json")
            .body(request.to_string())
            .send()
    }
}

impl Connection for Http {
    fn request_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    fn send_raw_request(&self, _: u64, request: Box<RawValue>) -> ResponseFuture {
        let future = self.http_request(request);
        Box::pin(async move {
            let response = future.await.map_err(ConnectionError::connection)?;
            let text = response.text().await.map_err(ConnectionError::connection)?.into_boxed_str();

            // try to parse as response (most likely)
            if let Ok(rpc::Response { result, .. }) = serde_json::from_str(&text) {
                return Ok(result.to_owned());
            }

            // try to parse as error (least likely)
            if let Ok(rpc::Error { error, .. }) = serde_json::from_str(&text) {
                return Err(ConnectionError::jsonrpc(error));
            }

            Err(ConnectionError::Json {
                input: text,
                source: de::Error::custom("invalid HTTP response"),
            })
        })
    }

    fn send_raw_batch_request(
        &self,
        ids: Box<[u64]>,
        request: Box<RawValue>,
    ) -> BatchResponseFuture {
        let future = self.http_request(request);
        Box::pin(async move {
            let response = future.await.map_err(ConnectionError::connection)?;
            let text = response.text().await.map_err(ConnectionError::connection)?.into_boxed_str();

            // try to parse as batch response
            if let Ok(mut batch) = rpc::deserialize_batch_response(&text) {
                let len = ids.len();
                if batch.len() != len {
                    return Err(BatchError::IncompleteBatch);
                }

                for i in 0..len {
                    for j in i..len {
                        if ids[i] == batch[j].id() && i != j {
                            batch.swap(i, j);
                        }
                    }
                }

                let responses = batch.into_iter().map(rpc::ResponseOrError::to_result).collect();
                return Ok(responses);
            }

            Err(ConnectionError::Json {
                input: text,
                source: de::Error::custom("invalid HTTP batch response"),
            }
            .into())
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
