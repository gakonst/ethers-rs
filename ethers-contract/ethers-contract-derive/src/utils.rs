use ethers_core::{abi::ParamType, macros::ethers_core_crate, types::Selector};
use proc_macro2::{Ident, Literal, Span};
use quote::{quote, quote_spanned};
use syn::{
    parse::Error, spanned::Spanned as _, Data, DeriveInput, Expr, Fields, GenericArgument, Lit,
    PathArguments, Type,
};

pub fn ident(name: &str) -> Ident {
    Ident::new(name, Span::call_site())
}

pub fn signature(hash: &[u8]) -> proc_macro2::TokenStream {
    let core_crate = ethers_core_crate();
    let bytes = hash.iter().copied().map(Literal::u8_unsuffixed);
    quote! {#core_crate::types::H256([#( #bytes ),*])}
}

pub fn selector(selector: Selector) -> proc_macro2::TokenStream {
    let bytes = selector.iter().copied().map(Literal::u8_unsuffixed);
    quote! {[#( #bytes ),*]}
}

/// Parses an int type from its string representation
pub fn parse_int_param_type(s: &str) -> Option<ParamType> {
    let size = s.chars().skip(1).collect::<String>().parse::<usize>().ok()?;
    if s.starts_with('u') {
        Some(ParamType::Uint(size))
    } else if s.starts_with('i') {
        Some(ParamType::Int(size))
    } else {
        None
    }
}

// Converts param types for indexed parameters to bytes32 where appropriate
// This applies to strings, arrays, structs and bytes to follow the encoding of
// these indexed param types according to
// <https://solidity.readthedocs.io/en/develop/abi-spec.html#encoding-of-indexed-event-parameters>
pub fn topic_param_type_quote(kind: &ParamType) -> proc_macro2::TokenStream {
    let core_crate = ethers_core_crate();
    match kind {
        ParamType::String |
        ParamType::Bytes |
        ParamType::Array(_) |
        ParamType::FixedArray(_, _) |
        ParamType::Tuple(_) => quote! {#core_crate::abi::ParamType::FixedBytes(32)},
        ty => param_type_quote(ty),
    }
}

/// Returns the rust type for the given parameter
pub fn param_type_quote(kind: &ParamType) -> proc_macro2::TokenStream {
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
            let ty = param_type_quote(ty);
            quote! {#core_crate::abi::ParamType::Array(Box::new(#ty))}
        }
        ParamType::FixedBytes(size) => {
            let size = Literal::usize_suffixed(*size);
            quote! {#core_crate::abi::ParamType::FixedBytes(#size)}
        }
        ParamType::FixedArray(ty, size) => {
            let ty = param_type_quote(ty);
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

/// Tries to find the corresponding `ParamType` used for tokenization for the
/// given type
pub fn find_parameter_type(ty: &Type) -> Result<ParamType, Error> {
    match ty {
        Type::Array(ty) => {
            let param = find_parameter_type(ty.elem.as_ref())?;
            if let Expr::Lit(ref expr) = ty.len {
                if let Lit::Int(ref len) = expr.lit {
                    if let Ok(size) = len.base10_parse::<usize>() {
                        return Ok(ParamType::FixedArray(Box::new(param), size))
                    }
                }
            }
            Err(Error::new(ty.span(), "Failed to derive proper ABI from array field"))
        }
        Type::Path(ty) => {
            // check for `Vec`
            if ty.path.segments.len() == 1 && ty.path.segments[0].ident == "Vec" {
                if let PathArguments::AngleBracketed(ref args) = ty.path.segments[0].arguments {
                    if args.args.len() == 1 {
                        if let GenericArgument::Type(ref ty) = args.args.iter().next().unwrap() {
                            let kind = find_parameter_type(ty)?;
                            return Ok(ParamType::Array(Box::new(kind)))
                        }
                    }
                }
            }
            let mut ident = ty.path.get_ident();
            if ident.is_none() {
                ident = ty.path.segments.last().map(|s| &s.ident);
            }
            if let Some(ident) = ident {
                let ident = ident.to_string().to_lowercase();
                return match ident.as_str() {
                    "address" => Ok(ParamType::Address),
                    "bytes" => Ok(ParamType::Bytes),
                    "string" => Ok(ParamType::String),
                    "bool" => Ok(ParamType::Bool),
                    "int" | "uint" => Ok(ParamType::Uint(256)),
                    "h160" => Ok(ParamType::FixedBytes(20)),
                    "h256" | "secret" | "hash" => Ok(ParamType::FixedBytes(32)),
                    "h512" | "public" => Ok(ParamType::FixedBytes(64)),
                    s => parse_int_param_type(s).ok_or_else(|| {
                        Error::new(ty.span(), "Failed to derive proper ABI from fields")
                    }),
                }
            }
            Err(Error::new(ty.span(), "Failed to derive proper ABI from fields"))
        }
        Type::Tuple(ty) => {
            let params = ty.elems.iter().map(find_parameter_type).collect::<Result<Vec<_>, _>>()?;
            Ok(ParamType::Tuple(params))
        }
        _ => Err(Error::new(ty.span(), "Failed to derive proper ABI from fields")),
    }
}

/// Attempts to determine the ABI Paramtypes from the type's AST
pub fn derive_abi_inputs_from_fields(
    input: &DeriveInput,
    trait_name: &str,
) -> Result<Vec<(String, ParamType)>, Error> {
    let fields: Vec<_> = match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => fields.named.iter().collect(),
            Fields::Unnamed(ref fields) => fields.unnamed.iter().collect(),
            Fields::Unit => {
                return Err(Error::new(
                    input.span(),
                    format!("{trait_name} cannot be derived for empty structs and unit"),
                ))
            }
        },
        Data::Enum(_) => {
            return Err(Error::new(
                input.span(),
                format!("{trait_name} cannot be derived for enums"),
            ))
        }
        Data::Union(_) => {
            return Err(Error::new(
                input.span(),
                format!("{trait_name} cannot be derived for unions"),
            ))
        }
    };

    fields
        .iter()
        .map(|f| {
            let name =
                f.ident.as_ref().map(|name| name.to_string()).unwrap_or_else(|| "".to_string());
            find_parameter_type(&f.ty).map(|ty| (name, ty))
        })
        .collect()
}

/// Use `AbiType::param_type` fo each field to construct the input types own param type
pub fn derive_param_type_with_abi_type(
    input: &DeriveInput,
    trait_name: &str,
) -> Result<proc_macro2::TokenStream, Error> {
    let core_crate = ethers_core_crate();
    let params = derive_abi_parameters_array(input, trait_name)?;
    Ok(quote! {
        #core_crate::abi::ParamType::Tuple(::std::vec!#params)
    })
}

/// Use `AbiType::param_type` fo each field to construct the whole signature `<name>(<params,>*)` as
/// `String`
pub fn derive_abi_signature_with_abi_type(
    input: &DeriveInput,
    function_name: &str,
    trait_name: &str,
) -> Result<proc_macro2::TokenStream, Error> {
    let params = derive_abi_parameters_array(input, trait_name)?;
    Ok(quote! {
        {
            let params: String = #params
                                 .iter()
                                 .map(|p| p.to_string())
                                 .collect::<::std::vec::Vec<_>>()
                                 .join(",");
            let function_name = #function_name;
            format!("{}({})", function_name, params)
        }
    })
}

/// Use `AbiType::param_type` fo each field to construct the signature's parameters as runtime array
/// `[param1, param2,...]`
pub fn derive_abi_parameters_array(
    input: &DeriveInput,
    trait_name: &str,
) -> Result<proc_macro2::TokenStream, Error> {
    let core_crate = ethers_core_crate();

    let param_types: Vec<_> = match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => fields
                .named
                .iter()
                .map(|f| {
                    let ty = &f.ty;
                    quote_spanned! { f.span() => <#ty as #core_crate::abi::AbiType>::param_type() }
                })
                .collect(),
            Fields::Unnamed(ref fields) => fields
                .unnamed
                .iter()
                .map(|f| {
                    let ty = &f.ty;
                    quote_spanned! { f.span() => <#ty as #core_crate::abi::AbiType>::param_type() }
                })
                .collect(),
            Fields::Unit => {
                return Err(Error::new(
                    input.span(),
                    format!("{trait_name} cannot be derived for empty structs and unit"),
                ))
            }
        },
        Data::Enum(_) => {
            return Err(Error::new(
                input.span(),
                format!("{trait_name} cannot be derived for enums"),
            ))
        }
        Data::Union(_) => {
            return Err(Error::new(
                input.span(),
                format!("{trait_name} cannot be derived for unions"),
            ))
        }
    };

    Ok(quote! {
        [#( #param_types ),*]
    })
}
