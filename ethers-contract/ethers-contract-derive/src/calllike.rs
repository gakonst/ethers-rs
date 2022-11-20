//! Code used by both `call` and `error`

use crate::{abi_ty, utils};
use ethers_core::{
    abi::Param,
    macros::{ethers_contract_crate, ethers_core_crate},
};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{parse::Error, spanned::Spanned as _, AttrStyle, DeriveInput, Lit, Meta, NestedMeta};

/// All the attributes the `EthCall`/`EthError` macro supports
#[derive(Default)]
pub struct EthCalllikeAttributes {
    pub name: Option<(String, Span)>,
    pub abi: Option<(String, Span)>,
}

/// extracts the attributes from the struct annotated with the given attribute
pub fn parse_calllike_attributes(
    input: &DeriveInput,
    attr_name: &str,
) -> Result<EthCalllikeAttributes, TokenStream> {
    let mut result = EthCalllikeAttributes::default();
    for a in input.attrs.iter() {
        if let AttrStyle::Outer = a.style {
            if let Ok(Meta::List(meta)) = a.parse_meta() {
                if meta.path.is_ident(attr_name) {
                    for n in meta.nested.iter() {
                        if let NestedMeta::Meta(meta) = n {
                            match meta {
                                Meta::Path(path) => {
                                    return Err(Error::new(
                                        path.span(),
                                        format!("unrecognized {attr_name} parameter"),
                                    )
                                    .to_compile_error())
                                }
                                Meta::List(meta) => {
                                    return Err(Error::new(
                                        meta.path.span(),
                                        format!("unrecognized {attr_name} parameter"),
                                    )
                                    .to_compile_error())
                                }
                                Meta::NameValue(meta) => {
                                    if meta.path.is_ident("name") {
                                        if let Lit::Str(ref lit_str) = meta.lit {
                                            if result.name.is_none() {
                                                result.name =
                                                    Some((lit_str.value(), lit_str.span()));
                                            } else {
                                                return Err(Error::new(
                                                    meta.span(),
                                                    "name already specified",
                                                )
                                                .to_compile_error())
                                            }
                                        } else {
                                            return Err(Error::new(
                                                meta.span(),
                                                "name must be a string",
                                            )
                                            .to_compile_error())
                                        }
                                    } else if meta.path.is_ident("abi") {
                                        if let Lit::Str(ref lit_str) = meta.lit {
                                            if result.abi.is_none() {
                                                result.abi =
                                                    Some((lit_str.value(), lit_str.span()));
                                            } else {
                                                return Err(Error::new(
                                                    meta.span(),
                                                    "abi already specified",
                                                )
                                                .to_compile_error())
                                            }
                                        } else {
                                            return Err(Error::new(
                                                meta.span(),
                                                "abi must be a string",
                                            )
                                            .to_compile_error())
                                        }
                                    } else {
                                        return Err(Error::new(
                                            meta.span(),
                                            format!("unrecognized {attr_name} parameter"),
                                        )
                                        .to_compile_error())
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(result)
}

/// Generates the decode implementation based on the type's runtime `AbiType` impl
pub fn derive_decode_impl_with_abi_type(
    input: &DeriveInput,
    trait_ident: Ident,
) -> Result<TokenStream, Error> {
    let datatypes_array = utils::derive_abi_parameters_array(input, &trait_ident.to_string())?;
    Ok(derive_decode_impl(datatypes_array, trait_ident))
}

/// Generates the decode implementation based on the params
pub fn derive_decode_impl_from_params(params: &[Param], trait_ident: Ident) -> TokenStream {
    let datatypes = params.iter().map(|input| utils::param_type_quote(&input.kind));
    let datatypes_array = quote! {[#( #datatypes ),*]};
    derive_decode_impl(datatypes_array, trait_ident)
}

pub fn derive_decode_impl(datatypes_array: TokenStream, trait_ident: Ident) -> TokenStream {
    let core_crate = ethers_core_crate();
    let contract_crate = ethers_contract_crate();
    let data_types_init = quote! {let data_types = #datatypes_array;};

    quote! {
        let bytes = bytes.as_ref();
        if bytes.len() < 4 || bytes[..4] != <Self as #contract_crate::#trait_ident>::selector() {
            return Err(#contract_crate::AbiError::WrongSelector);
        }
        #data_types_init
        let data_tokens = #core_crate::abi::decode(&data_types, &bytes[4..])?;
        Ok(<Self as #core_crate::abi::Tokenizable>::from_token( #core_crate::abi::Token::Tuple(data_tokens))?)
    }
}

/// Generates the Codec implementation
pub fn derive_codec_impls(
    input: &DeriveInput,
    decode_impl: TokenStream,
    trait_ident: Ident,
) -> TokenStream {
    // the ethers crates to use
    let core_crate = ethers_core_crate();
    let contract_crate = ethers_contract_crate();
    let struct_name = &input.ident;

    let codec_impl = quote! {

        impl #core_crate::abi::AbiDecode for #struct_name {
            fn decode(bytes: impl AsRef<[u8]>) -> ::std::result::Result<Self, #core_crate::abi::AbiError> {
                #decode_impl
            }
        }

        impl #core_crate::abi::AbiEncode for #struct_name {
            fn encode(self) -> ::std::vec::Vec<u8> {
                let tokens =  #core_crate::abi::Tokenize::into_tokens(self);
                let selector = <Self as #contract_crate::#trait_ident>::selector();
                let encoded = #core_crate::abi::encode(&tokens);
                selector
                    .iter()
                    .copied()
                    .chain(encoded.into_iter())
                    .collect()
            }
        }

    };
    let tokenize_impl = abi_ty::derive_tokenizeable_impl(input);

    quote! {
        #tokenize_impl
        #codec_impl
    }
}
