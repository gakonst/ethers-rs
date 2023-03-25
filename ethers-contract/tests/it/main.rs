#![allow(clippy::extra_unused_type_parameters)]
#![cfg(feature = "abigen")]

mod abigen;

mod derive;

mod contract_call;

mod eip712;

#[cfg(all(not(target_arch = "wasm32"), not(feature = "celo")))]
mod common;

#[cfg(all(not(target_arch = "wasm32"), not(feature = "celo")))]
mod contract;
