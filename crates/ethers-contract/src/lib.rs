mod contract;
pub use contract::Contract;

mod event;
pub use event::Event;

mod call;
pub use call::{ContractCall, ContractError};

#[cfg(feature = "abigen")]
pub use ethers_contract_abigen::Builder;

#[cfg(feature = "abigen")]
pub use ethers_contract_derive::abigen;

// re-export for convenience
pub use ethers_abi as abi;
pub use ethers_providers as providers;
pub use ethers_signers as signers;
pub use ethers_types as types;
pub use once_cell::sync::Lazy;
