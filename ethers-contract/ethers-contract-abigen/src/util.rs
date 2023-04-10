use crate::InternalStructs;
use ethers_core::abi::{
    struct_def::{FieldType, StructFieldType},
    ParamType, SolStruct,
};
use eyre::Result;
use inflector::Inflector;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use std::path::{Path, PathBuf};

/// Creates a new Ident with the given string at [`Span::call_site`].
///
/// # Panics
///
/// If the input string is neither a keyword nor a legal variable name.
pub(crate) fn ident(name: &str) -> Ident {
    Ident::new(name, Span::call_site())
}

/// Expands an identifier string into a token and appending `_` if the identifier is for a reserved
/// keyword.
///
/// Parsing keywords like `self` can fail, in this case we add an underscore.
pub(crate) fn safe_ident(name: &str) -> Ident {
    syn::parse_str::<Ident>(name).unwrap_or_else(|_| ident(&format!("{name}_")))
}

///  Converts a `&str` to `snake_case` `String` while respecting identifier rules
pub(crate) fn safe_snake_case(ident: &str) -> String {
    safe_identifier_name(ident.to_snake_case())
}

///  Converts a `&str` to `PascalCase` `String` while respecting identifier rules
pub(crate) fn safe_pascal_case(ident: &str) -> String {
    safe_identifier_name(ident.to_pascal_case())
}

/// respects identifier rules, such as, an identifier must not start with a numeric char
pub(crate) fn safe_identifier_name(name: String) -> String {
    if name.starts_with(char::is_numeric) {
        format!("_{name}")
    } else {
        name
    }
}

/// converts invalid rust module names to valid ones
pub(crate) fn safe_module_name(name: &str) -> String {
    // handle reserve words used in contracts (eg Enum is a gnosis contract)
    safe_ident(&safe_snake_case(name)).to_string()
}

/// Expands an identifier as snakecase and preserve any leading or trailing underscores
pub(crate) fn safe_snake_case_ident(name: &str) -> Ident {
    let i = name.to_snake_case();
    ident(&preserve_underscore_delim(&i, name))
}

/// Expands an identifier as pascal case and preserve any leading or trailing underscores
pub(crate) fn safe_pascal_case_ident(name: &str) -> Ident {
    let i = name.to_pascal_case();
    ident(&preserve_underscore_delim(&i, name))
}

/// Reapplies leading and trailing underscore chars to the ident
///
/// # Example
///
/// ```ignore
/// # use ethers_contract_abigen::util::preserve_underscore_delim;
/// assert_eq!(
///   preserve_underscore_delim("pascalCase", "__pascalcase__"),
///   "__pascalCase__"
/// );
/// ```
pub(crate) fn preserve_underscore_delim(ident: &str, original: &str) -> String {
    let is_underscore = |c: &char| *c == '_';
    let pre = original.chars().take_while(is_underscore);
    let post = original.chars().rev().take_while(is_underscore);
    pre.chain(ident.chars()).chain(post).collect()
}

/// Expands a positional identifier string that may be empty.
///
/// Note that this expands the parameter name with `safe_ident`, meaning that
/// identifiers that are reserved keywords get `_` appended to them.
pub(crate) fn expand_input_name(index: usize, name: &str) -> TokenStream {
    let name_str = match name {
        "" => format!("p{index}"),
        n => n.to_snake_case(),
    };
    let name = safe_ident(&name_str);

    quote! { #name }
}

/// Perform a blocking HTTP GET request and return the contents of the response as a String.
#[cfg(all(feature = "online", not(target_arch = "wasm32")))]
pub(crate) fn http_get(url: impl reqwest::IntoUrl) -> Result<String> {
    Ok(reqwest::blocking::get(url)?.text()?)
}

/// Replaces any occurrences of env vars in the `raw` str with their value
pub(crate) fn resolve_path(raw: &str) -> Result<PathBuf> {
    let mut unprocessed = raw;
    let mut resolved = String::new();

    while let Some(dollar_sign) = unprocessed.find('$') {
        let (head, tail) = unprocessed.split_at(dollar_sign);
        resolved.push_str(head);

        match parse_identifier(&tail[1..]) {
            Some((variable, rest)) => {
                let value = std::env::var(variable)?;
                resolved.push_str(&value);
                unprocessed = rest;
            }
            None => {
                eyre::bail!("Unable to parse a variable from \"{tail}\"")
            }
        }
    }
    resolved.push_str(unprocessed);

    Ok(PathBuf::from(resolved))
}

fn parse_identifier(text: &str) -> Option<(&str, &str)> {
    let mut calls = 0;

    let (head, tail) = take_while(text, |c| {
        calls += 1;
        match c {
            '_' => true,
            letter if letter.is_ascii_alphabetic() => true,
            digit if digit.is_ascii_digit() && calls > 1 => true,
            _ => false,
        }
    });

    if head.is_empty() {
        None
    } else {
        Some((head, tail))
    }
}

fn take_while(s: &str, mut predicate: impl FnMut(char) -> bool) -> (&str, &str) {
    let mut index = 0;
    for c in s.chars() {
        if predicate(c) {
            index += c.len_utf8();
        } else {
            break
        }
    }
    s.split_at(index)
}

/// Returns a list of absolute paths to all the json files under the root
pub(crate) fn json_files(root: impl AsRef<Path>) -> Vec<PathBuf> {
    walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().map(|ext| ext == "json").unwrap_or_default())
        .map(|e| e.path().into())
        .collect()
}

/// Returns whether all the given parameters can derive the builtin traits.
///
/// The following traits are only implemented on tuples of arity 12 or less:
///
/// - [PartialEq](https://doc.rust-lang.org/stable/std/cmp/trait.PartialEq.html)
/// - [Eq](https://doc.rust-lang.org/stable/std/cmp/trait.Eq.html)
/// - [PartialOrd](https://doc.rust-lang.org/stable/std/cmp/trait.PartialOrd.html)
/// - [Ord](https://doc.rust-lang.org/stable/std/cmp/trait.Ord.html)
/// - [Debug](https://doc.rust-lang.org/stable/std/fmt/trait.Debug.html)
/// - [Default](https://doc.rust-lang.org/stable/std/default/trait.Default.html)
/// - [Hash](https://doc.rust-lang.org/stable/std/hash/trait.Hash.html)
///
/// while the `Default` trait is only implemented on arrays of length 32 or less.
///
/// Tuple reference: <https://doc.rust-lang.org/stable/std/primitive.tuple.html#trait-implementations-1>
///
/// Array reference: <https://doc.rust-lang.org/stable/std/primitive.array.html>
///
/// `derive_default` should be set to false when calling this for enums.
pub(crate) fn derive_builtin_traits<'a>(
    params: impl IntoIterator<Item = &'a ParamType>,
    stream: &mut TokenStream,
    mut derive_default: bool,
    mut derive_others: bool,
) {
    for param in params {
        derive_default &= can_derive_default(param);
        derive_others &= can_derive_builtin_traits(param);
    }
    extend_derives(stream, derive_default, derive_others);
}

/// This has to be a seperate function since a sol struct is converted into a tuple, but for
/// deriving purposes it shouldn't count as one, so we recurse back the struct fields.
pub(crate) fn derive_builtin_traits_struct(
    structs: &InternalStructs,
    sol_struct: &SolStruct,
    params: &[ParamType],
    stream: &mut TokenStream,
) {
    if sol_struct.fields().iter().any(|field| field.ty.is_struct()) {
        let mut def = true;
        let mut others = true;
        _derive_builtin_traits_struct(structs, sol_struct, params, &mut def, &mut others);
        extend_derives(stream, def, others);
    } else {
        derive_builtin_traits(params, stream, true, true);
    }
}

fn _derive_builtin_traits_struct(
    structs: &InternalStructs,
    sol_struct: &SolStruct,
    params: &[ParamType],
    def: &mut bool,
    others: &mut bool,
) {
    let fields = sol_struct.fields();
    debug_assert_eq!(fields.len(), params.len());

    for (field, ty) in fields.iter().zip(params) {
        match &field.ty {
            FieldType::Struct(s_ty) => {
                // `ty` here can only be `Tuple`, `Array(Tuple)`, or `FixedArray(Tuple(), len)`.
                // We recurse on the Tuple's fields and check the FixedArray's length.
                if let StructFieldType::FixedArray(_, len) = s_ty {
                    *def &= *len <= MAX_SUPPORTED_ARRAY_LEN;
                }

                let id = s_ty.identifier();
                // TODO: InternalStructs does not contain this field's ID if the struct and field
                // are in 2 different modules, like in `can_generate_internal_structs_multiple`
                if let Some(recursed_struct) = structs.structs.get(&id) {
                    let recursed_params = get_struct_params(s_ty, ty);
                    _derive_builtin_traits_struct(
                        structs,
                        recursed_struct,
                        recursed_params,
                        def,
                        others,
                    );
                }
            }

            FieldType::Elementary(ty1) => {
                debug_assert_eq!(ty, ty1);
                *def &= can_derive_default(ty);
                *others &= can_derive_builtin_traits(ty);
            }

            FieldType::Mapping(_) => unreachable!(),
        }
    }
}

/// Recurses on the type until it reaches the struct tuple `ParamType`.
fn get_struct_params<'a>(s_ty: &StructFieldType, ty: &'a ParamType) -> &'a [ParamType] {
    match (s_ty, ty) {
        (_, ParamType::Tuple(params)) => params,
        (
            StructFieldType::Array(s_ty) | StructFieldType::FixedArray(s_ty, _),
            ParamType::Array(param) | ParamType::FixedArray(param, _),
        ) => get_struct_params(s_ty, param),
        _ => unreachable!("Unhandled struct field: {s_ty:?} | {ty:?}"),
    }
}

fn extend_derives(stream: &mut TokenStream, def: bool, others: bool) {
    if def {
        stream.extend(quote!(Default,))
    }
    if others {
        stream.extend(quote!(Debug, PartialEq, Eq, Hash))
    }
}

const MAX_SUPPORTED_ARRAY_LEN: usize = 32;
const MAX_SUPPORTED_TUPLE_LEN: usize = 12;

/// Whether the given type can derive the `Default` trait.
fn can_derive_default(param: &ParamType) -> bool {
    match param {
        ParamType::Array(ty) => can_derive_default(ty),
        ParamType::FixedBytes(len) => *len <= MAX_SUPPORTED_ARRAY_LEN,
        ParamType::FixedArray(ty, len) => {
            if *len > MAX_SUPPORTED_ARRAY_LEN {
                false
            } else {
                can_derive_default(ty)
            }
        }
        ParamType::Tuple(params) => {
            if params.len() > MAX_SUPPORTED_TUPLE_LEN {
                false
            } else {
                params.iter().all(can_derive_default)
            }
        }
        _ => true,
    }
}

/// Whether the given type can derive the builtin traits listed in [`derive_builtin_traits`], minus
/// `Default`.
fn can_derive_builtin_traits(param: &ParamType) -> bool {
    match param {
        ParamType::Array(ty) | ParamType::FixedArray(ty, _) => can_derive_builtin_traits(ty),
        ParamType::Tuple(params) => {
            if params.len() > MAX_SUPPORTED_TUPLE_LEN {
                false
            } else {
                params.iter().all(can_derive_builtin_traits)
            }
        }
        _ => true,
    }
}

/// Returns the formatted Solidity ABI signature.
pub(crate) fn abi_signature<'a, N, T>(name: N, types: T) -> String
where
    N: std::fmt::Display,
    T: IntoIterator<Item = &'a ParamType>,
{
    let types = abi_signature_types(types);
    format!("`{name}({types})`")
}

/// Returns the Solidity stringified ABI types joined by a single comma.
pub(crate) fn abi_signature_types<'a, T: IntoIterator<Item = &'a ParamType>>(types: T) -> String {
    types.into_iter().map(ToString::to_string).collect::<Vec<_>>().join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_detect_derives() {
        // array
        let param = ParamType::FixedArray(Box::new(ParamType::Uint(256)), 32);
        assert!(can_derive_default(&param));
        assert!(can_derive_builtin_traits(&param));

        let param = ParamType::FixedArray(Box::new(ParamType::Uint(256)), 33);
        assert!(!can_derive_default(&param));
        assert!(can_derive_builtin_traits(&param));

        // tuple
        let param = ParamType::Tuple(vec![ParamType::Uint(256); 12]);
        assert!(can_derive_default(&param));
        assert!(can_derive_builtin_traits(&param));

        let param = ParamType::Tuple(vec![ParamType::Uint(256); 13]);
        assert!(!can_derive_default(&param));
        assert!(!can_derive_builtin_traits(&param));
    }

    #[test]
    fn can_resolve_path() {
        let raw = "./$ENV_VAR";
        std::env::set_var("ENV_VAR", "file.txt");
        let resolved = resolve_path(raw).unwrap();
        assert_eq!(resolved.to_str().unwrap(), "./file.txt");
    }

    #[test]
    fn input_name_to_ident_empty() {
        assert_quote!(expand_input_name(0, ""), { p0 });
    }

    #[test]
    fn input_name_to_ident_keyword() {
        assert_quote!(expand_input_name(0, "self"), { self_ });
    }

    #[test]
    fn input_name_to_ident_snake_case() {
        assert_quote!(expand_input_name(0, "CamelCase1"), { camel_case_1 });
    }

    #[test]
    fn test_safe_module_name() {
        assert_eq!(safe_module_name("Valid"), "valid");
        assert_eq!(safe_module_name("Enum"), "enum_");
        assert_eq!(safe_module_name("Mod"), "mod_");
        assert_eq!(safe_module_name("2Two"), "_2_two");
    }
}
