//! Types expansion

use crate::{util, InternalStructs};
use ethers_core::{
    abi::{Event, EventParam, ParamType},
    macros::ethers_core_crate,
};
use eyre::{bail, Result};
use proc_macro2::{Literal, TokenStream};
use quote::quote;

/// Expands a ParamType Solidity type to its Rust equivalent.
pub fn expand(kind: &ParamType) -> Result<TokenStream> {
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
        ParamType::Array(ty) => {
            let ty = expand(ty)?;
            Ok(quote!(::std::vec::Vec<#ty>))
        }
        ParamType::FixedBytes(n) => {
            // TODO(nlordell): what is the performance impact of returning large
            //   `FixedBytes` and `FixedArray`s with `web3`?
            let size = Literal::usize_unsuffixed(*n);
            Ok(quote!([u8; #size]))
        }
        ParamType::FixedArray(ty, n) => {
            // TODO(nlordell): see above
            let ty = match **ty {
                // this prevents type ambiguity with `FixedBytes`
                // see: https://github.com/gakonst/ethers-rs/issues/1636
                ParamType::Uint(size) if size / 8 == 1 => quote!(#ethers_core::types::Uint8),
                _ => expand(ty)?,
            };
            let size = Literal::usize_unsuffixed(*n);
            Ok(quote!([#ty; #size]))
        }
        ParamType::Tuple(members) => {
            eyre::ensure!(!members.is_empty(), "Tuple must have at least 1 member");

            let members = members.iter().map(expand).collect::<Result<Vec<_>, _>>()?;
            Ok(quote!(( #( #members ),* )))
        }
    }
}

/// Expands the event's inputs.
pub fn expand_event_inputs(
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
                        ParamType::Array(_) => Ok(quote!(::std::vec::Vec<#ty>)),
                        ParamType::FixedArray(_, size) => {
                            let size = Literal::usize_unsuffixed(*size);
                            Ok(quote!([#ty; #size]))
                        }
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
