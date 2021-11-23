#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]
mod contract;
pub use contract::Contract;

mod base;
pub use base::{decode_function_data, encode_function_data, AbiError, BaseContract};

mod call;
pub use call::{ContractError, EthCall};

mod factory;
pub use factory::ContractFactory;

mod event;
pub use event::EthEvent;

mod log;
pub use log::{decode_logs, EthLogDecode, LogMeta};

mod stream;

mod multicall;
pub use multicall::Multicall;

/// This module exposes low lever builder structures which are only consumed by the
/// type-safe ABI bindings generators.
pub mod builders {
    pub use super::{call::ContractCall, event::Event, factory::Deployer};
}

#[cfg(any(test, feature = "abigen"))]
#[cfg_attr(docsrs, doc(cfg(feature = "abigen")))]
pub use ethers_contract_abigen::Abigen;

#[cfg(any(test, feature = "abigen"))]
#[cfg_attr(docsrs, doc(cfg(feature = "abigen")))]
pub use ethers_contract_derive::{abigen, EthAbiType, EthCall, EthDisplay, EthEvent};

// Hide the Lazy re-export, it's just for convenience
#[doc(hidden)]
pub use once_cell::sync::Lazy;

#[cfg(feature = "eip712")]
pub use ethers_derive_eip712::*;
