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
            signers::{Client, Signer},
            providers::JsonRpcClient,
        };
    }
}

pub(crate) fn struct_declaration(cx: &Context, abi_name: &proc_macro2::Ident) -> TokenStream {
    let name = &cx.contract_name;
    let abi = &cx.abi_str;

    quote! {
        // Inline ABI declaration
        pub static #abi_name: Lazy<Abi> = Lazy::new(|| serde_json::from_str(#abi)
                                          .expect("invalid abi"));

        // Struct declaration
        #[derive(Clone)]
        pub struct #name<P, S>(Contract<P, S>);


        // Deref to the inner contract in order to access more specific functions functions
        impl<P, S> std::ops::Deref for #name<P, S> {
            type Target = Contract<P, S>;

            fn deref(&self) -> &Self::Target { &self.0 }
        }

        impl<P: JsonRpcClient, S: Signer> std::fmt::Debug for #name<P, S> {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.debug_tuple(stringify!(#name))
                    .field(&self.address())
                    .finish()
            }
        }
    }
}
