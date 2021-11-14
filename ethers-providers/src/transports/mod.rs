mod common;

// only used with WS
#[cfg(feature = "ws")]
macro_rules! if_wasm {
    ($($item:item)*) => {$(
        #[cfg(target_arch = "wasm32")]
        $item
    )*}
}

macro_rules! if_not_wasm {
    ($($item:item)*) => {$(
        #[cfg(not(target_arch = "wasm32"))]
        $item
    )*}
}

if_not_wasm! {
    #[cfg(feature = "ipc")]
    mod ipc;
    #[cfg(feature = "ipc")]
    pub use ipc::Ipc;
}

mod http;
pub use http::{ClientError as HttpClientError, Provider as Http};

#[cfg(feature = "ws")]
mod ws;
#[cfg(feature = "ws")]
pub use ws::{ClientError as WsClientError, Ws};

mod quorum;
pub(crate) use quorum::JsonRpcClientWrapper;
pub use quorum::{Quorum, QuorumProvider, WeightedProvider};

mod mock;
pub use mock::{MockError, MockProvider};
