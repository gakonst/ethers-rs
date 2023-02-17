//! Types expansion

use crate::{util, InternalStructs};
use ethers_core::{
    abi::{struct_def::StructFieldType, Event, EventParam, Param, ParamType},
    macros::ethers_core_crate,
};
use eyre::{bail, Result};
use inflector::Inflector;
use proc_macro2::{Literal, TokenStream};
use quote::{quote, ToTokens};

/// Expands a ParamType Solidity type to its Rust equivalent.
pub(crate) fn expand(kind: &ParamType) -> Result<TokenStream> {
    let ethers_core = ethers_core_crate();

    match kind {
        ParamType::Address => Ok(quote! { #ethers_core::types::Address }),
        ParamType::Bytes => Ok(quote! { #ethers_core::types::Bytes }),
        ParamType::Int(n) => match n / 8 {
            1 => Ok(quote! { i8 }),
            2 => Ok(quote! { i16 }),
            3..=4 => Ok(quote! { i32 }),
            5..=8 => Ok(quote! { i64 }),
            9..=16 => Ok(quote! { i128 }),
            17..=32 => Ok(quote! { #ethers_core::types::I256 }),
            _ => bail!("unsupported solidity type int{n}"),
        },
        ParamType::Uint(n) => match n / 8 {
            1 => Ok(quote! { u8 }),
            2 => Ok(quote! { u16 }),
            3..=4 => Ok(quote! { u32 }),
            5..=8 => Ok(quote! { u64 }),
            9..=16 => Ok(quote! { u128 }),
            17..=32 => Ok(quote! { #ethers_core::types::U256 }),
            _ => bail!("unsupported solidity type uint{n}"),
        },
        ParamType::Bool => Ok(quote!(bool)),
        ParamType::String => Ok(quote!(::std::string::String)),
        ParamType::Array(ty) => Ok(array(expand(ty)?, None)),
        ParamType::FixedBytes(n) => Ok(array(quote!(u8), Some(*n))),
        ParamType::FixedArray(ty, n) => {
            let ty = match **ty {
                // this prevents type ambiguity with `FixedBytes`
                // see: https://github.com/gakonst/ethers-rs/issues/1636
                ParamType::Uint(size) if size / 8 == 1 => quote!(#ethers_core::types::Uint8),
                _ => expand(ty)?,
            };
            Ok(array(ty, Some(*n)))
        }
        ParamType::Tuple(members) => {
            eyre::ensure!(!members.is_empty(), "Tuple must have at least 1 member");

            let members = members.iter().map(expand).collect::<Result<Vec<_>, _>>()?;
            Ok(quote!(( #( #members ),* )))
        }
    }
}

/// Expands the event's inputs.
pub(crate) fn expand_event_inputs(
    event: &Event,
    internal_structs: &InternalStructs,
) -> Result<Vec<(TokenStream, TokenStream, bool)>> {
    event
        .inputs
        .iter()
        .enumerate()
        .map(|(index, input)| {
            // NOTE: Events can contain nameless values.
            expand_event_input(input, &event.name, index, internal_structs)
                .map(|ty| (util::expand_input_name(index, &input.name), ty, input.indexed))
        })
        .collect()
}

/// Expands an event property type.
///
/// Note that this is slightly different from expanding a Solidity type as complex types like arrays
/// and strings get emitted as hashes when they are indexed.
///
/// If a complex types matches with a struct previously parsed by the internal structs, we can
/// replace it.
fn expand_event_input(
    input: &EventParam,
    name: &str,
    index: usize,
    internal_structs: &InternalStructs,
) -> Result<TokenStream> {
    let kind = &input.kind;
    match (kind, input.indexed) {
        (ParamType::Array(_), true) |
        (ParamType::FixedArray(_, _), true) |
        (ParamType::Tuple(_), true) |
        (ParamType::Bytes, true) |
        (ParamType::String, true) => {
            let ethers_core = ethers_core_crate();
            Ok(quote!(#ethers_core::types::H256))
        }

        (ParamType::Array(_), false) |
        (ParamType::FixedArray(_, _), false) |
        (ParamType::Tuple(_), false) => {
            match internal_structs.get_event_input_struct_type(name, index) {
                Some(ty) => {
                    let ty = util::ident(ty);
                    match kind {
                        ParamType::Array(_) => Ok(array(ty, None)),
                        ParamType::FixedArray(_, size) => Ok(array(ty, Some(*size))),
                        ParamType::Tuple(_) => Ok(quote!(#ty)),
                        _ => unreachable!(),
                    }
                }
                None => expand(kind),
            }
        }

        _ => expand(kind),
    }
}

/// Expands `params` to `(name, type)` tokens pairs, while resolving tuples' types using the given
/// function.
pub(crate) fn expand_params<'a, 'b, F: Fn(&'a Param) -> Option<&'b str>>(
    params: &'a [Param],
    resolve_tuple: F,
) -> Result<Vec<(TokenStream, TokenStream)>> {
    params
        .iter()
        .enumerate()
        .map(|(idx, param)| {
            // NOTE: Params can be unnamed.
            expand_resolved(&param.kind, param, &resolve_tuple)
                .map(|ty| (util::expand_input_name(idx, &param.name), ty))
        })
        .collect()
}

/// Expands a ParamType Solidity type to its Rust equivalent, while resolving tuples' types using
/// the given function.
fn expand_resolved<'a, 'b, F: Fn(&'a Param) -> Option<&'b str>>(
    kind: &'a ParamType,
    param: &'a Param,
    resolve_tuple: &F,
) -> Result<TokenStream> {
    match kind {
        ParamType::Array(ty) => Ok(array(expand_resolved(ty, param, resolve_tuple)?, None)),
        ParamType::FixedArray(ty, size) => {
            Ok(array(expand_resolved(ty, param, resolve_tuple)?, Some(*size)))
        }
        ParamType::Tuple(_) => match resolve_tuple(param) {
            Some(ty) => {
                let ty = util::ident(ty);
                Ok(quote!(#ty))
            }
            None => expand(kind),
        },
        _ => expand(kind),
    }
}

/// Expands to the Rust struct type.
pub(crate) fn expand_struct_type(struct_ty: &StructFieldType) -> TokenStream {
    match struct_ty {
        StructFieldType::Type(ty) => {
            let ty = util::ident(&ty.name().to_pascal_case());
            quote!(#ty)
        }
        StructFieldType::Array(ty) => array(expand_struct_type(ty), None),
        StructFieldType::FixedArray(ty, size) => array(expand_struct_type(ty), Some(*size)),
    }
}

/// Expands `ty` into a Rust array or vector.
fn array<T: ToTokens>(ty: T, size: Option<usize>) -> TokenStream {
    match size {
        Some(size) => {
            let size = Literal::usize_unsuffixed(size);
            quote!([#ty; #size])
        }
        None => quote!(::std::vec::Vec<#ty>),
    }
}
