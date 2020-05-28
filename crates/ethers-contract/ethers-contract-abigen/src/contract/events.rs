use super::{types, util, Context};
use ethers_types::abi::{Event, EventExt, EventParam, Hash, ParamType};

use anyhow::Result;
use inflector::Inflector;
use proc_macro2::{Literal, TokenStream};
use quote::quote;
use syn::Path;

impl Context {
    /// Expands each event to a struct + its impl Detokenize block
    pub fn events_declaration(&self) -> Result<TokenStream> {
        let data_types = self
            .abi
            .events()
            .map(|event| expand_event(event, &self.event_derives))
            .collect::<Result<Vec<_>>>()?;

        if data_types.is_empty() {
            return Ok(quote! {});
        }

        Ok(quote! {
            #( #data_types )*
        })
    }

    pub fn events(&self) -> Result<TokenStream> {
        let data_types = self
            .abi
            .events()
            .map(|event| expand_filter(event))
            .collect::<Result<Vec<_>>>()?;

        if data_types.is_empty() {
            return Ok(quote! {});
        }

        Ok(quote! {
            #( #data_types )*
        })
    }
}

/// Expands into a single method for contracting an event stream.
fn expand_filter(event: &Event) -> Result<TokenStream> {
    let name = util::safe_ident(&event.name.to_snake_case());
    let ev_name = Literal::string(&event.name);
    let result = util::ident(&event.name.to_pascal_case());

    let doc = util::expand_doc(&format!("Gets the contract's `{}` event", event.name));
    Ok(quote! {

        #doc
        pub fn #name<'b>(&'a self) -> Event<'a, 'b, P, N, #result> where 'a: 'b, {
            self.0.event(#ev_name).expect("event not found (this should never happen)")
        }
    })
}

/// Expands an ABI event into a single event data type. This can expand either
/// into a structure or a tuple in the case where all event parameters (topics
/// and data) are anonymous.
fn expand_event(event: &Event, event_derives: &[Path]) -> Result<TokenStream> {
    let event_name = expand_struct_name(event);

    let signature = expand_hash(event.signature());

    let abi_signature = event.abi_signature();
    let abi_signature_lit = Literal::string(&abi_signature);
    let abi_signature_doc = util::expand_doc(&format!("`{}`", abi_signature));

    let params = expand_params(event)?;

    // expand as a tuple if all fields are anonymous
    let all_anonymous_fields = event.inputs.iter().all(|input| input.name.is_empty());
    let (data_type_definition, data_type_construction) = if all_anonymous_fields {
        expand_data_tuple(&event_name, &params)
    } else {
        expand_data_struct(&event_name, &params)
    };

    // read each token parameter as the required data type
    let params_len = Literal::usize_unsuffixed(params.len());
    let read_param_token = params
        .iter()
        .map(|(name, ty)| {
            quote! {
                let #name = #ty::from_token(tokens.next().expect("this should never happen"))?;
            }
        })
        .collect::<Vec<_>>();

    let derives = expand_derives(event_derives);

    Ok(quote! {
        #[derive(Clone, Debug, Default, Eq, PartialEq, #derives)]
        pub #data_type_definition

        impl #event_name {
            /// Retrieves the signature for the event this data corresponds to.
            /// This signature is the Keccak-256 hash of the ABI signature of
            /// this event.
            pub const fn signature() -> H256 {
                #signature
            }

            /// Retrieves the ABI signature for the event this data corresponds
            /// to. For this event the value should always be:
            ///
            #abi_signature_doc
            pub const fn abi_signature() -> &'static str {
                #abi_signature_lit
            }
        }

        impl Detokenize for #event_name {
            fn from_tokens(
                tokens: Vec<Token>,
            ) -> Result<Self, InvalidOutputType> {
                if tokens.len() != #params_len {
                    return Err(InvalidOutputType(format!(
                        "Expected {} tokens, got {}: {:?}",
                        #params_len,
                        tokens.len(),
                        tokens
                    )));
                }

                #[allow(unused_mut)]
                let mut tokens = tokens.into_iter();
                #( #read_param_token )*

                Ok(#data_type_construction)
            }
        }
    })
}

/// Expands an ABI event into an identifier for its event data type.
fn expand_struct_name(event: &Event) -> TokenStream {
    let event_name = util::ident(&event.name.to_pascal_case());
    quote! { #event_name }
}

/// Expands an ABI event into name-type pairs for each of its parameters.
fn expand_params(event: &Event) -> Result<Vec<(TokenStream, TokenStream)>> {
    event
        .inputs
        .iter()
        .enumerate()
        .map(|(i, input)| {
            // NOTE: Events can contain nameless values.
            let name = util::expand_input_name(i, &input.name);
            let ty = expand_input_type(&input)?;

            Ok((name, ty))
        })
        .collect()
}

/// Expands an event data structure from its name-type parameter pairs. Returns
/// a tuple with the type definition (i.e. the struct declaration) and
/// construction (i.e. code for creating an instance of the event data).
fn expand_data_struct(
    name: &TokenStream,
    params: &[(TokenStream, TokenStream)],
) -> (TokenStream, TokenStream) {
    let fields = params
        .iter()
        .map(|(name, ty)| quote! { pub #name: #ty })
        .collect::<Vec<_>>();

    let param_names = params
        .iter()
        .map(|(name, _)| name)
        .cloned()
        .collect::<Vec<_>>();

    let definition = quote! { struct #name { #( #fields, )* } };
    let construction = quote! { #name { #( #param_names ),* } };

    (definition, construction)
}

/// Expands an event data named tuple from its name-type parameter pairs.
/// Returns a tuple with the type definition and construction.
fn expand_data_tuple(
    name: &TokenStream,
    params: &[(TokenStream, TokenStream)],
) -> (TokenStream, TokenStream) {
    let fields = params
        .iter()
        .map(|(_, ty)| quote! { pub #ty })
        .collect::<Vec<_>>();

    let param_names = params
        .iter()
        .map(|(name, _)| name)
        .cloned()
        .collect::<Vec<_>>();

    let definition = quote! { struct #name( #( #fields ),* ); };
    let construction = quote! { #name( #( #param_names ),* ) };

    (definition, construction)
}

/// Expands an ABI event into filter methods for its indexed parameters.
fn expand_builder_topic_filters(event: &Event) -> Result<TokenStream> {
    let topic_filters = event
        .inputs
        .iter()
        .filter(|input| input.indexed)
        .enumerate()
        .map(|(topic_index, input)| expand_builder_topic_filter(topic_index, input))
        .collect::<Result<Vec<_>>>()?;

    Ok(quote! {
        #( #topic_filters )*
    })
}

/// Expands a event parameter into an event builder filter method for the
/// specified topic index.
fn expand_builder_topic_filter(topic_index: usize, param: &EventParam) -> Result<TokenStream> {
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
    let ty = expand_input_type(&param)?;

    Ok(quote! {
        #doc
        pub fn #name(mut self, topic: Topic<#ty>) -> Self {
            self.0 = (self.0).#topic(topic);
            self
        }
    })
}

/// Expands an ABI event into an identifier for its event data type.
fn expand_builder_name(event: &Event) -> TokenStream {
    let builder_name = util::ident(&format!("{}Builder", &event.name.to_pascal_case()));
    quote! { #builder_name }
}

fn expand_derives(derives: &[Path]) -> TokenStream {
    quote! {#(#derives),*}
}

/// Expands an event property type.
///
/// Note that this is slightly different than an expanding a Solidity type as
/// complex types like arrays and strings get emited as hashes when they are
/// indexed.
fn expand_input_type(input: &EventParam) -> Result<TokenStream> {
    Ok(match (&input.kind, input.indexed) {
        (ParamType::Array(..), true)
        | (ParamType::Bytes, true)
        | (ParamType::FixedArray(..), true)
        | (ParamType::String, true)
        | (ParamType::Tuple(..), true) => {
            quote! { H256 }
        }
        (kind, _) => types::expand(kind)?,
    })
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
    use ethers_types::abi::{EventParam, ParamType};

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

        assert_quote!(expand_filter(&event).unwrap(), {
            #[doc = "Gets the contract's `Transfer` event"]
            pub fn transfer<'b>(&'a self) -> Event<'a, 'b, P, N, Transfer>
            where
                'a: 'b,
            {
                self.0
                    .event("Transfer")
                    .expect("event not found (this should never happen)")
            }
        });
    }

    // #[test]
    // fn expand_transfer_builder_topic_filters() {
    //     let event = Event {
    //         name: "Transfer".into(),
    //         inputs: vec![
    //             EventParam {
    //                 name: "from".into(),
    //                 kind: ParamType::Address,
    //                 indexed: true,
    //             },
    //             EventParam {
    //                 name: "to".into(),
    //                 kind: ParamType::Address,
    //                 indexed: true,
    //             },
    //             EventParam {
    //                 name: "amount".into(),
    //                 kind: ParamType::Uint(256),
    //                 indexed: false,
    //             },
    //         ],
    //         anonymous: false,
    //     };

    //     #[rustfmt::skip]
    //     assert_quote!(expand_builder_topic_filters(&event).unwrap(), {
    //         #[doc = "Adds a filter for the from event parameter."]
    //         pub fn from(mut self, topic: self::ethcontract::Topic<self::ethcontract::Address>) -> Self {
    //             self.0 = (self.0).topic0(topic);
    //             self
    //         }

    //         #[doc = "Adds a filter for the to event parameter."]
    //         pub fn to(mut self, topic: self::ethcontract::Topic<self::ethcontract::Address>) -> Self {
    //             self.0 = (self.0).topic1(topic);
    //             self
    //         }
    //     });
    // }

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
        let params = expand_params(&event).unwrap();
        let (definition, construction) = expand_data_struct(&name, &params);

        assert_quote!(definition, {
            struct Foo {
                pub a: bool,
                pub p1: Address,
            }
        });
        assert_quote!(construction, { Foo { a, p1 } });
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
        let params = expand_params(&event).unwrap();
        let (definition, construction) = expand_data_tuple(&name, &params);

        assert_quote!(definition, {
            struct Foo(pub bool, pub Address);
        });
        assert_quote!(construction, { Foo(p0, p1) });
    }

    // #[test]
    // fn expand_enum_for_all_events() {
    //     let context = {
    //         let mut context = Context::default();
    //         context.abi.events.insert(
    //             "Foo".into(),
    //             vec![Event {
    //                 name: "Foo".into(),
    //                 inputs: vec![EventParam {
    //                     name: String::new(),
    //                     kind: ParamType::Bool,
    //                     indexed: false,
    //                 }],
    //                 anonymous: false,
    //             }],
    //         );
    //         context.abi.events.insert(
    //             "Bar".into(),
    //             vec![Event {
    //                 name: "Bar".into(),
    //                 inputs: vec![EventParam {
    //                     name: String::new(),
    //                     kind: ParamType::Address,
    //                     indexed: false,
    //                 }],
    //                 anonymous: true,
    //             }],
    //         );
    //         context.event_derives = ["Asdf", "a::B", "a::b::c::D"]
    //             .iter()
    //             .map(|derive| syn::parse_str::<Path>(derive).unwrap())
    //             .collect();
    //         context
    //     };

    //     assert_quote!(expand_event_enum(&context), {
    //         /// A contract event.
    //         #[derive(Clone, Debug, Eq, PartialEq, Asdf, a::B, a::b::c::D)]
    //         pub enum Event {
    //             Bar(self::event_data::Bar),
    //             Foo(self::event_data::Foo),
    //         }
    //     });
    // }

    #[test]
    // fn expand_parse_log_impl_for_all_events() {
    //     let context = {
    //         let mut context = Context::default();
    //         context.abi.events.insert(
    //             "Foo".into(),
    //             vec![Event {
    //                 name: "Foo".into(),
    //                 inputs: vec![EventParam {
    //                     name: String::new(),
    //                     kind: ParamType::Bool,
    //                     indexed: false,
    //                 }],
    //                 anonymous: false,
    //             }],
    //         );
    //         context.abi.events.insert(
    //             "Bar".into(),
    //             vec![Event {
    //                 name: "Bar".into(),
    //                 inputs: vec![EventParam {
    //                     name: String::new(),
    //                     kind: ParamType::Address,
    //                     indexed: false,
    //                 }],
    //                 anonymous: true,
    //             }],
    //         );
    //         context
    //     };

    //     let foo_signature = expand_hash(context.abi.event("Foo").unwrap().signature());
    //     let invalid_data = expand_invalid_data();

    //     assert_quote!(expand_event_parse_log(&context), {
    //         impl self::ethcontract::contract::ParseLog for Event {
    //             fn parse_log(
    //                 log: self::ethcontract::RawLog,
    //             ) -> Result<Self, self::ethcontract::errors::ExecutionError> {
    //                 let standard_event = log.topics
    //                     .get(0)
    //                     .copied()
    //                     .map(|topic| match topic {
    //                         #foo_signature => Ok(Event::Foo(
    //                             log.clone().decode(
    //                                 &Contract::artifact()
    //                                     .abi
    //                                     .event("Foo")
    //                                     .expect("generated event decode")
    //                             )?
    //                         )),
    //                         _ => #invalid_data,
    //                     });

    //                 if let Some(Ok(data)) = standard_event {
    //                     return Ok(data);
    //                 }

    //                 if let Ok(data) = log.clone().decode(
    //                     &Contract::artifact()
    //                         .abi
    //                         .event("Bar")
    //                         .expect("generated event decode")
    //                 ) {
    //                     return Ok(Event::Bar(data));
    //                 }

    //                 #invalid_data
    //             }
    //         }
    //     });
    // }

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
