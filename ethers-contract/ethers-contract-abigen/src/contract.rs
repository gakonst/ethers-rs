#![deny(missing_docs)]
mod common;
mod events;
mod methods;
mod types;

use super::util;
use super::Abigen;
use anyhow::{anyhow, Context as _, Result};
use ethers_core::{abi::Abi, types::Address};
use inflector::Inflector;
use proc_macro2::{Ident, Literal, TokenStream};
use quote::quote;
use std::collections::HashMap;
use syn::{Path, Visibility};

/// Internal shared context for generating smart contract bindings.
pub(crate) struct Context {
    /// The ABI string pre-parsing.
    abi_str: Literal,

    /// The parsed ABI.
    abi: Abi,

    /// The contract name as an identifier.
    contract_name: Ident,

    /// Manually specified method aliases.
    method_aliases: HashMap<String, Ident>,

    /// Derives added to event structs and enums.
    event_derives: Vec<Path>,
}

impl Context {
    pub(crate) fn expand(args: Abigen) -> Result<TokenStream> {
        let cx = Self::from_abigen(args)?;
        let name = &cx.contract_name;
        let name_mod = util::ident(&format!(
            "{}_mod",
            cx.contract_name.to_string().to_lowercase()
        ));

        let abi_name = super::util::safe_ident(&format!("{}_ABI", name.to_string().to_uppercase()));

        // 0. Imports
        let imports = common::imports();

        // 1. Declare Contract struct
        let struct_decl = common::struct_declaration(&cx, &abi_name);

        // 2. Declare events structs & impl FromTokens for each event
        let events_decl = cx.events_declaration()?;

        // 3. impl block for the event functions
        let contract_events = cx.events()?;

        // 4. impl block for the contract methods
        let contract_methods = cx.methods()?;

        Ok(quote! {
            // export all the created data types
            pub use #name_mod::*;

            mod #name_mod {
                #imports

                #struct_decl

                impl<'a, P: JsonRpcClient, S: Signer> #name<'a, P, S> {
                    /// Creates a new contract instance with the specified `ethers`
                    /// client at the given `Address`. The contract derefs to a `ethers::Contract`
                    /// object
                    pub fn new<T: Into<Address>>(address: T, client: &'a Client<P, S>) -> Self {
                        let contract = Contract::new(address.into(), &#abi_name, client);
                        Self(contract)
                    }

                    // TODO: Implement deployment.

                    #contract_methods

                    #contract_events
                }

                #events_decl
            }
        })
    }

    /// Create a context from the code generation arguments.
    fn from_abigen(args: Abigen) -> Result<Self> {
        // get the actual ABI string
        let abi_str = args.abi_source.get().context("failed to get ABI JSON")?;

        // parse it
        let abi: Abi = serde_json::from_str(&abi_str)
            .with_context(|| format!("invalid artifact JSON '{}'", abi_str))
            .with_context(|| {
                format!("failed to parse artifact from source {:?}", args.abi_source,)
            })?;

        let contract_name = util::ident(&args.contract_name);

        // NOTE: We only check for duplicate signatures here, since if there are
        //   duplicate aliases, the compiler will produce a warning because a
        //   method will be re-defined.
        let mut method_aliases = HashMap::new();
        for (signature, alias) in args.method_aliases.into_iter() {
            let alias = syn::parse_str(&alias)?;
            if method_aliases.insert(signature.clone(), alias).is_some() {
                return Err(anyhow!(
                    "duplicate method signature '{}' in method aliases",
                    signature,
                ));
            }
        }

        let event_derives = args
            .event_derives
            .iter()
            .map(|derive| syn::parse_str::<Path>(derive))
            .collect::<Result<Vec<_>, _>>()
            .context("failed to parse event derives")?;

        Ok(Context {
            abi,
            abi_str: Literal::string(&abi_str),
            contract_name,
            method_aliases,
            event_derives,
        })
    }
}
