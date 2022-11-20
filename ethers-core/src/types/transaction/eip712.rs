use crate::{
    abi,
    abi::{HumanReadableParser, ParamType, Token},
    types::{serde_helpers::StringifiedNumeric, Address, Bytes, U256},
    utils::keccak256,
};
use convert_case::{Case, Casing};
use core::convert::TryFrom;
use ethabi::encode;
use proc_macro2::TokenStream;
use serde::{Deserialize, Deserializer, Serialize};
use std::{
    collections::{BTreeMap, HashSet},
    convert::TryInto,
    iter::FromIterator,
};
use syn::{
    parse::Error, spanned::Spanned as _, AttrStyle, Data, DeriveInput, Expr, Fields,
    GenericArgument, Lit, NestedMeta, PathArguments, Type,
};

/// Custom types for `TypedData`
pub type Types = BTreeMap<String, Vec<Eip712DomainType>>;

/// Pre-computed value of the following statement:
///
/// `ethers_core::utils::keccak256("EIP712Domain(string name,string version,uint256 chainId,address
/// verifyingContract)")`
pub const EIP712_DOMAIN_TYPE_HASH: [u8; 32] = [
    139, 115, 195, 198, 155, 184, 254, 61, 81, 46, 204, 76, 247, 89, 204, 121, 35, 159, 123, 23,
    155, 15, 250, 202, 169, 167, 93, 82, 43, 57, 64, 15,
];

/// Pre-computed value of the following statement:
///
/// `ethers_core::utils::keccak256("EIP712Domain(string name,string version,uint256 chainId,address
/// verifyingContract,bytes32 salt)")`
pub const EIP712_DOMAIN_TYPE_HASH_WITH_SALT: [u8; 32] = [
    216, 124, 214, 239, 121, 212, 226, 185, 94, 21, 206, 138, 191, 115, 45, 181, 30, 199, 113, 241,
    202, 46, 220, 207, 34, 164, 108, 114, 154, 197, 100, 114,
];

/// Error typed used by Eip712 derive macro
#[derive(Debug, thiserror::Error)]
pub enum Eip712Error {
    #[error("Failed to serialize serde JSON object")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("Failed to decode hex value")]
    FromHexError(#[from] hex::FromHexError),
    #[error("Failed to make struct hash from values")]
    FailedToEncodeStruct,
    #[error("Failed to convert slice into byte array")]
    TryFromSliceError(#[from] std::array::TryFromSliceError),
    #[error("Nested Eip712 struct not implemented. Failed to parse.")]
    NestedEip712StructNotImplemented,
    #[error("Error from Eip712 struct: {0:?}")]
    Message(String),
}

/// The Eip712 trait provides helper methods for computing
/// the typed data hash used in `eth_signTypedData`.
///
/// The ethers-rs `derive_eip712` crate provides a derive macro to
/// implement the trait for a given struct. See documentation
/// for `derive_eip712` for more information and example usage.
///
/// For those who wish to manually implement this trait, see:
/// <https://eips.ethereum.org/EIPS/eip-712>
///
/// Any rust struct implementing Eip712 must also have a corresponding
/// struct in the verifying ethereum contract that matches its signature.
pub trait Eip712 {
    /// User defined error type;
    type Error: std::error::Error + Send + Sync + std::fmt::Debug;

    /// Default implementation of the domain separator;
    fn domain_separator(&self) -> Result<[u8; 32], Self::Error> {
        Ok(self.domain()?.separator())
    }

    /// Returns the current domain. The domain depends on the contract and unique domain
    /// for which the user is targeting. In the derive macro, these attributes
    /// are passed in as arguments to the macro. When manually deriving, the user
    /// will need to know the name of the domain, version of the contract, chain ID of
    /// where the contract lives and the address of the verifying contract.
    fn domain(&self) -> Result<EIP712Domain, Self::Error>;

    /// This method is used for calculating the hash of the type signature of the
    /// struct. The field types of the struct must map to primitive
    /// ethereum types or custom types defined in the contract.
    fn type_hash() -> Result<[u8; 32], Self::Error>;

    /// Hash of the struct, according to EIP-712 definition of `hashStruct`
    fn struct_hash(&self) -> Result<[u8; 32], Self::Error>;

    /// When using the derive macro, this is the primary method used for computing the final
    /// EIP-712 encoded payload. This method relies on the aforementioned methods for computing
    /// the final encoded payload.
    fn encode_eip712(&self) -> Result<[u8; 32], Self::Error> {
        // encode the digest to be compatible with solidity abi.encodePacked()
        // See: https://github.com/gakonst/ethers-rs/blob/master/examples/permit_hash.rs#L72

        let domain_separator = self.domain_separator()?;
        let struct_hash = self.struct_hash()?;

        let digest_input = [&[0x19, 0x01], &domain_separator[..], &struct_hash[..]].concat();

        Ok(keccak256(digest_input))
    }
}

/// Eip712 Domain attributes used in determining the domain separator;
/// Unused fields are left out of the struct type.
///
/// Protocol designers only need to include the fields that make sense for their signing domain.
/// Unused fields are left out of the struct type.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EIP712Domain {
    ///  The user readable name of signing domain, i.e. the name of the DApp or the protocol.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The current major version of the signing domain. Signatures from different versions are not
    /// compatible.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// The EIP-155 chain id. The user-agent should refuse signing if it does not match the
    /// currently active chain.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::types::serde_helpers::deserialize_stringified_numeric_opt"
    )]
    pub chain_id: Option<U256>,

    /// The address of the contract that will verify the signature.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verifying_contract: Option<Address>,

    /// A disambiguating salt for the protocol. This can be used as a domain separator of last
    /// resort.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub salt: Option<[u8; 32]>,
}

impl EIP712Domain {
    // Compute the domain separator;
    // See: https://github.com/gakonst/ethers-rs/blob/master/examples/permit_hash.rs#L41
    pub fn separator(&self) -> [u8; 32] {
        // full name is `EIP712Domain(string name,string version,uint256 chainId,address
        // verifyingContract,bytes32 salt)`
        let mut ty = "EIP712Domain(".to_string();

        let mut tokens = Vec::new();
        let mut needs_comma = false;
        if let Some(ref name) = self.name {
            ty += "string name";
            tokens.push(Token::Uint(U256::from(keccak256(name))));
            needs_comma = true;
        }

        if let Some(ref version) = self.version {
            if needs_comma {
                ty.push(',');
            }
            ty += "string version";
            tokens.push(Token::Uint(U256::from(keccak256(version))));
            needs_comma = true;
        }

        if let Some(chain_id) = self.chain_id {
            if needs_comma {
                ty.push(',');
            }
            ty += "uint256 chainId";
            tokens.push(Token::Uint(chain_id));
            needs_comma = true;
        }

        if let Some(verifying_contract) = self.verifying_contract {
            if needs_comma {
                ty.push(',');
            }
            ty += "address verifyingContract";
            tokens.push(Token::Address(verifying_contract));
            needs_comma = true;
        }

        if let Some(salt) = self.salt {
            if needs_comma {
                ty.push(',');
            }
            ty += "bytes32 salt";
            tokens.push(Token::Uint(U256::from(salt)));
        }

        ty.push(')');

        tokens.insert(0, Token::Uint(U256::from(keccak256(ty))));

        keccak256(encode(&tokens))
    }
}

#[derive(Debug, Clone)]
pub struct EIP712WithDomain<T>
where
    T: Clone + Eip712,
{
    pub domain: EIP712Domain,
    pub inner: T,
}

impl<T: Eip712 + Clone> EIP712WithDomain<T> {
    pub fn new(inner: T) -> Result<Self, Eip712Error> {
        let domain = inner.domain().map_err(|e| Eip712Error::Message(e.to_string()))?;

        Ok(Self { domain, inner })
    }

    #[must_use]
    pub fn set_domain(self, domain: EIP712Domain) -> Self {
        Self { domain, inner: self.inner }
    }
}

impl<T: Eip712 + Clone> Eip712 for EIP712WithDomain<T> {
    type Error = Eip712Error;

    fn domain(&self) -> Result<EIP712Domain, Self::Error> {
        Ok(self.domain.clone())
    }

    fn type_hash() -> Result<[u8; 32], Self::Error> {
        let type_hash = T::type_hash().map_err(|e| Self::Error::Message(e.to_string()))?;
        Ok(type_hash)
    }

    fn struct_hash(&self) -> Result<[u8; 32], Self::Error> {
        let struct_hash =
            self.inner.clone().struct_hash().map_err(|e| Self::Error::Message(e.to_string()))?;
        Ok(struct_hash)
    }
}

// Parse the AST of the struct to determine the domain attributes
impl TryFrom<&syn::DeriveInput> for EIP712Domain {
    type Error = TokenStream;
    fn try_from(input: &syn::DeriveInput) -> Result<EIP712Domain, Self::Error> {
        let mut domain = EIP712Domain::default();

        let mut found_eip712_attribute = false;

        'attribute_search: for attribute in input.attrs.iter() {
            if let AttrStyle::Outer = attribute.style {
                if let Ok(syn::Meta::List(meta)) = attribute.parse_meta() {
                    if meta.path.is_ident("eip712") {
                        found_eip712_attribute = true;

                        for n in meta.nested.iter() {
                            if let NestedMeta::Meta(meta) = n {
                                match meta {
                                    syn::Meta::NameValue(meta) => {
                                        let ident = meta.path.get_ident().ok_or_else(|| {
                                            Error::new(
                                                meta.path.span(),
                                                "unrecognized eip712 parameter",
                                            )
                                            .to_compile_error()
                                        })?;

                                        match ident.to_string().as_ref() {
                                            "name" => match meta.lit {
                                                syn::Lit::Str(ref lit_str) => {
                                                    if domain.name.is_some() {
                                                        return Err(Error::new(
                                                            meta.path.span(),
                                                            "domain name already specified",
                                                        )
                                                        .to_compile_error())
                                                    }

                                                    domain.name = Some(lit_str.value());
                                                }
                                                _ => {
                                                    return Err(Error::new(
                                                        meta.path.span(),
                                                        "domain name must be a string",
                                                    )
                                                    .to_compile_error())
                                                }
                                            },
                                            "version" => match meta.lit {
                                                syn::Lit::Str(ref lit_str) => {
                                                    if domain.version.is_some() {
                                                        return Err(Error::new(
                                                            meta.path.span(),
                                                            "domain version already specified",
                                                        )
                                                        .to_compile_error())
                                                    }

                                                    domain.version = Some(lit_str.value());
                                                }
                                                _ => {
                                                    return Err(Error::new(
                                                        meta.path.span(),
                                                        "domain version must be a string",
                                                    )
                                                    .to_compile_error())
                                                }
                                            },
                                            "chain_id" => match meta.lit {
                                                syn::Lit::Int(ref lit_int) => {
                                                    if domain.chain_id.is_some() {
                                                        return Err(Error::new(
                                                            meta.path.span(),
                                                            "domain chain_id already specified",
                                                        )
                                                        .to_compile_error())
                                                    }

                                                    domain.chain_id = Some(U256::from(
                                                        lit_int.base10_parse::<u64>().map_err(
                                                            |_| {
                                                                Error::new(
                                                                    meta.path.span(),
                                                                    "failed to parse chain id",
                                                                )
                                                                .to_compile_error()
                                                            },
                                                        )?,
                                                    ));
                                                }
                                                _ => {
                                                    return Err(Error::new(
                                                        meta.path.span(),
                                                        "domain chain_id must be a positive integer",
                                                    )
                                                    .to_compile_error());
                                                }
                                            },
                                            "verifying_contract" => match meta.lit {
                                                syn::Lit::Str(ref lit_str) => {
                                                    if domain.verifying_contract.is_some() {
                                                        return Err(Error::new(
                                                            meta.path.span(),
                                                            "domain verifying_contract already specified",
                                                        )
                                                        .to_compile_error());
                                                    }

                                                    domain.verifying_contract = Some(lit_str.value().parse().map_err(|_| {
                                                            Error::new(
                                                                meta.path.span(),
                                                                "failed to parse verifying contract into Address",
                                                            )
                                                            .to_compile_error()
                                                        })?);
                                                }
                                                _ => {
                                                    return Err(Error::new(
                                                        meta.path.span(),
                                                        "domain verifying_contract must be a string",
                                                    )
                                                    .to_compile_error());
                                                }
                                            },
                                            "salt" => match meta.lit {
                                                syn::Lit::Str(ref lit_str) => {
                                                    if domain.salt.is_some() {
                                                        return Err(Error::new(
                                                            meta.path.span(),
                                                            "domain salt already specified",
                                                        )
                                                        .to_compile_error())
                                                    }

                                                    // keccak256(<string>) to compute bytes32
                                                    // encoded domain salt
                                                    let salt = keccak256(lit_str.value());

                                                    domain.salt = Some(salt);
                                                }
                                                _ => {
                                                    return Err(Error::new(
                                                        meta.path.span(),
                                                        "domain salt must be a string",
                                                    )
                                                    .to_compile_error())
                                                }
                                            },
                                            _ => {
                                                return Err(Error::new(
                                                    meta.path.span(),
                                                    "unrecognized eip712 parameter; must be one of 'name', 'version', 'chain_id', or 'verifying_contract'",
                                                )
                                                .to_compile_error());
                                            }
                                        }
                                    }
                                    syn::Meta::Path(path) => {
                                        return Err(Error::new(
                                            path.span(),
                                            "unrecognized eip712 parameter",
                                        )
                                        .to_compile_error())
                                    }
                                    syn::Meta::List(meta) => {
                                        return Err(Error::new(
                                            meta.path.span(),
                                            "unrecognized eip712 parameter",
                                        )
                                        .to_compile_error())
                                    }
                                }
                            }
                        }
                    }

                    break 'attribute_search
                }
            }
        }

        if !found_eip712_attribute {
            return Err(Error::new_spanned(
                input,
                "missing required derive attribute: '#[eip712( ... )]'".to_string(),
            )
            .to_compile_error())
        }

        Ok(domain)
    }
}

/// Represents the [EIP-712](https://eips.ethereum.org/EIPS/eip-712) typed data object.
///
/// Typed data is a JSON object containing type information, domain separator parameters and the
/// message object which has the following schema
///
/// ```js
/// {
//   type: 'object',
//   properties: {
//     types: {
//       type: 'object',
//       properties: {
//         EIP712Domain: {type: 'array'},
//       },
//       additionalProperties: {
//         type: 'array',
//         items: {
//           type: 'object',
//           properties: {
//             name: {type: 'string'},
//             type: {type: 'string'}
//           },
//           required: ['name', 'type']
//         }
//       },
//       required: ['EIP712Domain']
//     },
//     primaryType: {type: 'string'},
//     domain: {type: 'object'},
//     message: {type: 'object'}
//   },
//   required: ['types', 'primaryType', 'domain', 'message']
// }
/// ```
///
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TypedData {
    /// Signing domain metadata. The signing domain is the intended context for the signature (e.g.
    /// the dapp, protocol, etc. that it's intended for). This data is used to construct the domain
    /// seperator of the message.
    pub domain: EIP712Domain,
    /// The custom types used by this message.
    pub types: Types,
    #[serde(rename = "primaryType")]
    /// The type of the message.
    pub primary_type: String,
    /// The message to be signed.
    pub message: BTreeMap<String, serde_json::Value>,
}

/// According to the MetaMask implementation,
/// the message parameter may be JSON stringified in versions later than V1
/// See <https://github.com/MetaMask/metamask-extension/blob/0dfdd44ae7728ed02cbf32c564c75b74f37acf77/app/scripts/metamask-controller.js#L1736>
/// In fact, ethers.js JSON stringifies the message at the time of writing.
impl<'de> Deserialize<'de> for TypedData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct TypedDataHelper {
            domain: EIP712Domain,
            types: Types,
            #[serde(rename = "primaryType")]
            primary_type: String,
            message: BTreeMap<String, serde_json::Value>,
        }

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Type {
            Val(TypedDataHelper),
            String(String),
        }

        match Type::deserialize(deserializer)? {
            Type::Val(v) => {
                let TypedDataHelper { domain, types, primary_type, message } = v;
                Ok(TypedData { domain, types, primary_type, message })
            }
            Type::String(s) => {
                let TypedDataHelper { domain, types, primary_type, message } =
                    serde_json::from_str(&s).map_err(serde::de::Error::custom)?;
                Ok(TypedData { domain, types, primary_type, message })
            }
        }
    }
}

// === impl TypedData ===

impl Eip712 for TypedData {
    type Error = Eip712Error;

    fn domain(&self) -> Result<EIP712Domain, Self::Error> {
        Ok(self.domain.clone())
    }

    fn type_hash() -> Result<[u8; 32], Self::Error> {
        Err(Eip712Error::Message("dynamic type".to_string()))
    }

    fn struct_hash(&self) -> Result<[u8; 32], Self::Error> {
        let tokens = encode_data(
            &self.primary_type,
            &serde_json::Value::Object(serde_json::Map::from_iter(self.message.clone())),
            &self.types,
        )?;
        Ok(keccak256(encode(&tokens)))
    }

    /// Hash a typed message according to EIP-712. The returned message starts with the EIP-712
    /// prefix, which is "1901", followed by the hash of the domain separator, then the data (if
    /// any). The result is hashed again and returned.
    fn encode_eip712(&self) -> Result<[u8; 32], Self::Error> {
        let domain_separator = self.domain.separator();
        let mut digest_input = [&[0x19, 0x01], &domain_separator[..]].concat().to_vec();

        if self.primary_type != "EIP712Domain" {
            // compatibility with <https://github.com/MetaMask/eth-sig-util>
            digest_input.extend(&self.struct_hash()?[..])
        }
        Ok(keccak256(digest_input))
    }
}

/// Represents the name and type pair
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Eip712DomainType {
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
}

/// Encodes an object by encoding and concatenating each of its members.
///
/// The encoding of a struct instance is `enc(value₁) ‖ enc(value₂) ‖ … ‖ enc(valueₙ)`, i.e. the
/// concatenation of the encoded member values in the order that they appear in the type. Each
/// encoded member value is exactly 32-byte long.
///
///   - `primaryType`: The root type.
///   - `data`: The object to encode.
///   - `types`: Type definitions for all types included in the message.
///
/// Returns an encoded representation of an object
pub fn encode_data(
    primary_type: &str,
    data: &serde_json::Value,
    types: &Types,
) -> Result<Vec<Token>, Eip712Error> {
    let hash = hash_type(primary_type, types)?;
    let mut tokens = vec![Token::Uint(U256::from(hash))];

    if let Some(fields) = types.get(primary_type) {
        for field in fields {
            // handle recursive types
            if let Some(value) = data.get(&field.name) {
                let field = encode_field(types, &field.name, &field.r#type, value)?;
                tokens.push(field);
            } else if types.contains_key(&field.r#type) {
                tokens.push(Token::Uint(U256::zero()));
            } else {
                return Err(Eip712Error::Message(format!("No data found for: `{}`", field.name)))
            }
        }
    }

    Ok(tokens)
}

/// Hashes an object
///
///   - `primary_type`: The root type to encode.
///   - `data`: The object to hash.
///   - `types`: All type definitions.
///
/// Returns the hash of the `primary_type` object
pub fn hash_struct(
    primary_type: &str,
    data: &serde_json::Value,
    types: &Types,
) -> Result<[u8; 32], Eip712Error> {
    let tokens = encode_data(primary_type, data, types)?;
    let encoded = encode(&tokens);
    Ok(keccak256(encoded))
}

/// Returns the hashed encoded type of `primary_type`
pub fn hash_type(primary_type: &str, types: &Types) -> Result<[u8; 32], Eip712Error> {
    encode_type(primary_type, types).map(keccak256)
}

///  Encodes the type of an object by encoding a comma delimited list of its members.
///
///   - `primary_type`: The root type to encode.
///   - `types`: All type definitions.
///
/// Returns the encoded representation of the field.
pub fn encode_type(primary_type: &str, types: &Types) -> Result<String, Eip712Error> {
    let mut names = HashSet::new();
    find_type_dependencies(primary_type, types, &mut names);
    // need to ensure primary_type is first in the list
    names.remove(primary_type);
    let mut deps: Vec<_> = names.into_iter().collect();
    deps.sort_unstable();
    deps.insert(0, primary_type);

    let mut res = String::new();

    for dep in deps.into_iter() {
        let fields = types.get(dep).ok_or_else(|| {
            Eip712Error::Message(format!("No type definition found for: `{dep}`"))
        })?;

        res += dep;
        res.push('(');
        res += &fields
            .iter()
            .map(|ty| format!("{} {}", ty.r#type, ty.name))
            .collect::<Vec<_>>()
            .join(",");

        res.push(')');
    }
    Ok(res)
}

/// Returns all the custom types used in the `primary_type`
fn find_type_dependencies<'a>(
    primary_type: &'a str,
    types: &'a Types,
    found: &mut HashSet<&'a str>,
) {
    if found.contains(primary_type) {
        return
    }
    if let Some(fields) = types.get(primary_type) {
        found.insert(primary_type);
        for field in fields {
            // need to strip the array tail
            let ty = field.r#type.split('[').next().unwrap();
            find_type_dependencies(ty, types, found)
        }
    }
}

/// Encode a single field.
///
///   - `types`: All type definitions.
///   - `field`: The name and type of the field being encoded.
///   - `value`: The value to encode.
///
/// Returns the encoded representation of the field.
pub fn encode_field(
    types: &Types,
    _field_name: &str,
    field_type: &str,
    value: &serde_json::Value,
) -> Result<Token, Eip712Error> {
    let token = {
        // check if field is custom data type
        if types.contains_key(field_type) {
            let tokens = encode_data(field_type, value, types)?;
            let encoded = encode(&tokens);
            encode_eip712_type(Token::Bytes(encoded.to_vec()))
        } else {
            match field_type {
                s if s.contains('[') => {
                    let (stripped_type, _) = s.rsplit_once('[').unwrap();
                    // ensure value is an array
                    let values = value.as_array().ok_or_else(|| {
                        Eip712Error::Message(format!(
                            "Expected array for type `{s}`, but got `{value}`",
                        ))
                    })?;
                    let tokens = values
                        .iter()
                        .map(|value| encode_field(types, _field_name, stripped_type, value))
                        .collect::<Result<Vec<_>, _>>()?;

                    let encoded = encode(&tokens);
                    encode_eip712_type(Token::Bytes(encoded))
                }
                s => {
                    // parse as param type
                    let param = HumanReadableParser::parse_type(s).map_err(|err| {
                        Eip712Error::Message(format!("Failed to parse type {s}: {err}",))
                    })?;

                    match param {
                        ParamType::Address => {
                            Token::Address(serde_json::from_value(value.clone())?)
                        }
                        ParamType::Bytes => {
                            let data: Bytes = serde_json::from_value(value.clone())?;
                            encode_eip712_type(Token::Bytes(data.to_vec()))
                        }
                        ParamType::Int(_) => Token::Uint(serde_json::from_value(value.clone())?),
                        ParamType::Uint(_) => {
                            // uints are commonly stringified due to how ethers-js encodes
                            let val: StringifiedNumeric = serde_json::from_value(value.clone())?;
                            let val = val.try_into().map_err(|err| {
                                Eip712Error::Message(format!("Failed to parse uint {err}"))
                            })?;

                            Token::Uint(val)
                        }
                        ParamType::Bool => {
                            encode_eip712_type(Token::Bool(serde_json::from_value(value.clone())?))
                        }
                        ParamType::String => {
                            let s: String = serde_json::from_value(value.clone())?;
                            encode_eip712_type(Token::String(s))
                        }
                        ParamType::FixedArray(_, _) | ParamType::Array(_) => {
                            unreachable!("is handled in separate arm")
                        }
                        ParamType::FixedBytes(_) => {
                            let data: Bytes = serde_json::from_value(value.clone())?;
                            encode_eip712_type(Token::FixedBytes(data.to_vec()))
                        }
                        ParamType::Tuple(_) => {
                            return Err(Eip712Error::Message(format!("Unexpected tuple type {s}",)))
                        }
                    }
                }
            }
        }
    };

    Ok(token)
}

/// Parse the eth abi parameter type based on the syntax type;
/// this method is copied from <https://github.com/gakonst/ethers-rs/blob/master/ethers-contract/ethers-contract-derive/src/lib.rs#L600>
/// with additional modifications for finding byte arrays
pub fn find_parameter_type(ty: &Type) -> Result<ParamType, TokenStream> {
    match ty {
        Type::Array(ty) => {
            let param = find_parameter_type(ty.elem.as_ref())?;
            if let Expr::Lit(ref expr) = ty.len {
                if let Lit::Int(ref len) = expr.lit {
                    if let Ok(size) = len.base10_parse::<usize>() {
                        if let ParamType::Uint(_) = param {
                            return Ok(ParamType::FixedBytes(size))
                        }

                        return Ok(ParamType::FixedArray(Box::new(param), size))
                    }
                }
            }
            Err(Error::new(ty.span(), "Failed to derive proper ABI from array field")
                .to_compile_error())
        }
        Type::Path(ty) => {
            if let Some(ident) = ty.path.get_ident() {
                let ident = ident.to_string().to_lowercase();
                return match ident.as_str() {
                    "address" => Ok(ParamType::Address),
                    "string" => Ok(ParamType::String),
                    "bool" => Ok(ParamType::Bool),
                    "int256" | "int" | "uint" | "uint256" => Ok(ParamType::Uint(256)),
                    "h160" => Ok(ParamType::FixedBytes(20)),
                    "h256" | "secret" | "hash" => Ok(ParamType::FixedBytes(32)),
                    "h512" | "public" => Ok(ParamType::FixedBytes(64)),
                    "bytes" => Ok(ParamType::Bytes),
                    s => parse_int_param_type(s).ok_or_else(|| {
                        Error::new(
                            ty.span(),
                            format!("Failed to derive proper ABI from field: {s})"),
                        )
                        .to_compile_error()
                    }),
                }
            }
            // check for `Vec`
            if ty.path.segments.len() == 1 && ty.path.segments[0].ident == "Vec" {
                if let PathArguments::AngleBracketed(ref args) = ty.path.segments[0].arguments {
                    if args.args.len() == 1 {
                        if let GenericArgument::Type(ref ty) = args.args.iter().next().unwrap() {
                            let kind = find_parameter_type(ty)?;

                            // Check if byte array is found
                            if let ParamType::Uint(size) = kind {
                                if size == 8 {
                                    return Ok(ParamType::Bytes)
                                }
                            }

                            return Ok(ParamType::Array(Box::new(kind)))
                        }
                    }
                }
            }

            Err(Error::new(ty.span(), "Failed to derive proper ABI from fields").to_compile_error())
        }
        Type::Tuple(ty) => {
            let params = ty.elems.iter().map(find_parameter_type).collect::<Result<Vec<_>, _>>()?;
            Ok(ParamType::Tuple(params))
        }
        _ => {
            Err(Error::new(ty.span(), "Failed to derive proper ABI from fields").to_compile_error())
        }
    }
}

fn parse_int_param_type(s: &str) -> Option<ParamType> {
    let size = s.chars().skip(1).collect::<String>().parse::<usize>().ok()?;
    if s.starts_with('u') {
        Some(ParamType::Uint(size))
    } else if s.starts_with('i') {
        Some(ParamType::Int(size))
    } else {
        None
    }
}

/// Return HashMap of the field name and the field type;
pub fn parse_fields(ast: &DeriveInput) -> Result<Vec<(String, ParamType)>, TokenStream> {
    let mut fields = Vec::new();

    let data = match &ast.data {
        Data::Struct(s) => s,
        _ => {
            return Err(Error::new(
                ast.span(),
                "invalid data type. can only derive Eip712 for a struct",
            )
            .to_compile_error())
        }
    };

    let named_fields = match &data.fields {
        Fields::Named(name) => name,
        _ => {
            return Err(Error::new(ast.span(), "unnamed fields are not supported").to_compile_error())
        }
    };

    for f in named_fields.named.iter() {
        let field_name =
            f.ident.clone().map(|i| i.to_string().to_case(Case::Camel)).ok_or_else(|| {
                Error::new(named_fields.span(), "fields must be named").to_compile_error()
            })?;

        let field_type =
            match f.attrs.iter().find(|a| a.path.segments.iter().any(|s| s.ident == "eip712")) {
                // Found nested Eip712 Struct
                // TODO: Implement custom
                Some(a) => {
                    return Err(Error::new(a.span(), "nested Eip712 struct are not yet supported")
                        .to_compile_error())
                }
                // Not a nested eip712 struct, return the field param type;
                None => find_parameter_type(&f.ty)?,
            };

        fields.push((field_name, field_type));
    }

    Ok(fields)
}

/// Convert hash map of field names and types into a type hash corresponding to enc types;
pub fn make_type_hash(primary_type: String, fields: &[(String, ParamType)]) -> [u8; 32] {
    let parameters =
        fields.iter().map(|(k, v)| format!("{v} {k}")).collect::<Vec<String>>().join(",");

    let sig = format!("{primary_type}({parameters})");

    keccak256(sig)
}

/// Parse token into Eip712 compliant ABI encoding
pub fn encode_eip712_type(token: Token) -> Token {
    match token {
        Token::Bytes(t) => Token::Uint(U256::from(keccak256(t))),
        Token::FixedBytes(t) => Token::Uint(U256::from(&t[..])),
        Token::String(t) => Token::Uint(U256::from(keccak256(t))),
        Token::Bool(t) => {
            // Boolean false and true are encoded as uint256 values 0 and 1 respectively
            Token::Uint(U256::from(t as i32))
        }
        Token::Int(t) => {
            // Integer values are sign-extended to 256-bit and encoded in big endian order.
            Token::Uint(t)
        }
        Token::Array(tokens) => Token::Uint(U256::from(keccak256(abi::encode(
            &tokens.into_iter().map(encode_eip712_type).collect::<Vec<Token>>(),
        )))),
        Token::FixedArray(tokens) => Token::Uint(U256::from(keccak256(abi::encode(
            &tokens.into_iter().map(encode_eip712_type).collect::<Vec<Token>>(),
        )))),
        Token::Tuple(tuple) => {
            let tokens = tuple.into_iter().map(encode_eip712_type).collect::<Vec<Token>>();
            let encoded = encode(&tokens);
            Token::Uint(U256::from(keccak256(encoded)))
        }
        _ => {
            // Return the ABI encoded token;
            token
        }
    }
}

// Adapted tests from <https://github.com/MetaMask/eth-sig-util/blob/main/src/sign-typed-data.test.ts>
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_domain() {
        let json = serde_json::json!({
          "types": {
            "EIP712Domain": [
              {
                "name": "name",
                "type": "string"
              },
              {
                "name": "version",
                "type": "string"
              },
              {
                "name": "chainId",
                "type": "uint256"
              },
              {
                "name": "verifyingContract",
                "type": "address"
              },
              {
                "name": "salt",
                "type": "bytes32"
              }
            ]
          },
          "primaryType": "EIP712Domain",
          "domain": {
            "name": "example.metamask.io",
            "version": "1",
            "chainId": 1,
            "verifyingContract": "0x0000000000000000000000000000000000000000"
          },
          "message": {}
        });

        let typed_data: TypedData = serde_json::from_value(json).unwrap();

        let hash = typed_data.encode_eip712().unwrap();
        assert_eq!(
            "122d1c8ef94b76dad44dcb03fa772361e20855c63311a15d5afe02d1b38f6077",
            hex::encode(&hash[..])
        );
    }

    #[test]
    fn test_minimal_message() {
        let json = serde_json::json!( {"types":{"EIP712Domain":[]},"primaryType":"EIP712Domain","domain":{},"message":{}});

        let typed_data: TypedData = serde_json::from_value(json).unwrap();

        let hash = typed_data.encode_eip712().unwrap();
        assert_eq!(
            "8d4a3f4082945b7879e2b55f181c31a77c8c0a464b70669458abbaaf99de4c38",
            hex::encode(&hash[..])
        );
    }

    #[test]
    fn test_encode_custom_array_type() {
        let json = serde_json::json!({"domain":{},"types":{"EIP712Domain":[],"Person":[{"name":"name","type":"string"},{"name":"wallet","type":"address[]"}],"Mail":[{"name":"from","type":"Person"},{"name":"to","type":"Person[]"},{"name":"contents","type":"string"}]},"primaryType":"Mail","message":{"from":{"name":"Cow","wallet":["0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826","0xDD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826"]},"to":[{"name":"Bob","wallet":["0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB"]}],"contents":"Hello, Bob!"}});

        let typed_data: TypedData = serde_json::from_value(json).unwrap();

        let hash = typed_data.encode_eip712().unwrap();
        assert_eq!(
            "80a3aeb51161cfc47884ddf8eac0d2343d6ae640efe78b6a69be65e3045c1321",
            hex::encode(&hash[..])
        );
    }

    #[test]
    fn test_hash_typed_message_with_data() {
        let json = serde_json::json!( {
          "types": {
            "EIP712Domain": [
              {
                "name": "name",
                "type": "string"
              },
              {
                "name": "version",
                "type": "string"
              },
              {
                "name": "chainId",
                "type": "uint256"
              },
              {
                "name": "verifyingContract",
                "type": "address"
              }
            ],
            "Message": [
              {
                "name": "data",
                "type": "string"
              }
            ]
          },
          "primaryType": "Message",
          "domain": {
            "name": "example.metamask.io",
            "version": "1",
            "chainId": "1",
            "verifyingContract": "0x0000000000000000000000000000000000000000"
          },
          "message": {
            "data": "Hello!"
          }
        });

        let typed_data: TypedData = serde_json::from_value(json).unwrap();

        let hash = typed_data.encode_eip712().unwrap();
        assert_eq!(
            "232cd3ec058eb935a709f093e3536ce26cc9e8e193584b0881992525f6236eef",
            hex::encode(&hash[..])
        );
    }

    #[test]
    fn test_hash_custom_data_type() {
        let json = serde_json::json!(  {"domain":{},"types":{"EIP712Domain":[],"Person":[{"name":"name","type":"string"},{"name":"wallet","type":"address"}],"Mail":[{"name":"from","type":"Person"},{"name":"to","type":"Person"},{"name":"contents","type":"string"}]},"primaryType":"Mail","message":{"from":{"name":"Cow","wallet":"0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826"},"to":{"name":"Bob","wallet":"0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB"},"contents":"Hello, Bob!"}});

        let typed_data: TypedData = serde_json::from_value(json).unwrap();

        let hash = typed_data.encode_eip712().unwrap();
        assert_eq!(
            "25c3d40a39e639a4d0b6e4d2ace5e1281e039c88494d97d8d08f99a6ea75d775",
            hex::encode(&hash[..])
        );
    }

    #[test]
    fn test_hash_recursive_types() {
        let json = serde_json::json!( {
          "domain": {},
          "types": {
            "EIP712Domain": [],
            "Person": [
              {
                "name": "name",
                "type": "string"
              },
              {
                "name": "wallet",
                "type": "address"
              }
            ],
            "Mail": [
              {
                "name": "from",
                "type": "Person"
              },
              {
                "name": "to",
                "type": "Person"
              },
              {
                "name": "contents",
                "type": "string"
              },
              {
                "name": "replyTo",
                "type": "Mail"
              }
            ]
          },
          "primaryType": "Mail",
          "message": {
            "from": {
              "name": "Cow",
              "wallet": "0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826"
            },
            "to": {
              "name": "Bob",
              "wallet": "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB"
            },
            "contents": "Hello, Bob!",
            "replyTo": {
              "to": {
                "name": "Cow",
                "wallet": "0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826"
              },
              "from": {
                "name": "Bob",
                "wallet": "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB"
              },
              "contents": "Hello!"
            }
          }
        });

        let typed_data: TypedData = serde_json::from_value(json).unwrap();

        let hash = typed_data.encode_eip712().unwrap();
        assert_eq!(
            "0808c17abba0aef844b0470b77df9c994bc0fa3e244dc718afd66a3901c4bd7b",
            hex::encode(&hash[..])
        );
    }

    #[test]
    fn test_hash_nested_struct_array() {
        let json = serde_json::json!({
          "types": {
            "EIP712Domain": [
              {
                "name": "name",
                "type": "string"
              },
              {
                "name": "version",
                "type": "string"
              },
              {
                "name": "chainId",
                "type": "uint256"
              },
              {
                "name": "verifyingContract",
                "type": "address"
              }
            ],
            "OrderComponents": [
              {
                "name": "offerer",
                "type": "address"
              },
              {
                "name": "zone",
                "type": "address"
              },
              {
                "name": "offer",
                "type": "OfferItem[]"
              },
              {
                "name": "startTime",
                "type": "uint256"
              },
              {
                "name": "endTime",
                "type": "uint256"
              },
              {
                "name": "zoneHash",
                "type": "bytes32"
              },
              {
                "name": "salt",
                "type": "uint256"
              },
              {
                "name": "conduitKey",
                "type": "bytes32"
              },
              {
                "name": "counter",
                "type": "uint256"
              }
            ],
            "OfferItem": [
              {
                "name": "token",
                "type": "address"
              }
            ],
            "ConsiderationItem": [
              {
                "name": "token",
                "type": "address"
              },
              {
                "name": "identifierOrCriteria",
                "type": "uint256"
              },
              {
                "name": "startAmount",
                "type": "uint256"
              },
              {
                "name": "endAmount",
                "type": "uint256"
              },
              {
                "name": "recipient",
                "type": "address"
              }
            ]
          },
          "primaryType": "OrderComponents",
          "domain": {
            "name": "Seaport",
            "version": "1.1",
            "chainId": "1",
            "verifyingContract": "0x00000000006c3852cbEf3e08E8dF289169EdE581"
          },
          "message": {
            "offerer": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
            "offer": [
              {
                "token": "0xA604060890923Ff400e8c6f5290461A83AEDACec"
              }
            ],
            "startTime": "1658645591",
            "endTime": "1659250386",
            "zone": "0x004C00500000aD104D7DBd00e3ae0A5C00560C00",
            "zoneHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
            "salt": "16178208897136618",
            "conduitKey": "0x0000007b02230091a7ed01230072f7006a004d60a8d4e71d599b8104250f0000",
            "totalOriginalConsiderationItems": "2",
            "counter": "0"
          }
        }
                );

        let typed_data: TypedData = serde_json::from_value(json).unwrap();

        let hash = typed_data.encode_eip712().unwrap();
        assert_eq!(
            "0b8aa9f3712df0034bc29fe5b24dd88cfdba02c7f499856ab24632e2969709a8",
            hex::encode(&hash[..])
        );
    }
}
