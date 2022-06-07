#![deny(missing_docs)]
mod common;
mod events;
mod methods;
mod structs;
mod types;

use super::{util, Abigen};
use crate::contract::structs::InternalStructs;
use ethers_core::{
    abi::{Abi, AbiParser},
    macros::{ethers_contract_crate, ethers_core_crate, ethers_providers_crate},
};
use eyre::{eyre, Context as _, Result};

use crate::contract::methods::MethodAlias;

use crate::rawabi::JsonAbi;
use ethers_core::{abi::EventExt, types::Bytes};
use proc_macro2::{Ident, Literal, TokenStream};
use quote::quote;
use serde::Deserialize;
use std::collections::BTreeMap;
use syn::Path;

/// The result of `Context::expand`
#[derive(Debug)]
pub struct ExpandedContract {
    /// The name of the contract module
    pub module: Ident,
    /// The contract module's imports
    pub imports: TokenStream,
    /// Contract, Middle related implementations
    pub contract: TokenStream,
    /// All event impls of the contract
    pub events: TokenStream,
    /// All contract call struct related types
    pub call_structs: TokenStream,
    /// The contract's internal structs
    pub abi_structs: TokenStream,
}

impl ExpandedContract {
    /// Merges everything into a single module
    pub fn into_tokens(self) -> TokenStream {
        let ExpandedContract { module, imports, contract, events, call_structs, abi_structs } =
            self;
        quote! {
           // export all the created data types
            pub use #module::*;

            #[allow(clippy::too_many_arguments, non_camel_case_types)]
            pub mod #module {
                #imports
                #contract
                #events
                #call_structs
                #abi_structs
            }
        }
    }
}

/// Internal shared context for generating smart contract bindings.
pub struct Context {
    /// The ABI string pre-parsing.
    abi_str: Literal,

    /// The parsed ABI.
    abi: Abi,

    /// The parser used for human readable format
    abi_parser: AbiParser,

    /// Contains all the solidity structs extracted from the JSON ABI.
    internal_structs: InternalStructs,

    /// Was the ABI in human readable format?
    human_readable: bool,

    /// The contract name as an identifier.
    contract_ident: Ident,

    /// The contract name as string
    contract_name: String,

    /// Manually specified method aliases.
    method_aliases: BTreeMap<String, MethodAlias>,

    /// Derives added to event structs and enums.
    event_derives: Vec<Path>,

    /// Manually specified event aliases.
    event_aliases: BTreeMap<String, Ident>,

    /// Bytecode extracted from the abi string input, if present.
    contract_bytecode: Option<Bytes>,
}

impl Context {
    /// Expands the whole rust contract
    pub fn expand(&self) -> Result<ExpandedContract> {
        let name = &self.contract_ident;
        let name_mod =
            util::ident(&format!("{}_mod", self.contract_ident.to_string().to_lowercase()));
        let abi_name = self.inline_abi_ident();

        // 0. Imports
        let imports = common::imports(&name.to_string());

        // 1. Declare Contract struct
        let struct_decl = common::struct_declaration(self);

        // 2. Declare events structs & impl FromTokens for each event
        let events_decl = self.events_declaration()?;

        // 3. impl block for the event functions
        let contract_events = self.event_methods()?;

        // 4. impl block for the contract methods and their corresponding types
        let (contract_methods, call_structs) = self.methods_and_call_structs()?;

        // 5. generate deploy function if
        let deployment_methods = self.deployment_methods();

        // 6. Declare the structs parsed from the human readable abi
        let abi_structs_decl = self.abi_structs()?;

        let ethers_core = ethers_core_crate();
        let ethers_contract = ethers_contract_crate();
        let ethers_providers = ethers_providers_crate();

        let contract = quote! {
                #struct_decl

                impl<M: #ethers_providers::Middleware> #name<M> {
                    /// Creates a new contract instance with the specified `ethers`
                    /// client at the given `Address`. The contract derefs to a `ethers::Contract`
                    /// object
                    pub fn new<T: Into<#ethers_core::types::Address>>(address: T, client: ::std::sync::Arc<M>) -> Self {
                        #ethers_contract::Contract::new(address.into(), #abi_name.clone(), client).into()
                    }

                    #deployment_methods

                    #contract_methods

                    #contract_events
                }

                impl<M : #ethers_providers::Middleware> From<#ethers_contract::Contract<M>> for #name<M> {
                    fn from(contract: #ethers_contract::Contract<M>) -> Self {
                       Self(contract)
                    }
                }
        };

        Ok(ExpandedContract {
            module: name_mod,
            imports,
            contract,
            events: events_decl,
            call_structs,
            abi_structs: abi_structs_decl,
        })
    }

    /// Create a context from the code generation arguments.
    pub fn from_abigen(args: Abigen) -> Result<Self> {
        // get the actual ABI string
        let mut abi_str =
            args.abi_source.get().map_err(|e| eyre!("failed to get ABI JSON: {}", e))?;

        // holds the bytecode parsed from the abi_str, if present
        let mut contract_bytecode = None;

        let (abi, human_readable, abi_parser) = parse_abi(&abi_str)?;

        // try to extract all the solidity structs from the normal JSON ABI
        // we need to parse the json abi again because we need the internalType fields which are
        // omitted by ethabi. If the ABI was defined as human readable we use the `internal_structs`
        // from the Abi Parser
        let internal_structs = if human_readable {
            let mut internal_structs = InternalStructs::default();
            // the types in the abi_parser are already valid rust types so simply clone them to make
            // it consistent with the `RawAbi` variant
            internal_structs
                .rust_type_names
                .extend(abi_parser.function_params.values().map(|ty| (ty.clone(), ty.clone())));
            internal_structs.function_params = abi_parser.function_params.clone();
            internal_structs.outputs = abi_parser.outputs.clone();

            internal_structs
        } else {
            match serde_json::from_str::<JsonAbi>(&abi_str)? {
                JsonAbi::Object(obj) => {
                    // need to update the `abi_str` here because we only want the `"abi": [...]`
                    // part of the json object in the contract binding
                    abi_str = serde_json::to_string(&obj.abi)?;
                    contract_bytecode = obj.bytecode;
                    InternalStructs::new(obj.abi)
                }
                JsonAbi::Array(abi) => InternalStructs::new(abi),
            }
        };

        let contract_ident = util::ident(&args.contract_name);

        // NOTE: We only check for duplicate signatures here, since if there are
        //   duplicate aliases, the compiler will produce a warning because a
        //   method will be re-defined.
        let mut method_aliases = BTreeMap::new();
        for (signature, alias) in args.method_aliases.into_iter() {
            let alias = MethodAlias {
                function_name: util::safe_ident(&alias),
                struct_name: util::safe_pascal_case_ident(&alias),
            };

            if method_aliases.insert(signature.clone(), alias).is_some() {
                eyre::bail!("duplicate method signature '{}' in method aliases", signature)
            }
        }

        let mut event_aliases = BTreeMap::new();
        for (signature, alias) in args.event_aliases.into_iter() {
            let alias = syn::parse_str(&alias)?;
            event_aliases.insert(signature, alias);
        }

        // also check for overloaded functions not covered by aliases, in which case we simply
        // numerate them
        for events in abi.events.values() {
            let not_aliased =
                events.iter().filter(|ev| !event_aliases.contains_key(&ev.abi_signature()));
            if not_aliased.clone().count() > 1 {
                let mut aliases = Vec::new();
                // overloaded events
                for (idx, event) in not_aliased.enumerate() {
                    let event_name = format!("{}{}", event.name, idx + 1);
                    aliases.push((event.abi_signature(), events::event_struct_alias(&event_name)));
                }
                event_aliases.extend(aliases);
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
            human_readable,
            abi_str: Literal::string(&abi_str),
            abi_parser,
            internal_structs,
            contract_ident,
            contract_name: args.contract_name,
            contract_bytecode,
            method_aliases,
            event_derives,
            event_aliases,
        })
    }

    /// The initial name fo the contract
    pub(crate) fn contract_name(&self) -> &str {
        &self.contract_name
    }

    /// name of the `Lazy` that stores the ABI
    pub(crate) fn inline_abi_ident(&self) -> Ident {
        util::safe_ident(&format!("{}_ABI", self.contract_ident.to_string().to_uppercase()))
    }

    /// name of the `Lazy` that stores the Bytecode
    pub(crate) fn inline_bytecode_ident(&self) -> Ident {
        util::safe_ident(&format!("{}_BYTECODE", self.contract_ident.to_string().to_uppercase()))
    }

    /// The internal abi struct mapping table
    pub fn internal_structs(&self) -> &InternalStructs {
        &self.internal_structs
    }

    /// The internal mutable abi struct mapping table
    pub fn internal_structs_mut(&mut self) -> &mut InternalStructs {
        &mut self.internal_structs
    }
}

/// Parse the abi via `Source::parse` and return if the abi defined as human readable
fn parse_abi(abi_str: &str) -> Result<(Abi, bool, AbiParser)> {
    let mut abi_parser = AbiParser::default();
    let res = if let Ok(abi) = abi_parser.parse_str(abi_str) {
        (abi, true, abi_parser)
    } else {
        // a best-effort coercion of an ABI or an artifact JSON into an artifact JSON.
        let contract: JsonContract = serde_json::from_str(abi_str)?;

        (contract.into_abi(), false, abi_parser)
    };
    Ok(res)
}

#[derive(Deserialize)]
struct ContractObject {
    abi: Abi,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum JsonContract {
    /// json object input as `{"abi": [..], "bin": "..."}`
    Object(ContractObject),
    /// json array input as `[]`
    Array(Abi),
}

impl JsonContract {
    fn into_abi(self) -> Abi {
        match self {
            JsonContract::Object(o) => o.abi,
            JsonContract::Array(abi) => abi,
        }
    }
}
