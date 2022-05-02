use std::{
    pin::Pin,
    str::FromStr,
    sync::atomic::{AtomicU64, Ordering},
};

use reqwest::{header::CONTENT_TYPE, Client};
use url::Url;

use crate::{err::TransportError, jsonrpc::Response, RequestFuture, Transport};

pub struct Http {
    next_id: AtomicU64,
    client: Client,
    url: Url,
}

impl Http {
    pub fn new(url: Url) -> Self {
        Self { next_id: AtomicU64::new(0), client: Client::new(), url }
    }
}

impl Transport for Http {
    fn request_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    fn send_raw_request(&self, request: String) -> RequestFuture {
        Box::pin(async move {
            let resp = self
                .client
                .post(self.url.as_ref())
                .header(CONTENT_TYPE, "application/json")
                .body(request.into())
                .send()
                .await
                .map_err(TransportError::transport)?;

            let text = resp.text().await.map_err(TransportError::transport)?;
            let raw =
                serde_json::from_str(&text).map_err(|source| TransportError::json(text, source))?;
            match raw {
                Response::Success { result, .. } => Ok(result.to_owned()),
                Response::Error { error, .. } => Err(TransportError::jsonrpc(error)),
                Response::Notification { .. } => todo!("return appropriate JSON error"),
            };
        })
    }
}

impl FromStr for Http {
    type Err = <Url as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url = Url::from_str(s)?;
        Ok(Self::new(url))
    }
}
