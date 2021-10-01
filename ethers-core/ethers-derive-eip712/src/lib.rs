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
//! All String values returned by the implemented methods are hex encoded and should be
//! decoded into `[u8; 32]` for signing. See example for decoding.
//!
//! # Example Usage
//!
//! ```rust
//! use ethers_derive_eip712::*;
//! use ethers_core::types::{transaction::eip712::Eip712, H160};
//! use serde::Serialize;
//!
//! #[derive(Debug, Eip712, Serialize)]
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
//!
//! let decoded: Vec<u8> = hex::decode(hash).expect("failed to decode")
//! let byte_array: [u8; 32] = <[u8; 32]>::try_from(&decoded[..])?;
//! ```
//!
//! # Limitations
//!
//! At the moment, the derive macro does not recursively encode nested Eip712 structs.
//!

use proc_macro::TokenStream;
use quote::quote;

// import eip712 utilities from ethers_core::types::transaction::eip712
use ethers_core::types::transaction::eip712;

// Main implementation macro, used to compute static values and define
// method for encoding the final eip712 payload;
fn impl_eip_712_macro(ast: &syn::DeriveInput) -> TokenStream {
    // Primary type should match the type in the ethereum verifying contract;
    let primary_type = &ast.ident;

    // Computer domain separator
    let domain_attributes: eip712::Eip712Domain = eip712::Eip712Domain::from(ast);
    let domain_separator = domain_attributes.separator();
    let domain_type_hash = hex::encode(eip712::eip712_domain_type_hash());

    // Must parse the AST at compile time.
    let parsed_fields = eip712::parse_fields(ast);

    // JSON Stringify the field names and types to pass into the
    // derived encode_eip712() method as a static string;
    // the AST of the struct is not available at runtime, so this is
    // a work around for passing in the struct fields;
    let fields: String = serde_json::to_string(&parsed_fields)
        .expect("failed to serialize parsed fields into JSON string");

    // Compute the type hash for the derived struct using the parsed fields from above;
    let type_hash = eip712::make_type_hash(primary_type.clone().to_string(), &parsed_fields);

    let implementation = quote! {
        impl Eip712 for #primary_type {
            type Error = ethers_core::types::transaction::eip712::Eip712Error;

            fn type_hash() -> String {
                #type_hash.to_string()
            }

            fn domain_separator() -> String {
                #domain_separator.to_string()
            }

            fn encode_eip712(&self) -> Result<String, Self::Error> {
                // Ok(make_struct_hash(self, #domain_separator, #type_hash, #fields)?.to_string())
                let json: serde_json::Value = serde_json::from_str(#fields)?;

                if let serde_json::Value::Object(fields) = json {
                    let mut keys = fields.keys().map(|f| f.to_string()).collect::<Vec<String>>();

                    // sort the fields alphabetically;
                    // NOTE: the solidity type hash should also use the same convention;
                    keys.sort();

                    let _values: serde_json::Value = serde_json::to_value(self)?;

                    if let serde_json::Value::Object(obj) = _values {
                        // Initialize the items with the type hash
                        let mut items = vec![ethers_core::abi::Token::Uint(
                            ethers_core::types::U256::from(&hex::decode(#type_hash)?[..]),
                        )];

                        for key in keys {
                            if let Some(v) = obj.get(&key) {
                                if let Some(ty) = fields.get(&key) {
                                    if let serde_json::Value::String(value) = v{
                                        if let serde_json::Value::String(field_type) = ty {
                                            // convert encoded type;
                                            let item = match field_type.as_ref() {
                                                // TODO: This following enc types are not exhaustive;
                                                // Check types against solidity abi.encodePacked()
                                                "uint128" => ethers_core::abi::Token::Uint(ethers_core::types::U256::from(value.parse::<usize>().expect("failed to parse unsigned integer"))),
                                                "uint256" => ethers_core::abi::Token::Uint(ethers_core::types::U256::from(value.parse::<usize>().expect("failed to parse unsigned integer"))),
                                                "address" => ethers_core::abi::Token::Address(value.parse::<ethers_core::types::Address>().expect("failed to parse address")),
                                                _ => { ethers_core::abi::Token::Uint(ethers_core::types::U256::from(ethers_core::utils::keccak256(value))) }
                                            };

                                            // Add the parsed field type to the items to be encoded;
                                            items.push(item);
                                        }
                                    }
                                }
                            }
                        }

                        let struct_hash = ethers_core::utils::keccak256(ethers_core::abi::encode(
                            &items,
                        ));

                        // encode the digest to be compatible with solidity abi.encodePacked()
                        // See: https://github.com/gakonst/ethers-rs/blob/master/examples/permit_hash.rs#L72
                        let digest_input = [
                            &[0x19, 0x01],
                            &hex::decode(#domain_separator)?[..],
                            &struct_hash[..]
                        ].concat();

                        return Ok(hex::encode(ethers_core::utils::keccak256(digest_input)));
                    }
                }

                // Reached Error:
                Err(ethers_core::types::transaction::eip712::Eip712Error::FailedToEncodeStruct)

            }

            fn eip712_domain_type_hash() -> String {
                #domain_type_hash.to_string()
            }
        }
    };

    implementation.into()
}

#[proc_macro_derive(Eip712, attributes(eip712))]
pub fn eip_712_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).expect("failed to parse token stream for Eip712 derived struct");

    impl_eip_712_macro(&ast)
}
