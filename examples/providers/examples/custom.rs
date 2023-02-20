//! Create a custom data transport to use with a Provider.

use async_trait::async_trait;
use ethers::{core::utils::Anvil, prelude::*};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use thiserror::Error;
use url::Url;

/// First we must create an error type, and implement [`From`] for
/// [`ProviderError`].
///
/// Here we are using [`thiserror`](https://docs.rs/thiserror) to wrap
/// [`WsClientError`] and [`IpcError`].
///
/// This also provides a conversion implementation ([`From`]) for both, so we
/// can use the [question mark operator](https://doc.rust-lang.org/rust-by-example/std/result/question_mark.html)
/// later on in our implementations.
#[derive(Debug, Error)]
pub enum WsOrIpcError {
    #[error(transparent)]
    Ws(#[from] WsClientError),

    #[error(transparent)]
    Ipc(#[from] IpcError),
}

/// In order to use our `WsOrIpcError` in the RPC client, we have to implement
/// this trait.
///
/// [`RpcError`] helps other parts off the stack get access to common provider
/// error cases. For example, any RPC connection may have a `serde_json` error,
/// so we want to make those easily accessible, so we implement
/// `as_serde_error()`
///
/// In addition, RPC requests may return JSON errors from the node, describing
/// why the request failed. In order to make these accessible, we implement
/// `as_error_response()`.
impl RpcError for WsOrIpcError {
    fn as_error_response(&self) -> Option<&ethers::providers::JsonRpcError> {
        match self {
            WsOrIpcError::Ws(e) => e.as_error_response(),
            WsOrIpcError::Ipc(e) => e.as_error_response(),
        }
    }

    fn as_serde_error(&self) -> Option<&serde_json::Error> {
        match self {
            WsOrIpcError::Ws(WsClientError::JsonError(e)) => Some(e),
            WsOrIpcError::Ipc(IpcError::JsonError(e)) => Some(e),
            _ => None,
        }
    }
}

/// This implementation helps us convert our Error to the library's
/// [`ProviderError`] so that we can use the `?` operator
impl From<WsOrIpcError> for ProviderError {
    fn from(value: WsOrIpcError) -> Self {
        Self::JsonRpcClientError(Box::new(value))
    }
}

/// Next, we create our transport type, which in this case will be an enum that contains
/// either [`Ws`] or [`Ipc`].
#[derive(Clone, Debug)]
enum WsOrIpc {
    Ws(Ws),
    Ipc(Ipc),
}

// We implement a convenience "constructor" method, to easily initialize the transport.
// This will connect to [`Ws`] if it's a valid [URL](url::Url), otherwise it'll
// default to [`Ipc`].
impl WsOrIpc {
    pub async fn connect(s: &str) -> Result<Self, WsOrIpcError> {
        let this = match Url::parse(s) {
            Ok(url) => Self::Ws(Ws::connect(url).await?),
            Err(_) => Self::Ipc(Ipc::connect(s).await?),
        };
        Ok(this)
    }
}

// Next, the most important step: implement [`JsonRpcClient`].
//
// For this implementation, we simply delegate to the wrapped transport and return the
// result.
//
// Note that we are using [`async-trait`](https://docs.rs/async-trait) for asynchronous
// functions in traits, as this is not yet supported in stable Rust; see:
// <https://blog.rust-lang.org/inside-rust/2022/11/17/async-fn-in-trait-nightly.html>
#[async_trait]
impl JsonRpcClient for WsOrIpc {
    type Error = WsOrIpcError;

    async fn request<T, R>(&self, method: &str, params: T) -> Result<R, Self::Error>
    where
        T: Debug + Serialize + Send + Sync,
        R: DeserializeOwned + Send,
    {
        let res = match self {
            Self::Ws(ws) => JsonRpcClient::request(ws, method, params).await?,
            Self::Ipc(ipc) => JsonRpcClient::request(ipc, method, params).await?,
        };
        Ok(res)
    }
}

// We can also implement [`PubsubClient`], since both `Ws` and `Ipc` implement it, by
// doing the same as in the `JsonRpcClient` implementation above.
impl PubsubClient for WsOrIpc {
    // Since both `Ws` and `Ipc`'s `NotificationStream` associated type is the same,
    // we can simply return one of them.
    // In case they differed, we would have to create a `WsOrIpcNotificationStream`,
    // similar to the error type.
    type NotificationStream = <Ws as PubsubClient>::NotificationStream;

    fn subscribe<T: Into<U256>>(&self, id: T) -> Result<Self::NotificationStream, Self::Error> {
        let stream = match self {
            Self::Ws(ws) => PubsubClient::subscribe(ws, id)?,
            Self::Ipc(ipc) => PubsubClient::subscribe(ipc, id)?,
        };
        Ok(stream)
    }

    fn unsubscribe<T: Into<U256>>(&self, id: T) -> Result<(), Self::Error> {
        match self {
            Self::Ws(ws) => PubsubClient::unsubscribe(ws, id)?,
            Self::Ipc(ipc) => PubsubClient::unsubscribe(ipc, id)?,
        };
        Ok(())
    }
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // Spawn Anvil
    let anvil = Anvil::new().block_time(1u64).spawn();

    // Connect to our transport
    let transport = WsOrIpc::connect(&anvil.ws_endpoint()).await?;

    // Wrap the transport in a provider
    let provider = Provider::new(transport);

    // Now we can use our custom transport provider like normal
    let block_number = provider.get_block_number().await?;
    println!("Current block: {block_number}");

    let mut subscription = provider.subscribe_blocks().await?.take(3);
    while let Some(block) = subscription.next().await {
        println!("New block: {:?}", block.number);
    }

    Ok(())
}
