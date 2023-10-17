use ethers_core::{
    abi::{AbiDecode, AbiEncode, Tokenizable},
    types::Selector,
    utils::id,
};
#[cfg(feature = "providers")]
use ethers_providers::JsonRpcError;
use std::borrow::Cow;

/// A trait for enums unifying [`EthError`] types. This trait is usually used
/// to represent the errors that a specific contract might throw. I.e. all
/// solidity custom errors + revert strings.
///
/// This trait should be accessed via
/// [`crate::ContractError::decode_contract_revert`]. It is generally
/// unnecessary to import this trait into your code.
///
/// # Implementor's Note
///
/// We do not recommend manual implementations of this trait. Instead, use the
/// automatically generated implementation in the [`crate::abigen`] macro
///
/// However, sophisticated users may wish to represent the errors of multiple
/// contracts as a single unified enum. E.g. if your contract calls Uniswap,
/// you may wish to implement this on `pub enum MyContractOrUniswapErrors`.
/// In that case, it should be straightforward to delegate to the inner types.
pub trait ContractRevert: AbiDecode + AbiEncode + Send + Sync {
    /// Decode the error from EVM revert data including an Error selector
    fn decode_with_selector(data: &[u8]) -> Option<Self> {
        if data.len() < 4 {
            return None
        }
        let selector = data[..4].try_into().expect("checked by len");
        if !Self::valid_selector(selector) {
            return None
        }

        if selector == String::selector() {
            <Self as AbiDecode>::decode(&data[4..]).ok()
        } else {
            // Try with and without a prefix, just in case.
            <Self as AbiDecode>::decode(data)
                .or_else(|_| <Self as AbiDecode>::decode(&data[4..]))
                .ok()
        }
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
    #[cfg(feature = "providers")]
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

    fn selector() -> Selector {
        [0x08, 0xc3, 0x79, 0xa0]
    }
}

#[cfg(all(test, feature = "abigen"))]
mod test {

    use ethers_core::types::Bytes;

    use crate::ContractRevert;

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

    #[test]
    fn custom_error() {
        use error::*;
        // Example of binary data returned by the contract after reverting with `revert
        // EmptyAddress();`.
        let example_revert: Bytes = "0x7138356f".parse().unwrap();

        let selector: [u8; 4] = example_revert.to_vec().try_into().unwrap();
        assert!(ExampleContractErrors::valid_selector(selector), "selector is valid");

        let e = ExampleContractErrors::decode_with_selector(&example_revert)
            .expect("failed to decode revert");

        assert_eq!(e, ExampleContractErrors::EmptyAddress(EmptyAddress));
    }

    #[test]
    fn custom_error_string() {
        use error::*;
        // Example of binary data returned by the contract after reverting with `revert("Multicall3:
        // call failed");`.
        let example_revert: Bytes = "0x08c379a0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000174d756c746963616c6c333a2063616c6c206661696c6564000000000000000000".parse().unwrap();

        let e = ExampleContractErrors::decode_with_selector(&example_revert)
            .expect("failed to decode revert string");

        assert_eq!(e, ExampleContractErrors::RevertString("Multicall3: call failed".into()));
    }

    /// This is an excerpt from the code generated by `Abigen` for custom errors used by a contract.
    mod error {
        ///Custom Error type `EmptyAddress` with signature `EmptyAddress()` and selector
        /// `0x7138356f`
        #[derive(
            Clone, crate::EthError, crate::EthDisplay, Default, Debug, PartialEq, Eq, Hash,
        )]
        #[etherror(name = "EmptyAddress", abi = "EmptyAddress()")]
        pub struct EmptyAddress;

        ///Custom Error type `CollateralIsZero` with signature `CollateralIsZero()` and selector
        /// `0xb4f18b02`
        #[derive(
            Clone, crate::EthError, crate::EthDisplay, Default, Debug, PartialEq, Eq, Hash,
        )]
        #[etherror(name = "CollateralIsZero", abi = "CollateralIsZero()")]
        pub struct CollateralIsZero;

        ///Container type for all of the contract's custom errors
        #[derive(Clone, crate::EthAbiType, Debug, PartialEq, Eq, Hash)]
        pub enum ExampleContractErrors {
            EmptyAddress(EmptyAddress),
            CollateralIsZero(CollateralIsZero),
            /// The standard solidity revert string, with selector
            /// Error(string) -- 0x08c379a0
            RevertString(::std::string::String),
        }
        impl ::ethers::core::abi::AbiDecode for ExampleContractErrors {
            fn decode(
                data: impl AsRef<[u8]>,
            ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
                let data = data.as_ref();
                if let Ok(decoded) =
                    <::std::string::String as ::ethers::core::abi::AbiDecode>::decode(data)
                {
                    return Ok(Self::RevertString(decoded))
                }
                if let Ok(decoded) = <EmptyAddress as ::ethers::core::abi::AbiDecode>::decode(data)
                {
                    return Ok(Self::EmptyAddress(decoded))
                }
                if let Ok(decoded) =
                    <CollateralIsZero as ::ethers::core::abi::AbiDecode>::decode(data)
                {
                    return Ok(Self::CollateralIsZero(decoded))
                }
                Err(::ethers::core::abi::Error::InvalidData.into())
            }
        }
        impl ::ethers::core::abi::AbiEncode for ExampleContractErrors {
            fn encode(self) -> ::std::vec::Vec<u8> {
                unimplemented!()
            }
        }
        impl crate::ContractRevert for ExampleContractErrors {
            fn valid_selector(selector: [u8; 4]) -> bool {
                match selector {
                    [0x08, 0xc3, 0x79, 0xa0] => true,
                    _ if selector == <EmptyAddress as crate::EthError>::selector() => true,
                    _ if selector == <CollateralIsZero as crate::EthError>::selector() => true,
                    _ => false,
                }
            }
        }
    }
}
