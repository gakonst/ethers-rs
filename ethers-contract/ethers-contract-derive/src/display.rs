//! Helper functions for deriving `Display`

use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned as _;
use syn::{parse::Error, Data, DeriveInput, Fields};
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

    let mut prints = Vec::with_capacity(fields.len());

    for field in fields {
        let param = utils::find_parameter_type(&field.ty) ?;


    }

    let name = &input.ident;

    Ok(quote! {
        impl ::std::fmt::Display for #name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {

                Ok(())
            }
        }
    })
}

struct X;
impl ::std::fmt::Display for X {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        let x: Vec<u8> = vec![];
        let x = hex::encode(&x);
        write!(f, "0x{:?}", x)
    }
}
