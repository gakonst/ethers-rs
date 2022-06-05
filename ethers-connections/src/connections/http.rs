use std::{
    borrow::Cow,
    str::FromStr,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use reqwest::{header::CONTENT_TYPE, Client};
use serde_json::value::RawValue;
use tokio::time::Instant;
use url::Url;

use crate::{err::TransportError, jsonrpc, BatchRequestFuture, Connection, RequestFuture};

/*
/// A rate-limit aware HTTP [`Connection`].
///
/// When this [`Connection`] encounters a rate-limit error, it will wait for a
/// short period of time before trying to send the request again.
/// The wait time inbetween retries increases with each iteration.
#[derive(Debug)]
pub struct DelayedHttp {
    http: Http,
    max_delay: Option<Duration>,
}

impl DelayedHttp {
    // this is the standard error code for rate limit errors and currently used
    // by at least Alchemy & Infura
    // cf. https://www.rfc-editor.org/rfc/rfc6585#section-4
    const RATE_LIMIT_CODE: i64 = 429;
    const INITIAL_BACKOFF: Duration = Duration::from_secs(1);

    /// Creates a new rate-limit aware HTTP [`Connection`] over the given `url`.
    ///
    /// If no `max_delay` is given, the connection will retry indefinitely.
    /// *Note*, that if it is set too short, no retries may ever happen, so it
    /// should be set no lower than at least 2 seconds.
    ///
    /// # Errors
    ///
    /// Fails, if `url` is not a valid URL.
    pub fn new(url: &str, max_delay: Option<Duration>) -> Result<Self, InvalidUrl> {
        let http = Http::new(url)?;
        Ok(Self { http, max_delay })
    }
}

impl Connection for DelayedHttp {
    fn request_id(&self) -> u64 {
        self.http.request_id()
    }

    fn send_raw_request(&self, id: u64, request: String) -> RequestFuture<'_> {
        Box::pin(async move {
            let deadline = self.max_delay.map(|dur| Instant::now() + dur);
            let mut backoff = Self::INITIAL_BACKOFF;

            loop {
                let response = self.http.send_raw_request(id, request.clone()).await;
                match &response {
                    Err(TransportError::JsonRpc(err)) if err.code == Self::RATE_LIMIT_CODE => {}
                    _ => return response,
                };

                // if a deadline is set and it is already reached, return the
                // response anyways.
                if let Some(deadline) = deadline {
                    if Instant::now() > deadline {
                        return response;
                    }
                }

                // sleep for the calculated backoff duration
                tokio::time::sleep(backoff).await;
                backoff *= 2;
            }
        })
    }
}*/

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
        Self { next_id: AtomicU64::new(0), client: Client::new(), url }
    }

    async fn send_http_request(&self, request: Box<RawValue>) -> Result<String, TransportError> {
        let response = self
            .client
            .post(self.url.as_ref())
            .header(CONTENT_TYPE, "application/json")
            .body(request.to_string())
            .send()
            .await
            .map_err(TransportError::transport)?;

        let text = response.text().await.map_err(|err| TransportError::transport(err))?;
        Ok(text)
    }
}

impl Connection for Http {
    fn request_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    fn send_raw_request(&self, _: u64, request: Box<RawValue>) -> RequestFuture {
        Box::pin(async move {
            let response = self.send_http_request(request).await?;

            if let Ok(jsonrpc::Response { result, .. }) = serde_json::from_str(&response)
                .map_err(|source| TransportError::json(&response, source))
            {
                return Ok(result.to_owned());
            }

            if let Ok(jsonrpc::Error { error, .. }) = serde_json::from_str(&response)
                .map_err(|source| TransportError::json(&response, source))
            {
                return Err(TransportError::jsonrpc(error));
            }

            Err(TransportError::Json { input: response, source: serde::de::Error::custom("TODO") })
        })
    }

    fn send_raw_batch_request(
        &self,
        batch: Vec<(u64, crate::PendingRequest)>,
        request: Box<RawValue>,
    ) -> BatchRequestFuture<'_> {
        let len = batch.len();
        Box::pin(async move {
            let response = self.send_http_request(request).await?;

            // TODO: success and error responses possible?
            if let Ok(mut responses) = serde_json::from_str::<Vec<jsonrpc::Response<'_>>>(&response)
                .map_err(|source| TransportError::json(&response, source))
            {
                for response in responses {
                    let index = match batch.iter().position(|(id, _)| response.id == *id) {
                        Some(index) => index,
                        None => todo!("error"),
                    };

                    let (_, tx) = batch.swap_remove(index);
                    let _ = tx.send(Ok(response.result.to_owned()));
                }

                for (_, tx) in batch {
                    todo!("respond with something, batch error? no response? just drop")
                }
            }

            // may be single error as well
            let raw: Vec<Response> = serde_json::from_str(&response)
                .map_err(|source| TransportError::json(&response, source))?;

            if raw.len() != len {
                todo!("error")
            }
            // TODO: no guaranteed order
            for i in 0..len {
                match raw[i] {
                    Response::Success { id, result } => {
                        if batch[i].0 != id {
                            todo!("error")
                        }

                        let _ = batch[i].1.send(Ok(result.to_owned()));
                    }
                    Response::Error { id, error } => todo!(),
                    _ => todo!("error"),
                }
            }

            Ok(())
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
