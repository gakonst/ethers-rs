//! Implementation of procedural macro for generating type-safe bindings to an Ethereum smart
//! contract.

use crate::spanned::Spanned;
use ethers_contract_abigen::{multi::MultiExpansion, Abigen};
use proc_macro2::TokenStream;
use std::collections::HashSet;
use syn::{
    braced,
    ext::IdentExt,
    parenthesized,
    parse::{Error, Parse, ParseStream, Result},
    punctuated::Punctuated,
    Ident, LitInt, LitStr, Path, Token,
};

/// A series of `ContractArgs` separated by `;`
#[derive(Clone, Debug)]
pub(crate) struct Contracts {
    pub(crate) inner: Vec<ContractArgs>,
}

impl Contracts {
    pub(crate) fn expand(self) -> Result<TokenStream> {
        let mut expansions = Vec::with_capacity(self.inner.len());

        // expand all contracts
        for contract in self.inner {
            let span = contract.abi.span();
            let contract = contract
                .into_builder()
                .and_then(|a| a.expand().map_err(|e| Error::new(span, e)))?;
            expansions.push(contract);
        }

        // expand all contract expansions
        Ok(MultiExpansion::new(expansions).expand_inplace())
    }
}

impl Parse for Contracts {
    fn parse(input: ParseStream) -> Result<Self> {
        let inner = input.parse_terminated(ContractArgs::parse, Token![;])?.into_iter().collect();
        Ok(Self { inner })
    }
}

/// Contract procedural macro arguments.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ContractArgs {
    name: Ident,
    abi: LitStr,
    parameters: Punctuated<Parameter, Token![,]>,
}

impl ContractArgs {
    fn into_builder(self) -> Result<Abigen> {
        // use the name's ident
        let contract_name = self.name;
        let abi = self.abi.value();
        let abi_source = abi.parse().map_err(|e| Error::new(self.abi.span(), e))?;
        let mut builder = Abigen::new_raw(contract_name, abi_source);

        for parameter in self.parameters {
            match parameter {
                Parameter::Methods(methods) => builder
                    .method_aliases_mut()
                    .extend(methods.into_iter().map(|m| (m.signature, m.alias.to_string()))),
                Parameter::Derives(derives) => builder.derives_mut().extend(derives),
            }
        }

        Ok(builder)
    }
}

impl Parse for ContractArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        // name
        let name = input.parse::<Ident>()?;

        input.parse::<Token![,]>()?;

        // abi
        // TODO(nlordell): Due to limitation with the proc-macro Span API, we
        //   can't currently get a path the the file where we were called from;
        //   therefore, the path will always be rooted on the cargo manifest
        //   directory. Eventually we can use the `Span::source_file` API to
        //   have a better experience.
        let abi = input.parse::<LitStr>()?;

        // optional parameters
        let mut parameters = Punctuated::default();
        if input.parse::<Token![,]>().is_ok() {
            loop {
                if input.is_empty() || input.peek(Token![;]) {
                    break
                }
                parameters.push_value(input.parse()?);
                if let Ok(comma) = input.parse() {
                    parameters.push_punct(comma);
                }
            }
        }

        Ok(ContractArgs { name, abi, parameters })
    }
}

/// A single procedural macro parameter.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Parameter {
    Methods(Vec<Method>),
    Derives(Punctuated<Path, Token![,]>),
}

impl Parse for Parameter {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = Ident::parse_any(input)?;
        match name.to_string().as_str() {
            "methods" => {
                let content;
                braced!(content in input);
                let parsed = content.parse_terminated(Spanned::<Method>::parse, Token![;])?;

                let mut methods = Vec::with_capacity(parsed.len());
                let mut signatures = HashSet::new();
                let mut aliases = HashSet::new();
                for method in parsed {
                    if !signatures.insert(method.signature.clone()) {
                        return Err(Error::new(method.span(), "duplicate method signature"))
                    }
                    if !aliases.insert(method.alias.clone()) {
                        return Err(Error::new(method.alias.span(), "duplicate method alias"))
                    }
                    methods.push(method.into_inner());
                }
                Ok(Parameter::Methods(methods))
            }
            "derives" | "event_derives" => {
                let content;
                parenthesized!(content in input);
                let derives = content.parse_terminated(Path::parse, Token![,])?;
                Ok(Parameter::Derives(derives))
            }
            _ => Err(Error::new(name.span(), "unexpected named parameter")),
        }
    }
}

/// An explicitely named contract method.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Method {
    signature: String,
    alias: Ident,
}

impl Parse for Method {
    fn parse(input: ParseStream) -> Result<Self> {
        // `{name}({params.join(",")})`
        let mut signature = String::with_capacity(64);

        // function name
        let name = input.parse::<Ident>()?;
        signature.push_str(&name.to_string());

        // function params
        let content;
        parenthesized!(content in input);
        let params = content.parse_terminated(IdentBracket::parse, Token![,])?;
        let last_i = params.len().saturating_sub(1);

        signature.push('(');
        for (i, param) in params.into_iter().enumerate() {
            let mut s = param.ident.to_string();
            if let Some((_, inside_brackets)) = param.bracket {
                s.push('[');
                if let Some(lit) = inside_brackets {
                    s.push_str(lit.base10_digits());
                }
                s.push(']');
            }
            // validate
            ethers_core::abi::ethabi::param_type::Reader::read(&s)
                .map_err(|e| Error::new(param.ident.span(), e))?;
            signature.push_str(&s);
            if i < last_i {
                signature.push(',');
            }
        }
        signature.push(')');

        input.parse::<Token![as]>()?;

        let alias = input.parse()?;

        Ok(Method { signature, alias })
    }
}

pub struct IdentBracket {
    pub ident: syn::Ident,
    pub bracket: Option<(syn::token::Bracket, Option<LitInt>)>,
}

impl Parse for IdentBracket {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        let ident = input.parse()?;
        let bracket = move || -> std::result::Result<_, _> {
            let content;
            Ok((syn::bracketed!(content in input), content.parse()?))
        };

        Ok(IdentBracket { ident, bracket: bracket().ok() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::Span;
    use quote::quote;
    use syn::parse::Parser;

    macro_rules! contract_args_result {
        ($($tt:tt)+) => {{
            Parser::parse2(Contracts::parse, quote!($($tt)+))
        }};
    }

    macro_rules! contract_args {
        ($($tt:tt)*) => {
            contract_args_result!($($tt)*)
                .expect("failed to parse contract args")
                .inner
        };
    }

    macro_rules! contract_args_err {
        ($($tt:tt)*) => {
            contract_args_result!($($tt)*)
                .expect_err("expected parse contract args to error")
        };
    }

    fn method(signature: &str, alias: &str) -> Method {
        Method { signature: signature.into(), alias: ident(alias) }
    }

    // Note: AST structs implement PartialEq by comparing the string repr, so the span is ignored.
    fn arg(
        name: &str,
        abi: &str,
        parameters: impl IntoIterator<Item = Parameter>,
        trailing: bool,
    ) -> ContractArgs {
        ContractArgs {
            name: ident(name),
            abi: lit_str(abi),
            parameters: params(parameters, trailing),
        }
    }

    fn ident(s: &str) -> Ident {
        Ident::new(s, Span::call_site())
    }

    fn lit_str(s: &str) -> LitStr {
        LitStr::new(s, Span::call_site())
    }

    fn params(
        v: impl IntoIterator<Item = Parameter>,
        trailing: bool,
    ) -> Punctuated<Parameter, Token![,]> {
        let mut punct: Punctuated<Parameter, Token![,]> = v.into_iter().collect();
        if trailing {
            punct.push_punct(Default::default());
        }
        punct
    }

    fn derives<'a>(v: impl IntoIterator<Item = &'a str>, trailing: bool) -> Parameter {
        let mut derives: Punctuated<_, _> =
            v.into_iter().map(|s| syn::parse_str::<syn::Path>(s).unwrap()).collect();
        if trailing {
            derives.push_punct(Default::default());
        }
        Parameter::Derives(derives)
    }

    #[test]
    fn parse_multi_contract_args_events() {
        let args = contract_args! {
            TestContract,
            "path/to/abi.json",
            event_derives(serde::Deserialize, serde::Serialize);

            TestContract2,
            "other.json",
            event_derives(serde::Deserialize, serde::Serialize);
        };

        assert_eq!(
            args,
            vec![
                arg(
                    "TestContract",
                    "path/to/abi.json",
                    [derives(["serde::Deserialize", "serde::Serialize"], false)],
                    false
                ),
                arg(
                    "TestContract2",
                    "other.json",
                    [derives(["serde::Deserialize", "serde::Serialize"], false)],
                    false
                ),
            ]
        );
    }

    #[test]
    fn parse_multi_contract_args_methods() {
        let args = contract_args! {
            TestContract,
            "path/to/abi.json",
            methods {
                myMethod(uint256, bool) as my_renamed_method;
                myOtherMethod() as my_other_renamed_method;
            };

            TestContract2,
            "other.json",
            event_derives(serde::Deserialize, serde::Serialize);
        };

        assert_eq!(
            args,
            vec![
                arg(
                    "TestContract",
                    "path/to/abi.json",
                    [Parameter::Methods(vec![
                        method("myMethod(uint256,bool)", "my_renamed_method"),
                        method("myOtherMethod()", "my_other_renamed_method"),
                    ])],
                    false
                ),
                arg(
                    "TestContract2",
                    "other.json",
                    [derives(["serde::Deserialize", "serde::Serialize"], false)],
                    false
                ),
            ]
        );
    }

    #[test]
    fn parse_multi_contract_args() {
        let args = contract_args! {
            TestContract,
            "path/to/abi.json",;

            TestContract2,
            "other.json",
            event_derives(serde::Deserialize, serde::Serialize,);
        };

        assert_eq!(
            args,
            vec![
                arg("TestContract", "path/to/abi.json", [], false),
                arg(
                    "TestContract2",
                    "other.json",
                    [derives(["serde::Deserialize", "serde::Serialize"], true)],
                    false
                ),
            ]
        );
    }

    #[test]
    fn parse_contract_args() {
        let args = contract_args!(TestContract, "path/to/abi.json");
        assert_eq!(*args.first().unwrap(), arg("TestContract", "path/to/abi.json", [], false));
    }

    #[test]
    fn parse_contract_args_with_defaults() {
        let args = contract_args!(TestContract, "[{}]");
        assert_eq!(*args.first().unwrap(), arg("TestContract", "[{}]", [], false));
    }

    #[test]
    fn parse_contract_args_with_parameters() {
        let args = contract_args!(
            TestContract,
            "abi.json",
            methods {
                myMethod(uint256, bool) as my_renamed_method;
                myOtherMethod() as my_other_renamed_method;
            },
            event_derives (Asdf, a::B, a::b::c::D)
        );
        assert_eq!(
            *args.first().unwrap(),
            arg(
                "TestContract",
                "abi.json",
                [
                    Parameter::Methods(vec![
                        method("myMethod(uint256,bool)", "my_renamed_method"),
                        method("myOtherMethod()", "my_other_renamed_method"),
                    ]),
                    derives(["Asdf", "a::B", "a::b::c::D"], false)
                ],
                false
            )
        );
    }

    #[test]
    fn duplicate_method_rename_error() {
        contract_args_err!(
            "abi.json",
            methods {
                myMethod(uint256) as my_method_1;
                myMethod(uint256) as my_method_2;
            }
        );
        contract_args_err!(
            "abi.json",
            methods {
                myMethod1(uint256) as my_method;
                myMethod2(uint256) as my_method;
            }
        );
    }

    #[test]
    fn method_invalid_method_parameter_type() {
        contract_args_err!(
            "abi.json",
            methods {
                myMethod(invalid) as my_method;
            }
        );
    }
}
