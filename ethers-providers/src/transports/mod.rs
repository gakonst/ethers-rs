mod common;
pub use common::Authorization;

// only used with WS
#[cfg(feature = "ws")]
macro_rules! if_wasm {
    ($($item:item)*) => {$(
        #[cfg(target_arch = "wasm32")]
        $item
    )*}
}

// only used with WS
#[cfg(feature = "ws")]
macro_rules! if_not_wasm {
    ($($item:item)*) => {$(
        #[cfg(not(target_arch = "wasm32"))]
        $item
    )*}
}

#[cfg(all(target_family = "unix", feature = "ipc"))]
mod ipc;
#[cfg(all(target_family = "unix", feature = "ipc"))]
pub use ipc::{Ipc, IpcError};

mod http;
pub use self::http::{ClientError as HttpClientError, Provider as Http};

#[cfg(feature = "ws")]
mod ws;
#[cfg(feature = "ws")]
pub use ws::{ClientError as WsClientError, Ws};

mod quorum;
pub(crate) use quorum::JsonRpcClientWrapper;
pub use quorum::{Quorum, QuorumError, QuorumProvider, WeightedProvider};

mod rw;
pub use rw::{RwClient, RwClientError};

mod mock;
pub use mock::{MockError, MockProvider};
