//! # EIP-712 Derive Macro
//! This crate provides a derive macro `Eip712` that is used to encode a rust struct
//! into a payload hash, according to [https://eips.ethereum.org/EIPS/eip-712](https://eips.ethereum.org/EIPS/eip-712)
//!
//! The trait used to derive the macro is found in `ethers_core::transaction::eip712::Eip712`
//! Both the derive macro and the trait must be in context when using
//!
//! This derive macro requires the `#[eip712]` attributes to be included
//! for specifying the domain separator used in encoding the hash.
//!
//! NOTE: In addition to deriving `Eip712` trait, the `EthAbiType` trait must also be derived.
//! This allows the struct to be parsed into `ethers_core::abi::Token` for encoding.
//!
//! # Optional Eip712 Parameters
//!
//! The only optional parameter is `salt`, which accepts a string
//! that is hashed using keccak256 and stored as bytes.
//!
//! # Example Usage
//!
//! ```ignore
//! use ethers_contract::EthAbiType;
//! use ethers_derive_eip712::*;
//! use ethers_core::types::{transaction::eip712::Eip712, H160};
//!
//! #[derive(Debug, Eip712, EthAbiType)]
//! #[eip712(
//!     name = "Radicle",
//!     version = "1",
//!     chain_id = 1,
//!     verifying_contract = "0x0000000000000000000000000000000000000000"
//!     // salt is an optional parameter
//!     salt = "my-unique-spice"
//! )]
//! pub struct Puzzle {
//!     pub organization: H160,
//!     pub contributor: H160,
//!     pub commit: String,
//!     pub project: String,
//! }
//!
//! let puzzle = Puzzle {
//!     organization: "0000000000000000000000000000000000000000"
//!         .parse::<H160>()
//!         .expect("failed to parse address"),
//!     contributor: "0000000000000000000000000000000000000000"
//!         .parse::<H160>()
//!         .expect("failed to parse address"),
//!     commit: "5693b7019eb3e4487a81273c6f5e1832d77acb53".to_string(),
//!     project: "radicle-reward".to_string(),
//! };
//!
//! let hash = puzzle.encode_eip712().unwrap();
//! ```
//!
//! # Limitations
//!
//! At the moment, the derive macro does not recursively encode nested Eip712 structs.
//!
//! There is an Inner helper attribute `#[eip712]` for fields that will eventually be used to
//! determine if there is a nested eip712 struct. However, this work is not yet complete.

#![deny(missing_docs, unsafe_code, rustdoc::broken_intra_doc_links)]
use ethers_core::{macros::ethers_core_crate, types::transaction::eip712};
use proc_macro::TokenStream;
use quote::quote;
use std::convert::TryFrom;
use syn::parse_macro_input;

/// Derive macro for `Eip712`
#[proc_macro_derive(Eip712, attributes(eip712))]
pub fn eip_712_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input);

    impl_eip_712_macro(&ast)
}

// Main implementation macro, used to compute static values and define
// method for encoding the final eip712 payload;
fn impl_eip_712_macro(ast: &syn::DeriveInput) -> TokenStream {
    // Primary type should match the type in the ethereum verifying contract;
    let primary_type = &ast.ident;

    // Instantiate domain from parsed attributes
    let domain = match eip712::EIP712Domain::try_from(ast) {
        Ok(attributes) => attributes,
        Err(e) => return TokenStream::from(e),
    };

    let domain_separator = hex::encode(domain.separator());

    //
    let domain_str = match serde_json::to_string(&domain) {
        Ok(s) => s,
        Err(e) => {
            return TokenStream::from(
                syn::Error::new(ast.ident.span(), e.to_string()).to_compile_error(),
            )
        }
    };

    // Must parse the AST at compile time.
    let parsed_fields = match eip712::parse_fields(ast) {
        Ok(fields) => fields,
        Err(e) => return TokenStream::from(e),
    };

    // Compute the type hash for the derived struct using the parsed fields from above.
    let type_hash =
        hex::encode(eip712::make_type_hash(primary_type.clone().to_string(), &parsed_fields));

    // Use reference to ethers_core instead of directly using the crate itself.
    let ethers_core = ethers_core_crate();

    let implementation = quote! {
        impl Eip712 for #primary_type {
            type Error = #ethers_core::types::transaction::eip712::Eip712Error;

            fn type_hash() -> Result<[u8; 32], Self::Error> {
                use std::convert::TryFrom;
                let decoded = #ethers_core::utils::hex::decode(#type_hash)?;
                let byte_array: [u8; 32] = <[u8; 32]>::try_from(&decoded[..])?;
                Ok(byte_array)
            }

            // Return the pre-computed domain separator from compile time;
            fn domain_separator(&self) -> Result<[u8; 32], Self::Error> {
                use std::convert::TryFrom;
                let decoded = #ethers_core::utils::hex::decode(#domain_separator)?;
                let byte_array: [u8; 32] = <[u8; 32]>::try_from(&decoded[..])?;
                Ok(byte_array)
            }

            fn domain(&self) -> Result<#ethers_core::types::transaction::eip712::EIP712Domain, Self::Error> {
                let domain: #ethers_core::types::transaction::eip712::EIP712Domain = # ethers_core::utils::__serde_json::from_str(#domain_str)?;

                Ok(domain)
            }

            fn struct_hash(&self) -> Result<[u8; 32], Self::Error> {
                use #ethers_core::abi::Tokenizable;
                let mut items = vec![#ethers_core::abi::Token::Uint(
                    #ethers_core::types::U256::from(&Self::type_hash()?[..]),
                )];

                if let #ethers_core::abi::Token::Tuple(tokens) = self.clone().into_token() {
                    for token in tokens {
                        match &token {
                            #ethers_core::abi::Token::Tuple(t) => {
                                // TODO: check for nested Eip712 Type;
                                // Challenge is determining the type hash
                                return Err(Self::Error::NestedEip712StructNotImplemented);
                            },
                            _ => {
                                items.push(#ethers_core::types::transaction::eip712::encode_eip712_type(token));
                            }
                        }
                    }
                }

                let struct_hash = #ethers_core::utils::keccak256(#ethers_core::abi::encode(
                    &items,
                ));

                Ok(struct_hash)
            }
        }
    };

    implementation.into()
}
