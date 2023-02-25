#![allow(clippy::extra_unused_type_parameters)]
#![cfg(not(target_arch = "wasm32"))]

use ethers_core::utils::{Anvil, AnvilInstance};
use ethers_providers::{Http, Provider, Ws};
use ethers_signers::{LocalWallet, Signer};
use std::time::Duration;

mod builder;

mod gas_escalator;

mod gas_oracle;

#[cfg(not(feature = "celo"))]
mod signer;

#[cfg(not(feature = "celo"))]
mod nonce_manager;

#[cfg(not(feature = "celo"))]
mod stack;

#[cfg(not(feature = "celo"))]
mod transformer;

/// Spawns Anvil and instantiates an HTTP provider.
pub fn spawn_anvil() -> (Provider<Http>, AnvilInstance) {
    let anvil = Anvil::new().spawn();
    let provider = Provider::<Http>::try_from(anvil.endpoint())
        .unwrap()
        .interval(Duration::from_millis(10u64));
    (provider, anvil)
}

/// Spawns Anvil and instantiates a WS provider.
pub async fn spawn_anvil_ws() -> (Provider<Ws>, AnvilInstance) {
    let anvil = Anvil::new().spawn();
    let provider = Provider::<Ws>::connect(anvil.ws_endpoint())
        .await
        .unwrap()
        .interval(Duration::from_millis(10u64));
    (provider, anvil)
}

/// Gets `idx` wallet from the given anvil instance.
pub fn get_wallet(anvil: &AnvilInstance, idx: usize) -> LocalWallet {
    LocalWallet::from(anvil.keys()[idx].clone()).with_chain_id(anvil.chain_id())
}
