//! Implementation of procedural macro for generating type-safe bindings to an
//! ethereum smart contract.
use crate::spanned::{ParseInner, Spanned};

use ethers_contract_abigen::Abigen;
use ethers_core::abi::{Function, FunctionExt, Param, StateMutability};

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::ToTokens;
use std::collections::HashSet;
use std::error::Error;
use syn::ext::IdentExt;
use syn::parse::{Error as ParseError, Parse, ParseStream, Result as ParseResult};
use syn::{braced, parenthesized, Ident, LitStr, Path, Token};

pub(crate) fn expand(args: ContractArgs) -> Result<TokenStream2, Box<dyn Error>> {
    Ok(args.into_builder()?.generate()?.into_tokens())
}

/// Contract procedural macro arguments.
#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
pub(crate) struct ContractArgs {
    name: String,
    abi: String,
    parameters: Vec<Parameter>,
}

impl ContractArgs {
    fn into_builder(self) -> Result<Abigen, Box<dyn Error>> {
        let mut builder = Abigen::new(&self.name, &self.abi)?;

        for parameter in self.parameters.into_iter() {
            builder = match parameter {
                Parameter::Methods(methods) => methods.into_iter().fold(builder, |builder, m| {
                    builder.add_method_alias(m.signature, m.alias)
                }),
                Parameter::EventDerives(derives) => derives
                    .into_iter()
                    .fold(builder, |builder, derive| builder.add_event_derive(derive)),
            };
        }

        Ok(builder)
    }
}

impl ParseInner for ContractArgs {
    fn spanned_parse(input: ParseStream) -> ParseResult<(Span, Self)> {
        // read the contract name
        let name = input.parse::<Ident>()?.to_string();

        // skip the comma
        input.parse::<Token![,]>()?;

        // TODO(nlordell): Due to limitation with the proc-macro Span API, we
        //   can't currently get a path the the file where we were called from;
        //   therefore, the path will always be rooted on the cargo manifest
        //   directory. Eventually we can use the `Span::source_file` API to
        //   have a better experience.
        let (span, abi) = {
            let literal = input.parse::<LitStr>()?;
            (literal.span(), literal.value())
        };

        if !input.is_empty() {
            input.parse::<Token![,]>()?;
        }

        let parameters = input
            .parse_terminated::<_, Token![,]>(Parameter::parse)?
            .into_iter()
            .collect();

        Ok((
            span,
            ContractArgs {
                name,
                abi,
                parameters,
            },
        ))
    }
}

/// A single procedural macro parameter.
#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
enum Parameter {
    Methods(Vec<Method>),
    EventDerives(Vec<String>),
}

impl Parse for Parameter {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let name = input.call(Ident::parse_any)?;
        let param = match name.to_string().as_str() {
            "methods" => {
                let content;
                braced!(content in input);
                let methods = {
                    let parsed =
                        content.parse_terminated::<_, Token![;]>(Spanned::<Method>::parse)?;

                    let mut methods = Vec::with_capacity(parsed.len());
                    let mut signatures = HashSet::new();
                    let mut aliases = HashSet::new();
                    for method in parsed {
                        if !signatures.insert(method.signature.clone()) {
                            return Err(ParseError::new(
                                method.span(),
                                "duplicate method signature in `abigen!` macro invocation",
                            ));
                        }
                        if !aliases.insert(method.alias.clone()) {
                            return Err(ParseError::new(
                                method.span(),
                                "duplicate method alias in `abigen!` macro invocation",
                            ));
                        }
                        methods.push(method.into_inner())
                    }

                    methods
                };

                Parameter::Methods(methods)
            }
            "event_derives" => {
                let content;
                parenthesized!(content in input);
                let derives = content
                    .parse_terminated::<_, Token![,]>(Path::parse)?
                    .into_iter()
                    .map(|path| path.to_token_stream().to_string())
                    .collect();
                Parameter::EventDerives(derives)
            }
            _ => {
                return Err(ParseError::new(
                    name.span(),
                    format!("unexpected named parameter `{}`", name),
                ))
            }
        };

        Ok(param)
    }
}

/// An explicitely named contract method.
#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
struct Method {
    signature: String,
    alias: String,
}

impl Parse for Method {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let function = {
            let name = input.parse::<Ident>()?.to_string();

            let content;
            parenthesized!(content in input);
            let inputs = content
                .parse_terminated::<_, Token![,]>(Ident::parse)?
                .iter()
                .map(|ident| {
                    let kind = serde_json::from_value(serde_json::json!(&ident.to_string()))
                        .map_err(|err| ParseError::new(ident.span(), err))?;
                    Ok(Param {
                        name: "".into(),
                        kind,
                    })
                })
                .collect::<ParseResult<Vec<_>>>()?;

            Function {
                name,
                inputs,

                // NOTE: The output types and const-ness of the function do not
                //   affect its signature.
                outputs: vec![],
                state_mutability: StateMutability::Nonpayable,
            }
        };
        let signature = function.abi_signature();
        input.parse::<Token![as]>()?;
        let alias = {
            let ident = input.parse::<Ident>()?;
            ident.to_string()
        };

        Ok(Method { signature, alias })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! contract_args_result {
        ($($arg:tt)*) => {{
            use syn::parse::Parser;
            <Spanned<ContractArgs> as Parse>::parse
                .parse2(quote::quote! { $($arg)* })
        }};
    }

    macro_rules! contract_args {
        ($($arg:tt)*) => {
            contract_args_result!($($arg)*)
                .expect("failed to parse contract args")
                .into_inner()
        };
    }

    macro_rules! contract_args_err {
        ($($arg:tt)*) => {
            contract_args_result!($($arg)*)
                .expect_err("expected parse contract args to error")
        };
    }

    #[allow(unused)]
    fn method(signature: &str, alias: &str) -> Method {
        Method {
            signature: signature.into(),
            alias: alias.into(),
        }
    }

    #[test]
    fn parse_contract_args() {
        let args = contract_args!(TestContract, "path/to/abi.json");
        assert_eq!(args.name, "TestContract");
        assert_eq!(args.abi, "path/to/abi.json");
    }

    #[test]
    fn parse_contract_args_with_defaults() {
        let args = contract_args!(TestContract, "[{}]");
        assert_eq!(
            args,
            ContractArgs {
                name: "TestContract".to_string(),
                abi: "[{}]".to_string(),
                parameters: vec![],
            },
        );
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
            args,
            ContractArgs {
                name: "TestContract".to_string(),
                abi: "abi.json".to_string(),
                parameters: vec![
                    // Parameter::Contract("Contract".into()),
                    Parameter::Methods(vec![
                        method("myMethod(uint256,bool)", "my_renamed_method"),
                        method("myOtherMethod()", "my_other_renamed_method"),
                    ]),
                    Parameter::EventDerives(vec![
                        "Asdf".into(),
                        "a :: B".into(),
                        "a :: b :: c :: D".into()
                    ])
                ],
            },
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
