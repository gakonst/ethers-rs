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
//! use derive_eip712::*;
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

use std::collections::HashMap;

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::quote;

use ethers_core::{
    abi,
    abi::Token,
    types::{Address, H160, U256},
    utils::keccak256,
};

/// This method provides the hex encoded domain type hash for EIP712Domain type;
/// This is used by all Eip712 structs.
fn eip712_domain_type_hash() -> [u8; 32] {
    keccak256("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)")
}

/// Eip712 Domain attributes used in determining the domain separator;
#[derive(Debug, Default)]
struct Eip712Domain {
    name: String,
    version: String,
    chain_id: U256,
    verifying_contract: Address,
}

impl Eip712Domain {
    // Compute the domain separator;
    // See: https://github.com/gakonst/ethers-rs/blob/master/examples/permit_hash.rs#L41
    pub fn separator(&self) -> String {
        hex::encode(keccak256(abi::encode(&[
            Token::Uint(U256::from(eip712_domain_type_hash())),
            Token::Uint(U256::from(keccak256(&self.name))),
            Token::Uint(U256::from(keccak256(&self.version))),
            Token::Uint(self.chain_id),
            Token::Address(self.verifying_contract),
        ])))
    }
}

// Parse the AST of the struct to determine the domain attributes
impl From<&syn::DeriveInput> for Eip712Domain {
    fn from(input: &syn::DeriveInput) -> Eip712Domain {
        let mut domain = Eip712Domain::default();

        let attributes = input.attrs.first().expect("missing macro arguments");

        let is_segment_valid = attributes
            .path
            .segments
            .first()
            .map(|s| s.ident == "eip712")
            .expect("missing eip712 macro arguments");

        if !is_segment_valid {
            panic!("invalid path segment, identity does not match 'eip712'")
        }

        let mut token_stream = attributes.tokens.clone().into_iter();

        if let Some(quote::__private::TokenTree::Group(g)) = token_stream.next() {
            let group_stream = g.stream().into_iter();
            let mut current_arg = String::new();
            for item in group_stream {
                if let quote::__private::TokenTree::Ident(ident) = item {
                    current_arg = ident.to_string();
                } else if let quote::__private::TokenTree::Literal(literal) = item {
                    match current_arg.as_ref() {
                        "name" => {
                            domain.name = literal.to_string().replace("\"", "");
                        }
                        "version" => {
                            domain.version = literal.to_string().replace("\"", "");
                        }
                        "chain_id" => {
                            domain.chain_id = literal
                                .to_string()
                                .parse::<U256>()
                                .expect("failed to parse chain id from macro arguments");
                        }
                        "verifying_contract" => {
                            domain.verifying_contract = literal
                                .to_string()
                                .replace("\"", "")
                                .parse::<H160>()
                                .expect("failed to parse verifying contract");
                        }
                        _ => {
                            panic!("expected arguments: 'name', 'version', 'chain_id' and 'verifying_contract'; found: {}", current_arg);
                        }
                    }
                }
            }
        };

        domain
    }
}

// Convert rust types to enc types. This is used in determining the type hash;
// NOTE: this is not an exhaustive list
fn parse_field_type(field_type: String) -> String {
    match field_type.as_ref() {
        "U128" => "uint128",
        "U256" => "uint256",
        "H128" => "bytes16",
        "H160" => "address",
        "H256" => "bytes32",
        "String" => "string",
        "Bytes" => "bytes",
        "Vec<u8>" => "bytes",
        "Vec<U128>" => "uint128[]",
        "Vec<U256>" => "uint256[]",
        "Vec<H128?" => "bytes16[]",
        "Vec<H160>" => "address[]",
        "Vec<H256?" => "bytes32[]",
        "Vec<String>" => "string[]",
        "Vec<Bytes>" => "bytes[]",
        _ => {
            // NOTE: This will fail if the field type does not match an ethereum type;
            &field_type
        }
    }
    .to_string()
}

// Parse the field type from the derived struct
fn parse_field(field: &syn::Field) -> String {
    let field_path = match &field.ty {
        syn::Type::Path(p) => p,
        _ => {
            panic!("field type must be a path")
        }
    };

    let segment = field_path
        .path
        .segments
        .first()
        .expect("field must have a type");

    let mut field_type = segment.ident.to_string();

    if let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments {
        field_type.push('<');
        for arg in &arguments.args {
            if let syn::GenericArgument::Type(syn::Type::Path(p)) = arg {
                let arg_identity = p
                    .path
                    .segments
                    .first()
                    .map(|s| s.ident.to_string())
                    .expect("argument must have an identity");

                field_type.push_str(&arg_identity);
            }
        }
        field_type.push('>');
    }

    parse_field_type(field_type)
}

// Return HashMap of the field name and the field type;
fn parse_fields(ast: &syn::DeriveInput) -> HashMap<String, String> {
    let mut parsed_fields = HashMap::new();

    let data = match &ast.data {
        syn::Data::Struct(s) => s,
        _ => {
            panic!("Eip712 can only be derived for a struct")
        }
    };

    let named_fields = match &data.fields {
        syn::Fields::Named(name) => name,
        _ => {
            panic!("unnamed fields are not supported")
        }
    };

    named_fields.named.iter().for_each(|f| {
        let field_name = f
            .ident
            .clone()
            .expect("field must be named")
            .to_string()
            .to_case(Case::Camel);

        let field_type = parse_field(f);

        parsed_fields.insert(field_name, field_type);
    });

    parsed_fields
}

// Convert hash map of field names and types into a type hash corresponding to enc types;
fn make_type_hash(primary_type: String, fields: &HashMap<String, String>) -> String {
    let parameters = fields
        .iter()
        .map(|(k, v)| format!("{} {}", v, k))
        .collect::<Vec<String>>()
        .join(",");

    let sig = format!("{}({})", primary_type, parameters);

    hex::encode(keccak256(sig))
}

// Main implementation macro, used to compute static values and define
// method for encoding the final eip712 payload;
fn impl_eip_712_macro(ast: &syn::DeriveInput) -> TokenStream {
    // Primary type should match the type in the ethereum verifying contract;
    let primary_type = &ast.ident;

    // Computer domain separator
    let domain_attributes: Eip712Domain = Eip712Domain::from(ast);
    let domain_separator = domain_attributes.separator();
    let domain_type_hash = hex::encode(eip712_domain_type_hash());

    // Must parse the AST at compile time.
    let parsed_fields = parse_fields(ast);

    // JSON Stringify the field names and types to pass into the
    // derived encode_eip712() method as a static string;
    // the AST of the struct is not available at runtime, so this is
    // a work around for passing in the struct fields;
    let fields: String = serde_json::to_string(&parsed_fields)
        .expect("failed to serialize parsed fields into JSON string");

    // Compute the type hash for the derived struct using the parsed fields from above;
    let type_hash = make_type_hash(primary_type.clone().to_string(), &parsed_fields);

    let implementation = quote! {
        #[derive(Debug, thiserror::Error)]
        pub enum Eip712Error {
            #[error("Failed to serialize serde JSON object")]
            SerdeJsonError(#[from] serde_json::Error),
            #[error("Failed to decode hex value")]
            FromHexError(#[from] hex::FromHexError),
            #[error("Failed to make struct hash from values")]
            FailedToEncodeStruct
        }

        fn make_struct_hash<T: std::fmt::Debug + serde::Serialize>(
            data: &T,
            domain_separator: &'static str,
            type_hash: &'static str,
            _fields: &'static str,
        ) -> Result<String, Eip712Error> {
            let _fields: serde_json::Value = serde_json::from_str(_fields)?;

            if let serde_json::Value::Object(fields) = _fields {
                let mut keys = fields.keys().map(|f| f.to_string()).collect::<Vec<String>>();

                // sort the fields alphabetically;
                // NOTE: the solidity type hash should also use the same convention;
                keys.sort();

                let _values: serde_json::Value = serde_json::to_value(data)?;

                if let serde_json::Value::Object(obj) = _values {
                    // Initialize the items with the type hash
                    let mut items = vec![ethers_core::abi::Token::Uint(
                        ethers_core::types::U256::from(&hex::decode(type_hash)?[..]),
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
                        &hex::decode(domain_separator)?[..],
                        &struct_hash[..]
                    ].concat();

                    return Ok(hex::encode(ethers_core::utils::keccak256(digest_input)));
                }
            }

            // Reached Error:
            Err(Eip712Error::FailedToEncodeStruct)
        }

        impl Eip712 for #primary_type {
            type Error = Eip712Error;

            fn type_hash() -> String {
                #type_hash.to_string()
            }

            fn domain_separator() -> String {
                #domain_separator.to_string()
            }

            fn encode_eip712(&self) -> Result<String, Self::Error> {
                Ok(make_struct_hash(self, #domain_separator, #type_hash, #fields)?.to_string())
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
