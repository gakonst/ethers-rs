use super::Context;

use ethers_types::Address;
use proc_macro2::{Literal, TokenStream};
use quote::quote;

pub(crate) fn imports() -> TokenStream {
    quote! {
        // TODO: Can we make this context aware so that it imports either ethers_contract
        // or ethers::contract?
        use ethers_contract::{
            abi::{Abi, Token, Detokenize, InvalidOutputType, Tokenizable},
            Contract, ContractCall, Event, Lazy,
            signers::{Client, Signer},
            types::*, // import all the types so that we can codegen for everything
            providers::JsonRpcClient,
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
        pub struct #name<'a, S, P>(Contract<'a, S, P>);


        // Deref to the inner contract in order to access more specific functions functions
        impl<'a, S, P> std::ops::Deref for #name<'a, S, P> {
            type Target = Contract<'a, S, P>;

            fn deref(&self) -> &Self::Target { &self.0 }
        }

        impl<'a, S: Signer, P: JsonRpcClient> std::fmt::Debug for #name<'a, S, P> {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.debug_tuple(stringify!(#name))
                    .field(&self.address())
                    .finish()
            }
        }
    }
}
