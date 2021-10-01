//! TL;DR you're probably looking for `ethers-derive-eip712` Eip712 derive macro.
//!
//! The eip712 module contains helper methods and types mainly used
//! by the derive-eip712 procedural macro. Note that many of the methods
//! used in this module may panic!. While this is desired behavior for a
//! procedural macro, it may not be the behavior you wish to use in your
//! application if using these methods manually.
use std::collections::HashMap;

use convert_case::{Case, Casing};

use crate::{
    abi,
    abi::Token,
    types::{Address, H160, U256},
    utils::keccak256,
};

/// Error typed used by Eip712 derive macro
#[derive(Debug, thiserror::Error)]
pub enum Eip712Error {
    #[error("Failed to serialize serde JSON object")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("Failed to decode hex value")]
    FromHexError(#[from] hex::FromHexError),
    #[error("Failed to make struct hash from values")]
    FailedToEncodeStruct,
}

/// The Eip712 trait provides helper methods for computing
/// the typed data hash used in `eth_signTypedData`.
///
/// The ethers-rs `derive_eip712` crate provides a derive macro to
/// implement the trait for a given struct. See documentation
/// for `derive_eip712` for more information and example usage.
///
/// For those who wish to manually implement this trait, see:
/// https://eips.ethereum.org/EIPS/eip-712
///
/// Any rust struct implementing Eip712 must also have a corresponding
/// struct in the verifying ethereum contract that matches its signature.
///
/// NOTE: Due to limitations of the derive macro not supporting return types of
/// [u8; 32] or Vec<u8>, all methods should return the hex encoded values of the keccak256
/// byte array.
pub trait Eip712 {
    /// User defined error type;
    type Error: std::error::Error + Send + Sync + std::fmt::Debug;

    /// The eip712 domain is the same for all Eip712 implementations,
    /// This method does not need to be manually implemented, but may be overridden
    /// if needed.
    fn eip712_domain_type_hash() -> String {
        hex::encode(crate::utils::keccak256(
            "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)",
        ))
    }

    /// The domain separator depends on the contract and unique domain
    /// for which the user is targeting. In the derive macro, these attributes
    /// are passed in as arguments to the macro. When manually deriving, the user
    /// will need to know the name of the domain, version of the contract, chain ID of
    /// where the contract lives and the address of the verifying contract.
    fn domain_separator() -> String;

    /// This method is used for calculating the hash of the type signature of the
    /// struct. The field types of the struct must map to primitive
    /// ethereum types or custom types defined in the contract.
    fn type_hash() -> String;

    /// When using the derive macro, this is the primary method used for computing the final
    /// EIP-712 encoded payload. This method relies on the aforementioned methods for computing
    /// the final encoded payload.
    fn encode_eip712(&self) -> Result<String, Self::Error>;
}

/// This method provides the hex encoded domain type hash for EIP712Domain type;
/// This is used by all Eip712 structs.
pub fn eip712_domain_type_hash() -> [u8; 32] {
    keccak256("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)")
}

/// Eip712 Domain attributes used in determining the domain separator;
#[derive(Debug, Default)]
pub struct Eip712Domain {
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
// NOTE: this is not an exhaustive list, and there may already be an existing mapping
// in another library.
pub fn parse_field_type(field_type: String) -> String {
    match field_type.as_ref() {
        "U128" => "uint128",
        "U256" => "uint256",
        "H128" => "bytes16",
        "H160" => "address",
        "H256" => "bytes32",
        "String" => "string",
        "Boolean" => "boolean",
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

/// Parse the field type from the derived struct
pub fn parse_field(field: &syn::Field) -> String {
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

/// Return HashMap of the field name and the field type;
pub fn parse_fields(ast: &syn::DeriveInput) -> HashMap<String, String> {
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

/// Convert hash map of field names and types into a type hash corresponding to enc types;
pub fn make_type_hash(primary_type: String, fields: &HashMap<String, String>) -> String {
    let parameters = fields
        .iter()
        .map(|(k, v)| format!("{} {}", v, k))
        .collect::<Vec<String>>()
        .join(",");

    let sig = format!("{}({})", primary_type, parameters);

    hex::encode(keccak256(sig))
}
