//! Events expansion

use super::{structs::expand_event_struct, types, Context};
use crate::util;
use ethers_core::{
    abi::{Event, EventExt},
    macros::{ethers_contract_crate, ethers_core_crate},
};
use eyre::Result;
use inflector::Inflector;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use std::collections::BTreeMap;

impl Context {
    /// Expands each event to a struct + its impl Detokenize block
    pub fn events_declaration(&self) -> Result<TokenStream> {
        let sorted_events: BTreeMap<_, _> = self.abi.events.clone().into_iter().collect();
        let data_types = sorted_events
            .values()
            .flatten()
            .map(|event| self.expand_event(event))
            .collect::<Result<Vec<_>>>()?;

        // only expand enums when multiple events are present
        let events_enum_decl =
            if data_types.len() > 1 { Some(self.expand_events_enum()) } else { None };

        Ok(quote! {
            #( #data_types )*

            #events_enum_decl
        })
    }

    /// Generate the event filter methods for the contract
    pub fn event_methods(&self) -> Result<TokenStream> {
        let sorted_events: BTreeMap<_, _> = self.abi.events.iter().collect();
        let filter_methods = sorted_events
            .values()
            .flat_map(std::ops::Deref::deref)
            .map(|event| self.expand_filter(event));

        let events_method = self.expand_events_method();

        Ok(quote! {
            #( #filter_methods )*

            #events_method
        })
    }

    /// Generate an enum with a variant for each event
    fn expand_events_enum(&self) -> TokenStream {
        let variants = self
            .abi
            .events
            .values()
            .flatten()
            .map(|e| {
                event_struct_name(&e.name, self.event_aliases.get(&e.abi_signature()).cloned())
            })
            .collect::<Vec<_>>();

        let enum_name = self.expand_event_enum_name();

        let mut derives = self.expand_extra_derives();
        let params =
            self.abi.events.values().flatten().flat_map(|err| &err.inputs).map(|param| &param.kind);
        util::derive_builtin_traits(params, &mut derives, false, true);

        let ethers_core = ethers_core_crate();
        let ethers_contract = ethers_contract_crate();

        quote! {
            #[doc = "Container type for all of the contract's events"]
            #[derive(Clone, #ethers_contract::EthAbiType, #derives)]
            pub enum #enum_name {
                #( #variants(#variants), )*
            }

            impl #ethers_contract::EthLogDecode for #enum_name {
                fn decode_log(log: &#ethers_core::abi::RawLog) -> ::core::result::Result<Self, #ethers_core::abi::Error> {
                    #(
                        if let Ok(decoded) = #variants::decode_log(log) {
                            return Ok(#enum_name::#variants(decoded))
                        }
                    )*
                    Err(#ethers_core::abi::Error::InvalidData)
                }
            }

            impl ::core::fmt::Display for #enum_name {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match self {
                        #(
                            Self::#variants(element) => ::core::fmt::Display::fmt(element, f),
                        )*
                    }
                }
            }

            #(
                impl ::core::convert::From<#variants> for #enum_name {
                    fn from(value: #variants) -> Self {
                        Self::#variants(value)
                    }
                }
            )*
        }
    }

    /// The name ident of the events enum
    fn expand_event_enum_name(&self) -> Ident {
        util::ident(&format!("{}Events", self.contract_ident))
    }

    /// Expands the `events` function that bundles all declared events of this contract
    fn expand_events_method(&self) -> Option<TokenStream> {
        let sorted_events: BTreeMap<_, _> = self.abi.events.clone().into_iter().collect();

        let mut iter = sorted_events.values().flatten();
        let ethers_contract = ethers_contract_crate();

        if let Some(event) = iter.next() {
            let ty = if iter.next().is_some() {
                self.expand_event_enum_name()
            } else {
                event_struct_name(
                    &event.name,
                    self.event_aliases.get(&event.abi_signature()).cloned(),
                )
            };

            Some(quote! {
                /// Returns an `Event` builder for all the events of this contract.
                pub fn events(&self) -> #ethers_contract::builders::Event<
                    ::std::sync::Arc<M>,
                    M,
                    #ty,
                > {
                    self.0.event_with_filter(::core::default::Default::default())
                }
            })
        } else {
            None
        }
    }

    /// Expands into a single method for contracting an event stream.
    fn expand_filter(&self, event: &Event) -> TokenStream {
        let name = &event.name;
        let sig = event.abi_signature();
        let alias = self.event_aliases.get(&sig).cloned();

        // append `filter` to disambiguate with potentially conflicting function names
        let function_name = {
            let name = if let Some(ref id) = alias {
                id.to_string().to_snake_case()
            } else {
                name.to_snake_case()
            };
            util::safe_ident(&format!("{name}_filter"))
        };
        let struct_name = event_struct_name(name, alias);

        let doc_str = format!("Gets the contract's `{name}` event");

        let ethers_contract = ethers_contract_crate();

        quote! {
            #[doc = #doc_str]
            pub fn #function_name(&self) -> #ethers_contract::builders::Event<
                ::std::sync::Arc<M>,
                M,
                #struct_name
            > {
                self.0.event()
            }
        }
    }

    /// Expands an ABI event into a single event data type. This can expand either
    /// into a structure or a tuple in the case where all event parameters (topics
    /// and data) are anonymous.
    fn expand_event(&self, event: &Event) -> Result<TokenStream> {
        let name = &event.name;
        let abi_signature = event.abi_signature();
        let alias = self.event_aliases.get(&abi_signature).cloned();

        let struct_name = event_struct_name(name, alias);

        let fields = types::expand_event_inputs(event, &self.internal_structs)?;
        // expand as a tuple if all fields are anonymous
        let all_anonymous_fields = event.inputs.iter().all(|input| input.name.is_empty());
        let data_type_definition = expand_event_struct(&struct_name, &fields, all_anonymous_fields);

        let mut derives = self.expand_extra_derives();
        let params = event.inputs.iter().map(|param| &param.kind);
        util::derive_builtin_traits(params, &mut derives, true, true);

        let ethers_contract = ethers_contract_crate();

        Ok(quote! {
            #[derive(Clone, #ethers_contract::EthEvent, #ethers_contract::EthDisplay, #derives)]
            #[ethevent(name = #name, abi = #abi_signature)]
            pub #data_type_definition
        })
    }
}

/// Expands an ABI event into an identifier for its event data type.
fn event_struct_name(event_name: &str, alias: Option<Ident>) -> Ident {
    // TODO: get rid of `Filter` suffix?

    let name = if let Some(id) = alias {
        format!("{}Filter", id.to_string().to_pascal_case())
    } else {
        format!("{}Filter", event_name.to_pascal_case())
    };
    util::ident(&name)
}

/// Returns the alias name for an event
pub(crate) fn event_struct_alias(event_name: &str) -> Ident {
    util::ident(&event_name.to_pascal_case())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Abigen;
    use ethers_core::abi::{EventParam, ParamType};

    fn test_context() -> Context {
        Context::from_abigen(Abigen::new("TestToken", "[]").unwrap()).unwrap()
    }

    fn test_context_with_alias(sig: &str, alias: &str) -> Context {
        Context::from_abigen(Abigen::new("TestToken", "[]").unwrap().add_event_alias(sig, alias))
            .unwrap()
    }

    #[test]
    fn expand_transfer_filter_with_alias() {
        let event = Event {
            name: "Transfer".into(),
            inputs: vec![
                EventParam { name: "from".into(), kind: ParamType::Address, indexed: true },
                EventParam { name: "to".into(), kind: ParamType::Address, indexed: true },
                EventParam { name: "amount".into(), kind: ParamType::Uint(256), indexed: false },
            ],
            anonymous: false,
        };
        let sig = "Transfer(address,address,uint256)";
        let cx = test_context_with_alias(sig, "TransferEvent");
        #[rustfmt::skip]
        assert_quote!(cx.expand_filter(&event), {
            #[doc = "Gets the contract's `Transfer` event"]
            pub fn transfer_event_filter(
                &self
            ) -> ::ethers_contract::builders::Event<::std::sync::Arc<M>, M, TransferEventFilter>
            {
                self.0.event()
            }
        });
    }

    #[test]
    fn expand_transfer_filter() {
        let event = Event {
            name: "Transfer".into(),
            inputs: vec![
                EventParam { name: "from".into(), kind: ParamType::Address, indexed: true },
                EventParam { name: "to".into(), kind: ParamType::Address, indexed: true },
                EventParam { name: "amount".into(), kind: ParamType::Uint(256), indexed: false },
            ],
            anonymous: false,
        };
        let cx = test_context();
        #[rustfmt::skip]
        assert_quote!(cx.expand_filter(&event), {
            #[doc = "Gets the contract's `Transfer` event"]
            pub fn transfer_filter(
                &self
            ) -> ::ethers_contract::builders::Event<::std::sync::Arc<M>, M, TransferFilter>
            {
                self.0.event()
            }
        });
    }

    #[test]
    fn expand_data_struct_value() {
        let event = Event {
            name: "Foo".into(),
            inputs: vec![
                EventParam { name: "a".into(), kind: ParamType::Bool, indexed: false },
                EventParam { name: String::new(), kind: ParamType::Address, indexed: false },
            ],
            anonymous: false,
        };

        let cx = test_context();
        let params = types::expand_event_inputs(&event, &cx.internal_structs).unwrap();
        let name = event_struct_name(&event.name, None);
        let definition = expand_event_struct(&name, &params, false);

        assert_quote!(definition, {
            struct FooFilter {
                pub a: bool,
                pub p1: ::ethers_core::types::Address,
            }
        });
    }

    #[test]
    fn expand_data_struct_with_alias() {
        let event = Event {
            name: "Foo".into(),
            inputs: vec![
                EventParam { name: "a".into(), kind: ParamType::Bool, indexed: false },
                EventParam { name: String::new(), kind: ParamType::Address, indexed: false },
            ],
            anonymous: false,
        };

        let cx = test_context_with_alias("Foo(bool,address)", "FooAliased");
        let params = types::expand_event_inputs(&event, &cx.internal_structs).unwrap();
        let alias = Some(util::ident("FooAliased"));
        let name = event_struct_name(&event.name, alias);
        let definition = expand_event_struct(&name, &params, false);

        assert_quote!(definition, {
            struct FooAliasedFilter {
                pub a: bool,
                pub p1: ::ethers_core::types::Address,
            }
        });
    }

    #[test]
    fn expand_data_tuple_value() {
        let event = Event {
            name: "Foo".into(),
            inputs: vec![
                EventParam { name: String::new(), kind: ParamType::Bool, indexed: false },
                EventParam { name: String::new(), kind: ParamType::Address, indexed: false },
            ],
            anonymous: false,
        };

        let cx = test_context();
        let params = types::expand_event_inputs(&event, &cx.internal_structs).unwrap();
        let name = event_struct_name(&event.name, None);
        let definition = expand_event_struct(&name, &params, true);

        assert_quote!(definition, {
            struct FooFilter(pub bool, pub ::ethers_core::types::Address);
        });
    }

    #[test]
    fn expand_data_tuple_value_with_alias() {
        let event = Event {
            name: "Foo".into(),
            inputs: vec![
                EventParam { name: String::new(), kind: ParamType::Bool, indexed: false },
                EventParam { name: String::new(), kind: ParamType::Address, indexed: false },
            ],
            anonymous: false,
        };

        let cx = test_context_with_alias("Foo(bool,address)", "FooAliased");
        let params = types::expand_event_inputs(&event, &cx.internal_structs).unwrap();
        let alias = Some(util::ident("FooAliased"));
        let name = event_struct_name(&event.name, alias);
        let definition = expand_event_struct(&name, &params, true);

        assert_quote!(definition, {
            struct FooAliasedFilter(pub bool, pub ::ethers_core::types::Address);
        });
    }
}
