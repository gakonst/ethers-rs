use super::{util, Context};

use ethers_core::types::Address;
use proc_macro2::{Literal, TokenStream};
use quote::quote;

pub(crate) fn imports(name: &str) -> TokenStream {
    let doc = util::expand_doc(&format!("{} was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs", name));

    quote! {
        #![allow(dead_code)]
        #![allow(unused_imports)]
        #doc

        use std::sync::Arc;
        use ethers::{
            core::{
                abi::{Abi, Token, Detokenize, InvalidOutputType, Tokenizable},
                types::*, // import all the types so that we can codegen for everything
            },
            contract::{Contract, builders::{ContractCall, Event}, Lazy},
            providers::Middleware,
        };
    }
}

/// Generates the static `Abi` constants and the contract struct
pub(crate) fn struct_declaration(cx: &Context, abi_name: &proc_macro2::Ident) -> TokenStream {
    let name = &cx.contract_name;
    let abi = &cx.abi_str;

    let abi_parse = if !cx.human_readable {
        quote! {
            pub static #abi_name: Lazy<Abi> = Lazy::new(|| serde_json::from_str(#abi)
                                              .expect("invalid abi"));
        }
    } else {
        quote! {
            pub static #abi_name: Lazy<Abi> = Lazy::new(|| ethers::core::abi::parse_abi_str(#abi)
                                                .expect("invalid abi"));
        }
    };

    quote! {
        // Inline ABI declaration
        #abi_parse

        // Struct declaration
        #[derive(Clone)]
        pub struct #name<M>(Contract<M>);


        // Deref to the inner contract in order to access more specific functions functions
        impl<M> std::ops::Deref for #name<M> {
            type Target = Contract<M>;

            fn deref(&self) -> &Self::Target { &self.0 }
        }

        impl<M: Middleware> std::fmt::Debug for #name<M> {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.debug_tuple(stringify!(#name))
                    .field(&self.address())
                    .finish()
            }
        }
    }
}
