pub(crate) mod common;
pub use common::{Authorization, JsonRpcError};

mod http;
pub use self::http::{ClientError as HttpClientError, Provider as Http};

#[cfg(all(feature = "ipc", any(unix, windows)))]
mod ipc;
#[cfg(all(feature = "ipc", any(unix, windows)))]
pub use ipc::{Ipc, IpcError};

#[cfg(feature = "ws")]
mod ws;
#[cfg(feature = "ws")]
pub use ws::{ConnectionDetails, WsClient as Ws, WsClientError};
// pub use ws::{ClientError as WsClientError, Ws};

mod quorum;
pub use quorum::{JsonRpcClientWrapper, Quorum, QuorumError, QuorumProvider, WeightedProvider};

mod rw;
pub use rw::{RwClient, RwClientError};

mod retry;
pub use retry::*;

mod mock;
pub use mock::{MockError, MockProvider};

/// archival websocket
#[cfg(feature = "ws")]
pub mod ws2;
// pub use ws2::WsClient as Ws2Client;
