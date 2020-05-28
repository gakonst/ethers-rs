#![deny(missing_docs)]

//! Crate for generating type-safe bindings to Ethereum smart contracts. This
//! crate is intended to be used either indirectly with the `ethcontract`
//! crate's `contract` procedural macro or directly from a build script.

mod common;
mod events;
mod methods;
mod types;

use super::util;
use super::Args;
use anyhow::{anyhow, Context as _, Result};
use ethers_types::{abi::Abi, Address};
use inflector::Inflector;
use proc_macro2::{Ident, Literal, TokenStream};
use quote::quote;
use std::collections::HashMap;
use syn::{Path, Visibility};

/// Internal shared context for generating smart contract bindings.
pub(crate) struct Context {
    /// The contract name
    name: String,

    /// The ABI string pre-parsing.
    abi_str: Literal,

    /// The parsed ABI.
    abi: Abi,

    /// The identifier for the runtime crate. Usually this is `ethcontract` but
    /// it can be different if the crate was renamed in the Cargo manifest for
    /// example.
    runtime_crate: Ident,

    /// The visibility for the generated module and re-exported contract type.
    visibility: Visibility,

    /// The contract name as an identifier.
    contract_name: Ident,

    /// Manually specified method aliases.
    method_aliases: HashMap<String, Ident>,

    /// Derives added to event structs and enums.
    event_derives: Vec<Path>,
}

impl Context {
    pub fn expand(args: Args) -> Result<TokenStream> {
        let cx = Self::from_args(args)?;
        let name = &cx.contract_name;
        let name_mod = util::ident(&format!(
            "{}_mod",
            cx.contract_name.to_string().to_lowercase()
        ));

        // 0. Imports
        let imports = common::imports();

        // 1. Declare Contract struct
        let struct_decl = common::struct_declaration(&cx);

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

                impl<'a, P: JsonRpcClient, N: Network, S: Signer> #name<'a, P, N, S> {
                    /// Creates a new contract instance with the specified `ethers`
                    /// client at the given `Address`. The contract derefs to a `ethers::Contract`
                    /// object
                    pub fn new<T: Into<Address>>(address: T, client: &'a Client<'a, P, N, S>) -> Self {
                        let contract = Contract::new(client, &ABI, address.into());
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
    fn from_args(args: Args) -> Result<Self> {
        // get the actual ABI string
        let abi_str = args.abi_source.get().context("failed to get ABI JSON")?;

        // parse it
        let abi: Abi = serde_json::from_str(&abi_str)
            .with_context(|| format!("invalid artifact JSON '{}'", abi_str))
            .with_context(|| {
                format!("failed to parse artifact from source {:?}", args.abi_source,)
            })?;

        let raw_contract_name = args.contract_name;

        let runtime_crate = util::ident(&args.runtime_crate_name);

        let visibility = match args.visibility_modifier.as_ref() {
            Some(vis) => syn::parse_str(vis)?,
            None => Visibility::Inherited,
        };

        let contract_name = util::ident(&raw_contract_name);

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
            name: raw_contract_name,
            abi,
            abi_str: Literal::string(&abi_str),
            runtime_crate,
            visibility,
            contract_name,
            method_aliases,
            event_derives,
        })
    }
}
