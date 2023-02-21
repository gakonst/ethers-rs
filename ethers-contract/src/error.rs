use ethers_core::{
    abi::{AbiDecode, AbiEncode, Tokenizable},
    types::Selector,
    utils::id,
};
use ethers_providers::JsonRpcError;
use std::borrow::Cow;

/// A helper trait for types that represents a custom error type
pub trait EthError: Tokenizable + AbiDecode + AbiEncode + Send + Sync {
    fn from_rpc_response(response: &JsonRpcError) -> Option<Self>
    where
        Self: Sized,
    {
        response.as_revert_data().and_then(|data| Self::decode(data).ok())
    }

    /// The name of the error
    fn error_name() -> Cow<'static, str>;

    /// Retrieves the ABI signature for the error
    fn abi_signature() -> Cow<'static, str>;

    /// The selector of the error
    fn selector() -> Selector {
        id(Self::abi_signature())
    }
}

impl EthError for String {
    fn error_name() -> Cow<'static, str> {
        Cow::Borrowed("Error")
    }

    fn abi_signature() -> Cow<'static, str> {
        Cow::Borrowed("Error(string)")
    }
}
