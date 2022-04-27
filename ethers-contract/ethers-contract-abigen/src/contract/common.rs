use super::{util, Context};

use proc_macro2::TokenStream;
use quote::quote;

use ethers_core::macros::{ethers_contract_crate, ethers_core_crate, ethers_providers_crate};

pub(crate) fn imports(name: &str) -> TokenStream {
    let doc = util::expand_doc(&format!("{} was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs", name));

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
    let ethers_providers = ethers_providers_crate();
    let ethers_contract = ethers_contract_crate();

    let abi_parse = if !cx.human_readable {
        quote! {
            pub static #abi_name: #ethers_contract::Lazy<#ethers_core::abi::Abi> = #ethers_contract::Lazy::new(|| serde_json::from_str(#abi)
                                              .expect("invalid abi"));
        }
    } else {
        quote! {
            pub static #abi_name: #ethers_contract::Lazy<#ethers_core::abi::Abi> = #ethers_contract::Lazy::new(|| #ethers_core::abi::parse_abi_str(#abi)
                                                .expect("invalid abi"));
        }
    };

    let bytecode = if let Some(ref bytecode) = cx.contract_bytecode {
        let bytecode_name = cx.inline_bytecode_ident();
        let hex_bytecode = format!("{}", bytecode);
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

        impl<M: #ethers_providers::Middleware> std::fmt::Debug for #name<M> {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.debug_tuple(stringify!(#name))
                    .field(&self.address())
                    .finish()
            }
        }
    }
}
