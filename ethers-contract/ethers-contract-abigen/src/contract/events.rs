use super::{types, util, Context};
use anyhow::Result;
use ethers_core::{
    abi::{Event, EventExt, EventParam, ParamType, SolStruct},
    macros::{ethers_contract_crate, ethers_core_crate},
};
use inflector::Inflector;
use proc_macro2::{Ident, Literal, TokenStream};
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
        let events_enum_decl = if sorted_events.values().flatten().count() > 1 {
            self.expand_events_enum()
        } else {
            quote! {}
        };

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
            .map(std::ops::Deref::deref)
            .flatten()
            .map(|event| self.expand_filter(event))
            .collect::<Vec<_>>();

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
            .map(|e| expand_struct_name(e, self.event_aliases.get(&e.abi_signature()).cloned()))
            .collect::<Vec<_>>();

        let enum_name = self.expand_event_enum_name();

        let ethers_core = ethers_core_crate();
        let ethers_contract = ethers_contract_crate();

        quote! {
            #[derive(Debug, Clone, PartialEq, Eq, #ethers_contract::EthAbiType)]
            pub enum #enum_name {
                #(#variants(#variants)),*
            }

             impl #ethers_contract::EthLogDecode for #enum_name {
                fn decode_log(log: &#ethers_core::abi::RawLog) -> Result<Self, #ethers_core::abi::Error>
                where
                    Self: Sized,
                {
                     #(
                        if let Ok(decoded) = #variants::decode_log(log) {
                            return Ok(#enum_name::#variants(decoded))
                        }
                    )*
                    Err(#ethers_core::abi::Error::InvalidData)
                }
            }

            impl ::std::fmt::Display for #enum_name {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    match self {
                        #(
                            #enum_name::#variants(element) => element.fmt(f)
                        ),*
                    }
                }
            }
        }
    }

    /// The name ident of the events enum
    fn expand_event_enum_name(&self) -> Ident {
        util::ident(&format!("{}Events", self.contract_name))
    }

    /// Expands the `events` function that bundles all declared events of this contract
    fn expand_events_method(&self) -> TokenStream {
        let sorted_events: BTreeMap<_, _> = self.abi.events.clone().into_iter().collect();

        let mut iter = sorted_events.values().flatten();
        let ethers_contract = ethers_contract_crate();

        if let Some(event) = iter.next() {
            let ty = if iter.next().is_some() {
                self.expand_event_enum_name()
            } else {
                expand_struct_name(event, self.event_aliases.get(&event.abi_signature()).cloned())
            };

            quote! {
                /// Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract
                pub fn events(&self) -> #ethers_contract::builders::Event<M, #ty> {
                    self.0.event_with_filter(Default::default())
                }
            }
        } else {
            quote! {}
        }
    }

    /// Expands an event property type.
    ///
    /// Note that this is slightly different from expanding a Solidity type as
    /// complex types like arrays and strings get emitted as hashes when they are
    /// indexed.
    /// If a complex types matches with a struct previously parsed by the AbiParser,
    /// we can replace it
    fn expand_input_type(&self, input: &EventParam) -> Result<TokenStream> {
        let ethers_core = ethers_core_crate();
        Ok(match (&input.kind, input.indexed) {
            (ParamType::Array(ty), true) => {
                if let ParamType::Tuple(..) = **ty {
                    // represents an array of a struct
                    if let Some(ty) = self
                        .abi_parser
                        .structs
                        .get(&input.name)
                        .map(SolStruct::name)
                        .map(util::ident)
                    {
                        return Ok(quote! {::std::vec::Vec<#ty>})
                    }
                }
                quote! { #ethers_core::types::H256 }
            }
            (ParamType::FixedArray(ty, size), true) => {
                if let ParamType::Tuple(..) = **ty {
                    // represents a fixed array of a struct
                    if let Some(ty) = self
                        .abi_parser
                        .structs
                        .get(&input.name)
                        .map(SolStruct::name)
                        .map(util::ident)
                    {
                        let size = Literal::usize_unsuffixed(*size);
                        return Ok(quote! {[#ty; #size]})
                    }
                }
                quote! { #ethers_core::types::H256 }
            }
            (ParamType::Tuple(..), true) => {
                // represents a struct
                if let Some(ty) =
                    self.abi_parser.structs.get(&input.name).map(SolStruct::name).map(util::ident)
                {
                    quote! {#ty}
                } else {
                    quote! { #ethers_core::types::H256 }
                }
            }
            (ParamType::Bytes, true) | (ParamType::String, true) => {
                quote! { #ethers_core::types::H256 }
            }
            (kind, _) => types::expand(kind)?,
        })
    }

    /// Expands the name-type pairs for the given inputs
    fn expand_event_params(&self, event: &Event) -> Result<Vec<(TokenStream, TokenStream, bool)>> {
        event
            .inputs
            .iter()
            .enumerate()
            .map(|(i, input)| {
                // NOTE: Events can contain nameless values.
                let name = util::expand_input_name(i, &input.name);
                let ty = self.expand_input_type(input)?;

                Ok((name, ty, input.indexed))
            })
            .collect()
    }

    /// Expands into a single method for contracting an event stream.
    fn expand_filter(&self, event: &Event) -> TokenStream {
        let ethers_contract = ethers_contract_crate();
        let alias = self.event_aliases.get(&event.abi_signature()).cloned();

        let name = if let Some(id) = alias.clone() {
            util::safe_ident(&format!("{}_filter", id.to_string().to_snake_case()))
        } else {
            util::safe_ident(&format!("{}_filter", event.name.to_snake_case()))
        };

        // append `filter` to disambiguate with potentially conflicting
        // function names

        let result = expand_struct_name(event, alias);

        let doc = util::expand_doc(&format!("Gets the contract's `{}` event", event.name));
        quote! {
            #doc
            pub fn #name(&self) -> #ethers_contract::builders::Event<M, #result> {
                self.0.event()
            }
        }
    }

    /// Expands an ABI event into a single event data type. This can expand either
    /// into a structure or a tuple in the case where all event parameters (topics
    /// and data) are anonymous.
    fn expand_event(&self, event: &Event) -> Result<TokenStream> {
        let sig = self.event_aliases.get(&event.abi_signature()).cloned();
        let abi_signature = event.abi_signature();
        let event_abi_name = event.name.clone();

        let event_name = expand_struct_name(event, sig);

        let params = self.expand_event_params(event)?;
        // expand as a tuple if all fields are anonymous
        let all_anonymous_fields = event.inputs.iter().all(|input| input.name.is_empty());
        let data_type_definition = if all_anonymous_fields {
            expand_data_tuple(&event_name, &params)
        } else {
            expand_data_struct(&event_name, &params)
        };

        let derives = util::expand_derives(&self.event_derives);

        let ethers_contract = ethers_contract_crate();

        Ok(quote! {
            #[derive(Clone, Debug, Default, Eq, PartialEq, #ethers_contract::EthEvent, #ethers_contract::EthDisplay, #derives)]
            #[ethevent( name = #event_abi_name, abi = #abi_signature )]
            pub #data_type_definition
        })
    }
}

/// Expands an ABI event into an identifier for its event data type.
fn expand_struct_name(event: &Event, alias: Option<Ident>) -> Ident {
    // TODO: get rid of `Filter` suffix?

    let name = if let Some(id) = alias {
        format!("{}Filter", id.to_string().to_pascal_case())
    } else {
        format!("{}Filter", event.name.to_pascal_case())
    };
    util::ident(&name)
}

/// Expands an event data structure from its name-type parameter pairs. Returns
/// a tuple with the type definition (i.e. the struct declaration) and
/// construction (i.e. code for creating an instance of the event data).
fn expand_data_struct(name: &Ident, params: &[(TokenStream, TokenStream, bool)]) -> TokenStream {
    let fields = params
        .iter()
        .map(|(name, ty, indexed)| {
            if *indexed {
                quote! {
                    #[ethevent(indexed)]
                    pub #name: #ty
                }
            } else {
                quote! { pub #name: #ty }
            }
        })
        .collect::<Vec<_>>();

    quote! { struct #name { #( #fields, )* } }
}

/// Expands an event data named tuple from its name-type parameter pairs.
/// Returns a tuple with the type definition and construction.
fn expand_data_tuple(name: &Ident, params: &[(TokenStream, TokenStream, bool)]) -> TokenStream {
    let fields = params
        .iter()
        .map(|(_, ty, indexed)| {
            if *indexed {
                quote! {
                #[ethevent(indexed)] pub #ty }
            } else {
                quote! {
                pub #ty }
            }
        })
        .collect::<Vec<_>>();

    quote! { struct #name( #( #fields ),* ); }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Abigen;
    use ethers_core::abi::{EventParam, Hash, ParamType};

    /// Expands a 256-bit `Hash` into a literal representation that can be used with
    /// quasi-quoting for code generation. We do this to avoid allocating at runtime
    fn expand_hash(hash: Hash) -> TokenStream {
        let bytes = hash.as_bytes().iter().copied().map(Literal::u8_unsuffixed);
        let ethers_core = ethers_core_crate();

        quote! {
            #ethers_core::types::H256([#( #bytes ),*])
        }
    }

    fn test_context() -> Context {
        Context::from_abigen(Abigen::new("TestToken", "[]").unwrap()).unwrap()
    }

    fn test_context_with_alias(sig: &str, alias: &str) -> Context {
        Context::from_abigen(Abigen::new("TestToken", "[]").unwrap().add_event_alias(sig, alias))
            .unwrap()
    }

    #[test]
    #[rustfmt::skip]
    fn expand_transfer_filter_with_alias() {
        let event = Event {
            name: "Transfer".into(),
            inputs: vec![
                EventParam {
                    name: "from".into(),
                    kind: ParamType::Address,
                    indexed: true,
                },
                EventParam {
                    name: "to".into(),
                    kind: ParamType::Address,
                    indexed: true,
                },
                EventParam {
                    name: "amount".into(),
                    kind: ParamType::Uint(256),
                    indexed: false,
                },
            ],
            anonymous: false,
        };
        let sig = "Transfer(address,address,uint256)";
        let cx = test_context_with_alias(sig, "TransferEvent");
        assert_quote!(cx.expand_filter(&event), {
            #[doc = "Gets the contract's `Transfer` event"]
            pub fn transfer_event_filter(
                &self
            ) -> ethers_contract::builders::Event<M, TransferEventFilter> {
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
        assert_quote!(cx.expand_filter(&event), {
            #[doc = "Gets the contract's `Transfer` event"]
            pub fn transfer_filter(&self) -> ethers_contract::builders::Event<M, TransferFilter> {
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
        let params = cx.expand_event_params(&event).unwrap();
        let name = expand_struct_name(&event, None);
        let definition = expand_data_struct(&name, &params);

        assert_quote!(definition, {
            struct FooFilter {
                pub a: bool,
                pub p1: ethers_core::types::Address,
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
        let params = cx.expand_event_params(&event).unwrap();
        let alias = Some(util::ident("FooAliased"));
        let name = expand_struct_name(&event, alias);
        let definition = expand_data_struct(&name, &params);

        assert_quote!(definition, {
            struct FooAliasedFilter {
                pub a: bool,
                pub p1: ethers_core::types::Address,
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
        let params = cx.expand_event_params(&event).unwrap();
        let name = expand_struct_name(&event, None);
        let definition = expand_data_tuple(&name, &params);

        assert_quote!(definition, {
            struct FooFilter(pub bool, pub ethers_core::types::Address);
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
        let params = cx.expand_event_params(&event).unwrap();
        let alias = Some(util::ident("FooAliased"));
        let name = expand_struct_name(&event, alias);
        let definition = expand_data_tuple(&name, &params);

        assert_quote!(definition, {
            struct FooAliasedFilter(pub bool, pub ethers_core::types::Address);
        });
    }

    #[test]
    #[rustfmt::skip]
    fn expand_hash_value() {
        assert_quote!(
            expand_hash(
                "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f".parse().unwrap()
            ),
            {
                ethers_core::types::H256([
                    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
                    16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31
                ])
            },
        );
    }
}
