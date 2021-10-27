//! Helper functions for deriving `EthCall`

use ethers_contract_abigen::{ethers_contract_crate, ethers_core_crate};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::spanned::Spanned as _;
use syn::{parse::Error, AttrStyle, DeriveInput, Lit, Meta, NestedMeta};

use ethers_core::abi::{param_type::Reader, AbiParser, Function, FunctionExt, Param, ParamType};

use crate::abi_ty;
use crate::utils;

/// Generates the `ethcall` trait support
pub(crate) fn derive_eth_call_impl(input: DeriveInput) -> TokenStream {
    // the ethers crates to use
    let core_crate = ethers_core_crate();
    let contract_crate = ethers_contract_crate();

    let name = &input.ident;
    let attributes = match parse_call_attributes(&input) {
        Ok(attributes) => attributes,
        Err(errors) => return errors,
    };

    let function_call_name = attributes
        .name
        .map(|(s, _)| s)
        .unwrap_or_else(|| input.ident.to_string());

    let mut function = if let Some((src, span)) = attributes.abi {
        // try to parse as solidity function
        if let Ok(fun) = parse_function(&src) {
            fun
        } else {
            // try as tuple
            if let Some(inputs) = Reader::read(
                src.trim_start_matches("function ")
                    .trim_start()
                    .trim_start_matches(&function_call_name),
            )
            .ok()
            .and_then(|param| match param {
                ParamType::Tuple(params) => Some(
                    params
                        .into_iter()
                        .map(|kind| Param {
                            name: "".to_string(),
                            kind,
                            internal_type: None,
                        })
                        .collect(),
                ),
                _ => None,
            }) {
                #[allow(deprecated)]
                Function {
                    name: function_call_name.clone(),
                    inputs,
                    outputs: vec![],
                    constant: false,
                    state_mutability: Default::default(),
                }
            } else {
                return Error::new(span, format!("Unable to determine ABI: {}", src))
                    .to_compile_error();
            }
        }
    } else {
        // // try to determine the abi from the fields
        match derive_abi_function_from_fields(&input) {
            Ok(event) => event,
            Err(err) => return err.to_compile_error(),
        }
    };

    function.name = function_call_name.clone();

    let abi = function.abi_signature();
    let selector = utils::selector(function.selector());

    let decode_impl = derive_decode_impl(&function);

    let ethcall_impl = quote! {
        impl #contract_crate::EthCall for #name {

            fn function_name() -> ::std::borrow::Cow<'static, str> {
                #function_call_name.into()
            }

            fn selector() -> #core_crate::types::Selector {
                #selector
            }

            fn abi_signature() -> ::std::borrow::Cow<'static, str> {
                #abi.into()
            }
        }

        impl  #core_crate::abi::AbiDecode for #name {
            fn decode(bytes: impl AsRef<[u8]>) -> Result<Self, #core_crate::abi::AbiError> {
                #decode_impl
            }
        }

        impl #core_crate::abi::AbiEncode for #name {
            fn encode(self) -> ::std::vec::Vec<u8> {
                let tokens =  #core_crate::abi::Tokenize::into_tokens(self);
                let selector = <Self as #contract_crate::EthCall>::selector();
                let encoded = #core_crate::abi::encode(&tokens);
                selector
                    .iter()
                    .copied()
                    .chain(encoded.into_iter())
                    .collect()
            }
        }

    };
    let tokenize_impl = abi_ty::derive_tokenizeable_impl(&input);

    quote! {
        #tokenize_impl
        #ethcall_impl
    }
}

fn derive_decode_impl(function: &Function) -> TokenStream {
    let core_crate = ethers_core_crate();
    let contract_crate = ethers_contract_crate();
    let data_types = function
        .inputs
        .iter()
        .map(|input| utils::param_type_quote(&input.kind));

    let data_types_init = quote! {let data_types = [#( #data_types ),*];};

    quote! {
        let bytes = bytes.as_ref();
        if bytes.len() < 4 || bytes[..4] != <Self as #contract_crate::EthCall>::selector() {
            return Err(#contract_crate::AbiError::WrongSelector);
        }
        #data_types_init
        let data_tokens = #core_crate::abi::decode(&data_types, &bytes[4..])?;
        Ok(<Self as #core_crate::abi::Tokenizable>::from_token( #core_crate::abi::Token::Tuple(data_tokens))?)
    }
}

/// Determine the function's ABI by parsing the AST
fn derive_abi_function_from_fields(input: &DeriveInput) -> Result<Function, Error> {
    #[allow(deprecated)]
    let function = Function {
        name: "".to_string(),
        inputs: utils::derive_abi_inputs_from_fields(input, "EthCall")?
            .into_iter()
            .map(|(name, kind)| Param {
                name,
                kind,
                internal_type: None,
            })
            .collect(),
        outputs: vec![],
        constant: false,
        state_mutability: Default::default(),
    };
    Ok(function)
}

/// All the attributes the `EthCall` macro supports
#[derive(Default)]
struct EthCallAttributes {
    name: Option<(String, Span)>,
    abi: Option<(String, Span)>,
}

/// extracts the attributes from the struct annotated with `EthCall`
fn parse_call_attributes(input: &DeriveInput) -> Result<EthCallAttributes, TokenStream> {
    let mut result = EthCallAttributes::default();
    for a in input.attrs.iter() {
        if let AttrStyle::Outer = a.style {
            if let Ok(Meta::List(meta)) = a.parse_meta() {
                if meta.path.is_ident("ethcall") {
                    for n in meta.nested.iter() {
                        if let NestedMeta::Meta(meta) = n {
                            match meta {
                                Meta::Path(path) => {
                                    return Err(Error::new(
                                        path.span(),
                                        "unrecognized ethcall parameter",
                                    )
                                    .to_compile_error());
                                }
                                Meta::List(meta) => {
                                    return Err(Error::new(
                                        meta.path.span(),
                                        "unrecognized ethcall parameter",
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
                                    } else {
                                        return Err(Error::new(
                                            meta.span(),
                                            "unrecognized ethcall parameter",
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

fn parse_function(abi: &str) -> Result<Function, String> {
    let abi = if !abi.trim_start().starts_with("function ") {
        format!("function {}", abi)
    } else {
        abi.to_string()
    };

    AbiParser::default()
        .parse_function(&abi)
        .map_err(|err| format!("Failed to parse the function ABI: {:?}", err))
}
