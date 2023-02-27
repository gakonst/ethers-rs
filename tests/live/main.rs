#![cfg(not(target_arch = "wasm32"))]

#[cfg(feature = "celo")]
mod celo;

pub(crate) mod simple_storage {
    ethers::contract::abigen!(SimpleStorage, "./tests/testdata/SimpleStorage.json");
}
