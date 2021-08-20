mod common;

#[cfg(not(target_arch = "wasm32"))]
mod http;
#[cfg(not(target_arch = "wasm32"))]
pub use http::Provider as Http;

#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "ws")]
mod ws;
#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "ws")]
pub use ws::Ws;

#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "ipc")]
mod ipc;
#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "ipc")]
pub use ipc::Ipc;

mod mock;
pub use mock::{MockError, MockProvider};
