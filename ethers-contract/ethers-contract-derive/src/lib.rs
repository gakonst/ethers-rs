//! Implementation of procedural macro for generating type-safe bindings to an
//! ethereum smart contract.
#![deny(missing_docs, unsafe_code)]

use ethers_contract_abigen::Source;
use proc_macro::TokenStream;
use proc_macro2::{Literal, Span};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned as _;
use syn::{
    parse::Error, parse_macro_input, AttrStyle, Data, DeriveInput, Expr, Fields, GenericArgument,
    Lit, Meta, NestedMeta, PathArguments, Type,
};

use abigen::{expand, ContractArgs};
use ethers_core::abi::{AbiParser, Event, EventExt, EventParam, ParamType};
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
/// - `name`, `name = "..."`: Overrides the generated `EthEvent` name, default is the struct's name.
/// - `signature`, `signature = "..."`: The signature as hex string to override the event's signature.
/// - `abi`, `abi = "..."`: The ABI signature for the event this event's data corresponds to.
#[proc_macro_derive(EthEvent, attributes(ethevent))]
pub fn derive_abi_event(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let attributes = match parse_attributes(&input) {
        Ok(attributes) => attributes,
        Err(errors) => return TokenStream::from(errors),
    };

    let event_name = attributes
        .name
        .map(|(n, _)| n)
        .unwrap_or_else(|| input.ident.to_string());

    let (abi, hash) = if let Some((src, span)) = attributes.abi {
        if let Ok(mut event) = parse_event(&src) {
            event.name = event_name.clone();
            (event.abi_signature(), event.signature())
        } else {
            match src.parse::<Source>().and_then(|s| s.get()) {
                Ok(abi) => {
                    // try to derive the signature from the abi from the parsed abi
                    // TODO(mattsse): this will fail for events that contain other non elementary types in their abi
                    //  because the parser doesn't know how to substitute the types
                    //  this could be mitigated by getting the ABI of each non elementary type at runtime
                    //  and computing the the signature as `static Lazy::...`
                    match parse_event(&abi) {
                        Ok(mut event) => {
                            event.name = event_name.clone();
                            (event.abi_signature(), event.signature())
                        }
                        Err(err) => {
                            return TokenStream::from(Error::new(span, err).to_compile_error())
                        }
                    }
                }
                Err(err) => return TokenStream::from(Error::new(span, err).to_compile_error()),
            }
        }
    } else {
        // try to determine the abi from the fields
        match derive_abi_event_from_fields(&input) {
            Ok(mut event) => {
                event.name = event_name.clone();
                (event.abi_signature(), event.signature())
            }
            Err(err) => return TokenStream::from(err.to_compile_error()),
        }
    };

    let signature = if let Some((hash, _)) = attributes.signature_hash {
        signature(&hash)
    } else {
        signature(hash.as_bytes())
    };

    let ethevent_impl = quote! {
        impl ethers_contract::EthEvent for #name {

            fn name(&self) -> ::std::borrow::Cow<'static, str> {
                #event_name.into()
            }

            fn signature() -> ethers_core::types::H256 {
                #signature
            }

            fn abi_signature() -> ::std::borrow::Cow<'static, str> {
                #abi.into()
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

fn derive_abi_event_from_fields(input: &DeriveInput) -> Result<Event, Error> {
    let types: Vec<_> = match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => fields.named.iter().map(|f| &f.ty).collect(),
            Fields::Unnamed(ref fields) => fields.unnamed.iter().map(|f| &f.ty).collect(),
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

    let inputs = types
        .iter()
        .map(|ty| find_parameter_type(ty))
        .collect::<Result<Vec<_>, _>>()?;

    let event = Event {
        name: "".to_string(),
        inputs: inputs
            .into_iter()
            .map(|kind| EventParam {
                name: "".to_string(),
                kind,
                indexed: false,
            })
            .collect(),
        anonymous: false,
    };
    Ok(event)
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
        _ => {
            eprintln!("Found other types");
            Err(Error::new(
                ty.span(),
                "Failed to derive proper ABI from fields",
            ))
        }
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
    let bytes = hash.iter().copied().map(Literal::u8_unsuffixed);
    quote! {ethers_core::types::H256([#( #bytes ),*])}
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
                    quote_spanned! { f.span() => #ty: ethers_core::abi::Tokenize }
                });
                let tokenize_predicates = quote! { #(#tokenize_predicates,)* };

                let assignments = fields.named.iter().map(|f| {
                    let name = f.ident.as_ref().expect("Named fields have names");
                    quote_spanned! { f.span() => #name: ethers_core::abi::Tokenizable::from_token(iter.next().expect("tokens size is sufficient qed").into_token())? }
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
                    quote_spanned! { f.span() => #ty: ethers_core::abi::Tokenize }
                });
                let tokenize_predicates = quote! { #(#tokenize_predicates,)* };

                let assignments = fields.unnamed.iter().map(|f| {
                    quote_spanned! { f.span() => ethers_core::abi::Tokenizable::from_token(iter.next().expect("tokens size is sufficient qed").into_token())? }
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

    quote! {
         impl<#generic_params> ethers_core::abi::Tokenizable for #name<#generic_args>
         where
             #generic_predicates
             #tokenize_predicates
         {

             fn from_token(token: ethers_core::abi::Token) -> Result<Self, ethers_core::abi::InvalidOutputType> where
                 Self: Sized {
                if let ethers_core::abi::Token::Tuple(tokens) = token {
                    if tokens.len() != #params_len {
                        return Err(ethers_core::abi::InvalidOutputType(format!(
                            "Expected {} tokens, got {}: {:?}",
                            #params_len,
                            tokens.len(),
                            tokens
                        )));
                    }

                    let mut iter = tokens.into_iter();

                    Ok(#init_struct_impl)
                } else {
                    Err(ethers_core::abi::InvalidOutputType(format!(
                        "Expected Tuple, got {:?}",
                        token
                    )))
                }
             }

             fn into_token(self) -> ethers_core::abi::Token {
                ethers_core::abi::Token::Tuple(
                    vec![
                        #into_token_impl
                    ]
                )
             }
         }
    }
}

struct Attributes {
    name: Option<(String, Span)>,
    abi: Option<(String, Span)>,
    signature_hash: Option<(Vec<u8>, Span)>,
}

impl Default for Attributes {
    fn default() -> Self {
        Self {
            name: None,
            abi: None,
            signature_hash: None,
        }
    }
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
                                    return Err(Error::new(
                                        path.span(),
                                        "unrecognized ethevent parameter",
                                    )
                                    .to_compile_error());
                                }
                                Meta::List(meta) => {
                                    // TODO support raw list
                                    return Err(Error::new(
                                        meta.path.span(),
                                        "unrecognized ethevent parameter",
                                    )
                                    .to_compile_error());
                                }
                                Meta::NameValue(meta) => {
                                    if meta.path.is_ident("name") {
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
