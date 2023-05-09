use ethers_core::{abi::ParamType, macros::ethers_core_crate, types::Selector};
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{
    parse::Error, spanned::Spanned, Data, DeriveInput, Expr, Fields, GenericArgument, Lit,
    PathArguments, Type,
};

/// Parses the specified attributes from a `syn::Attribute` iterator.
macro_rules! parse_attributes {
    ($attrs:expr, $attr_ident:literal, $meta:ident, $($field:pat, $opt:expr => $block:block)*) => {
        const ERROR: &str = concat!("unrecognized ", $attr_ident, " attribute");
        const ALREADY_SPECIFIED: &str = concat!($attr_ident, " attribute already specified");

        for attr in $attrs {
            if !attr.path().is_ident($attr_ident) {
                continue;
            }

            attr.parse_nested_meta(|$meta| {
                let ident = $meta.path.get_ident().ok_or_else(|| $meta.error(ERROR))?.to_string();
                match ident.as_str() {
                    $(
                        $field if $opt.is_none() => $block,
                        $field => return Err($meta.error(ALREADY_SPECIFIED)),
                    )*

                    _ => return Err($meta.error(ERROR)),
                }

                Ok(())
            })?;
        }
    };
}
pub(crate) use parse_attributes;

pub fn ident(name: &str) -> Ident {
    Ident::new(name, Span::call_site())
}

pub fn signature(hash: &[u8]) -> TokenStream {
    let ethers_core = ethers_core_crate();
    let bytes = hash.iter().copied().map(Literal::u8_unsuffixed);
    quote! {#ethers_core::types::H256([#( #bytes ),*])}
}

pub fn selector(selector: Selector) -> TokenStream {
    let bytes = selector.iter().copied().map(Literal::u8_unsuffixed);
    quote! {[#( #bytes ),*]}
}

/// Parses an int / hash type from its string representation
pub fn parse_param_type(s: &str) -> Option<ParamType> {
    match s.chars().next() {
        Some('H' | 'h') => {
            let size = s[1..].parse::<usize>().ok()? / 8;
            Some(ParamType::FixedBytes(size))
        }

        Some(c @ 'U' | c @ 'I' | c @ 'u' | c @ 'i') => {
            let size = s[1..].parse::<usize>().ok()?;
            if matches!(c, 'U' | 'u') {
                Some(ParamType::Uint(size))
            } else {
                Some(ParamType::Int(size))
            }
        }
        _ => None,
    }
}

// Converts param types for indexed parameters to bytes32 where appropriate
// This applies to strings, arrays, structs and bytes to follow the encoding of
// these indexed param types according to
// <https://solidity.readthedocs.io/en/develop/abi-spec.html#encoding-of-indexed-event-parameters>
pub fn topic_param_type_quote(kind: &ParamType) -> TokenStream {
    let ethers_core = ethers_core_crate();
    match kind {
        ParamType::String |
        ParamType::Bytes |
        ParamType::Array(_) |
        ParamType::FixedArray(_, _) |
        ParamType::Tuple(_) => quote! {#ethers_core::abi::ParamType::FixedBytes(32)},
        ty => param_type_quote(ty),
    }
}

/// Returns the rust type for the given parameter
pub fn param_type_quote(kind: &ParamType) -> TokenStream {
    let ethers_core = ethers_core_crate();
    match kind {
        ParamType::Address => {
            quote! {#ethers_core::abi::ParamType::Address}
        }
        ParamType::Bytes => {
            quote! {#ethers_core::abi::ParamType::Bytes}
        }
        ParamType::Int(size) => {
            let size = Literal::usize_suffixed(*size);
            quote! {#ethers_core::abi::ParamType::Int(#size)}
        }
        ParamType::Uint(size) => {
            let size = Literal::usize_suffixed(*size);
            quote! {#ethers_core::abi::ParamType::Uint(#size)}
        }
        ParamType::Bool => {
            quote! {#ethers_core::abi::ParamType::Bool}
        }
        ParamType::String => {
            quote! {#ethers_core::abi::ParamType::String}
        }
        ParamType::Array(ty) => {
            let ty = param_type_quote(ty);
            quote! {#ethers_core::abi::ParamType::Array(Box::new(#ty))}
        }
        ParamType::FixedBytes(size) => {
            let size = Literal::usize_suffixed(*size);
            quote! {#ethers_core::abi::ParamType::FixedBytes(#size)}
        }
        ParamType::FixedArray(ty, size) => {
            let ty = param_type_quote(ty);
            let size = Literal::usize_suffixed(*size);
            quote! {#ethers_core::abi::ParamType::FixedArray(Box::new(#ty), #size)}
        }
        ParamType::Tuple(tuple) => {
            let elements = tuple.iter().map(param_type_quote);
            quote!(#ethers_core::abi::ParamType::Tuple(::std::vec![#( #elements ),*]))
        }
    }
}

/// Tries to find the corresponding `ParamType` used for tokenization for the
/// given type
pub fn find_parameter_type(ty: &Type) -> Result<ParamType, Error> {
    const ERROR: &str = "Failed to derive proper ABI from array field";

    match ty {
        Type::Array(arr) => {
            let ty = find_parameter_type(&arr.elem)?;
            if let Expr::Lit(ref expr) = arr.len {
                if let Lit::Int(ref len) = expr.lit {
                    if let Ok(len) = len.base10_parse::<usize>() {
                        return match (ty, len) {
                            (ParamType::Uint(8), 32) => Ok(ParamType::FixedBytes(32)),
                            (ty, len) => Ok(ParamType::FixedArray(Box::new(ty), len)),
                        }
                    }
                }
            }
            Err(Error::new(arr.span(), ERROR))
        }

        Type::Path(ty) => {
            // check for `Vec`
            if let Some(segment) = ty.path.segments.iter().find(|s| s.ident == "Vec") {
                if let PathArguments::AngleBracketed(ref args) = segment.arguments {
                    // Vec<T, A?>
                    debug_assert!(matches!(args.args.len(), 1 | 2));
                    let ty = args.args.iter().next().unwrap();
                    if let GenericArgument::Type(ref ty) = ty {
                        return find_parameter_type(ty).map(|kind| ParamType::Array(Box::new(kind)))
                    }
                }
            }

            // match on the last segment of the path
            ty.path
                .get_ident()
                .or_else(|| ty.path.segments.last().map(|s| &s.ident))
                .and_then(|ident| {
                    match ident.to_string().as_str() {
                        // eth types
                        "Address" => Some(ParamType::Address),
                        "Bytes" => Some(ParamType::Bytes),
                        "Uint8" => Some(ParamType::Uint(8)),

                        // core types
                        "String" => Some(ParamType::String),
                        "bool" => Some(ParamType::Bool),
                        // usize / isize, shouldn't happen but use max width
                        "usize" => Some(ParamType::Uint(64)),
                        "isize" => Some(ParamType::Int(64)),

                        s => parse_param_type(s),
                    }
                })
                .ok_or_else(|| Error::new(ty.span(), ERROR))
        }

        Type::Tuple(ty) => ty
            .elems
            .iter()
            .map(find_parameter_type)
            .collect::<Result<Vec<_>, _>>()
            .map(ParamType::Tuple),

        _ => Err(Error::new(ty.span(), ERROR)),
    }
}

/// Attempts to determine the ABI Paramtypes from the type's AST
pub fn derive_abi_inputs_from_fields(
    input: &DeriveInput,
    trait_name: &str,
) -> Result<Vec<(String, ParamType)>, Error> {
    let fields: Vec<_> = match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => fields.named.iter().collect(),
            Fields::Unnamed(ref fields) => fields.unnamed.iter().collect(),
            Fields::Unit => {
                return Err(Error::new(
                    input.span(),
                    format!("{trait_name} cannot be derived for empty structs and unit"),
                ))
            }
        },
        Data::Enum(_) => {
            return Err(Error::new(
                input.span(),
                format!("{trait_name} cannot be derived for enums"),
            ))
        }
        Data::Union(_) => {
            return Err(Error::new(
                input.span(),
                format!("{trait_name} cannot be derived for unions"),
            ))
        }
    };

    fields
        .iter()
        .map(|f| {
            let name =
                f.ident.as_ref().map(|name| name.to_string()).unwrap_or_else(|| "".to_string());
            find_parameter_type(&f.ty).map(|ty| (name, ty))
        })
        .collect()
}

/// Use `AbiType::param_type` fo each field to construct the input types own param type
pub fn derive_param_type_with_abi_type(
    input: &DeriveInput,
    trait_name: &str,
) -> Result<TokenStream, Error> {
    let ethers_core = ethers_core_crate();
    let params = abi_parameters_array(input, trait_name)?;
    Ok(quote! {
        #ethers_core::abi::ParamType::Tuple(::std::vec!#params)
    })
}

/// Use `AbiType::param_type` fo each field to construct the whole signature `<name>(<params,>*)` as
/// `String`.
pub fn abi_signature_with_abi_type(
    input: &DeriveInput,
    function_name: &str,
    trait_name: &str,
) -> Result<TokenStream, Error> {
    let params = abi_parameters_array(input, trait_name)?;
    Ok(quote! {
        {
            let params: String = #params
                .iter()
                .map(|p| p.to_string())
                .collect::<::std::vec::Vec<_>>()
                .join(",");
            let function_name = #function_name;
            format!("{}({})", function_name, params)
        }
    })
}

/// Use `AbiType::param_type` fo each field to construct the signature's parameters as runtime array
/// `[param1, param2,...]`
pub fn abi_parameters_array(input: &DeriveInput, trait_name: &str) -> Result<TokenStream, Error> {
    let ethers_core = ethers_core_crate();

    let fields = match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => &fields.named,
            Fields::Unnamed(ref fields) => &fields.unnamed,
            Fields::Unit => {
                return Err(Error::new(
                    input.span(),
                    format!("{trait_name} cannot be derived for empty structs and unit"),
                ))
            }
        },
        Data::Enum(_) => {
            return Err(Error::new(
                input.span(),
                format!("{trait_name} cannot be derived for enums"),
            ))
        }
        Data::Union(_) => {
            return Err(Error::new(
                input.span(),
                format!("{trait_name} cannot be derived for unions"),
            ))
        }
    };

    let iter = fields.iter().map(|f| {
        let ty = &f.ty;
        quote_spanned!(f.span() => <#ty as #ethers_core::abi::AbiType>::param_type())
    });

    Ok(quote! {
        [#( #iter ),*]
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    macro_rules! type_test_cases {
        ($($t:ty => $e:expr),+ $(,)?) => {{
            &[
                $(
                    (parse_quote!($t), $e),
                )+
            ]
        }};
    }

    fn arr(ty: ParamType) -> ParamType {
        ParamType::Array(Box::new(ty))
    }

    fn farr(ty: ParamType, len: usize) -> ParamType {
        ParamType::FixedArray(Box::new(ty), len)
    }

    #[test]
    fn can_find_params() {
        use ParamType as PT;
        let test_cases: &[(Type, ParamType)] = type_test_cases! {
            u8 => PT::Uint(8),
            u16 => PT::Uint(16),
            u32 => PT::Uint(32),
            u64 => PT::Uint(64),
            usize => PT::Uint(64),
            u128 => PT::Uint(128),
            ::ethers::types::U256 => PT::Uint(256),
            ethers::types::U256 => PT::Uint(256),
            ::ethers_core::types::U256 => PT::Uint(256),
            ethers_core::types::U256 => PT::Uint(256),
            U256 => PT::Uint(256),

            i8 => PT::Int(8),
            i16 => PT::Int(16),
            i32 => PT::Int(32),
            i64 => PT::Int(64),
            isize => PT::Int(64),
            i128 => PT::Int(128),
            ::ethers::types::I256 => PT::Int(256),
            ethers::types::I256 => PT::Int(256),
            ::ethers_core::types::I256 => PT::Int(256),
            ethers_core::types::I256 => PT::Int(256),
            I256 => PT::Int(256),


            ::ethers::types::H160 => PT::FixedBytes(20),
            H160 => PT::FixedBytes(20),
            ::ethers::types::H256 => PT::FixedBytes(32),
            H256 => PT::FixedBytes(32),
            ::ethers::types::H512 => PT::FixedBytes(64),
            H512 => PT::FixedBytes(64),

            ::std::vec::Vec<::ethers_core::types::U256, ::std::alloc::Global> => arr(PT::Uint(256)),
            ::std::vec::Vec<::ethers_core::types::U256, Global> => arr(PT::Uint(256)),
            ::std::vec::Vec<::ethers_core::types::U256> => arr(PT::Uint(256)),
            ::std::vec::Vec<ethers::types::U256> => arr(PT::Uint(256)),
            ::std::vec::Vec<U256> => arr(PT::Uint(256)),
            std::vec::Vec<U256> => arr(PT::Uint(256)),
            vec::Vec<U256> => arr(PT::Uint(256)),
            Vec<U256> => arr(PT::Uint(256)),

            [u64; 8] => farr(PT::Uint(64), 8),
            [u64; 16] => farr(PT::Uint(64), 16),
            [::ethers_core::types::U256; 2] => farr(PT::Uint(256), 2),
            [String; 4] => farr(PT::String, 4),
            [Address; 2] => farr(PT::Address, 2),

            (String, String, Address) => PT::Tuple(vec![PT::String, PT::String, PT::Address]),
            (::ethers_core::types::U256, u8, ::ethers_core::types::Address)
                => PT::Tuple(vec![PT::Uint(256), PT::Uint(8), PT::Address]),
            (::ethers::types::Bytes, ::ethers::types::H256, (::ethers::types::Address, ::std::string::String))
                => PT::Tuple(vec![
                    PT::Bytes,
                    PT::FixedBytes(32),
                    PT::Tuple(vec![PT::Address, PT::String])
                ]),
        };

        for (ty, expected) in test_cases {
            match find_parameter_type(ty) {
                Ok(ty) => assert_eq!(ty, *expected),
                Err(e) => panic!("{e}: {ty:#?}\n{expected}"),
            }
        }
    }
}
