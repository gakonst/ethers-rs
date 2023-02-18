use super::Context;
use ethers_core::macros::{ethers_contract_crate, ethers_core_crate, ethers_providers_crate};
use proc_macro2::{Ident, Literal, TokenStream};
use quote::quote;

pub(crate) fn imports(name: &str) -> TokenStream {
    let doc_str = format!("{name} was auto-generated with ethers-rs Abigen. More information at: <https://github.com/gakonst/ethers-rs>");

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
        let bytecode = bytecode.iter().copied().map(Literal::u8_unsuffixed);
        let bytecode_name = cx.inline_bytecode_ident();
        quote! {
            #[rustfmt::skip]
            const __BYTECODE: &[u8] = &[ #( #bytecode ),* ];

            #[doc = "The bytecode of the contract."]
            pub static #bytecode_name: #ethers_core::types::Bytes = #ethers_core::types::Bytes::from_static(__BYTECODE);
        }
    });

    let deployed_bytecode = cx.contract_deployed_bytecode.as_ref().map(|bytecode| {
        let bytecode = bytecode.iter().copied().map(Literal::u8_unsuffixed);
        let bytecode_name = cx.inline_deployed_bytecode_ident();
        quote! {
            #[rustfmt::skip]
            const __DEPLOYED_BYTECODE: &[u8] = &[ #( #bytecode ),* ];

            #[doc = "The deployed bytecode of the contract."]
            pub static #bytecode_name: #ethers_core::types::Bytes = #ethers_core::types::Bytes::from_static(__DEPLOYED_BYTECODE);
        }
    });

    quote! {
        // The `Lazy` ABI
        #abi

        // The static Bytecode, if present
        #bytecode

         // The static deployed Bytecode, if present
         #deployed_bytecode

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
