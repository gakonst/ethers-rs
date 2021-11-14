use ethers_core::types::Address;

use anyhow::{anyhow, Result};
use cfg_if::cfg_if;
use inflector::Inflector;
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;

use syn::{Ident as SynIdent, Path};

/// Expands a identifier string into an token.
pub fn ident(name: &str) -> Ident {
    Ident::new(name, Span::call_site())
}

/// Expands an identifier string into a token and appending `_` if the
/// identifier is for a reserved keyword.
///
/// Parsing keywords like `self` can fail, in this case we add an underscore.
pub fn safe_ident(name: &str) -> Ident {
    syn::parse_str::<SynIdent>(name).unwrap_or_else(|_| ident(&format!("{}_", name)))
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
        "" => format!("p{}", index),
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
    if !address_str.starts_with("0x") {
        return Err(anyhow!("address must start with '0x'"))
    }
    Ok(address_str[2..].parse()?)
}

/// Perform an HTTP GET request and return the contents of the response.
pub fn http_get(_url: &str) -> Result<String> {
    cfg_if! {
        if #[cfg(any(not(target_arch = "wasm32"), not(features = "reqwest")))]{
            Err(anyhow!("HTTP is unsupported"))
        } else {
            Ok(reqwest::blocking::get(_url)?.text()?)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(
            !parse_address("0000000000000000000000000000000000000000").is_ok(),
            "parsing address not starting with 0x should fail"
        );
    }

    #[test]
    fn parse_address_address_too_short() {
        assert!(
            !parse_address("0x00000000000000").is_ok(),
            "parsing address not starting with 0x should fail"
        );
    }

    #[test]
    fn parse_address_ok() {
        let expected =
            Address::from([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19]);
        assert_eq!(parse_address("0x000102030405060708090a0b0c0d0e0f10111213").unwrap(), expected);
    }
}
