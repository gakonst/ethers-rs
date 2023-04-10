//! Code used by both `call` and `error`

use crate::{abi_ty, utils};
use ethers_core::{
    abi::Param,
    macros::{ethers_contract_crate, ethers_core_crate},
};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{DeriveInput, LitStr, Result};

/// All the attributes the `EthCall`/`EthError` macro supports
#[derive(Default)]
pub struct EthCalllikeAttributes {
    pub name: Option<LitStr>,
    pub abi: Option<LitStr>,
}

impl EthCalllikeAttributes {
    pub fn name(&self, fallback: &Ident) -> String {
        self.name.as_ref().map(|s| s.value()).unwrap_or_else(|| fallback.to_string())
    }

    pub fn abi(&self) -> Option<(String, Span)> {
        self.abi.as_ref().map(|s| (s.value(), s.span()))
    }
}

macro_rules! parse_calllike_attributes {
    ($input:ident, $attr_ident:literal) => {{
        let mut result = EthCalllikeAttributes::default();
        $crate::utils::parse_attributes!($input.attrs.iter(), $attr_ident, meta,
            "name", result.name => {
                meta.input.parse::<::syn::Token![=]>()?;
                let litstr: ::syn::LitStr = meta.input.parse()?;
                result.name = Some(litstr);
            }
            "abi", result.abi => {
                meta.input.parse::<::syn::Token![=]>()?;
                let litstr: ::syn::LitStr = meta.input.parse()?;
                result.abi = Some(litstr);
            }
        );
        result
    }};
}
pub(crate) use parse_calllike_attributes;

/// Generates the decode implementation based on the type's runtime `AbiType` impl
pub fn derive_decode_impl_with_abi_type(
    input: &DeriveInput,
    trait_ident: Ident,
) -> Result<TokenStream> {
    let datatypes_array = utils::abi_parameters_array(input, &trait_ident.to_string())?;
    Ok(derive_decode_impl(datatypes_array, trait_ident))
}

/// Generates the decode implementation based on the params
pub fn derive_decode_impl_from_params(params: &[Param], trait_ident: Ident) -> TokenStream {
    let datatypes = params.iter().map(|input| utils::param_type_quote(&input.kind));
    let datatypes_array = quote! {[#( #datatypes ),*]};
    derive_decode_impl(datatypes_array, trait_ident)
}

pub fn derive_decode_impl(datatypes_array: TokenStream, trait_ident: Ident) -> TokenStream {
    let ethers_core = ethers_core_crate();
    let ethers_contract = ethers_contract_crate();
    let data_types_init = quote! {let data_types = #datatypes_array;};

    quote! {
        let bytes = bytes.as_ref();
        if bytes.len() < 4 || bytes[..4] != <Self as #ethers_contract::#trait_ident>::selector() {
            return Err(#ethers_contract::AbiError::WrongSelector);
        }
        #data_types_init
        let data_tokens = #ethers_core::abi::decode(&data_types, &bytes[4..])?;
        Ok(<Self as #ethers_core::abi::Tokenizable>::from_token(#ethers_core::abi::Token::Tuple(data_tokens))?)
    }
}

/// Generates the Codec implementation
pub fn derive_codec_impls(
    input: &DeriveInput,
    decode_impl: TokenStream,
    trait_ident: Ident,
) -> Result<TokenStream> {
    // the ethers crates to use
    let ethers_core = ethers_core_crate();
    let ethers_contract = ethers_contract_crate();
    let struct_name = &input.ident;

    let codec_impl = quote! {
        impl #ethers_core::abi::AbiDecode for #struct_name {
            fn decode(bytes: impl AsRef<[u8]>) -> ::core::result::Result<Self, #ethers_core::abi::AbiError> {
                #decode_impl
            }
        }

        impl #ethers_core::abi::AbiEncode for #struct_name {
            fn encode(self) -> ::std::vec::Vec<u8> {
                let tokens =  #ethers_core::abi::Tokenize::into_tokens(self);
                let selector = <Self as #ethers_contract::#trait_ident>::selector();
                let encoded = #ethers_core::abi::encode(&tokens);
                selector
                    .iter()
                    .copied()
                    .chain(encoded.into_iter())
                    .collect()
            }
        }

    };
    let tokenize_impl = abi_ty::derive_tokenizeable_impl(input)?;

    Ok(quote! {
        #tokenize_impl
        #codec_impl
    })
}
