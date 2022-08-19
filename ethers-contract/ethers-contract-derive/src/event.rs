//! Helper functions for deriving `EthEvent`

use ethers_contract_abigen::Source;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse::Error, spanned::Spanned as _, AttrStyle, Data, DeriveInput, Field, Fields, Lit, Meta,
    NestedMeta,
};

use ethers_core::{
    abi::{Event, EventExt, EventParam, HumanReadableParser},
    macros::{ethers_contract_crate, ethers_core_crate},
};
use hex::FromHex;

use crate::{abi_ty, utils};

/// Generates the `EthEvent` trait support
pub(crate) fn derive_eth_event_impl(input: DeriveInput) -> TokenStream {
    // the ethers crates to use
    let core_crate = ethers_core_crate();
    let contract_crate = ethers_contract_crate();

    let name = &input.ident;
    let attributes = match parse_event_attributes(&input) {
        Ok(attributes) => attributes,
        Err(errors) => return errors,
    };

    let event_name = attributes.name.map(|(s, _)| s).unwrap_or_else(|| input.ident.to_string());

    let mut event = if let Some((src, span)) = attributes.abi {
        // try to parse as solidity event
        if let Ok(event) = HumanReadableParser::parse_event(&src) {
            event
        } else {
            match src.parse::<Source>().and_then(|s| s.get()) {
                Ok(abi) => {
                    // try to derive the signature from the abi from the parsed abi
                    // TODO(mattsse): this will fail for events that contain other non
                    // elementary types in their abi  because the parser
                    // doesn't know how to substitute the types
                    //  this could be mitigated by getting the ABI of each non elementary type
                    // at runtime  and computing the the signature as
                    // `static Lazy::...`
                    match HumanReadableParser::parse_event(&abi) {
                        Ok(event) => event,
                        Err(err) => return Error::new(span, err).to_compile_error(),
                    }
                }
                Err(err) => return Error::new(span, err).to_compile_error(),
            }
        }
    } else {
        // try to determine the abi from the fields
        match derive_abi_event_from_fields(&input) {
            Ok(event) => event,
            Err(err) => return err.to_compile_error(),
        }
    };

    event.name = event_name.clone();
    if let Some((anon, _)) = attributes.anonymous.as_ref() {
        event.anonymous = *anon;
    }

    let decode_log_impl = match derive_decode_from_log_impl(&input, &event) {
        Ok(log) => log,
        Err(err) => return err.to_compile_error(),
    };

    let (abi, hash) = (event.abi_signature(), event.signature());

    let signature = if let Some((hash, _)) = attributes.signature_hash {
        utils::signature(&hash)
    } else {
        utils::signature(hash.as_bytes())
    };

    let anon = attributes.anonymous.map(|(b, _)| b).unwrap_or_default();

    let ethevent_impl = quote! {
        impl #contract_crate::EthEvent for #name {

            fn name() -> ::std::borrow::Cow<'static, str> {
                #event_name.into()
            }

            fn signature() -> #core_crate::types::H256 {
                #signature
            }

            fn abi_signature() -> ::std::borrow::Cow<'static, str> {
                #abi.into()
            }

            fn decode_log(log: &#core_crate::abi::RawLog) -> ::std::result::Result<Self, #core_crate::abi::Error> where Self: Sized {
                #decode_log_impl
            }

            fn is_anonymous() -> bool {
                #anon
            }
        }
    };

    let tokenize_impl = abi_ty::derive_tokenizeable_impl(&input);

    quote! {
        #tokenize_impl
        #ethevent_impl
    }
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

fn derive_decode_from_log_impl(
    input: &DeriveInput,
    event: &Event,
) -> Result<proc_macro2::TokenStream, Error> {
    let core_crate = ethers_core_crate();

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
                return Err(Error::new(
                    input.span(),
                    "EthEvent cannot be derived for empty structs and unit",
                ))
            }
        },
        Data::Enum(_) => {
            return Err(Error::new(input.span(), "EthEvent cannot be derived for enums"))
        }
        Data::Union(_) => {
            return Err(Error::new(input.span(), "EthEvent cannot be derived for unions"))
        }
    };

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

    // decode
    let (signature_check, flat_topics_init, topic_tokens_len_check) = if event.anonymous {
        (
            quote! {},
            quote! {
                  let flat_topics = topics.iter().flat_map(|t| t.as_ref().to_vec()).collect::<Vec<u8>>();
            },
            quote! {
                if topic_tokens.len() != topics.len() {
                    return Err(#core_crate::abi::Error::InvalidData);
                }
            },
        )
    } else {
        (
            quote! {
                let event_signature = topics.get(0).ok_or(#core_crate::abi::Error::InvalidData)?;
                if event_signature != &Self::signature() {
                    return Err(#core_crate::abi::Error::InvalidData);
                }
            },
            quote! {
                let flat_topics = topics.iter().skip(1).flat_map(|t| t.as_ref().to_vec()).collect::<Vec<u8>>();
            },
            quote! {
                if topic_tokens.len() != topics.len() - 1 {
                    return Err(#core_crate::abi::Error::InvalidData);
                }
            },
        )
    };

    // check if indexed are sorted
    let tokens_init = if event_fields
        .iter()
        .filter(|f| f.is_indexed())
        .enumerate()
        .all(|(idx, f)| f.index == idx)
    {
        quote! {
            let topic_tokens = #core_crate::abi::decode(&topic_types, &flat_topics)?;
            #topic_tokens_len_check
            let data_tokens = #core_crate::abi::decode(&data_types, data)?;
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
            let mut topic_tokens = #core_crate::abi::decode(&topic_types, &flat_topics)?;
            #topic_tokens_len_check
            let mut data_tokens = #core_crate::abi::decode(&data_types, &data)?;
            let mut tokens = Vec::with_capacity(topics.len() + data_tokens.len());
            #( tokens.push(#swap_tokens); )*
        }
    };
    Ok(quote! {

        let #core_crate::abi::RawLog {data, topics} = log;

        #signature_check

        #topic_types_init
        #data_types_init

        #flat_topics_init

        #tokens_init

        #core_crate::abi::Tokenizable::from_token(#core_crate::abi::Token::Tuple(tokens)).map_err(|_|#core_crate::abi::Error::InvalidData)
    })
}

/// Determine the event's ABI by parsing the AST
fn derive_abi_event_from_fields(input: &DeriveInput) -> Result<Event, Error> {
    let event = Event {
        name: "".to_string(),
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
fn parse_event_attributes(
    input: &DeriveInput,
) -> Result<EthEventAttributes, proc_macro2::TokenStream> {
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
                                                )
                                                .to_compile_error())
                                            }
                                        }
                                    }
                                    return Err(Error::new(
                                        path.span(),
                                        "unrecognized ethevent parameter",
                                    )
                                    .to_compile_error())
                                }
                                Meta::List(meta) => {
                                    return Err(Error::new(
                                        meta.path.span(),
                                        "unrecognized ethevent parameter",
                                    )
                                    .to_compile_error())
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
                                                )
                                                .to_compile_error())
                                            }
                                        } else {
                                            return Err(Error::new(
                                                meta.span(),
                                                "name must be a string",
                                            )
                                            .to_compile_error())
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
                                                )
                                                .to_compile_error())
                                            }
                                        } else {
                                            return Err(Error::new(
                                                meta.span(),
                                                "name must be a string",
                                            )
                                            .to_compile_error())
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
                                                )
                                                .to_compile_error())
                                            }
                                        } else {
                                            return Err(Error::new(
                                                meta.span(),
                                                "abi must be a string",
                                            )
                                            .to_compile_error())
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
                                                                "Expected hex signature: {:?}",
                                                                err
                                                            ),
                                                        )
                                                        .to_compile_error())
                                                    }
                                                }
                                            } else {
                                                return Err(Error::new(
                                                    meta.span(),
                                                    "signature already specified",
                                                )
                                                .to_compile_error())
                                            }
                                        } else {
                                            return Err(Error::new(
                                                meta.span(),
                                                "signature must be a hex string",
                                            )
                                            .to_compile_error())
                                        }
                                    } else {
                                        return Err(Error::new(
                                            meta.span(),
                                            "unrecognized ethevent parameter",
                                        )
                                        .to_compile_error())
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
