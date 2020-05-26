use anyhow::{anyhow, Result};
use ethcontract_common::abi::ParamType;
use proc_macro2::{Literal, TokenStream};
use quote::quote;

pub(crate) fn expand(kind: &ParamType) -> Result<TokenStream> {
    match kind {
        ParamType::Address => Ok(quote! { self::ethcontract::Address }),
        ParamType::Bytes => Ok(quote! { Vec<u8> }),
        ParamType::Int(n) => match n / 8 {
            1 => Ok(quote! { i8 }),
            2 => Ok(quote! { i16 }),
            3..=4 => Ok(quote! { i32 }),
            5..=8 => Ok(quote! { i64 }),
            9..=16 => Ok(quote! { i128 }),
            17..=32 => Ok(quote! { self::ethcontract::I256 }),
            _ => Err(anyhow!("unsupported solidity type int{}", n)),
        },
        ParamType::Uint(n) => match n / 8 {
            1 => Ok(quote! { u8 }),
            2 => Ok(quote! { u16 }),
            3..=4 => Ok(quote! { u32 }),
            5..=8 => Ok(quote! { u64 }),
            9..=16 => Ok(quote! { u128 }),
            17..=32 => Ok(quote! { self::ethcontract::U256 }),
            _ => Err(anyhow!("unsupported solidity type uint{}", n)),
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
            let inner = expand(t)?;
            let size = Literal::usize_unsuffixed(*n);
            Ok(quote! { [#inner; #size] })
        }
        ParamType::Tuple(_) => Err(anyhow!("ABIEncoderV2 is currently not supported")),
    }
}
