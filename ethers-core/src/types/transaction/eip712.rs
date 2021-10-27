use convert_case::{Case, Casing};
use core::convert::TryFrom;
use proc_macro2::TokenStream;
use syn::{
    parse::Error, spanned::Spanned as _, AttrStyle, Data, DeriveInput, Expr, Fields,
    GenericArgument, Lit, NestedMeta, PathArguments, Type,
};

use crate::{
    abi,
    abi::{ParamType, Token},
    types::{Address, H160, U256},
    utils::keccak256,
};

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
    Inner(String),
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
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct EIP712Domain {
    ///  The user readable name of signing domain, i.e. the name of the DApp or the protocol.
    pub name: String,

    /// The current major version of the signing domain. Signatures from different versions are not
    /// compatible.
    pub version: String,

    /// The EIP-155 chain id. The user-agent should refuse signing if it does not match the
    /// currently active chain.
    pub chain_id: U256,

    /// The address of the contract that will verify the signature.
    pub verifying_contract: Address,

    /// A disambiguating salt for the protocol. This can be used as a domain separator of last
    /// resort.
    pub salt: Option<[u8; 32]>,
}

impl EIP712Domain {
    // Compute the domain separator;
    // See: https://github.com/gakonst/ethers-rs/blob/master/examples/permit_hash.rs#L41
    pub fn separator(&self) -> [u8; 32] {
        let domain_type_hash = if self.salt.is_some() {
            EIP712_DOMAIN_TYPE_HASH_WITH_SALT
        } else {
            EIP712_DOMAIN_TYPE_HASH
        };

        let mut tokens = vec![
            Token::Uint(U256::from(domain_type_hash)),
            Token::Uint(U256::from(keccak256(&self.name))),
            Token::Uint(U256::from(keccak256(&self.version))),
            Token::Uint(self.chain_id),
            Token::Address(self.verifying_contract),
        ];

        // Add the salt to the struct to be hashed if it exists;
        if let Some(salt) = &self.salt {
            tokens.push(Token::Uint(U256::from(salt)));
        }

        keccak256(abi::encode(&tokens))
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
        let domain = inner.domain().map_err(|e| Eip712Error::Inner(e.to_string()))?;

        Ok(Self { domain, inner })
    }

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
        let type_hash = T::type_hash().map_err(|e| Self::Error::Inner(e.to_string()))?;
        Ok(type_hash)
    }

    fn struct_hash(&self) -> Result<[u8; 32], Self::Error> {
        let struct_hash =
            self.inner.clone().struct_hash().map_err(|e| Self::Error::Inner(e.to_string()))?;
        Ok(struct_hash)
    }
}

// Parse the AST of the struct to determine the domain attributes
impl TryFrom<&syn::DeriveInput> for EIP712Domain {
    type Error = TokenStream;
    fn try_from(input: &syn::DeriveInput) -> Result<EIP712Domain, Self::Error> {
        let mut domain = EIP712Domain::default();
        let mut found_eip712_attribute = false;

        for attribute in input.attrs.iter() {
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
                                                    if domain.name != String::default() {
                                                        return Err(Error::new(
                                                            meta.path.span(),
                                                            "domain name already specified",
                                                        )
                                                        .to_compile_error())
                                                    }

                                                    domain.name = lit_str.value();
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
                                                    if domain.version != String::default() {
                                                        return Err(Error::new(
                                                            meta.path.span(),
                                                            "domain version already specified",
                                                        )
                                                        .to_compile_error())
                                                    }

                                                    domain.version = lit_str.value();
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
                                                    if domain.chain_id != U256::default() {
                                                        return Err(Error::new(
                                                            meta.path.span(),
                                                            "domain chain_id already specified",
                                                        )
                                                        .to_compile_error())
                                                    }

                                                    domain.chain_id = lit_int
                                                        .base10_digits()
                                                        .parse()
                                                        .map_err(|_| {
                                                            Error::new(
                                                                meta.path.span(),
                                                                "failed to parse chain id",
                                                            )
                                                            .to_compile_error()
                                                        })?;
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
                                                    if domain.verifying_contract != H160::default()
                                                    {
                                                        return Err(Error::new(
                                                            meta.path.span(),
                                                            "domain verifying_contract already specified",
                                                        )
                                                        .to_compile_error());
                                                    }

                                                    domain.verifying_contract = lit_str.value().parse().map_err(|_| {
                                                            Error::new(
                                                                meta.path.span(),
                                                                "failed to parse verifying contract into Address",
                                                            )
                                                            .to_compile_error()
                                                        })?;
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
                                                    if domain.salt != Option::None {
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

                        if domain.name == String::default() {
                            return Err(Error::new(
                                meta.path.span(),
                                "missing required domain attribute: 'name'".to_string(),
                            )
                            .to_compile_error())
                        }
                        if domain.version == String::default() {
                            return Err(Error::new(
                                meta.path.span(),
                                "missing required domain attribute: 'version'".to_string(),
                            )
                            .to_compile_error())
                        }
                        if domain.chain_id == U256::default() {
                            return Err(Error::new(
                                meta.path.span(),
                                "missing required domain attribute: 'chain_id'".to_string(),
                            )
                            .to_compile_error())
                        }
                        if domain.verifying_contract == H160::default() {
                            return Err(Error::new(
                                meta.path.span(),
                                "missing required domain attribute: 'verifying_contract'"
                                    .to_string(),
                            )
                            .to_compile_error())
                        }
                    }
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

/// Parse the eth abi parameter type based on the syntax type;
/// this method is copied from https://github.com/gakonst/ethers-rs/blob/master/ethers-contract/ethers-contract-derive/src/lib.rs#L600
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
                            format!("Failed to derive proper ABI from field: {})", s),
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
        fields.iter().map(|(k, v)| format!("{} {}", v, k)).collect::<Vec<String>>().join(",");

    let sig = format!("{}({})", primary_type, parameters);

    keccak256(sig)
}

/// Parse token into Eip712 compliant ABI encoding
/// NOTE: Token::Tuple() is currently not supported for solidity structs;
/// this is needed for nested Eip712 types, but is not implemented.
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
        _ => {
            // Return the ABI encoded token;
            token
        }
    }
}
