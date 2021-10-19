//! Support for compiling contracts

pub mod artifacts;
pub use artifacts::{CompilerInput, CompilerOutput, EvmVersion};
pub mod cache;
mod compile;
pub use compile::Solc;
