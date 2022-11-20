use ethers_core::{
    abi::{Param, ParamType},
    types::Address,
};
use eyre::Result;
use inflector::Inflector;
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;
use std::path::PathBuf;
use syn::{Ident as SynIdent, Path};

/// Expands a identifier string into a token.
pub fn ident(name: &str) -> Ident {
    Ident::new(name, Span::call_site())
}

/// Expands an identifier string into a token and appending `_` if the
/// identifier is for a reserved keyword.
///
/// Parsing keywords like `self` can fail, in this case we add an underscore.
pub fn safe_ident(name: &str) -> Ident {
    syn::parse_str::<SynIdent>(name).unwrap_or_else(|_| ident(&format!("{name}_")))
}

///  Converts a `&str` to `snake_case` `String` while respecting identifier rules
pub fn safe_snake_case(ident: &str) -> String {
    safe_identifier_name(ident.to_snake_case())
}

///  Converts a `&str` to `PascalCase` `String` while respecting identifier rules
pub fn safe_pascal_case(ident: &str) -> String {
    safe_identifier_name(ident.to_pascal_case())
}

/// respects identifier rules, such as, an identifier must not start with a numeric char
fn safe_identifier_name(name: String) -> String {
    if name.starts_with(|c: char| c.is_numeric()) {
        format!("_{name}")
    } else {
        name
    }
}

/// converts invalid rust module names to valid ones
pub fn safe_module_name(name: &str) -> String {
    // handle reserve words used in contracts (eg Enum is a gnosis contract)
    safe_ident(&safe_snake_case(name)).to_string()
}

/// Expands an identifier as snakecase and preserve any leading or trailing underscores
pub fn safe_snake_case_ident(name: &str) -> Ident {
    let i = name.to_snake_case();
    ident(&preserve_underscore_delim(&i, name))
}

/// Expands an identifier as pascal case and preserve any leading or trailing underscores
pub fn safe_pascal_case_ident(name: &str) -> Ident {
    let i = name.to_pascal_case();
    ident(&preserve_underscore_delim(&i, name))
}

/// Reapplies leading and trailing underscore chars to the ident
/// Example `ident = "pascalCase"; alias = __pascalcase__` -> `__pascalCase__`
pub fn preserve_underscore_delim(ident: &str, alias: &str) -> String {
    alias
        .chars()
        .take_while(|c| *c == '_')
        .chain(ident.chars())
        .chain(alias.chars().rev().take_while(|c| *c == '_'))
        .collect()
}

/// Expands a positional identifier string that may be empty.
///
/// Note that this expands the parameter name with `safe_ident`, meaning that
/// identifiers that are reserved keywords get `_` appended to them.
pub fn expand_input_name(index: usize, name: &str) -> TokenStream {
    let name_str = match name {
        "" => format!("p{index}"),
        n => n.to_snake_case(),
    };
    let name = safe_ident(&name_str);

    quote! { #name }
}

/// Expands a doc string into an attribute token stream.
pub fn expand_doc(s: &str) -> TokenStream {
    let doc = Literal::string(s);
    quote! {
        #[doc = #doc]
    }
}

pub fn expand_derives(derives: &[Path]) -> TokenStream {
    quote! {#(#derives),*}
}

/// Parses the given address string
pub fn parse_address<S>(address_str: S) -> Result<Address>
where
    S: AsRef<str>,
{
    let address_str = address_str.as_ref();
    eyre::ensure!(address_str.starts_with("0x"), "address must start with '0x'");
    Ok(address_str[2..].parse()?)
}

/// Perform an HTTP GET request and return the contents of the response.
#[cfg(not(target_arch = "wasm32"))]
pub fn http_get(_url: &str) -> Result<String> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "reqwest")]{
            Ok(reqwest::blocking::get(_url)?.text()?)
        } else {
            eyre::bail!("HTTP is unsupported")
        }
    }
}

/// Replaces any occurrences of env vars in the `raw` str with their value
pub fn resolve_path(raw: &str) -> Result<PathBuf> {
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
                eyre::bail!("Unable to parse a variable from \"{}\"", tail)
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
pub fn json_files(root: impl AsRef<std::path::Path>) -> Vec<PathBuf> {
    walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().map(|ext| ext == "json").unwrap_or_default())
        .map(|e| e.path().into())
        .collect()
}

/// rust-std derives `Default` automatically only for arrays len <= 32
///
/// Returns whether the corresponding struct can derive `Default`
pub fn can_derive_defaults(params: &[Param]) -> bool {
    params.iter().map(|param| &param.kind).all(can_derive_default)
}

pub fn can_derive_default(param: &ParamType) -> bool {
    const MAX_SUPPORTED_LEN: usize = 32;
    match param {
        ParamType::FixedBytes(len) => *len <= MAX_SUPPORTED_LEN,
        ParamType::FixedArray(ty, len) => {
            if *len > MAX_SUPPORTED_LEN {
                false
            } else {
                can_derive_default(ty)
            }
        }
        ParamType::Tuple(params) => params.iter().all(can_derive_default),
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_detect_non_default() {
        let param = ParamType::FixedArray(Box::new(ParamType::Uint(64)), 128);
        assert!(!can_derive_default(&param));

        let param = ParamType::FixedArray(Box::new(ParamType::Uint(64)), 32);
        assert!(can_derive_default(&param));
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
    fn parse_address_missing_prefix() {
        let _ = parse_address("0000000000000000000000000000000000000000").unwrap_err();
    }

    #[test]
    fn parse_address_address_too_short() {
        let _ = parse_address("0x00000000000000").unwrap_err();
    }

    #[test]
    fn parse_address_ok() {
        let expected =
            Address::from([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19]);
        assert_eq!(parse_address("0x000102030405060708090a0b0c0d0e0f10111213").unwrap(), expected);
    }

    #[test]
    fn test_safe_module_name() {
        assert_eq!(safe_module_name("Valid"), "valid");
        assert_eq!(safe_module_name("Enum"), "enum_");
        assert_eq!(safe_module_name("Mod"), "mod_");
        assert_eq!(safe_module_name("2Two"), "_2_two");
    }
}
