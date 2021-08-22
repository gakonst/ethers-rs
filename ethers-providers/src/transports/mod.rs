mod common;

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
    mod http;
    pub use http::Provider as Http;


    #[cfg(feature = "ipc")]
    mod ipc;
    #[cfg(feature = "ipc")]
    pub use ipc::Ipc;
}

#[cfg(feature = "ws")]
mod ws;
#[cfg(feature = "ws")]
pub use ws::Ws;

mod mock;
pub use mock::{MockError, MockProvider};
