use crate::utils;
use ethers_core::{
    abi::{Address, ParamType},
    macros::ethers_core_crate,
    types::{transaction::eip712::EIP712Domain, H256},
    utils::keccak256,
};
use inflector::Inflector;
use proc_macro2::{Literal, TokenStream};
use quote::quote;
use syn::{spanned::Spanned, Data, DeriveInput, Error, Fields, LitInt, LitStr, Result, Token};

pub(crate) fn impl_derive_eip712(input: &DeriveInput) -> Result<TokenStream> {
    // Primary type should match the type in the ethereum verifying contract;
    let primary_type = &input.ident;

    // Instantiate domain from parsed attributes
    let domain = parse_attributes(input)?;

    let domain_separator = into_tokens(domain.separator());

    let domain_str = serde_json::to_string(&domain).unwrap();

    // Must parse the AST at compile time.
    let parsed_fields = parse_fields(input)?;

    // Compute the type hash for the derived struct using the parsed fields from above.
    let type_hash = into_tokens(make_type_hash(primary_type.to_string(), &parsed_fields));

    // Use reference to ethers_core instead of directly using the crate itself.
    let ethers_core = ethers_core_crate();

    let tokens = quote! {
        impl #ethers_core::types::transaction::eip712::Eip712 for #primary_type {
            type Error = #ethers_core::types::transaction::eip712::Eip712Error;

            #[inline]
            fn type_hash() -> ::core::result::Result<[u8; 32], Self::Error> {
                Ok([#(#type_hash),*])
            }

            #[inline]
            fn domain_separator(&self) -> ::core::result::Result<[u8; 32], Self::Error> {
                Ok([#(#domain_separator),*])
            }

            fn domain(&self) -> ::core::result::Result<#ethers_core::types::transaction::eip712::EIP712Domain, Self::Error> {
                #ethers_core::utils::__serde_json::from_str(#domain_str).map_err(::core::convert::Into::into)
            }

            fn struct_hash(&self) -> ::core::result::Result<[u8; 32], Self::Error> {
                let mut items = vec![#ethers_core::abi::Token::Uint(
                    #ethers_core::types::U256::from(&Self::type_hash()?[..]),
                )];

                if let #ethers_core::abi::Token::Tuple(tokens) =
                    #ethers_core::abi::Tokenizable::into_token(::core::clone::Clone::clone(self))
                {
                    items.reserve(tokens.len());
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

    Ok(tokens)
}

fn parse_attributes(input: &DeriveInput) -> Result<EIP712Domain> {
    let mut domain = EIP712Domain::default();
    utils::parse_attributes!(input.attrs.iter(), "eip712", meta,
        "name", domain.name => {
            meta.input.parse::<Token![=]>()?;
            let litstr: LitStr = meta.input.parse()?;
            domain.name = Some(litstr.value());
        }
        "version", domain.version => {
            meta.input.parse::<Token![=]>()?;
            let litstr: LitStr = meta.input.parse()?;
            domain.version = Some(litstr.value());
        }
        "chain_id", domain.chain_id => {
            meta.input.parse::<Token![=]>()?;
            let litint: LitInt = meta.input.parse()?;
            let n: u64 = litint.base10_parse()?;
            domain.chain_id = Some(n.into());
        }
        "verifying_contract", domain.verifying_contract => {
            meta.input.parse::<Token![=]>()?;
            let litstr: LitStr = meta.input.parse()?;
            let addr: Address =
                litstr.value().parse().map_err(|e| Error::new(litstr.span(), e))?;
            domain.verifying_contract = Some(addr);
        }
        // hash string
        "salt", domain.salt => {
            meta.input.parse::<Token![=]>()?;
            let litstr: LitStr = meta.input.parse()?;
            let hash = keccak256(litstr.value());
            domain.salt = Some(hash);
        }
        // parse string as H256
        "raw_salt", domain.salt => {
            meta.input.parse::<Token![=]>()?;
            let litstr: LitStr = meta.input.parse()?;
            let bytes = litstr.value().parse::<H256>().map_err(|e| Error::new(litstr.span(), e))?;
            domain.salt = Some(bytes.0);
        }
    );
    Ok(domain)
}

/// Returns a Vec of `(name, param_type)`
fn parse_fields(input: &DeriveInput) -> Result<Vec<(String, ParamType)>> {
    let data = match &input.data {
        Data::Struct(s) => s,
        Data::Enum(e) => {
            return Err(Error::new(e.enum_token.span, "Eip712 is not derivable for enums"))
        }
        Data::Union(u) => {
            return Err(Error::new(u.union_token.span, "Eip712 is not derivable for unions"))
        }
    };

    let named_fields = match &data.fields {
        Fields::Named(fields) => fields,
        _ => return Err(Error::new(input.span(), "unnamed fields are not supported")),
    };

    let mut fields = Vec::with_capacity(named_fields.named.len());
    for f in named_fields.named.iter() {
        // strip the raw identifier prefix
        let name = f.ident.as_ref().unwrap().to_string();
        let s = name.strip_prefix("r#").unwrap_or(&name);
        let name = s.to_camel_case();

        let ty =
            match f.attrs.iter().find(|a| a.path().segments.iter().any(|s| s.ident == "eip712")) {
                // Found nested Eip712 Struct
                // TODO: Implement custom
                Some(a) => {
                    return Err(Error::new(a.span(), "nested Eip712 struct are not yet supported"))
                }
                // Not a nested eip712 struct, return the field param type;
                None => crate::utils::find_parameter_type(&f.ty)?,
            };

        fields.push((name, ty));
    }

    Ok(fields)
}

/// Convert hash map of field names and types into a type hash corresponding to enc types;
fn make_type_hash(primary_type: String, fields: &[(String, ParamType)]) -> [u8; 32] {
    let mut sig = String::with_capacity(256);

    sig.push_str(&primary_type);

    sig.push('(');
    for (i, (name, ty)) in fields.iter().enumerate() {
        sig.push_str(&ty.to_string());
        sig.push(' ');
        sig.push_str(name);
        if i < fields.len() - 1 {
            sig.push(',');
        }
    }
    sig.push(')');

    keccak256(sig)
}

fn into_tokens(bytes: [u8; 32]) -> impl Iterator<Item = Literal> {
    bytes.into_iter().map(Literal::u8_suffixed)
}
