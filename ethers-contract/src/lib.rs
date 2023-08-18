#![doc = include_str!("../README.md")]
#![deny(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

macro_rules! if_providers {
    ($($item:item)*) => {$(
        #[cfg(feature = "providers")]
        #[cfg_attr(docsrs, doc(cfg(feature = "providers")))]
        $item
    )*}
}

mod base;
pub use base::{decode_function_data, encode_function_data, AbiError, BaseContract};

mod call_core;
pub use call_core::EthCall;

mod error;
pub use error::{ContractRevert, EthError};

mod event_core;
pub use event_core::{parse_log, EthEvent};

mod log;
pub use log::{decode_logs, EthLogDecode, LogMeta};

pub mod stream;

#[cfg(feature = "abigen")]
#[cfg_attr(docsrs, doc(cfg(feature = "abigen")))]
mod multicall;
#[cfg(feature = "abigen")]
#[cfg_attr(docsrs, doc(cfg(feature = "abigen")))]
pub use multicall::{
    constants::{MULTICALL_ADDRESS, MULTICALL_SUPPORTED_CHAIN_IDS},
    contract as multicall_contract, MulticallVersion,
};

#[cfg(feature = "abigen")]
#[cfg_attr(docsrs, doc(cfg(feature = "abigen")))]
pub use ethers_contract_abigen::{
    Abigen, ContractFilter, ExcludeContracts, InternalStructs, MultiAbigen, SelectContracts,
};

#[cfg(feature = "abigen")]
#[cfg_attr(docsrs, doc(cfg(feature = "abigen")))]
pub use ethers_contract_derive::{
    abigen, Eip712, EthAbiCodec, EthAbiType, EthCall, EthDisplay, EthError, EthEvent,
};

// Hide the Lazy re-export, it's just for convenience
#[doc(hidden)]
pub use once_cell::sync::Lazy;

// For macro expansions only, not public API.
// See: [#2235](https://github.com/gakonst/ethers-rs/pull/2235)

#[doc(hidden)]
#[allow(unused_extern_crates)]
extern crate self as ethers_contract;

#[doc(hidden)]
#[allow(unused_extern_crates)]
extern crate self as ethers;

#[doc(hidden)]
pub mod contract {
    pub use crate::*;
}

#[doc(hidden)]
pub use ethers_core as core;

if_providers! {
    mod event;
    pub use event::Event;

    #[path = "contract.rs"]
    mod _contract;
    pub use _contract::{Contract, ContractInstance};

    mod call;
    pub use call::{ContractCall, ContractError, FunctionCall};

    mod factory;
    pub use factory::{ContractDeployer, ContractDeploymentTx, ContractFactory, DeploymentTxFactory};

    #[cfg(all(feature = "abigen"))]
    #[cfg_attr(docsrs, doc(cfg(feature = "abigen")))]
    pub use multicall::{error::MulticallError, Call, Multicall, MulticallContract};

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

    #[doc(hidden)]
    pub use ethers_providers as providers;
}
