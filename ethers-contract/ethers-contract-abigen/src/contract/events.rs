use super::{types, util, Context};
use anyhow::Result;
use ethers_core::abi::{Event, EventExt, EventParam, Hash, ParamType, SolStruct};
use inflector::Inflector;
use proc_macro2::{Literal, TokenStream};
use quote::quote;
use std::collections::BTreeMap;
use syn::Path;

impl Context {
    /// Expands each event to a struct + its impl Detokenize block
    pub fn events_declaration(&self) -> Result<TokenStream> {
        let sorted_events: BTreeMap<_, _> = self.abi.events.clone().into_iter().collect();
        let data_types = sorted_events
            .values()
            .flatten()
            .map(|event| self.expand_event(event))
            .collect::<Result<Vec<_>>>()?;

        if data_types.is_empty() {
            return Ok(quote! {});
        }

        Ok(quote! {
            #( #data_types )*
        })
    }

    /// Generate the event filter methods for the contract
    pub fn event_methods(&self) -> Result<TokenStream> {
        let sorted_events: BTreeMap<_, _> = self.abi.events.clone().into_iter().collect();
        let data_types = sorted_events
            .values()
            .flatten()
            .map(|event| self.expand_filter(event))
            .collect::<Vec<_>>();

        if data_types.is_empty() {
            return Ok(quote! {});
        }

        Ok(quote! {
            #( #data_types )*
        })
    }

    /// Expands an event property type.
    ///
    /// Note that this is slightly different than an expanding a Solidity type as
    /// complex types like arrays and strings get emited as hashes when they are
    /// indexed.
    /// If a complex types matches with a struct previously parsed by the AbiParser,
    /// we can replace it
    fn expand_input_type(&self, input: &EventParam) -> Result<TokenStream> {
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
                        return Ok(quote! {::std::vec::Vec<#ty>});
                    }
                }
                quote! { H256 }
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
                        return Ok(quote! {[#ty; #size]});
                    }
                }
                quote! { H256 }
            }
            (ParamType::Tuple(..), true) => {
                // represents an struct
                if let Some(ty) = self
                    .abi_parser
                    .structs
                    .get(&input.name)
                    .map(SolStruct::name)
                    .map(util::ident)
                {
                    quote! {#ty}
                } else {
                    quote! { H256 }
                }
            }
            (ParamType::Bytes, true) | (ParamType::String, true) => {
                quote! { H256 }
            }
            (kind, _) => types::expand(kind)?,
        })
    }

    /// Expands an ABI event into name-type pairs for each of its parameters.
    fn expand_params(&self, event: &Event) -> Result<Vec<(TokenStream, TokenStream)>> {
        event
            .inputs
            .iter()
            .enumerate()
            .map(|(i, input)| {
                // NOTE: Events can contain nameless values.
                let name = util::expand_input_name(i, &input.name);
                let ty = self.expand_input_type(&input)?;

                Ok((name, ty))
            })
            .collect()
    }

    /// Expands into a single method for contracting an event stream.
    fn expand_filter(&self, event: &Event) -> TokenStream {
        // append `filter` to disambiguate with potentially conflicting
        // function names
        let name = util::safe_ident(&format!("{}_filter", event.name.to_snake_case()));
        // let result = util::ident(&event.name.to_pascal_case());
        let result = expand_struct_name(event);
        let ev_name = Literal::string(&event.name);

        let doc = util::expand_doc(&format!("Gets the contract's `{}` event", event.name));
        quote! {
            #doc
            pub fn #name(&self) -> Event<M, #result> {
                self.0.event(#ev_name).expect("event not found (this should never happen)")
            }
        }
    }

    /// Expands an ABI event into a single event data type. This can expand either
    /// into a structure or a tuple in the case where all event parameters (topics
    /// and data) are anonymous.
    fn expand_event(&self, event: &Event) -> Result<TokenStream> {
        let event_name = expand_struct_name(event);

        let params = self.expand_params(event)?;
        // expand as a tuple if all fields are anonymous
        let all_anonymous_fields = event.inputs.iter().all(|input| input.name.is_empty());
        let data_type_definition = if all_anonymous_fields {
            expand_data_tuple(&event_name, &params)
        } else {
            expand_data_struct(&event_name, &params)
        };

        let derives = expand_derives(&self.event_derives);
        let abi_signature = event.abi_signature();
        let event_abi_name = &event.name;

        Ok(quote! {
            #[derive(Clone, Debug, Default, Eq, PartialEq, ethers::contract::EthEvent, #derives)]
            #[ethevent( name = #event_abi_name, abi = #abi_signature )]
            pub #data_type_definition
        })
    }

    /// Expands a event parameter into an event builder filter method for the
    /// specified topic index.
    fn expand_builder_topic_filter(
        &self,
        topic_index: usize,
        param: &EventParam,
    ) -> Result<TokenStream> {
        let doc = util::expand_doc(&format!(
            "Adds a filter for the `{}` event parameter.",
            param.name,
        ));
        let topic = util::ident(&format!("topic{}", topic_index));
        let name = if param.name.is_empty() {
            topic.clone()
        } else {
            util::safe_ident(&param.name.to_snake_case())
        };
        let ty = self.expand_input_type(&param)?;

        Ok(quote! {
            #doc
            pub fn #name(mut self, topic: Topic<#ty>) -> Self {
                self.0 = (self.0).#topic(topic);
                self
            }
        })
    }

    /// Expands an ABI event into filter methods for its indexed parameters.
    fn expand_builder_topic_filters(&self, event: &Event) -> Result<TokenStream> {
        let topic_filters = event
            .inputs
            .iter()
            .filter(|input| input.indexed)
            .enumerate()
            .map(|(topic_index, input)| self.expand_builder_topic_filter(topic_index, input))
            .collect::<Result<Vec<_>>>()?;

        Ok(quote! {
            #( #topic_filters )*
        })
    }
}

/// Expands an ABI event into an identifier for its event data type.
fn expand_struct_name(event: &Event) -> TokenStream {
    // TODO: get rid of `Filter` suffix?
    let name = format!("{}Filter", event.name.to_pascal_case());
    let event_name = util::ident(&name);
    quote! { #event_name }
}

/// Expands an event data structure from its name-type parameter pairs. Returns
/// a tuple with the type definition (i.e. the struct declaration) and
/// construction (i.e. code for creating an instance of the event data).
fn expand_data_struct(name: &TokenStream, params: &[(TokenStream, TokenStream)]) -> TokenStream {
    let fields = params
        .iter()
        .map(|(name, ty)| quote! { pub #name: #ty })
        .collect::<Vec<_>>();

    quote! { struct #name { #( #fields, )* } }
}

/// Expands an event data named tuple from its name-type parameter pairs.
/// Returns a tuple with the type definition and construction.
fn expand_data_tuple(name: &TokenStream, params: &[(TokenStream, TokenStream)]) -> TokenStream {
    let fields = params
        .iter()
        .map(|(_, ty)| quote! { pub #ty })
        .collect::<Vec<_>>();

    quote! { struct #name( #( #fields ),* ); }
}

/// Expands an ABI event into an identifier for its event data type.
fn expand_builder_name(event: &Event) -> TokenStream {
    let builder_name = util::ident(&format!("{}Builder", &event.name.to_pascal_case()));
    quote! { #builder_name }
}

fn expand_derives(derives: &[Path]) -> TokenStream {
    quote! {#(#derives),*}
}

/// Expands a 256-bit `Hash` into a literal representation that can be used with
/// quasi-quoting for code generation. We do this to avoid allocating at runtime
fn expand_hash(hash: Hash) -> TokenStream {
    let bytes = hash.as_bytes().iter().copied().map(Literal::u8_unsuffixed);

    quote! {
        H256([#( #bytes ),*])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethers_core::abi::{EventParam, ParamType};

    #[test]
    fn expand_transfer_filter() {
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

        // assert_quote!(expand_filter(&event), {
        //     #[doc = "Gets the contract's `Transfer` event"]
        //     pub fn transfer_filter(&self) -> Event<M, TransferFilter> {
        //         self.0
        //             .event("Transfer")
        //             .expect("event not found (this should never happen)")
        //     }
        // });
    }

    #[test]
    fn expand_data_struct_value() {
        let event = Event {
            name: "Foo".into(),
            inputs: vec![
                EventParam {
                    name: "a".into(),
                    kind: ParamType::Bool,
                    indexed: false,
                },
                EventParam {
                    name: String::new(),
                    kind: ParamType::Address,
                    indexed: false,
                },
            ],
            anonymous: false,
        };

        let name = expand_struct_name(&event);
        // let params = expand_params(&event).unwrap();
        // let definition = expand_data_struct(&name, &params);
        //
        // assert_quote!(definition, {
        //     struct FooFilter {
        //         pub a: bool,
        //         pub p1: Address,
        //     }
        // });
    }

    #[test]
    fn expand_data_tuple_value() {
        let event = Event {
            name: "Foo".into(),
            inputs: vec![
                EventParam {
                    name: String::new(),
                    kind: ParamType::Bool,
                    indexed: false,
                },
                EventParam {
                    name: String::new(),
                    kind: ParamType::Address,
                    indexed: false,
                },
            ],
            anonymous: false,
        };

        let name = expand_struct_name(&event);
        // let params = expand_params(&event).unwrap();
        // let definition = expand_data_tuple(&name, &params);
        //
        // assert_quote!(definition, {
        //     struct FooFilter(pub bool, pub Address);
        // });
    }

    #[test]
    #[rustfmt::skip]
    fn expand_hash_value() {
        assert_quote!(
            expand_hash(
                "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f".parse().unwrap()
            ),
            {
                H256([
                    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
                    16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31
                ])
            },
        );
    }
}
