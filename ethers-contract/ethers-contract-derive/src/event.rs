//! Helper functions for deriving `EthEvent`

use crate::{abi_ty, utils};
use ethers_contract_abigen::Source;
use ethers_core::{
    abi::{Event, EventExt, EventParam, HumanReadableParser},
    macros::{ethers_contract_crate, ethers_core_crate},
};
use hex::FromHex;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse::Error, spanned::Spanned, AttrStyle, Data, DeriveInput, Field, Fields, Lit, Meta,
    NestedMeta,
};

/// Generates the `EthEvent` trait support
pub(crate) fn derive_eth_event_impl(input: DeriveInput) -> Result<TokenStream, Error> {
    let name = &input.ident;
    let attributes = parse_event_attributes(&input)?;

    let mut event = if let Some((src, span)) = attributes.abi {
        // try to parse as a Solidity event
        match HumanReadableParser::parse_event(&src) {
            Ok(event) => Ok(event),
            Err(parse_err) => {
                match src.parse::<Source>().and_then(|s| s.get()) {
                    Ok(abi) => {
                        // try to derive the signature from the abi from the parsed abi
                        // TODO(mattsse): this will fail for events that contain other non
                        // elementary types in their abi because the parser
                        // doesn't know how to substitute the types.
                        // This could be mitigated by getting the ABI of each non elementary type
                        // at runtime and computing the the signature as a Lazy static.
                        match HumanReadableParser::parse_event(&abi) {
                            Ok(event) => Ok(event),
                            // Ignore parse_err since this is a valid [Source]
                            Err(err) => Err(Error::new(span, err)),
                        }
                    }
                    Err(source_err) => {
                        // Return both error messages
                        let mut error = Error::new(span, parse_err);
                        error.combine(Error::new(span, source_err));
                        Err(error)
                    }
                }
            }
        }
    } else {
        // try to determine the abi from the fields
        derive_abi_event_from_fields(&input)
    }?;

    if let Some((attribute_name, _)) = attributes.name {
        event.name = attribute_name;
    }

    if let Some((anon, _)) = attributes.anonymous.as_ref() {
        event.anonymous = *anon;
    }

    let decode_log_impl = derive_decode_from_log_impl(&input, &event)?;

    let (abi, event_sig) = (event.abi_signature(), event.signature());

    let signature = if let Some((hash, _)) = attributes.signature_hash {
        utils::signature(&hash)
    } else {
        utils::signature(event_sig.as_bytes())
    };

    let anon = attributes.anonymous.map(|(b, _)| b).unwrap_or_default();
    let event_name = &event.name;

    let ethers_core = ethers_core_crate();
    let ethers_contract = ethers_contract_crate();

    let ethevent_impl = quote! {
        impl #ethers_contract::EthEvent for #name {

            fn name() -> ::std::borrow::Cow<'static, str> {
                #event_name.into()
            }

            fn signature() -> #ethers_core::types::H256 {
                #signature
            }

            fn abi_signature() -> ::std::borrow::Cow<'static, str> {
                #abi.into()
            }

            fn decode_log(log: &#ethers_core::abi::RawLog) -> ::core::result::Result<Self, #ethers_core::abi::Error> where Self: Sized {
                #decode_log_impl
            }

            fn is_anonymous() -> bool {
                #anon
            }
        }
    };

    let tokenize_impl = abi_ty::derive_tokenizeable_impl(&input)?;

    Ok(quote! {
        #tokenize_impl
        #ethevent_impl
    })
}

/// Internal helper type for an event/log
struct EventField {
    topic_name: Option<String>,
    index: usize,
    param: EventParam,
}

impl EventField {
    fn is_indexed(&self) -> bool {
        self.topic_name.is_some()
    }
}

fn derive_decode_from_log_impl(input: &DeriveInput, event: &Event) -> Result<TokenStream, Error> {
    let ethers_core = ethers_core_crate();

    let fields: Vec<_> = match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                if fields.named.len() != event.inputs.len() {
                    return Err(Error::new(
                        fields.span(),
                        format!(
                            "EthEvent {}'s fields length don't match with signature inputs {}",
                            event.name,
                            event.abi_signature()
                        ),
                    ))
                }
                fields.named.iter().collect()
            }
            Fields::Unnamed(ref fields) => {
                if fields.unnamed.len() != event.inputs.len() {
                    return Err(Error::new(
                        fields.span(),
                        format!(
                            "EthEvent {}'s fields length don't match with signature inputs {}",
                            event.name,
                            event.abi_signature()
                        ),
                    ))
                }
                fields.unnamed.iter().collect()
            }
            Fields::Unit => {
                // Empty structs or unit, no fields
                vec![]
            }
        },
        Data::Enum(_) => {
            return Err(Error::new(input.span(), "EthEvent cannot be derived for enums"))
        }
        Data::Union(_) => {
            return Err(Error::new(input.span(), "EthEvent cannot be derived for unions"))
        }
    };

    // decode
    let (signature_check, flat_topics_init, topic_tokens_len_check) = if event.anonymous {
        (
            None,
            quote! {
                let flat_topics = topics.iter().flat_map(|t| t.as_ref().to_vec()).collect::<Vec<u8>>();
            },
            quote! {
                if topic_tokens.len() != topics.len() {
                    return Err(#ethers_core::abi::Error::InvalidData);
                }
            },
        )
    } else {
        (
            Some(quote! {
                let event_signature = topics.get(0).ok_or(#ethers_core::abi::Error::InvalidData)?;
                if event_signature != &Self::signature() {
                    return Err(#ethers_core::abi::Error::InvalidData);
                }
            }),
            quote! {
                let flat_topics = topics.iter().skip(1).flat_map(|t| t.as_ref().to_vec()).collect::<Vec<u8>>();
            },
            quote! {
                if topic_tokens.len() != topics.len() - 1 {
                    return Err(#ethers_core::abi::Error::InvalidData);
                }
            },
        )
    };

    // Event with no fields, can skip decoding
    if fields.is_empty() {
        return Ok(quote! {

            let #ethers_core::abi::RawLog {topics, data} = log;

            #signature_check

            if topics.len() != 1usize || !data.is_empty() {
                return Err(#ethers_core::abi::Error::InvalidData);
            }

            #ethers_core::abi::Tokenizable::from_token(#ethers_core::abi::Token::Tuple(::std::vec::Vec::new())).map_err(|_|#ethers_core::abi::Error::InvalidData)
        })
    }

    let mut event_fields = Vec::with_capacity(fields.len());
    for (index, field) in fields.iter().enumerate() {
        let mut param = event.inputs[index].clone();

        let (topic_name, indexed) = parse_field_attributes(field)?;
        if indexed {
            param.indexed = true;
        }
        let topic_name =
            param.indexed.then(|| topic_name.or_else(|| Some(param.name.clone()))).flatten();

        event_fields.push(EventField { topic_name, index, param });
    }

    // convert fields to params list
    let topic_types = event_fields
        .iter()
        .filter(|f| f.is_indexed())
        .map(|f| utils::topic_param_type_quote(&f.param.kind));

    let topic_types_init = quote! {let topic_types = ::std::vec![#( #topic_types ),*];};

    let data_types = event_fields
        .iter()
        .filter(|f| !f.is_indexed())
        .map(|f| utils::param_type_quote(&f.param.kind));

    let data_types_init = quote! {let data_types = [#( #data_types ),*];};

    // check if indexed are sorted
    let tokens_init = if event_fields
        .iter()
        .filter(|f| f.is_indexed())
        .enumerate()
        .all(|(idx, f)| f.index == idx)
    {
        quote! {
            let topic_tokens = #ethers_core::abi::decode(&topic_types, &flat_topics)?;
            #topic_tokens_len_check
            let data_tokens = #ethers_core::abi::decode(&data_types, data)?;
            let tokens:Vec<_> = topic_tokens.into_iter().chain(data_tokens.into_iter()).collect();
        }
    } else {
        let swap_tokens = event_fields.iter().map(|field| {
            if field.is_indexed() {
                quote! { topic_tokens.remove(0) }
            } else {
                quote! { data_tokens.remove(0) }
            }
        });

        quote! {
            let mut topic_tokens = #ethers_core::abi::decode(&topic_types, &flat_topics)?;
            #topic_tokens_len_check
            let mut data_tokens = #ethers_core::abi::decode(&data_types, &data)?;
            let mut tokens = Vec::with_capacity(topics.len() + data_tokens.len());
            #( tokens.push(#swap_tokens); )*
        }
    };
    Ok(quote! {

        let #ethers_core::abi::RawLog {data, topics} = log;

        #signature_check

        #topic_types_init
        #data_types_init

        #flat_topics_init

        #tokens_init

        #ethers_core::abi::Tokenizable::from_token(#ethers_core::abi::Token::Tuple(tokens)).map_err(|_|#ethers_core::abi::Error::InvalidData)
    })
}

/// Determine the event's ABI by parsing the AST
fn derive_abi_event_from_fields(input: &DeriveInput) -> Result<Event, Error> {
    let event = Event {
        name: input.ident.to_string(),
        inputs: utils::derive_abi_inputs_from_fields(input, "EthEvent")?
            .into_iter()
            .map(|(name, kind)| EventParam { name, kind, indexed: false })
            .collect(),
        anonymous: false,
    };
    Ok(event)
}

fn parse_field_attributes(field: &Field) -> Result<(Option<String>, bool), Error> {
    let mut indexed = false;
    let mut topic_name = None;
    for a in field.attrs.iter() {
        if let AttrStyle::Outer = a.style {
            if let Ok(Meta::List(meta)) = a.parse_meta() {
                if meta.path.is_ident("ethevent") {
                    for n in meta.nested.iter() {
                        if let NestedMeta::Meta(meta) = n {
                            match meta {
                                Meta::Path(path) => {
                                    if path.is_ident("indexed") {
                                        indexed = true;
                                    } else {
                                        return Err(Error::new(
                                            path.span(),
                                            "unrecognized ethevent parameter",
                                        ))
                                    }
                                }
                                Meta::List(meta) => {
                                    return Err(Error::new(
                                        meta.path.span(),
                                        "unrecognized ethevent parameter",
                                    ))
                                }
                                Meta::NameValue(meta) => {
                                    if meta.path.is_ident("name") {
                                        if let Lit::Str(ref lit_str) = meta.lit {
                                            topic_name = Some(lit_str.value());
                                        } else {
                                            return Err(Error::new(
                                                meta.span(),
                                                "name attribute must be a string",
                                            ))
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok((topic_name, indexed))
}

/// All the attributes the `EthEvent` macro supports
#[derive(Default)]
struct EthEventAttributes {
    name: Option<(String, Span)>,
    abi: Option<(String, Span)>,
    signature_hash: Option<(Vec<u8>, Span)>,
    anonymous: Option<(bool, Span)>,
}

/// extracts the attributes from the struct annotated with `EthEvent`
fn parse_event_attributes(input: &DeriveInput) -> Result<EthEventAttributes, Error> {
    let mut result = EthEventAttributes::default();
    for a in input.attrs.iter() {
        if let AttrStyle::Outer = a.style {
            if let Ok(Meta::List(meta)) = a.parse_meta() {
                if meta.path.is_ident("ethevent") {
                    for n in meta.nested.iter() {
                        if let NestedMeta::Meta(meta) = n {
                            match meta {
                                Meta::Path(path) => {
                                    if let Some(name) = path.get_ident() {
                                        if &*name.to_string() == "anonymous" {
                                            if result.anonymous.is_none() {
                                                result.anonymous = Some((true, name.span()));
                                                continue
                                            } else {
                                                return Err(Error::new(
                                                    name.span(),
                                                    "anonymous already specified",
                                                ))
                                            }
                                        }
                                    }
                                    return Err(Error::new(
                                        path.span(),
                                        "unrecognized ethevent parameter",
                                    ))
                                }
                                Meta::List(meta) => {
                                    return Err(Error::new(
                                        meta.path.span(),
                                        "unrecognized ethevent parameter",
                                    ))
                                }
                                Meta::NameValue(meta) => {
                                    if meta.path.is_ident("anonymous") {
                                        if let Lit::Bool(ref bool_lit) = meta.lit {
                                            if result.anonymous.is_none() {
                                                result.anonymous =
                                                    Some((bool_lit.value, bool_lit.span()));
                                            } else {
                                                return Err(Error::new(
                                                    meta.span(),
                                                    "anonymous already specified",
                                                ))
                                            }
                                        } else {
                                            return Err(Error::new(
                                                meta.span(),
                                                "name must be a string",
                                            ))
                                        }
                                    } else if meta.path.is_ident("name") {
                                        if let Lit::Str(ref lit_str) = meta.lit {
                                            if result.name.is_none() {
                                                result.name =
                                                    Some((lit_str.value(), lit_str.span()));
                                            } else {
                                                return Err(Error::new(
                                                    meta.span(),
                                                    "name already specified",
                                                ))
                                            }
                                        } else {
                                            return Err(Error::new(
                                                meta.span(),
                                                "name must be a string",
                                            ))
                                        }
                                    } else if meta.path.is_ident("abi") {
                                        if let Lit::Str(ref lit_str) = meta.lit {
                                            if result.abi.is_none() {
                                                result.abi =
                                                    Some((lit_str.value(), lit_str.span()));
                                            } else {
                                                return Err(Error::new(
                                                    meta.span(),
                                                    "abi already specified",
                                                ))
                                            }
                                        } else {
                                            return Err(Error::new(
                                                meta.span(),
                                                "abi must be a string",
                                            ))
                                        }
                                    } else if meta.path.is_ident("signature") {
                                        if let Lit::Str(ref lit_str) = meta.lit {
                                            if result.signature_hash.is_none() {
                                                match Vec::from_hex(lit_str.value()) {
                                                    Ok(sig) => {
                                                        result.signature_hash =
                                                            Some((sig, lit_str.span()))
                                                    }
                                                    Err(err) => {
                                                        return Err(Error::new(
                                                            meta.span(),
                                                            format!(
                                                                "Expected hex signature: {err:?}"
                                                            ),
                                                        ))
                                                    }
                                                }
                                            } else {
                                                return Err(Error::new(
                                                    meta.span(),
                                                    "signature already specified",
                                                ))
                                            }
                                        } else {
                                            return Err(Error::new(
                                                meta.span(),
                                                "signature must be a hex string",
                                            ))
                                        }
                                    } else {
                                        return Err(Error::new(
                                            meta.span(),
                                            "unrecognized ethevent parameter",
                                        ))
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(result)
}
