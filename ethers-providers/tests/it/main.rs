#![cfg(not(target_arch = "wasm32"))]

use ethers_core::utils::{Anvil, AnvilInstance};
use ethers_providers::{Http, Provider};
use std::time::Duration;

#[cfg(feature = "ipc")]
use ethers_providers::Ipc;
#[cfg(feature = "ipc")]
use tempfile::NamedTempFile;

#[cfg(feature = "ws")]
use ethers_providers::Ws;

mod provider;

mod txpool;

#[cfg(all(feature = "ws", not(feature = "legacy-ws"), not(feature = "celo")))]
mod ws_errors;

/// Spawns Anvil and instantiates an Http provider.
pub fn spawn_anvil() -> (Provider<Http>, AnvilInstance) {
    let anvil = Anvil::new().block_time(1u64).spawn();
    let provider = Provider::<Http>::try_from(anvil.endpoint())
        .unwrap()
        .interval(Duration::from_millis(50u64));
    (provider, anvil)
}

/// Spawns Anvil and instantiates a Ws provider.
#[cfg(feature = "ws")]
pub async fn spawn_anvil_ws() -> (Provider<Ws>, AnvilInstance) {
    let anvil = Anvil::new().block_time(1u64).spawn();
    let provider = Provider::<Ws>::connect(anvil.ws_endpoint())
        .await
        .unwrap()
        .interval(Duration::from_millis(50u64));
    (provider, anvil)
}

/// Spawns Anvil and instantiates a Ipc provider.
#[cfg(feature = "ipc")]
pub async fn spawn_anvil_ipc() -> (Provider<Ipc>, AnvilInstance, NamedTempFile) {
    let ipc = NamedTempFile::new().unwrap();
    let anvil =
        Anvil::new().block_time(1u64).arg("--ipc").arg(ipc.path().display().to_string()).spawn();
    let provider = Provider::<Ipc>::connect_ipc(ipc.path())
        .await
        .unwrap()
        .interval(Duration::from_millis(50u64));
    (provider, anvil, ipc)
}
