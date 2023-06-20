//! Contains types to generate Rust bindings for Solidity contracts.

mod errors;
mod events;
mod methods;
pub(crate) mod structs;
mod types;

use super::{util, Abigen};
use crate::contract::{methods::MethodAlias, structs::InternalStructs};
use ethers_core::{
    abi::{Abi, AbiParser, ErrorExt, EventExt, JsonAbi},
    macros::{ethers_contract_crate, ethers_core_crate, ethers_providers_crate},
    types::Bytes,
};
use eyre::{eyre, Context as _, Result};
use proc_macro2::{Ident, Literal, TokenStream};
use quote::{format_ident, quote};
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
    /// All error impls of the contract
    pub errors: TokenStream,
    /// All contract call struct related types
    pub call_structs: TokenStream,
    /// The contract's internal structs
    pub abi_structs: TokenStream,
}

impl ExpandedContract {
    /// Merges everything into a single module
    pub fn into_tokens(self) -> TokenStream {
        self.into_tokens_with_path(None)
    }

    /// Merges everything into a single module, with an `include_bytes!` to the given path
    pub fn into_tokens_with_path(self, path: Option<&std::path::Path>) -> TokenStream {
        let ExpandedContract {
            module,
            imports,
            contract,
            events,
            call_structs,
            abi_structs,
            errors,
        } = self;

        let include_tokens = path.and_then(|path| path.to_str()).map(|s| {
            quote! {
                const _: () = { ::core::include_bytes!(#s); };
            }
        });

        quote! {
            pub use #module::*;

            /// This module was auto-generated with ethers-rs Abigen.
            /// More information at: <https://github.com/gakonst/ethers-rs>
            #[allow(
                clippy::enum_variant_names,
                clippy::too_many_arguments,
                clippy::upper_case_acronyms,
                clippy::type_complexity,
                dead_code,
                non_camel_case_types,
            )]
            pub mod #module {
                #imports
                #include_tokens
                #contract
                #errors
                #events
                #call_structs
                #abi_structs
            }
        }
    }
}

/// Internal shared context for generating smart contract bindings.
pub struct Context {
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

    /// Manually specified method aliases.
    error_aliases: BTreeMap<String, Ident>,

    /// Derives added to event structs and enums.
    extra_derives: Vec<Path>,

    /// Manually specified event aliases.
    event_aliases: BTreeMap<String, Ident>,

    /// Bytecode extracted from the abi string input, if present.
    contract_bytecode: Option<Bytes>,

    /// Deployed bytecode extracted from the abi string input, if present.
    contract_deployed_bytecode: Option<Bytes>,
}

impl Context {
    /// Generates the tokens.
    pub fn expand(&self) -> Result<ExpandedContract> {
        let name = &self.contract_ident;
        let name_mod = util::ident(&util::safe_module_name(&self.contract_name));
        let abi_name = self.inline_abi_ident();

        // 1. Declare Contract struct
        let struct_decl = self.struct_declaration();

        // 2. Declare events structs & impl FromTokens for each event
        let events_decl = self.events_declaration()?;

        // 3. impl block for the event functions
        let contract_events = self.event_methods()?;

        // 4. impl block for the contract methods and their corresponding types
        let (contract_methods, call_structs) = self.methods_and_call_structs()?;

        // 5. The deploy method, only if the contract has a bytecode object
        let deployment_methods = self.deployment_methods();

        // 6. Declare the structs parsed from the human readable abi
        let abi_structs_decl = self.abi_structs()?;

        // 7. declare all error types
        let errors_decl = self.errors()?;

        let ethers_core = ethers_core_crate();
        let ethers_contract = ethers_contract_crate();
        let ethers_providers = ethers_providers_crate();

        let contract = quote! {
                #struct_decl

                impl<M: #ethers_providers::Middleware> #name<M> {
                    /// Creates a new contract instance with the specified `ethers` client at
                    /// `address`. The contract derefs to a `ethers::Contract` object.
                    pub fn new<T: Into<#ethers_core::types::Address>>(address: T, client: ::std::sync::Arc<M>) -> Self {
                        Self(#ethers_contract::Contract::new(address.into(), #abi_name.clone(), client))
                    }

                    #deployment_methods

                    #contract_methods

                    #contract_events
                }

                impl<M: #ethers_providers::Middleware> From<#ethers_contract::Contract<M>> for #name<M> {
                    fn from(contract: #ethers_contract::Contract<M>) -> Self {
                        Self::new(contract.address(), contract.client())
                    }
                }
        };

        Ok(ExpandedContract {
            module: name_mod,
            imports: quote!(),
            contract,
            events: events_decl,
            errors: errors_decl,
            call_structs,
            abi_structs: abi_structs_decl,
        })
    }

    /// Create a context from the code generation arguments.
    pub fn from_abigen(args: Abigen) -> Result<Self> {
        // get the actual ABI string
        let abi_str = args.abi_source.get().map_err(|e| eyre!("failed to get ABI JSON: {e}"))?;

        // holds the bytecode parsed from the abi_str, if present
        let mut contract_bytecode = None;

        // holds the deployed bytecode parsed from the abi_str, if present
        let mut contract_deployed_bytecode = None;

        let (abi, human_readable, abi_parser) = parse_abi(&abi_str).wrap_err_with(|| {
            eyre::eyre!("error parsing abi for contract: {}", args.contract_name)
        })?;

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
            internal_structs.event_params = abi_parser.event_params.clone();
            internal_structs.outputs = abi_parser.outputs.clone();

            internal_structs
        } else {
            match serde_json::from_str::<JsonAbi>(&abi_str)? {
                JsonAbi::Object(obj) => {
                    contract_bytecode = obj.bytecode;
                    contract_deployed_bytecode = obj.deployed_bytecode;
                    InternalStructs::new(obj.abi)
                }
                JsonAbi::Array(abi) => InternalStructs::new(abi),
            }
        };

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
                eyre::bail!("duplicate method signature {signature:?} in method aliases")
            }
        }

        let mut event_aliases = BTreeMap::new();
        for (signature, alias) in args.event_aliases.into_iter() {
            let alias = syn::parse_str(&alias)?;
            event_aliases.insert(signature, alias);
        }

        // also check for overloaded events not covered by aliases, in which case we simply
        // numerate them
        for events in abi.events.values() {
            insert_alias_names(
                &mut event_aliases,
                events.iter().map(|e| (e.abi_signature(), e.name.as_str())),
                events::event_struct_alias,
            );
        }

        let mut error_aliases = BTreeMap::new();
        for (signature, alias) in args.error_aliases.into_iter() {
            let alias = syn::parse_str(&alias)?;
            error_aliases.insert(signature, alias);
        }

        // also check for overloaded errors not covered by aliases, in which case we simply
        // numerate them
        for errors in abi.errors.values() {
            insert_alias_names(
                &mut error_aliases,
                errors.iter().map(|e| (e.abi_signature(), e.name.as_str())),
                errors::error_struct_alias,
            );
        }

        Ok(Self {
            abi,
            human_readable,
            abi_parser,
            internal_structs,
            contract_name: args.contract_name.to_string(),
            contract_ident: args.contract_name,
            contract_bytecode,
            contract_deployed_bytecode,
            method_aliases,
            error_aliases: Default::default(),
            event_aliases,
            extra_derives: args.derives,
        })
    }

    /// The name of the contract.
    pub(crate) fn contract_name(&self) -> &str {
        &self.contract_name
    }

    /// Name of the `Lazy` that stores the ABI.
    pub(crate) fn inline_abi_ident(&self) -> Ident {
        format_ident!("{}_ABI", self.contract_name.to_uppercase())
    }

    /// Name of the `Lazy` that stores the Bytecode.
    pub(crate) fn inline_bytecode_ident(&self) -> Ident {
        format_ident!("{}_BYTECODE", self.contract_name.to_uppercase())
    }

    /// Name of the `Lazy` that stores the Deployed Bytecode.
    pub(crate) fn inline_deployed_bytecode_ident(&self) -> Ident {
        format_ident!("{}_DEPLOYED_BYTECODE", self.contract_name.to_uppercase())
    }

    /// Returns a reference to the internal ABI struct mapping table.
    pub fn internal_structs(&self) -> &InternalStructs {
        &self.internal_structs
    }

    /// Returns a mutable reference to the internal ABI struct mapping table.
    pub fn internal_structs_mut(&mut self) -> &mut InternalStructs {
        &mut self.internal_structs
    }

    /// Expands `self.extra_derives` into a comma separated list to be inserted in a
    /// `#[derive(...)]` attribute.
    pub(crate) fn expand_extra_derives(&self) -> TokenStream {
        let extra_derives = &self.extra_derives;
        quote!(#( #extra_derives, )*)
    }

    /// Generates the token stream for the contract's ABI, bytecode and struct declarations.
    pub(crate) fn struct_declaration(&self) -> TokenStream {
        let name = &self.contract_ident;

        let ethers_core = ethers_core_crate();
        let ethers_contract = ethers_contract_crate();

        let abi = {
            let doc_str = if self.human_readable {
                "The parsed human-readable ABI of the contract."
            } else {
                "The parsed JSON ABI of the contract."
            };
            let abi_name = self.inline_abi_ident();
            let abi = crate::verbatim::generate(&self.abi, &ethers_core);
            quote! {
                #[allow(deprecated)]
                fn __abi() -> #ethers_core::abi::Abi {
                    #abi
                }

                #[doc = #doc_str]
                pub static #abi_name: #ethers_contract::Lazy<#ethers_core::abi::Abi> =
                    #ethers_contract::Lazy::new(__abi);
            }
        };

        let bytecode = self.contract_bytecode.as_ref().map(|bytecode| {
            let bytecode = Literal::byte_string(bytecode);
            let bytecode_name = self.inline_bytecode_ident();
            quote! {
                #[rustfmt::skip]
                const __BYTECODE: &[u8] = #bytecode;

                /// The bytecode of the contract.
                pub static #bytecode_name: #ethers_core::types::Bytes =
                    #ethers_core::types::Bytes::from_static(__BYTECODE);
            }
        });

        let deployed_bytecode = self.contract_deployed_bytecode.as_ref().map(|bytecode| {
            let bytecode = Literal::byte_string(bytecode);
            let bytecode_name = self.inline_deployed_bytecode_ident();
            quote! {
                #[rustfmt::skip]
                const __DEPLOYED_BYTECODE: &[u8] = #bytecode;

                /// The deployed bytecode of the contract.
                pub static #bytecode_name: #ethers_core::types::Bytes =
                    #ethers_core::types::Bytes::from_static(__DEPLOYED_BYTECODE);
            }
        });

        quote! {
            // The `Lazy` ABI
            #abi

            // The static Bytecode, if present
            #bytecode

            // The static deployed Bytecode, if present
            #deployed_bytecode

            // Struct declaration
            pub struct #name<M>(#ethers_contract::Contract<M>);

            // Manual implementation since `M` is stored in `Arc<M>` and does not need to be `Clone`
            impl<M> ::core::clone::Clone for #name<M> {
                fn clone(&self) -> Self {
                    Self(::core::clone::Clone::clone(&self.0))
                }
            }

            // Deref to the inner contract to have access to all its methods
            impl<M> ::core::ops::Deref for #name<M> {
                type Target = #ethers_contract::Contract<M>;

                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }

            impl<M> ::core::ops::DerefMut for #name<M> {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.0
                }
            }

            // `<name>(<address>)`
            impl<M> ::core::fmt::Debug for #name<M> {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    f.debug_tuple(::core::stringify!(#name))
                        .field(&self.address())
                        .finish()
                }
            }
        }
    }
}

/// Solidity supports overloading as long as the signature of an event, error, function is unique,
/// which results in a mapping `(name -> Vec<Element>)`
///
///
/// This will populate the alias map for the value in the mapping (`Vec<Element>`) via `abi
/// signature -> name` using the given aliases and merge it with all names not yet aliased.
///
/// If the iterator yields more than one element, this will simply numerate them
fn insert_alias_names<'a, I, F>(aliases: &mut BTreeMap<String, Ident>, elements: I, get_ident: F)
where
    I: IntoIterator<Item = (String, &'a str)>,
    F: Fn(&str) -> Ident,
{
    let not_aliased =
        elements.into_iter().filter(|(sig, _name)| !aliases.contains_key(sig)).collect::<Vec<_>>();
    if not_aliased.len() > 1 {
        let mut overloaded_aliases = Vec::new();
        for (idx, (sig, name)) in not_aliased.into_iter().enumerate() {
            let unique_name = format!("{name}{}", idx + 1);
            overloaded_aliases.push((sig, get_ident(&unique_name)));
        }
        aliases.extend(overloaded_aliases);
    }
}

/// Parse the abi via `Source::parse` and return if the abi defined as human readable
fn parse_abi(abi_str: &str) -> Result<(Abi, bool, AbiParser)> {
    let mut abi_parser = AbiParser::default();
    let res = if let Ok(abi) = abi_parser.parse_str(abi_str) {
        (abi, true, abi_parser)
    } else {
        // a best-effort coercion of an ABI or an artifact JSON into an artifact JSON.
        let contract: JsonContract = serde_json::from_str(abi_str)
            .wrap_err_with(|| eyre::eyre!("failed deserializing abi:\n{}", abi_str))?;

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
