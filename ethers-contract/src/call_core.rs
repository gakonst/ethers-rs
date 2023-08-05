use ethers_core::{
    abi::{AbiDecode, AbiEncode, Tokenizable},
    types::Selector,
    utils::id,
};
use std::borrow::Cow;

/// A helper trait for types that represent all call input parameters of a specific function
pub trait EthCall: Tokenizable + AbiDecode + AbiEncode + Send + Sync {
    /// The name of the function
    fn function_name() -> Cow<'static, str>;

    /// Retrieves the ABI signature for the call
    fn abi_signature() -> Cow<'static, str>;

    /// The selector of the function
    fn selector() -> Selector {
        id(Self::abi_signature())
    }
}
