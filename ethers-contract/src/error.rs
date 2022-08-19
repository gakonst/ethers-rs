use ethers_core::{
    abi::{AbiDecode, AbiEncode, Tokenizable},
    types::Selector,
    utils::id,
};
use std::borrow::Cow;

/// A helper trait for types that represents a custom error type
pub trait EthError: Tokenizable + AbiDecode + AbiEncode + Send + Sync {
    /// The name of the error
    fn error_name() -> Cow<'static, str>;

    /// Retrieves the ABI signature for the error
    fn abi_signature() -> Cow<'static, str>;

    /// The selector of the error
    fn selector() -> Selector {
        id(Self::abi_signature())
    }
}
