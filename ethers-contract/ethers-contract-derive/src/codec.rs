//! Helper functions for deriving `EthAbiType`

use ethers_core::macros::ethers_core_crate;

use quote::quote;
use syn::DeriveInput;

/// Generates the `AbiEncode` + `AbiDecode` implementation
pub fn derive_codec_impl(input: &DeriveInput) -> proc_macro2::TokenStream {
    let name = &input.ident;
    let core_crate = ethers_core_crate();

    quote! {
        impl  #core_crate::abi::AbiDecode for #name {
            fn decode(bytes: impl AsRef<[u8]>) -> ::std::result::Result<Self, #core_crate::abi::AbiError> {
                if let #core_crate::abi::ParamType::Tuple(params) = <Self as #core_crate::abi::AbiType>::param_type() {
                  let tokens = #core_crate::abi::decode(&params, bytes.as_ref())?;
                  Ok(<Self as #core_crate::abi::Tokenizable>::from_token(#core_crate::abi::Token::Tuple(tokens))?)
                } else {
                    Err(
                        #core_crate::abi::InvalidOutputType("Expected tuple".to_string()).into()
                    )
                }
            }
        }
        impl #core_crate::abi::AbiEncode for #name {
            fn encode(self) -> ::std::vec::Vec<u8> {
                let tokens =  #core_crate::abi::Tokenize::into_tokens(self);
                #core_crate::abi::encode(&tokens)
            }
        }
    }
}
