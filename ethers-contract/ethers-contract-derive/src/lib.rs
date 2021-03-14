//! Implementation of procedural macro for generating type-safe bindings to an
//! ethereum smart contract.
#![deny(missing_docs, unsafe_code)]

use proc_macro::TokenStream;

use quote::{quote, quote_spanned};
use syn::spanned::Spanned as _;
use syn::{parse::Error, parse_macro_input, Data, DeriveInput, Fields};

use abigen::{expand, ContractArgs};
use spanned::Spanned;

mod abigen;
mod spanned;

/// Proc macro to generate type-safe bindings to a contract. This macro accepts
/// an Ethereum contract ABI or a path. Note that this path is rooted in
/// the crate's root `CARGO_MANIFEST_DIR`.
///
/// # Examples
///
/// ```ignore
/// # use ethers_contract_derive::abigen;
/// // ABI Path
/// abigen!(MyContract, "MyContract.json");
///
/// // HTTP(S) source
/// abigen!(MyContract, "https://my.domain.local/path/to/contract.json");
///
/// // Etherscan.io
/// abigen!(MyContract, "etherscan:0x0001020304050607080910111213141516171819");
/// abigen!(MyContract, "https://etherscan.io/address/0x0001020304050607080910111213141516171819");
///
/// // npmjs
/// abigen!(MyContract, "npm:@org/package@1.0.0/path/to/contract.json");
/// ```
///
/// Note that Etherscan rate-limits requests to their API, to avoid this an
/// `ETHERSCAN_API_KEY` environment variable can be set. If it is, it will use
/// that API key when retrieving the contract ABI.
///
/// Currently the proc macro accepts additional parameters to configure some
/// aspects of the code generation. Specifically it accepts:
/// - `methods`: A list of mappings from method signatures to method names
///   allowing methods names to be explicitely set for contract methods. This
///   also provides a workaround for generating code for contracts with multiple
///   methods with the same name.
/// - `event_derives`: A list of additional derives that should be added to
///   contract event structs and enums.
///
/// ```ignore
/// abigen!(
///     MyContract,
///     "path/to/MyContract.json",
///     methods {
///         myMethod(uint256,bool) as my_renamed_method;
///     },
///     event_derives (serde::Deserialize, serde::Serialize),
/// );
/// ```
#[proc_macro]
pub fn abigen(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as Spanned<ContractArgs>);

    let span = args.span();
    expand(args.into_inner())
        .unwrap_or_else(|e| Error::new(span, format!("{:?}", e)).to_compile_error())
        .into()
}

///
#[proc_macro_derive(EthEvent, attributes(rename))]
pub fn derive_abi_event(input: TokenStream) -> TokenStream {
    "fn answer() -> u32 { 42 }".parse().unwrap()
}

/// Derives the `Tokenizable` trait for the labeled type.
///
/// This derive macro automatically adds a type bound `field: Tokenizable` for each
/// field type.
#[proc_macro_derive(EthAbiType)]
pub fn derive_abi_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

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
                    quote_spanned! { f.span() => #ty: ethers_core::abi::Tokenize }
                });
                let tokenize_predicates = quote! { #(#tokenize_predicates,)* };

                let assignments = fields.named.iter().map(|f| {
                    let name = f.ident.as_ref().expect("Named fields have names");
                    quote_spanned! { f.span() => #name: ethers_core::abi::Tokenizable::from_token(iter.next().expect("tokens size is sufficient qed").into_token())? }
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
                    quote_spanned! { f.span() => #ty: ethers_core::abi::Tokenize }
                });
                let tokenize_predicates = quote! { #(#tokenize_predicates,)* };

                let assignments = fields.unnamed.iter().map(|f| {
                    quote_spanned! { f.span() => ethers_core::abi::Tokenizable::from_token(iter.next().expect("tokens size is sufficient qed").into_token())? }
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
                return TokenStream::from(
                    Error::new(
                        input.span(),
                        "EthAbiType cannot be derived for empty structs and unit",
                    )
                    .to_compile_error(),
                )
            }
        },
        Data::Enum(_) => {
            return TokenStream::from(
                Error::new(input.span(), "EthAbiType cannot be derived for enums")
                    .to_compile_error(),
            )
        }
        Data::Union(_) => {
            return TokenStream::from(
                Error::new(input.span(), "EthAbiType cannot be derived for unions")
                    .to_compile_error(),
            )
        }
    };

    let tokenizeable_impl = quote! {
             impl<#generic_params> ethers_core::abi::Tokenizable for #name<#generic_args>
             where
                 #generic_predicates
                 #tokenize_predicates
             {

                 fn from_token(token: ethers_core::abi::Token) -> Result<Self, ethers_core::abi::InvalidOutputType> where
                     Self: Sized {
                    if let ethers_core::abi::Token::Tuple(tokens) = token {
                        if tokens.len() != #params_len {
                            return Err(ethers_core::abi::InvalidOutputType(format!(
                                "Expected {} tokens, got {}: {:?}",
                                #params_len,
                                tokens.len(),
                                tokens
                            )));
                        }

                        let mut iter = tokens.into_iter();

                        Ok(#init_struct_impl)
                    } else {
                        Err(ethers_core::abi::InvalidOutputType(format!(
                            "Expected Tuple, got {:?}",
                            token
                        )))
                    }
                 }

                 fn into_token(self) -> ethers_core::abi::Token {
                    ethers_core::abi::Token::Tuple(
                        vec![
                            #into_token_impl
                        ]
                    )
                 }
             }
    };

    TokenStream::from(tokenizeable_impl)
}
