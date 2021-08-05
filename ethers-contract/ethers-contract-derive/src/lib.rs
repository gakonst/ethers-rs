//! Implementation of procedural macro for generating type-safe bindings to an
//! ethereum smart contract.
#![deny(missing_docs, unsafe_code)]

use ethers_contract_abigen::{ethers_contract_crate, ethers_core_crate, Source};
use proc_macro::TokenStream;
use proc_macro2::{Literal, Span};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned as _;
use syn::{
    parse::Error, parse_macro_input, AttrStyle, Data, DeriveInput, Expr, Field, Fields,
    GenericArgument, Lit, Meta, NestedMeta, PathArguments, Type,
};

use abigen::{expand, ContractArgs};
use ethers_core::abi::{param_type::Reader, AbiParser, Event, EventExt, EventParam, ParamType};
use hex::FromHex;
use spanned::Spanned;

mod abigen;
mod spanned;

/// Proc macro to generate type-safe bindings to a contract. This macro accepts
/// an Ethereum contract ABI or a path. Note that this path is rooted in
/// the crate's root `CARGO_MANIFEST_DIR`.
///
/// # Examples
///
/// ```ignore
/// # use ethers_contract_derive::abigen;
/// // ABI Path
/// abigen!(MyContract, "MyContract.json");
///
/// // HTTP(S) source
/// abigen!(MyContract, "https://my.domain.local/path/to/contract.json");
///
/// // Etherscan.io
/// abigen!(MyContract, "etherscan:0x0001020304050607080910111213141516171819");
/// abigen!(MyContract, "https://etherscan.io/address/0x0001020304050607080910111213141516171819");
///
/// // npmjs
/// abigen!(MyContract, "npm:@org/package@1.0.0/path/to/contract.json");
/// ```
///
/// Note that Etherscan rate-limits requests to their API, to avoid this an
/// `ETHERSCAN_API_KEY` environment variable can be set. If it is, it will use
/// that API key when retrieving the contract ABI.
///
/// Currently the proc macro accepts additional parameters to configure some
/// aspects of the code generation. Specifically it accepts:
/// - `methods`: A list of mappings from method signatures to method names
///   allowing methods names to be explicitely set for contract methods. This
///   also provides a workaround for generating code for contracts with multiple
///   methods with the same name.
/// - `event_derives`: A list of additional derives that should be added to
///   contract event structs and enums.
///
/// ```ignore
/// abigen!(
///     MyContract,
///     "path/to/MyContract.json",
///     methods {
///         myMethod(uint256,bool) as my_renamed_method;
///     },
///     event_derives (serde::Deserialize, serde::Serialize),
/// );
/// ```
#[proc_macro]
pub fn abigen(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as Spanned<ContractArgs>);

    let span = args.span();
    expand(args.into_inner())
        .unwrap_or_else(|e| Error::new(span, format!("{:?}", e)).to_compile_error())
        .into()
}

/// Derives the `EthEvent` and `Tokenizeable` trait for the labeled type.
///
/// Additional arguments can be specified using the `#[ethevent(...)]` attribute:
///
/// For the struct:
///
/// - `name`, `name = "..."`: Overrides the generated `EthEvent` name, default is the
///  struct's name.
/// - `signature`, `signature = "..."`: The signature as hex string to override the
///  event's signature.
/// - `abi`, `abi = "..."`: The ABI signature for the event this event's data corresponds to.
///  The `abi` should be solidity event definition or a tuple of the event's types in case the
///  event has non elementary (other `EthAbiType`) types as members
/// - `anonymous`: A flag to mark this as an anonymous event
///
/// For fields:
///
/// - `indexed`: flag to mark a field as an indexed event input
/// - `name`: override the name of an indexed event input, default is the rust field name
///
/// # Example
/// ```ignore
/// # use ethers_core::types::Address;
///
/// #[derive(Debug, EthAbiType)]
/// struct Inner {
///     inner: Address,
///     msg: String,
/// }
///
/// #[derive(Debug, EthEvent)]
/// #[ethevent(abi = "ValueChangedEvent((address,string),string)")]
/// struct ValueChangedEvent {
///     #[ethevent(indexed, name = "_target")]
///     target: Address,
///     msg: String,
///     inner: Inner,
/// }
/// ```
#[proc_macro_derive(EthEvent, attributes(ethevent))]
pub fn derive_abi_event(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // the ethers crates to use
    let core_crate = ethers_core_crate();
    let contract_crate = ethers_contract_crate();

    let name = &input.ident;
    let attributes = match parse_attributes(&input) {
        Ok(attributes) => attributes,
        Err(errors) => return TokenStream::from(errors),
    };

    let event_name = attributes
        .name
        .map(|(s, _)| s)
        .unwrap_or_else(|| input.ident.to_string());

    let mut event = if let Some((src, span)) = attributes.abi {
        // try to parse as solidity event
        if let Ok(event) = parse_event(&src) {
            event
        } else {
            // try as tuple
            if let Some(inputs) = Reader::read(
                src.trim_start_matches("event ")
                    .trim_start()
                    .trim_start_matches(&event_name),
            )
            .ok()
            .and_then(|param| match param {
                ParamType::Tuple(params) => Some(
                    params
                        .into_iter()
                        .map(|kind| EventParam {
                            name: "".to_string(),
                            indexed: false,
                            kind,
                        })
                        .collect(),
                ),
                _ => None,
            }) {
                Event {
                    name: event_name.clone(),
                    inputs,
                    anonymous: false,
                }
            } else {
                match src.parse::<Source>().and_then(|s| s.get()) {
                    Ok(abi) => {
                        // try to derive the signature from the abi from the parsed abi
                        // TODO(mattsse): this will fail for events that contain other non elementary types in their abi
                        //  because the parser doesn't know how to substitute the types
                        //  this could be mitigated by getting the ABI of each non elementary type at runtime
                        //  and computing the the signature as `static Lazy::...`
                        match parse_event(&abi) {
                            Ok(event) => event,
                            Err(err) => {
                                return TokenStream::from(Error::new(span, err).to_compile_error())
                            }
                        }
                    }
                    Err(err) => return TokenStream::from(Error::new(span, err).to_compile_error()),
                }
            }
        }
    } else {
        // try to determine the abi from the fields
        match derive_abi_event_from_fields(&input) {
            Ok(event) => event,
            Err(err) => return TokenStream::from(err.to_compile_error()),
        }
    };

    event.name = event_name.clone();
    if let Some((anon, _)) = attributes.anonymous.as_ref() {
        event.anonymous = *anon;
    }

    let decode_log_impl = match derive_decode_from_log_impl(&input, &event) {
        Ok(log) => log,
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };

    let (abi, hash) = (event.abi_signature(), event.signature());

    let signature = if let Some((hash, _)) = attributes.signature_hash {
        signature(&hash)
    } else {
        signature(hash.as_bytes())
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

            fn decode_log(log: &#core_crate::abi::RawLog) -> Result<Self, #core_crate::abi::Error> where Self: Sized {
                #decode_log_impl
            }

            fn is_anonymous() -> bool {
                #anon
            }
        }
    };

    let tokenize_impl = derive_tokenizeable_impl(&input);

    // parse attributes abi into source
    TokenStream::from(quote! {
        #tokenize_impl
        #ethevent_impl
    })
}

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

// Converts param types for indexed parameters to bytes32 where appropriate
// This applies to strings, arrays, structs and bytes to follow the encoding of
// these indexed param types according to
// https://solidity.readthedocs.io/en/develop/abi-spec.html#encoding-of-indexed-event-parameters
fn topic_param_type_quote(kind: &ParamType) -> proc_macro2::TokenStream {
    let core_crate = ethers_core_crate();
    match kind {
        ParamType::String
        | ParamType::Bytes
        | ParamType::Array(_)
        | ParamType::FixedArray(_, _)
        | ParamType::Tuple(_) => quote! {#core_crate::abi::ParamType::FixedBytes(32)},
        ty => param_type_quote(ty),
    }
}

fn param_type_quote(kind: &ParamType) -> proc_macro2::TokenStream {
    let core_crate = ethers_core_crate();
    match kind {
        ParamType::Address => {
            quote! {#core_crate::abi::ParamType::Address}
        }
        ParamType::Bytes => {
            quote! {#core_crate::abi::ParamType::Bytes}
        }
        ParamType::Int(size) => {
            let size = Literal::usize_suffixed(*size);
            quote! {#core_crate::abi::ParamType::Int(#size)}
        }
        ParamType::Uint(size) => {
            let size = Literal::usize_suffixed(*size);
            quote! {#core_crate::abi::ParamType::Uint(#size)}
        }
        ParamType::Bool => {
            quote! {#core_crate::abi::ParamType::Bool}
        }
        ParamType::String => {
            quote! {#core_crate::abi::ParamType::String}
        }
        ParamType::Array(ty) => {
            let ty = param_type_quote(&*ty);
            quote! {#core_crate::abi::ParamType::Array(Box::new(#ty))}
        }
        ParamType::FixedBytes(size) => {
            let size = Literal::usize_suffixed(*size);
            quote! {#core_crate::abi::ParamType::FixedBytes(#size)}
        }
        ParamType::FixedArray(ty, size) => {
            let ty = param_type_quote(&*ty);
            let size = Literal::usize_suffixed(*size);
            quote! {#core_crate::abi::ParamType::FixedArray(Box::new(#ty),#size)}
        }
        ParamType::Tuple(tuple) => {
            let elements = tuple.iter().map(param_type_quote);
            quote! {
                #core_crate::abi::ParamType::Tuple(
                    ::std::vec![
                        #( #elements ),*
                    ]
                )
            }
        }
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
                    ));
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
                    ));
                }
                fields.unnamed.iter().collect()
            }
            Fields::Unit => {
                return Err(Error::new(
                    input.span(),
                    "EthEvent cannot be derived for empty structs and unit",
                ));
            }
        },
        Data::Enum(_) => {
            return Err(Error::new(
                input.span(),
                "EthEvent cannot be derived for enums",
            ));
        }
        Data::Union(_) => {
            return Err(Error::new(
                input.span(),
                "EthEvent cannot be derived for unions",
            ));
        }
    };

    let mut event_fields = Vec::with_capacity(fields.len());
    for (index, field) in fields.iter().enumerate() {
        let mut param = event.inputs[index].clone();

        let (topic_name, indexed) = parse_field_attributes(field)?;
        if indexed {
            param.indexed = true;
        }
        let topic_name = if param.indexed {
            if topic_name.is_none() {
                Some(param.name.clone())
            } else {
                topic_name
            }
        } else {
            None
        };

        event_fields.push(EventField {
            topic_name,
            index,
            param,
        });
    }

    // convert fields to params list
    let topic_types = event_fields
        .iter()
        .filter(|f| f.is_indexed())
        .map(|f| topic_param_type_quote(&f.param.kind));

    let topic_types_init = quote! {let topic_types = ::std::vec![#( #topic_types ),*];};

    let data_types = event_fields
        .iter()
        .filter(|f| !f.is_indexed())
        .map(|f| param_type_quote(&f.param.kind));

    let data_types_init = quote! {let data_types = ::std::vec![#( #data_types ),*];};

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

        #core_crate::abi::Detokenize::from_tokens(tokens).map_err(|_|#core_crate::abi::Error::InvalidData)
    })
}

fn derive_abi_event_from_fields(input: &DeriveInput) -> Result<Event, Error> {
    let fields: Vec<_> = match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => fields.named.iter().collect(),
            Fields::Unnamed(ref fields) => fields.unnamed.iter().collect(),
            Fields::Unit => {
                return Err(Error::new(
                    input.span(),
                    "EthEvent cannot be derived for empty structs and unit",
                ))
            }
        },
        Data::Enum(_) => {
            return Err(Error::new(
                input.span(),
                "EthEvent cannot be derived for enums",
            ));
        }
        Data::Union(_) => {
            return Err(Error::new(
                input.span(),
                "EthEvent cannot be derived for unions",
            ));
        }
    };

    let inputs = fields
        .iter()
        .map(|f| {
            let name = f
                .ident
                .as_ref()
                .map(|name| name.to_string())
                .unwrap_or_else(|| "".to_string());
            find_parameter_type(&f.ty).map(|ty| (name, ty))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let event = Event {
        name: "".to_string(),
        inputs: inputs
            .into_iter()
            .map(|(name, kind)| EventParam {
                name,
                kind,
                indexed: false,
            })
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
                                        ));
                                    }
                                }
                                Meta::List(meta) => {
                                    return Err(Error::new(
                                        meta.path.span(),
                                        "unrecognized ethevent parameter",
                                    ));
                                }
                                Meta::NameValue(meta) => {
                                    if meta.path.is_ident("name") {
                                        if let Lit::Str(ref lit_str) = meta.lit {
                                            topic_name = Some(lit_str.value());
                                        } else {
                                            return Err(Error::new(
                                                meta.span(),
                                                "name attribute must be a string",
                                            ));
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

fn find_parameter_type(ty: &Type) -> Result<ParamType, Error> {
    match ty {
        Type::Array(ty) => {
            let param = find_parameter_type(ty.elem.as_ref())?;
            if let Expr::Lit(ref expr) = ty.len {
                if let Lit::Int(ref len) = expr.lit {
                    if let Ok(size) = len.base10_parse::<usize>() {
                        return Ok(ParamType::FixedArray(Box::new(param), size));
                    }
                }
            }
            Err(Error::new(
                ty.span(),
                "Failed to derive proper ABI from array field",
            ))
        }
        Type::Path(ty) => {
            if let Some(ident) = ty.path.get_ident() {
                return match ident.to_string().to_lowercase().as_str() {
                    "address" => Ok(ParamType::Address),
                    "string" => Ok(ParamType::String),
                    "bool" => Ok(ParamType::Bool),
                    "int" | "uint" => Ok(ParamType::Uint(256)),
                    "h160" => Ok(ParamType::FixedBytes(20)),
                    "h256" | "secret" | "hash" => Ok(ParamType::FixedBytes(32)),
                    "h512" | "public" => Ok(ParamType::FixedBytes(64)),
                    s => parse_int_param_type(s).ok_or_else(|| {
                        Error::new(ty.span(), "Failed to derive proper ABI from fields")
                    }),
                };
            }
            // check for `Vec`
            if ty.path.segments.len() == 1 && ty.path.segments[0].ident == "Vec" {
                if let PathArguments::AngleBracketed(ref args) = ty.path.segments[0].arguments {
                    if args.args.len() == 1 {
                        if let GenericArgument::Type(ref ty) = args.args.iter().next().unwrap() {
                            let kind = find_parameter_type(ty)?;
                            return Ok(ParamType::Array(Box::new(kind)));
                        }
                    }
                }
            }

            Err(Error::new(
                ty.span(),
                "Failed to derive proper ABI from fields",
            ))
        }
        Type::Tuple(ty) => {
            let params = ty
                .elems
                .iter()
                .map(|t| find_parameter_type(t))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(ParamType::Tuple(params))
        }
        _ => Err(Error::new(
            ty.span(),
            "Failed to derive proper ABI from fields",
        )),
    }
}

fn parse_int_param_type(s: &str) -> Option<ParamType> {
    let size = s
        .chars()
        .skip(1)
        .collect::<String>()
        .parse::<usize>()
        .ok()?;
    if s.starts_with('u') {
        Some(ParamType::Uint(size))
    } else if s.starts_with('i') {
        Some(ParamType::Int(size))
    } else {
        None
    }
}

fn signature(hash: &[u8]) -> proc_macro2::TokenStream {
    let core_crate = ethers_core_crate();
    let bytes = hash.iter().copied().map(Literal::u8_unsuffixed);
    quote! {#core_crate::types::H256([#( #bytes ),*])}
}

fn parse_event(abi: &str) -> Result<Event, String> {
    let abi = if !abi.trim_start().starts_with("event ") {
        format!("event {}", abi)
    } else {
        abi.to_string()
    };
    AbiParser::default()
        .parse_event(&abi)
        .map_err(|err| format!("Failed to parse the event ABI: {:?}", err))
}

/// Derives the `Tokenizable` trait for the labeled type.
///
/// This derive macro automatically adds a type bound `field: Tokenizable` for each
/// field type.
#[proc_macro_derive(EthAbiType)]
pub fn derive_abi_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    TokenStream::from(derive_tokenizeable_impl(&input))
}

fn derive_tokenizeable_impl(input: &DeriveInput) -> proc_macro2::TokenStream {
    let core_crate = ethers_core_crate();
    let name = &input.ident;
    let generic_params = input.generics.params.iter().map(|p| quote! { #p });
    let generic_params = quote! { #(#generic_params,)* };

    let generic_args = input.generics.type_params().map(|p| {
        let name = &p.ident;
        quote_spanned! { p.ident.span() => #name }
    });

    let generic_args = quote! { #(#generic_args,)* };

    let generic_predicates = match input.generics.where_clause {
        Some(ref clause) => {
            let predicates = clause.predicates.iter().map(|p| quote! { #p });
            quote! { #(#predicates,)* }
        }
        None => quote! {},
    };

    let (tokenize_predicates, params_len, init_struct_impl, into_token_impl) = match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let tokenize_predicates = fields.named.iter().map(|f| {
                    let ty = &f.ty;
                    quote_spanned! { f.span() => #ty: #core_crate::abi::Tokenize }
                });
                let tokenize_predicates = quote! { #(#tokenize_predicates,)* };

                let assignments = fields.named.iter().map(|f| {
                    let name = f.ident.as_ref().expect("Named fields have names");
                    quote_spanned! { f.span() => #name: #core_crate::abi::Tokenizable::from_token(iter.next().expect("tokens size is sufficient qed").into_token())? }
                });
                let init_struct_impl = quote! { Self { #(#assignments,)* } };

                let into_token = fields.named.iter().map(|f| {
                    let name = f.ident.as_ref().expect("Named fields have names");
                    quote_spanned! { f.span() => self.#name.into_token() }
                });
                let into_token_impl = quote! { #(#into_token,)* };

                (
                    tokenize_predicates,
                    fields.named.len(),
                    init_struct_impl,
                    into_token_impl,
                )
            }
            Fields::Unnamed(ref fields) => {
                let tokenize_predicates = fields.unnamed.iter().map(|f| {
                    let ty = &f.ty;
                    quote_spanned! { f.span() => #ty: #core_crate::abi::Tokenize }
                });
                let tokenize_predicates = quote! { #(#tokenize_predicates,)* };

                let assignments = fields.unnamed.iter().map(|f| {
                    quote_spanned! { f.span() => #core_crate::abi::Tokenizable::from_token(iter.next().expect("tokens size is sufficient qed").into_token())? }
                });
                let init_struct_impl = quote! { Self(#(#assignments,)* ) };

                let into_token = fields.unnamed.iter().enumerate().map(|(i, f)| {
                    let idx = syn::Index::from(i);
                    quote_spanned! { f.span() => self.#idx.into_token() }
                });
                let into_token_impl = quote! { #(#into_token,)* };

                (
                    tokenize_predicates,
                    fields.unnamed.len(),
                    init_struct_impl,
                    into_token_impl,
                )
            }
            Fields::Unit => {
                return Error::new(
                    input.span(),
                    "EthAbiType cannot be derived for empty structs and unit",
                )
                .to_compile_error();
            }
        },
        Data::Enum(_) => {
            return Error::new(input.span(), "EthAbiType cannot be derived for enums")
                .to_compile_error();
        }
        Data::Union(_) => {
            return Error::new(input.span(), "EthAbiType cannot be derived for unions")
                .to_compile_error();
        }
    };

    // there might be the case that the event has only 1 params, which is not a tuple
    let (from_token_impl, into_token_impl) = match params_len {
        0 => (
            quote! {
               Ok(#init_struct_impl)
            },
            // can't encode an empty struct
            // TODO: panic instead?
            quote! {
                #core_crate::abi::Token::Tuple(Vec::new())
            },
        ),
        1 => {
            // This is a hacky solution in order to keep the same tokenstream as for tuples
            let from_token = quote! {
                let mut iter = Some(token).into_iter();
                Ok(#init_struct_impl)
            };

            // This is a hack to get rid of the trailing comma introduced in the macro that concatenates all the fields
            if let Ok(into_token) = into_token_impl
                .to_string()
                .as_str()
                .trim_end_matches(',')
                .parse()
            {
                (from_token, into_token)
            } else {
                return Error::new(input.span(), "Failed to derive Tokenizeable implementation")
                    .to_compile_error();
            }
        }
        _ => {
            let from_token = quote! {
                if let #core_crate::abi::Token::Tuple(tokens) = token {
                    if tokens.len() != #params_len {
                        return Err(#core_crate::abi::InvalidOutputType(::std::format!(
                            "Expected {} tokens, got {}: {:?}",
                            #params_len,
                            tokens.len(),
                            tokens
                        )));
                    }

                    let mut iter = tokens.into_iter();

                    Ok(#init_struct_impl)
                } else {
                    Err(#core_crate::abi::InvalidOutputType(::std::format!(
                        "Expected Tuple, got {:?}",
                        token
                    )))
                }
            };

            let into_token = quote! {
                #core_crate::abi::Token::Tuple(
                    ::std::vec![
                        #into_token_impl
                    ]
                )
            };
            (from_token, into_token)
        }
    };

    quote! {
         impl<#generic_params> #core_crate::abi::Tokenizable for #name<#generic_args>
         where
             #generic_predicates
             #tokenize_predicates
         {

             fn from_token(token: #core_crate::abi::Token) -> Result<Self, #core_crate::abi::InvalidOutputType> where
                 Self: Sized {
                #from_token_impl
             }

             fn into_token(self) -> #core_crate::abi::Token {
                #into_token_impl
             }
         }

        impl<#generic_params> #core_crate::abi::TokenizableItem for #name<#generic_args>
         where
             #generic_predicates
             #tokenize_predicates
         { }
    }
}

#[derive(Default)]
struct Attributes {
    name: Option<(String, Span)>,
    abi: Option<(String, Span)>,
    signature_hash: Option<(Vec<u8>, Span)>,
    anonymous: Option<(bool, Span)>,
}

fn parse_attributes(input: &DeriveInput) -> Result<Attributes, proc_macro2::TokenStream> {
    let mut result = Attributes::default();
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
                                                continue;
                                            } else {
                                                return Err(Error::new(
                                                    name.span(),
                                                    "anonymous already specified",
                                                )
                                                .to_compile_error());
                                            }
                                        }
                                    }
                                    return Err(Error::new(
                                        path.span(),
                                        "unrecognized ethevent parameter",
                                    )
                                    .to_compile_error());
                                }
                                Meta::List(meta) => {
                                    return Err(Error::new(
                                        meta.path.span(),
                                        "unrecognized ethevent parameter",
                                    )
                                    .to_compile_error());
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
                                                .to_compile_error());
                                            }
                                        } else {
                                            return Err(Error::new(
                                                meta.span(),
                                                "name must be a string",
                                            )
                                            .to_compile_error());
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
                                                .to_compile_error());
                                            }
                                        } else {
                                            return Err(Error::new(
                                                meta.span(),
                                                "name must be a string",
                                            )
                                            .to_compile_error());
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
                                                .to_compile_error());
                                            }
                                        } else {
                                            return Err(Error::new(
                                                meta.span(),
                                                "abi must be a string",
                                            )
                                            .to_compile_error());
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
                                                        .to_compile_error());
                                                    }
                                                }
                                            } else {
                                                return Err(Error::new(
                                                    meta.span(),
                                                    "signature already specified",
                                                )
                                                .to_compile_error());
                                            }
                                        } else {
                                            return Err(Error::new(
                                                meta.span(),
                                                "signature must be a hex string",
                                            )
                                            .to_compile_error());
                                        }
                                    } else {
                                        return Err(Error::new(
                                            meta.span(),
                                            "unrecognized ethevent parameter",
                                        )
                                        .to_compile_error());
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
