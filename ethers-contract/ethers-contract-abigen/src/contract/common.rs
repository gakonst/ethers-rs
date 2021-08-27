use super::{util, Context};

use ethers_core::types::Address;
use proc_macro2::{Literal, TokenStream};
use quote::quote;

use super::util::{ethers_contract_crate, ethers_core_crate, ethers_providers_crate};

pub(crate) fn imports(name: &str) -> TokenStream {
    let doc = util::expand_doc(&format!("{} was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs", name));

    let ethers_core = ethers_core_crate();
    let ethers_providers = ethers_providers_crate();
    let ethers_contract = ethers_contract_crate();

    quote! {
        #![allow(clippy::enum_variant_names)]
        #![allow(dead_code)]
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
pub(crate) fn struct_declaration(cx: &Context, abi_name: &proc_macro2::Ident) -> TokenStream {
    let name = &cx.contract_name;
    let abi = &cx.abi_str;

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

    quote! {
        // Inline ABI declaration
        #abi_parse

        // Struct declaration
        #[derive(Clone)]
        pub struct #name<M>(#ethers_contract::Contract<M>);


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
