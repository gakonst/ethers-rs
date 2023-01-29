use super::{util, Context};

use crate::contract::types;
use ethers_core::{
    abi::{Param, ParamType},
    macros::{ethers_contract_crate, ethers_core_crate, ethers_providers_crate},
};
use proc_macro2::{Ident, TokenStream};
use quote::quote;

/// Expands to the `name, type` pairs for the params
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
    let doc_str = format!("{name} was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs");

    let ethers_core = ethers_core_crate();
    let ethers_providers = ethers_providers_crate();
    let ethers_contract = ethers_contract_crate();

    quote! {
        #![allow(clippy::enum_variant_names)]
        #![allow(dead_code)]
        #![allow(clippy::type_complexity)]
        #![allow(unused_imports)]
        #![doc = #doc_str]

        use std::sync::Arc;
        use #ethers_core::{
            abi::{Abi, Token, Detokenize, InvalidOutputType, Tokenizable},
            types::*, // import all the types so that we can codegen for everything
        };
        use #ethers_contract::{Contract, builders::{ContractCall, Event}, Lazy};
        use #ethers_providers::Middleware;
    }
}

/// Generates the token stream for the contract's ABI, bytecode and struct declarations.
pub(crate) fn struct_declaration(cx: &Context) -> TokenStream {
    let name = &cx.contract_ident;

    let ethers_core = ethers_core_crate();
    let ethers_contract = ethers_contract_crate();

    let abi = {
        let abi_name = cx.inline_abi_ident();
        let abi = &cx.abi_str;
        let (doc_str, parse) = if cx.human_readable {
            // Human readable: use abi::parse_abi_str
            let doc_str = "The parsed human-readable ABI of the contract.";
            let parse = quote!(#ethers_core::abi::parse_abi_str(__ABI));
            (doc_str, parse)
        } else {
            // JSON ABI: use serde_json::from_str
            let doc_str = "The parsed JSON ABI of the contract.";
            let parse = quote!(#ethers_core::utils::__serde_json::from_str(__ABI));
            (doc_str, parse)
        };

        quote! {
            #[rustfmt::skip]
            const __ABI: &str = #abi;

            // This never fails as we are parsing the ABI in this macro
            #[doc = #doc_str]
            pub static #abi_name: #ethers_contract::Lazy<#ethers_core::abi::Abi> =
                #ethers_contract::Lazy::new(|| #parse.expect("ABI is always valid"));
        }
    };

    let bytecode = cx.contract_bytecode.as_ref().map(|bytecode| {
        // TODO: use static &[u8], which eliminates the need for a Lazy by using const fn
        // Bytes::from_static(). Also uses less memory.
        let bytecode = bytecode.to_string();
        let bytecode_name = cx.inline_bytecode_ident();
        quote! {
            const __BYTECODE: &str = #bytecode;

            #[doc = "The bytecode of the contract."]
            pub static #bytecode_name: #ethers_contract::Lazy<#ethers_core::types::Bytes> =
                #ethers_contract::Lazy::new(|| __BYTECODE.parse().expect("Invalid bytecode"));
        }
    });

    quote! {
        // Lazy ABI
        #abi

        // Lazy Bytecode, if present
        #bytecode

        // Struct declaration
        pub struct #name<M>(#ethers_contract::Contract<M>);

        // Manual implementation since `M` is stored in `Arc<M>` and does not need to be `Clone`
        impl<M> ::core::clone::Clone for #name<M> {
            fn clone(&self) -> Self {
                Self(::core::clone::Clone::clone(&self.0))
            }
        }

        // Deref to the inner contract to have access to all its methods
        impl<M> ::core::ops::Deref for #name<M> {
            type Target = #ethers_contract::Contract<M>;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl<M> ::core::ops::DerefMut for #name<M> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        // `<name>(<address>)`
        impl<M> ::core::fmt::Debug for #name<M> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_tuple(stringify!(#name))
                    .field(&self.address())
                    .finish()
            }
        }
    }
}

pub(crate) fn expand_struct(
    name: &Ident,
    fields: &[(TokenStream, TokenStream)],
    is_tuple: bool,
) -> TokenStream {
    _expand_struct(name, fields.iter().map(|(a, b)| (a, b, false)), is_tuple)
}

pub(crate) fn expand_event_struct(
    name: &Ident,
    fields: &[(TokenStream, TokenStream, bool)],
    is_tuple: bool,
) -> TokenStream {
    _expand_struct(name, fields.iter().map(|(a, b, c)| (a, b, *c)), is_tuple)
}

fn _expand_struct<'a>(
    name: &Ident,
    fields: impl IntoIterator<Item = (&'a TokenStream, &'a TokenStream, bool)>,
    is_tuple: bool,
) -> TokenStream {
    let fields = fields.into_iter();
    let (lower, upper) = fields.size_hint();
    let fields = if lower == 0 || upper == Some(0) {
        // unit struct
        quote!(;)
    } else if is_tuple {
        // tuple struct
        let fields = fields.map(|(_, ty, indexed)| {
            let indexed = indexed_attribute(indexed);
            quote!(#indexed pub #ty)
        });
        quote!(( #( #fields ),* );)
    } else {
        // struct
        let fields = fields.map(|(field, ty, indexed)| {
            let indexed = indexed_attribute(indexed);
            quote!(#indexed pub #field: #ty)
        });
        quote!({ #( #fields, )* })
    };
    quote!(struct #name #fields)
}

fn indexed_attribute(indexed: bool) -> Option<TokenStream> {
    if indexed {
        Some(quote!(#[ethevent(indexed)]))
    } else {
        None
    }
}
