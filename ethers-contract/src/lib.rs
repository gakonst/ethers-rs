mod contract;
pub use contract::Contract;

mod call;
pub use call::ContractError;

mod factory;
pub use factory::ContractFactory;

mod event;

/// This module exposes low lever builder structures which are only consumed by the
/// type-safe ABI bindings generators.
pub mod builders {
    pub use super::call::ContractCall;
    pub use super::event::Event;
    pub use super::factory::Deployer;
}

#[cfg(feature = "abigen")]
pub use ethers_contract_abigen::Abigen;

#[cfg(feature = "abigen")]
pub use ethers_contract_derive::abigen;

// Hide the Lazy re-export, it's just for convenience
#[doc(hidden)]
pub use once_cell::sync::Lazy;
