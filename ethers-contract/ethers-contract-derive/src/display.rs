//! Helper functions for deriving `Display`

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::Error, spanned::Spanned as _, Data, DeriveInput, Fields, Index};

use ethers_core::{abi::ParamType, macros::ethers_core_crate};

use crate::utils;

/// Derive `fmt::Display` for the given type
pub(crate) fn derive_eth_display_impl(input: DeriveInput) -> Result<TokenStream, Error> {
    let fields: Vec<_> = match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => fields.named.iter().collect(),
            Fields::Unnamed(ref fields) => fields.unnamed.iter().collect(),
            Fields::Unit => {
                vec![]
            }
        },
        Data::Enum(_) => {
            return Err(Error::new(input.span(), "Enum types are not supported by EthDisplay"))
        }
        Data::Union(_) => {
            return Err(Error::new(input.span(), "Union types are not supported by EthDisplay"))
        }
    };
    let core_crate = ethers_core_crate();
    let hex_encode = quote! {#core_crate::utils::hex::encode};
    let mut fmts = TokenStream::new();
    for (idx, field) in fields.iter().enumerate() {
        let ident = field.ident.clone().map(|id| quote! {#id}).unwrap_or_else(|| {
            let idx = Index::from(idx);
            quote! {#idx}
        });
        let tokens = if let Ok(param) = utils::find_parameter_type(&field.ty) {
            match param {
                ParamType::Address | ParamType::Uint(_) | ParamType::Int(_) => {
                    quote! {
                         write!(f, "{:?}", self.#ident)?;
                    }
                }
                ParamType::Bytes => {
                    quote! {
                         write!(f, "0x{}", #hex_encode(&self.#ident))?;
                    }
                }
                ParamType::Bool | ParamType::String => {
                    quote! {
                         self.#ident.fmt(f)?;
                    }
                }
                ParamType::Tuple(_) => {
                    quote! {
                        write!(f, "{:?}", &self.#ident)?;
                    }
                }
                ParamType::Array(ty) | ParamType::FixedArray(ty, _) => {
                    if *ty == ParamType::Uint(8) {
                        // `u8`
                        quote! {
                             write!(f, "0x{}", #hex_encode(&self.#ident[..]))?;
                        }
                    } else {
                        // format as array with `[arr[0].display, arr[1].display,...]`
                        quote! {
                           write!(f, "[")?;
                           for (idx, val) in self.#ident.iter().enumerate() {
                               write!(f, "{:?}", val)?;
                               if idx < self.#ident.len() - 1 {
                                   write!(f, ", ")?;
                               }
                           }
                           write!(f, "]")?;
                        }
                    }
                }
                ParamType::FixedBytes(_) => {
                    quote! {
                         write!(f, "0x{}", #hex_encode(&self.#ident))?;
                    }
                }
            }
        } else {
            // could not detect the parameter type and rely on using debug fmt
            quote! {
                 write!(f, "{:?}", &self.#ident)?;
            }
        };
        fmts.extend(tokens);
        if idx < fields.len() - 1 {
            fmts.extend(quote! { write!(f, ", ")?;});
        }
    }
    let name = &input.ident;
    Ok(quote! {
        impl ::std::fmt::Display for #name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                #fmts
                Ok(())
            }
        }
    })
}
