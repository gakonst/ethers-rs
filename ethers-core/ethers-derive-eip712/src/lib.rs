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
//! # Example Usage
//!
//! ```rust
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
//! let hash = puzzle.encode_eip712()?;
//! ```
//!
//! # Limitations
//!
//! At the moment, the derive macro does not recursively encode nested Eip712 structs.
//!
//! There is an Inner helper attribute `#[eip712]` for fields that will eventually be used to
//! determine if there is a nested eip712 struct. However, this work is not yet complete.
//!
use std::convert::TryFrom;

use ethers_core::types::transaction::eip712;
use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(Eip712, attributes(eip712))]
pub fn eip_712_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).expect("failed to parse token stream for Eip712 derived struct");

    impl_eip_712_macro(&ast)
}

// Main implementation macro, used to compute static values and define
// method for encoding the final eip712 payload;
fn impl_eip_712_macro(ast: &syn::DeriveInput) -> TokenStream {
    // Primary type should match the type in the ethereum verifying contract;
    let primary_type = &ast.ident;

    // Computer domain separator
    let domain = match eip712::EIP712Domain::try_from(ast) {
        Ok(attributes) => attributes,
        Err(e) => return TokenStream::from(e),
    };

    let domain_separator = hex::encode(domain.separator());

    // Must parse the AST at compile time.
    let parsed_fields = match eip712::parse_fields(ast) {
        Ok(fields) => fields,
        Err(e) => return TokenStream::from(e),
    };

    // Compute the type hash for the derived struct using the parsed fields from above;
    let type_hash = hex::encode(eip712::make_type_hash(
        primary_type.clone().to_string(),
        &parsed_fields,
    ));

    let implementation = quote! {
        impl Eip712 for #primary_type {
            type Error = ethers_core::types::transaction::eip712::Eip712Error;

            fn type_hash() -> Result<[u8; 32], Self::Error> {
                use std::convert::TryFrom;
                let decoded = hex::decode(#type_hash.to_string())?;
                let byte_array: [u8; 32] = <[u8; 32]>::try_from(&decoded[..])?;
                Ok(byte_array)
            }

            fn domain_separator() -> Result<[u8; 32], Self::Error> {
                use std::convert::TryFrom;
                let decoded = hex::decode(#domain_separator.to_string())?;
                let byte_array: [u8; 32] = <[u8; 32]>::try_from(&decoded[..])?;
                Ok(byte_array)
            }

            fn struct_hash(self) -> Result<[u8; 32], Self::Error> {
                use ethers_core::abi::Tokenizable;
                let mut items = vec![ethers_core::abi::Token::Uint(
                    ethers_core::types::U256::from(&Self::type_hash()?[..]),
                )];

                if let ethers_core::abi::Token::Tuple(tokens) = self.clone().into_token() {
                    for token in tokens {
                        match &token {
                            ethers_core::abi::Token::Tuple(t) => {
                                // TODO: check for nested Eip712 Type;
                                // Challenge is determining the type hash
                            },
                            _ => {
                                items.push(ethers_core::types::transaction::eip712::encode_eip712_type(token));
                            }
                        }
                    }
                }

                let struct_hash = ethers_core::utils::keccak256(ethers_core::abi::encode(
                    &items,
                ));

                Ok(struct_hash)
            }

            fn encode_eip712(self) -> Result<[u8; 32], Self::Error> {
                let struct_hash = self.struct_hash()?;

                // encode the digest to be compatible with solidity abi.encodePacked()
                // See: https://github.com/gakonst/ethers-rs/blob/master/examples/permit_hash.rs#L72
                let digest_input = [
                    &[0x19, 0x01],
                    &Self::domain_separator()?[..],
                    &struct_hash[..]
                ].concat();

                return Ok(ethers_core::utils::keccak256(digest_input));

            }
        }
    };

    implementation.into()
}
