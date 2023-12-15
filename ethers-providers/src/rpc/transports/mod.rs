pub(crate) mod common;
pub use common::{Authorization, JsonRpcError, JwtAuth, JwtKey};

mod http;
use self::http::{ClientError, JsonRpcClient as RawJsonRpcClient};
pub use self::http::{ClientError as HttpClientError, Provider as HttpProvider};

///the old strict Http provider -- fails if the response has a `method:` entry
pub type Http = HttpProvider<false>;

///the new loose Http provider
pub type RelaxedHttp = HttpProvider<true>;

///when you want either Http or LooseHttp
pub type HttpClient = dyn RawJsonRpcClient<Error = ClientError>;

#[cfg(all(feature = "ipc", any(unix, windows)))]
mod ipc;
#[cfg(all(feature = "ipc", any(unix, windows)))]
pub use ipc::{Ipc, IpcError};

mod quorum;
pub use quorum::{JsonRpcClientWrapper, Quorum, QuorumError, QuorumProvider, WeightedProvider};

mod rw;
pub use rw::{RwClient, RwClientError};

mod retry;
pub use retry::*;

#[cfg(all(feature = "ws", not(feature = "legacy-ws")))]
mod ws;
#[cfg(all(feature = "ws", not(feature = "legacy-ws")))]
pub use ws::{ConnectionDetails, WsClient as Ws, WsClientError};

/// archival websocket
#[cfg(feature = "legacy-ws")]
pub mod legacy_ws;
#[cfg(feature = "legacy-ws")]
pub use legacy_ws::{ClientError as WsClientError, Ws};

mod mock;
pub use mock::{MockError, MockProvider, MockResponse};
