/// Utilities for launching a ganache-cli testnet instance
mod ganache;
pub use ganache::Ganache;

/// Solidity compiler bindings
mod solc;
pub use solc::{CompiledContract, Solc};

mod hash;
pub use hash::{hash_message, id, keccak256, serialize};
