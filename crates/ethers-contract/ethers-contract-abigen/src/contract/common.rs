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
            providers::{JsonRpcClient, networks::Network},
        };
    }
}

pub(crate) fn struct_declaration(cx: &Context) -> TokenStream {
    let name = &cx.contract_name;
    let abi = &cx.abi_str;

    quote! {
        // Inline ABI declaration
        static ABI: Lazy<Abi> = Lazy::new(|| serde_json::from_str(#abi)
                                          .expect("invalid abi"));

        // Struct declaration
        #[derive(Clone)]
        pub struct #name<'a, P, N, S>(Contract<'a, P, N, S>);


        // Deref to the inner contract in order to access more specific functions functions
        impl<'a, P, N, S> std::ops::Deref for #name<'a, P, N, S> {
            type Target = Contract<'a, P, N, S>;

            fn deref(&self) -> &Self::Target { &self.0 }
        }

        impl<'a, P: JsonRpcClient, N: Network, S: Signer> std::fmt::Debug for #name<'a, P, N, S> {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.debug_tuple(stringify!(#name))
                    .field(&self.address())
                    .finish()
            }
        }
    }
}
