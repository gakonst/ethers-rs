use std::{
    str::FromStr,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use reqwest::{header::CONTENT_TYPE, Client};
use tokio::time::Instant;
use url::Url;

use crate::{err::TransportError, jsonrpc::Response, Connection, RequestFuture};

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
}

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
