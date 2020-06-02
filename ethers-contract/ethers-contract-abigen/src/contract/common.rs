use super::Context;

use ethers_core::types::Address;
use proc_macro2::{Literal, TokenStream};
use quote::quote;

pub(crate) fn imports() -> TokenStream {
    quote! {
        // TODO: Can we make this context aware so that it imports either ethers_contract
        // or ethers::contract?
        use ethers::{
            core::{
                abi::{Abi, Token, Detokenize, InvalidOutputType, Tokenizable},
                types::*, // import all the types so that we can codegen for everything
            },
            contract::{Contract, ContractCall, Event, Lazy},
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
        pub struct #name<'a, P, S>(Contract<'a, P, S>);


        // Deref to the inner contract in order to access more specific functions functions
        impl<'a, P, S> std::ops::Deref for #name<'a, P, S> {
            type Target = Contract<'a, P, S>;

            fn deref(&self) -> &Self::Target { &self.0 }
        }

        impl<'a, P: JsonRpcClient, S: Signer> std::fmt::Debug for #name<'a, P, S> {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.debug_tuple(stringify!(#name))
                    .field(&self.address())
                    .finish()
            }
        }
    }
}
