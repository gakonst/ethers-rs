//! Helper functions for deriving `EthAbiType`

use crate::utils;
use ethers_core::macros::ethers_core_crate;
use proc_macro2::{Ident, Literal, TokenStream};
use quote::{quote, quote_spanned};
use syn::{parse::Error, spanned::Spanned, Data, DeriveInput, Fields, Variant};

/// Generates the tokenize implementation
pub fn derive_tokenizeable_impl(input: &DeriveInput) -> Result<TokenStream, Error> {
    let ethers_core = ethers_core_crate();
    let name = &input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let generic_predicates = where_clause.map(|c| &c.predicates);

    let (tokenize_predicates, params_len, init_struct_impl, into_token_impl) = match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let tokenize_predicates = fields.named.iter().map(|f| {
                    let ty = &f.ty;
                    quote_spanned! { f.span() => #ty: #ethers_core::abi::Tokenize }
                });
                let tokenize_predicates = quote! { #(#tokenize_predicates,)* };

                let assignments = fields.named.iter().map(|f| {
                    let name = f.ident.as_ref().expect("Named fields have names");
                    quote_spanned! { f.span() =>
                        #name: #ethers_core::abi::Tokenizable::from_token(
                            iter.next().expect("The iter is guaranteed to be something due to the size check")
                        )?
                    }
                });
                let init_struct_impl = quote! { Self { #(#assignments,)* } };

                let into_token = fields.named.iter().map(|f| {
                    let name = f.ident.as_ref().expect("Named fields have names");
                    quote_spanned! { f.span() => self.#name.into_token() }
                });
                let into_token_impl = quote! { #(#into_token,)* };

                (tokenize_predicates, fields.named.len(), init_struct_impl, into_token_impl)
            }
            Fields::Unnamed(ref fields) => {
                let tokenize_predicates = fields.unnamed.iter().map(|f| {
                    let ty = &f.ty;
                    quote_spanned! { f.span() => #ty: #ethers_core::abi::Tokenize }
                });
                let tokenize_predicates = quote! { #(#tokenize_predicates,)* };

                let assignments = fields.unnamed.iter().map(|f| {
                    quote_spanned! { f.span() =>
                        #ethers_core::abi::Tokenizable::from_token(
                            iter.next().expect("The iter is guaranteed to be something due to the size check")
                        )?
                    }
                });
                let init_struct_impl = quote! { Self(#(#assignments,)* ) };

                let into_token = fields.unnamed.iter().enumerate().map(|(i, f)| {
                    let idx = syn::Index::from(i);
                    quote_spanned! { f.span() => self.#idx.into_token() }
                });
                let into_token_impl = quote! { #(#into_token,)* };

                (tokenize_predicates, fields.unnamed.len(), init_struct_impl, into_token_impl)
            }
            Fields::Unit => return Ok(tokenize_unit_type(&input.ident)),
        },
        Data::Enum(ref data) => return tokenize_enum(name, data.variants.iter()),
        Data::Union(_) => {
            return Err(Error::new(input.span(), "EthAbiType cannot be derived for unions"))
        }
    };

    // there might be the case that the event has only 1 params, which is not a
    // tuple
    let (from_token_impl, into_token_impl) = match params_len {
        0 => (
            quote! {
               Ok(#init_struct_impl)
            },
            // can't encode an empty struct
            // TODO: panic instead?
            quote! {
                #ethers_core::abi::Token::Tuple(Vec::new())
            },
        ),
        _ => {
            let from_token = quote! {
                if let #ethers_core::abi::Token::Tuple(tokens) = token {
                    if tokens.len() != #params_len {
                        return Err(#ethers_core::abi::InvalidOutputType(::std::format!(
                            "Expected {} tokens, got {}: {:?}",
                            #params_len,
                            tokens.len(),
                            tokens
                        )));
                    }

                    let mut iter = tokens.into_iter();

                    Ok(#init_struct_impl)
                } else {
                    Err(#ethers_core::abi::InvalidOutputType(::std::format!(
                        "Expected Tuple, got {:?}",
                        token
                    )))
                }
            };

            let into_token = quote! {
                #ethers_core::abi::Token::Tuple(
                    ::std::vec![
                        #into_token_impl
                    ]
                )
            };
            (from_token, into_token)
        }
    };

    let params = utils::derive_param_type_with_abi_type(input, "EthAbiType")?;

    Ok(quote! {
        impl #impl_generics #ethers_core::abi::AbiType for #name #ty_generics #where_clause {
            fn param_type() -> #ethers_core::abi::ParamType {
                #params
            }
        }

        impl #impl_generics #ethers_core::abi::AbiArrayType for #name #ty_generics #where_clause {}

        impl #impl_generics #ethers_core::abi::Tokenizable for #name #ty_generics
        where
            #generic_predicates
            #tokenize_predicates
        {
            fn from_token(token: #ethers_core::abi::Token) -> ::std::result::Result<Self, #ethers_core::abi::InvalidOutputType>
            where
                Self: Sized,
            {
                #from_token_impl
            }

            fn into_token(self) -> #ethers_core::abi::Token {
                #into_token_impl
            }
        }

        impl #impl_generics #ethers_core::abi::TokenizableItem for #name #ty_generics
        where
            #generic_predicates
            #tokenize_predicates
        {}
    })
}

fn tokenize_unit_type(name: &Ident) -> TokenStream {
    let ethers_core = ethers_core_crate();

    quote! {
        impl #ethers_core::abi::Tokenizable for #name {
            fn from_token(token: #ethers_core::abi::Token) -> ::std::result::Result<Self, #ethers_core::abi::InvalidOutputType>
            where
                Self: Sized,
            {
                if let #ethers_core::abi::Token::Tuple(tokens) = token {
                    if !tokens.is_empty() {
                        Err(#ethers_core::abi::InvalidOutputType(::std::format!(
                            "Expected empty tuple, got {:?}",
                            tokens
                        )))
                    } else {
                        Ok(#name{})
                    }
                } else {
                    Err(#ethers_core::abi::InvalidOutputType(::std::format!(
                        "Expected Tuple, got {:?}",
                        token
                    )))
                }
            }

            fn into_token(self) -> #ethers_core::abi::Token {
                #ethers_core::abi::Token::Tuple(::std::vec::Vec::new())
            }
        }

        impl #ethers_core::abi::TokenizableItem for #name {}
    }
}

/// Derive for an enum
///
/// An enum can be a [solidity enum](https://docs.soliditylang.org/en/v0.5.3/types.html#enums) or a
/// bundled set of different types.
///
/// Decoding works like untagged decoding
fn tokenize_enum<'a>(
    enum_name: &Ident,
    variants: impl Iterator<Item = &'a Variant> + 'a,
) -> Result<TokenStream, Error> {
    let ethers_core = ethers_core_crate();

    let mut into_tokens = TokenStream::new();
    let mut from_tokens = TokenStream::new();
    for (idx, variant) in variants.into_iter().enumerate() {
        let var_ident = &variant.ident;
        if variant.fields.len() > 1 {
            return Err(Error::new(
                variant.span(),
                "EthAbiType cannot be derived for enum variants with multiple fields",
            ))
        } else if variant.fields.is_empty() {
            let value = Literal::u8_unsuffixed(idx as u8);
            from_tokens.extend(quote! {
                if let Ok(#value) = u8::from_token(token.clone()) {
                    return Ok(#enum_name::#var_ident)
                }
            });
            into_tokens.extend(quote! {
                #enum_name::#var_ident => #value.into_token(),
            });
        } else if let Some(field) = variant.fields.iter().next() {
            let ty = &field.ty;
            from_tokens.extend(quote! {
                if let Ok(decoded) = #ty::from_token(token.clone()) {
                    return Ok(#enum_name::#var_ident(decoded))
                }
            });
            into_tokens.extend(quote! {
                #enum_name::#var_ident(element) => element.into_token(),
            });
        } else {
            into_tokens.extend(quote! {
                #enum_name::#var_ident(element) => # ethers_core::abi::Token::Tuple(::std::vec::Vec::new()),
            });
        }
    }

    Ok(quote! {
        impl #ethers_core::abi::Tokenizable for #enum_name {
            fn from_token(token: #ethers_core::abi::Token) -> ::std::result::Result<Self, #ethers_core::abi::InvalidOutputType>
            where
                Self: Sized,
            {
                #from_tokens
                Err(#ethers_core::abi::InvalidOutputType("Failed to decode all type variants".to_string()))
            }

            fn into_token(self) -> #ethers_core::abi::Token {
                match self {
                    #into_tokens
                }
            }
        }

        impl #ethers_core::abi::TokenizableItem for #enum_name {}
    })
}
