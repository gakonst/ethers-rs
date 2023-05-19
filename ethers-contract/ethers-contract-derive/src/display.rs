//! Helper functions for deriving `Display`

use crate::utils;
use ethers_core::{abi::ParamType, macros::ethers_core_crate};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::Error, spanned::Spanned, Data, DeriveInput, Index};

/// Derive `fmt::Display` for the given type
pub(crate) fn derive_eth_display_impl(input: DeriveInput) -> Result<TokenStream, Error> {
    let fields = match input.data {
        Data::Struct(data) => data.fields.into_iter(),
        Data::Enum(e) => {
            return Err(Error::new(e.enum_token.span(), "Enums are not supported by EthDisplay"))
        }
        Data::Union(u) => {
            return Err(Error::new(u.union_token.span(), "Unions are not supported by EthDisplay"))
        }
    };

    let mut expressions = TokenStream::new();
    for (i, field) in fields.enumerate() {
        // comma separator
        if i > 0 {
            let tokens = quote! {
                ::core::fmt::Write::write_str(f, ", ")?;
            };
            expressions.extend(tokens);
        }

        let ident = field.ident.as_ref().map(|id| quote!(#id)).unwrap_or_else(|| {
            let idx = Index::from(i);
            quote!(#idx)
        });
        if let Ok(param) = utils::find_parameter_type(&field.ty) {
            let ethers_core = ethers_core_crate();
            let hex_encode = quote!(#ethers_core::utils::hex::encode);
            fmt_params_tokens(&param, ident, &mut expressions, &hex_encode);
        } else {
            // could not detect the parameter type and rely on using debug fmt
            fmt_debug_tokens(&ident, &mut expressions);
        }
    }

    let name = &input.ident;
    Ok(quote! {
        impl ::core::fmt::Display for #name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                #expressions
                Ok(())
            }
        }
    })
}

/// Recursive for tuples len > 12.
fn fmt_params_tokens(
    param: &ParamType,
    ident: TokenStream,
    out: &mut TokenStream,
    hex_encode: &TokenStream,
) {
    match param {
        // Display
        ParamType::Bool | ParamType::String | ParamType::Uint(_) | ParamType::Int(_) => {
            fmt_display_tokens(&ident, out);
        }

        // Debug
        ParamType::Address => fmt_debug_tokens(&ident, out),

        // 0x ++ hex::encode
        ParamType::Bytes | ParamType::FixedBytes(_) => hex_encode_tokens(&ident, out, hex_encode),

        // Debug or recurse
        ParamType::Tuple(params) => {
            // Debug is implemented automatically only for tuples with arity <= 12
            if params.len() <= 12 {
                fmt_debug_tokens(&ident, out);
            } else {
                for (i, new_param) in params.iter().enumerate() {
                    let idx = Index::from(i);
                    let new_ident = quote!(#ident.#idx);
                    fmt_params_tokens(new_param, new_ident, out, hex_encode);
                }
            }
        }

        // 0x ++ hex::encode or DebugList
        ParamType::Array(ty) | ParamType::FixedArray(ty, _) => match &**ty {
            ParamType::Uint(8) => hex_encode_tokens(&ident, out, hex_encode),
            ParamType::Tuple(params) if params.len() > 12 => {
                // TODO: Recurse this
                let idx = (0..params.len()).map(Index::from);
                let tokens = quote! {
                    let mut list = f.debug_list();
                    for entry in self.#ident.iter() {
                        #( list.entry(&entry.#idx); )*
                    }
                    list.finish()?;
                };
                out.extend(tokens);
            }
            _ => {
                let tokens = quote! {
                    f.debug_list().entries(self.#ident.iter()).finish()?;
                };
                out.extend(tokens);
            }
        },
    }
}

fn fmt_display_tokens(ident: &TokenStream, out: &mut TokenStream) {
    let tokens = quote! {
        ::core::fmt::Display::fmt(&self.#ident, f)?;
    };
    out.extend(tokens);
}

fn fmt_debug_tokens(ident: &TokenStream, out: &mut TokenStream) {
    let tokens = quote! {
        ::core::fmt::Debug::fmt(&self.#ident, f)?;
    };
    out.extend(tokens);
}

fn hex_encode_tokens(ident: &TokenStream, out: &mut TokenStream, hex_encode: &TokenStream) {
    let tokens = quote! {
        ::core::fmt::Write::write_str(f, "0x")?;
        ::core::fmt::Write::write_str(f, #hex_encode(&self.#ident).as_str())?;
    };
    out.extend(tokens);
}
