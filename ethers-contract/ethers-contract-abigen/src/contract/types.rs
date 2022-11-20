use ethers_core::{abi::ParamType, macros::ethers_core_crate};
use eyre::{bail, Result};
use proc_macro2::{Literal, TokenStream};
use quote::quote;

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
            17..=32 => Ok(quote! { I256 }),
            _ => bail!("unsupported solidity type int{}", n),
        },
        ParamType::Uint(n) => match n / 8 {
            1 => Ok(quote! { u8 }),
            2 => Ok(quote! { u16 }),
            3..=4 => Ok(quote! { u32 }),
            5..=8 => Ok(quote! { u64 }),
            9..=16 => Ok(quote! { u128 }),
            17..=32 => Ok(quote! { #ethers_core::types::U256 }),
            _ => bail!("unsupported solidity type uint{}", n),
        },
        ParamType::Bool => Ok(quote! { bool }),
        ParamType::String => Ok(quote! { String }),
        ParamType::Array(t) => {
            let inner = expand(t)?;
            Ok(quote! { Vec<#inner> })
        }
        ParamType::FixedBytes(n) => {
            // TODO(nlordell): what is the performance impact of returning large
            //   `FixedBytes` and `FixedArray`s with `web3`?
            let size = Literal::usize_unsuffixed(*n);
            Ok(quote! { [u8; #size] })
        }
        ParamType::FixedArray(t, n) => {
            // TODO(nlordell): see above
            let inner = match **t {
                ParamType::Uint(size) => {
                    if size / 8 == 1 {
                        // this prevents type ambiguity with `FixedBytes`
                        quote! { #ethers_core::types::Uint8}
                    } else {
                        expand(t)?
                    }
                }
                _ => expand(t)?,
            };
            let size = Literal::usize_unsuffixed(*n);
            Ok(quote! { [#inner; #size] })
        }
        ParamType::Tuple(members) => {
            eyre::ensure!(!members.is_empty(), "Tuple must have at least 1 member");

            let members = members.iter().map(expand).collect::<Result<Vec<_>, _>>()?;
            Ok(quote! { (#(#members,)*) })
        }
    }
}
