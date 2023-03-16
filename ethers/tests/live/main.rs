//! Ethers live tests.
//!
//! If a feature or external binary is added, like Solc, please also update
//! `.github/workflows/ci.yml` at `job.live-test`.

#![cfg(not(target_arch = "wasm32"))]

#[cfg(feature = "celo")]
mod celo;

pub(crate) mod simple_storage {
    ethers::contract::abigen!(SimpleStorage, "../testdata/SimpleStorage.json");
}
