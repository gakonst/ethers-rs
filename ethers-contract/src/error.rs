use ethers_core::{
    abi::{AbiDecode, AbiEncode, Tokenizable},
    types::Selector,
    utils::id,
};
use ethers_providers::JsonRpcError;
use std::borrow::Cow;

/// A trait for enums unifying [`EthError`] types.
///
/// We do not recommend manual implementations of this trait. Instead, use the
/// automatically generated implementation in the [`crate::abigen`] macro
pub trait ContractRevert: AbiDecode + AbiEncode + Send + Sync {
    /// Decode the error from EVM revert data including an Error selector
    fn decode_with_selector(data: &[u8]) -> Option<Self> {
        if data.len() < 4 {
            return None
        }
        let selector = data.try_into().expect("checked by len");
        if !Self::valid_selector(selector) {
            return None
        }
        <Self as AbiDecode>::decode(&data[4..]).ok()
    }

    /// `true` if the selector corresponds to an error that this contract can
    /// revert. False otherwise
    fn valid_selector(selector: Selector) -> bool;
}

/// A helper trait for types that represents a custom error type
pub trait EthError: Tokenizable + AbiDecode + AbiEncode + Send + Sync {
    /// Attempt to decode from a [`JsonRpcError`] by extracting revert data
    ///
    /// Fails if the error is not a revert, or decoding fails
    fn from_rpc_response(response: &JsonRpcError) -> Option<Self> {
        Self::decode_with_selector(&response.as_revert_data()?)
    }

    /// Decode the error from EVM revert data including an Error selector
    fn decode_with_selector(data: &[u8]) -> Option<Self> {
        // This will return none if selector mismatch.
        <Self as AbiDecode>::decode(data.strip_prefix(&Self::selector())?).ok()
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

#[cfg(test)]
mod test {
    use ethers_core::types::Bytes;

    use super::EthError;

    #[test]
    fn string_error() {
        let multicall_revert_string: Bytes = "0x08c379a0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000174d756c746963616c6c333a2063616c6c206661696c6564000000000000000000".parse().unwrap();
        assert_eq!(String::selector().as_slice(), &multicall_revert_string[0..4]);
        assert_eq!(
            String::decode_with_selector(&multicall_revert_string).unwrap().as_str(),
            "Multicall3: call failed"
        );
    }
}
