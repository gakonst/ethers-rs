//! Helper functions for deriving `EthAbiType`

use ethers_core::macros::ethers_core_crate;
use quote::quote;
use syn::DeriveInput;

/// Generates the `AbiEncode` + `AbiDecode` implementation
pub fn derive_codec_impl(input: &DeriveInput) -> proc_macro2::TokenStream {
    let name = &input.ident;
    let ethers_core = ethers_core_crate();

    quote! {
        impl #ethers_core::abi::AbiDecode for #name {
            fn decode(bytes: impl AsRef<[u8]>) -> ::core::result::Result<Self, #ethers_core::abi::AbiError> {
                fn _decode(bytes: &[u8]) -> ::core::result::Result<#name, #ethers_core::abi::AbiError> {
                    let #ethers_core::abi::ParamType::Tuple(params) =
                        <#name as #ethers_core::abi::AbiType>::param_type() else { unreachable!() };
                    let min_len: usize = params.iter().map(#ethers_core::abi::minimum_size).sum();
                    if bytes.len() < min_len {
                        Err(#ethers_core::abi::AbiError::DecodingError(#ethers_core::abi::ethabi::Error::InvalidData))
                    } else {
                        let tokens = #ethers_core::abi::decode(&params, bytes)?;
                        let tuple = #ethers_core::abi::Token::Tuple(tokens);
                        let this = <#name as #ethers_core::abi::Tokenizable>::from_token(tuple)?;
                        Ok(this)
                    }
                }

                _decode(bytes.as_ref())
            }
        }

        impl #ethers_core::abi::AbiEncode for #name {
            fn encode(self) -> ::std::vec::Vec<u8> {
                let tokens = #ethers_core::abi::Tokenize::into_tokens(self);
                #ethers_core::abi::encode(&tokens)
            }
        }
    }
}
