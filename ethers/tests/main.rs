//! Ethers integration tests.
#![cfg(not(target_arch = "wasm32"))]

#[cfg(feature = "celo")]
mod celo;

mod eip712;
