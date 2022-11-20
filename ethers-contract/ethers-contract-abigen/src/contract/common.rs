use super::{util, Context};

use crate::contract::types;
use ethers_core::{
    abi::{Param, ParamType},
    macros::{ethers_contract_crate, ethers_core_crate, ethers_providers_crate},
};
use proc_macro2::{Ident, TokenStream};
use quote::quote;

/// Expands to the `name : type` pairs for the params
pub(crate) fn expand_params<'a, F>(
    params: &[Param],
    resolve_tuple: F,
) -> eyre::Result<Vec<(TokenStream, TokenStream)>>
where
    F: Fn(&str) -> Option<&'a str>,
{
    params
        .iter()
        .enumerate()
        .map(|(idx, param)| {
            let name = util::expand_input_name(idx, &param.name);
            let ty = expand_param_type(param, &param.kind, |s| resolve_tuple(s))?;
            Ok((name, ty))
        })
        .collect()
}

/// returns the Tokenstream for the corresponding rust type
pub(crate) fn expand_param_type<'a, F>(
    param: &Param,
    kind: &ParamType,
    resolve_tuple: F,
) -> eyre::Result<TokenStream>
where
    F: Fn(&str) -> Option<&'a str>,
{
    match kind {
        ParamType::Array(ty) => {
            let ty = expand_param_type(param, ty, resolve_tuple)?;
            Ok(quote! {
                ::std::vec::Vec<#ty>
            })
        }
        ParamType::FixedArray(ty, size) => {
            let ty = expand_param_type(param, ty, resolve_tuple)?;
            let size = *size;
            Ok(quote! {[#ty; #size]})
        }
        ParamType::Tuple(_) => {
            let ty = if let Some(rust_struct_name) =
                param.internal_type.as_ref().and_then(|s| resolve_tuple(s.as_str()))
            {
                let ident = util::ident(rust_struct_name);
                quote! {#ident}
            } else {
                types::expand(kind)?
            };
            Ok(ty)
        }
        _ => types::expand(kind),
    }
}

pub(crate) fn imports(name: &str) -> TokenStream {
    let doc = util::expand_doc(&format!("{name} was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"));

    let ethers_core = ethers_core_crate();
    let ethers_providers = ethers_providers_crate();
    let ethers_contract = ethers_contract_crate();

    quote! {
        #![allow(clippy::enum_variant_names)]
        #![allow(dead_code)]
        #![allow(clippy::type_complexity)]
        #![allow(unused_imports)]
        #doc

        use std::sync::Arc;
        use #ethers_core::{
            abi::{Abi, Token, Detokenize, InvalidOutputType, Tokenizable},
            types::*, // import all the types so that we can codegen for everything
        };
        use #ethers_contract::{Contract, builders::{ContractCall, Event}, Lazy};
        use #ethers_providers::Middleware;
    }
}

/// Generates the static `Abi` constants and the contract struct
pub(crate) fn struct_declaration(cx: &Context) -> TokenStream {
    let name = &cx.contract_ident;
    let abi = &cx.abi_str;

    let abi_name = cx.inline_abi_ident();

    let ethers_core = ethers_core_crate();
    let ethers_contract = ethers_contract_crate();

    let abi_parse = if !cx.human_readable {
        quote! {
            #[rustfmt::skip]
            const __ABI: &str = #abi;

            /// The parsed JSON-ABI of the contract.
            pub static #abi_name: #ethers_contract::Lazy<#ethers_core::abi::Abi> = #ethers_contract::Lazy::new(|| #ethers_core::utils::__serde_json::from_str(__ABI)
                                              .expect("invalid abi"));
        }
    } else {
        quote! {
            /// The parsed human readable ABI of the contract.
            pub static #abi_name: #ethers_contract::Lazy<#ethers_core::abi::Abi> = #ethers_contract::Lazy::new(|| #ethers_core::abi::parse_abi_str(#abi)
                                                .expect("invalid abi"));
        }
    };

    let bytecode = if let Some(ref bytecode) = cx.contract_bytecode {
        let bytecode_name = cx.inline_bytecode_ident();
        let hex_bytecode = format!("{bytecode}");
        quote! {
            /// Bytecode of the #name contract
            pub static #bytecode_name: #ethers_contract::Lazy<#ethers_core::types::Bytes> = #ethers_contract::Lazy::new(|| #hex_bytecode.parse()
                                                .expect("invalid bytecode"));
        }
    } else {
        quote! {}
    };

    quote! {
        // Inline ABI declaration
        #abi_parse

        #bytecode

        // Struct declaration
        pub struct #name<M>(#ethers_contract::Contract<M>);

        impl<M> Clone for #name<M> {
            fn clone(&self) -> Self {
                #name(self.0.clone())
            }
        }

        // Deref to the inner contract in order to access more specific functions functions
        impl<M> std::ops::Deref for #name<M> {
            type Target = #ethers_contract::Contract<M>;

            fn deref(&self) -> &Self::Target { &self.0 }
        }

        impl<M> std::fmt::Debug for #name<M> {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.debug_tuple(stringify!(#name))
                    .field(&self.address())
                    .finish()
            }
        }
    }
}

/// Expands to the tuple struct definition
pub(crate) fn expand_data_tuple(
    name: &Ident,
    params: &[(TokenStream, TokenStream)],
) -> TokenStream {
    let fields = params
        .iter()
        .map(|(_, ty)| {
            quote! {
            pub #ty }
        })
        .collect::<Vec<_>>();

    if fields.is_empty() {
        quote! { struct #name; }
    } else {
        quote! { struct #name( #( #fields ),* ); }
    }
}

/// Expands to a struct definition with named fields
pub(crate) fn expand_data_struct(
    name: &Ident,
    params: &[(TokenStream, TokenStream)],
) -> TokenStream {
    let fields = params
        .iter()
        .map(|(name, ty)| {
            quote! { pub #name: #ty }
        })
        .collect::<Vec<_>>();

    quote! { struct #name { #( #fields, )* } }
}
