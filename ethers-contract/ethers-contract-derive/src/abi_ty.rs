//! Helper functions for deriving `EthAbiType`

use ethers_contract_abigen::ethers_core_crate;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned as _;
use syn::{parse::Error, Data, DeriveInput, Fields};

/// Generates the tokenize implementation
pub fn derive_tokenizeable_impl(input: &DeriveInput) -> proc_macro2::TokenStream {
    let core_crate = ethers_core_crate();
    let name = &input.ident;
    let generic_params = input.generics.params.iter().map(|p| quote! { #p });
    let generic_params = quote! { #(#generic_params,)* };

    let generic_args = input.generics.type_params().map(|p| {
        let name = &p.ident;
        quote_spanned! { p.ident.span() => #name }
    });

    let generic_args = quote! { #(#generic_args,)* };

    let generic_predicates = match input.generics.where_clause {
        Some(ref clause) => {
            let predicates = clause.predicates.iter().map(|p| quote! { #p });
            quote! { #(#predicates,)* }
        }
        None => quote! {},
    };

    let (tokenize_predicates, params_len, init_struct_impl, into_token_impl) = match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let tokenize_predicates = fields.named.iter().map(|f| {
                    let ty = &f.ty;
                    quote_spanned! { f.span() => #ty: #core_crate::abi::Tokenize }
                });
                let tokenize_predicates = quote! { #(#tokenize_predicates,)* };

                let assignments = fields.named.iter().map(|f| {
                    let name = f.ident.as_ref().expect("Named fields have names");
                    quote_spanned! { f.span() => #name: #core_crate::abi::Tokenizable::from_token(iter.next().expect("tokens size is sufficient qed").into_token())? }
                });
                let init_struct_impl = quote! { Self { #(#assignments,)* } };

                let into_token = fields.named.iter().map(|f| {
                    let name = f.ident.as_ref().expect("Named fields have names");
                    quote_spanned! { f.span() => self.#name.into_token() }
                });
                let into_token_impl = quote! { #(#into_token,)* };

                (
                    tokenize_predicates,
                    fields.named.len(),
                    init_struct_impl,
                    into_token_impl,
                )
            }
            Fields::Unnamed(ref fields) => {
                let tokenize_predicates = fields.unnamed.iter().map(|f| {
                    let ty = &f.ty;
                    quote_spanned! { f.span() => #ty: #core_crate::abi::Tokenize }
                });
                let tokenize_predicates = quote! { #(#tokenize_predicates,)* };

                let assignments = fields.unnamed.iter().map(|f| {
                    quote_spanned! { f.span() => #core_crate::abi::Tokenizable::from_token(iter.next().expect("tokens size is sufficient qed").into_token())? }
                });
                let init_struct_impl = quote! { Self(#(#assignments,)* ) };

                let into_token = fields.unnamed.iter().enumerate().map(|(i, f)| {
                    let idx = syn::Index::from(i);
                    quote_spanned! { f.span() => self.#idx.into_token() }
                });
                let into_token_impl = quote! { #(#into_token,)* };

                (
                    tokenize_predicates,
                    fields.unnamed.len(),
                    init_struct_impl,
                    into_token_impl,
                )
            }
            Fields::Unit => {
                return Error::new(
                    input.span(),
                    "EthAbiType cannot be derived for empty structs and unit",
                )
                .to_compile_error();
            }
        },
        Data::Enum(_) => {
            return Error::new(input.span(), "EthAbiType cannot be derived for enums")
                .to_compile_error();
        }
        Data::Union(_) => {
            return Error::new(input.span(), "EthAbiType cannot be derived for unions")
                .to_compile_error();
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
                #core_crate::abi::Token::Tuple(Vec::new())
            },
        ),
        _ => {
            let from_token = quote! {
                if let #core_crate::abi::Token::Tuple(tokens) = token {
                    if tokens.len() != #params_len {
                        return Err(#core_crate::abi::InvalidOutputType(::std::format!(
                            "Expected {} tokens, got {}: {:?}",
                            #params_len,
                            tokens.len(),
                            tokens
                        )));
                    }

                    let mut iter = tokens.into_iter();

                    Ok(#init_struct_impl)
                } else {
                    Err(#core_crate::abi::InvalidOutputType(::std::format!(
                        "Expected Tuple, got {:?}",
                        token
                    )))
                }
            };

            let into_token = quote! {
                #core_crate::abi::Token::Tuple(
                    ::std::vec![
                        #into_token_impl
                    ]
                )
            };
            (from_token, into_token)
        }
    };

    quote! {
         impl<#generic_params> #core_crate::abi::Tokenizable for #name<#generic_args>
         where
             #generic_predicates
             #tokenize_predicates
         {

             fn from_token(token: #core_crate::abi::Token) -> Result<Self, #core_crate::abi::InvalidOutputType> where
                 Self: Sized {
                #from_token_impl
             }

             fn into_token(self) -> #core_crate::abi::Token {
                #into_token_impl
             }
         }

        impl<#generic_params> #core_crate::abi::TokenizableItem for #name<#generic_args>
         where
             #generic_predicates
             #tokenize_predicates
         { }
    }
}
