#![cfg_attr(docsrs, feature(doc_cfg))]
//! Type-safe abstractions for interacting with Ethereum smart contracts
//!
//! Interacting with a smart contract requires broadcasting carefully crafted
//! [transactions](ethers_core::types::TransactionRequest) where the `data` field contains
//! the [function's
//! selector](https://ethereum.stackexchange.com/questions/72363/what-is-a-function-selector)
//! along with the arguments of the called function. This module provides the
//! [`Contract`] and [`ContractFactory`] abstractions so that you do not have to worry about that.
//! It also provides typesafe bindings via the [`abigen`] macro and the [`Abigen` builder].
//!
//! [`ContractFactory`]: crate::ContractFactory
//! [`Contract`]: crate::Contract
//! [`abigen`]: ./macro.abigen.html
//! [`Abigen` builder]: crate::Abigen
mod contract;
pub use contract::Contract;

mod base;
pub use base::{decode_function_data, encode_function_data, AbiError, BaseContract};

mod call;
pub use call::ContractError;

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
    pub use super::call::ContractCall;
    pub use super::event::Event;
    pub use super::factory::Deployer;
}

#[cfg(feature = "abigen")]
#[cfg_attr(docsrs, doc(cfg(feature = "abigen")))]
pub use ethers_contract_abigen::Abigen;

#[cfg(feature = "abigen")]
#[cfg_attr(docsrs, doc(cfg(feature = "abigen")))]
pub use ethers_contract_derive::{abigen, EthAbiType, EthEvent};

// Hide the Lazy re-export, it's just for convenience
#[doc(hidden)]
pub use once_cell::sync::Lazy;
