#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]
#![deny(unsafe_code)]

mod contract;
pub use contract::Contract;

mod base;
pub use base::{decode_function_data, encode_function_data, AbiError, BaseContract};

mod call;
pub use call::{ContractError, EthCall};

mod error;
pub use error::EthError;

mod factory;
pub use factory::{ContractDeployer, ContractFactory};

mod event;
pub use event::{EthEvent, Event};

mod log;
pub use log::{decode_logs, EthLogDecode, LogMeta};

pub mod stream;

#[cfg(any(test, feature = "abigen"))]
#[cfg_attr(docsrs, doc(cfg(feature = "abigen")))]
mod multicall;
#[cfg(any(test, feature = "abigen"))]
#[cfg_attr(docsrs, doc(cfg(feature = "abigen")))]
pub use multicall::{
    multicall_contract, Call, Multicall, MulticallContract, MulticallError, MulticallVersion,
    MULTICALL_ADDRESS, MULTICALL_SUPPORTED_CHAIN_IDS,
};

/// This module exposes low lever builder structures which are only consumed by the
/// type-safe ABI bindings generators.
#[doc(hidden)]
pub mod builders {
    pub use super::{
        call::ContractCall,
        event::Event,
        factory::{ContractDeployer, Deployer},
    };
}

#[cfg(any(test, feature = "abigen"))]
#[cfg_attr(docsrs, doc(cfg(feature = "abigen")))]
pub use ethers_contract_abigen::{
    Abigen, ContractFilter, ExcludeContracts, InternalStructs, MultiAbigen, SelectContracts,
};

#[cfg(any(test, feature = "abigen"))]
#[cfg_attr(docsrs, doc(cfg(feature = "abigen")))]
pub use ethers_contract_derive::{
    abigen, EthAbiCodec, EthAbiType, EthCall, EthDisplay, EthError, EthEvent,
};

// Hide the Lazy re-export, it's just for convenience
#[doc(hidden)]
pub use once_cell::sync::Lazy;

#[cfg(feature = "eip712")]
pub use ethers_derive_eip712::*;
