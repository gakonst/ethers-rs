//! Implementation of procedural macro for generating type-safe bindings to an
//! ethereum smart contract.
#![deny(missing_docs, unsafe_code)]

extern crate proc_macro;

mod spanned;
use crate::spanned::{ParseInner, Spanned, parse_address, Address, Builder};

use ethers::abi::{Function, Param, ParamType, FunctionExt, ParamTypeExt};

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens as _};
use std::collections::HashSet;
use std::error::Error;
use syn::ext::IdentExt;
use syn::parse::{Error as ParseError, Parse, ParseStream, Result as ParseResult};
use syn::{
    braced, parenthesized, parse_macro_input, Error as SynError, Ident, LitInt, LitStr, Path,
    Token, Visibility,
};

// TODO: Make it accept an inline ABI array
/// Proc macro to generate type-safe bindings to a contract. This macro accepts
/// an Ethereum contract ABIABI or a path. Note that this path is rooted in
/// the crate's root `CARGO_MANIFEST_DIR`.
///
/// ```ignore
/// ethcontract::contract!("build/contracts/MyContract.json");
/// ```
///
/// Alternatively, other sources may be used, for full details consult the
/// `ethcontract-generate::source` documentation. Some basic examples:
///
/// ```ignore
/// // HTTP(S) source
/// ethcontract::contract!("https://my.domain.local/path/to/contract.json")
/// // Etherscan.io
/// ethcontract::contract!("etherscan:0x0001020304050607080910111213141516171819");
/// ethcontract::contract!("https://etherscan.io/address/0x0001020304050607080910111213141516171819");
/// // npmjs
/// ethcontract::contract!("npm:@org/package@1.0.0/path/to/contract.json")
/// ```
///
/// Note that Etherscan rate-limits requests to their API, to avoid this an
/// `ETHERSCAN_API_KEY` environment variable can be set. If it is, it will use
/// that API key when retrieving the contract ABI.
///
/// Currently the proc macro accepts additional parameters to configure some
/// aspects of the code generation. Specifically it accepts:
/// - `crate`: The name of the `ethcontract` crate. This is useful if the crate
///   was renamed in the `Cargo.toml` for whatever reason.
/// - `contract`: Override the contract name that is used for the generated
///   type. This is required when using sources that do not provide the contract
///   name in the artifact JSON such as Etherscan.
/// - `mod`: The name of the contract module to place generated code in. Note
///   that the root contract type gets re-exported in the context where the
///   macro was invoked. This defaults to the contract name converted into snake
///   case.
/// - `methods`: A list of mappings from method signatures to method names
///   allowing methods names to be explicitely set for contract methods. This
///   also provides a workaround for generating code for contracts with multiple
///   methods with the same name.
/// - `event_derives`: A list of additional derives that should be added to
///   contract event structs and enums.
///
/// Additionally, the ABI source can be preceeded by a visibility modifier such
/// as `pub` or `pub(crate)`. This visibility modifier is applied to both the
/// generated module and contract re-export. If no visibility modifier is
/// provided, then none is used for the generated code as well, making the
/// module and contract private to the scope where the macro was invoked.
///
/// ```ignore
/// ethcontract::contract!(
///     pub(crate) "build/contracts/MyContract.json",
///     crate = ethcontract_rename,
///     mod = my_contract_instance,
///     contract = MyContractInstance,
///     deployments {
///         4 => "0x000102030405060708090a0b0c0d0e0f10111213",
///         5777 => "0x0123456789012345678901234567890123456789",
///     },
///     methods {
///         myMethod(uint256,bool) as my_renamed_method;
///     },
///     event_derives (serde::Deserialize, serde::Serialize),
/// );
/// ```
///
/// See [`ethcontract`](ethcontract) module level documentation for additional
/// information.
#[proc_macro]
pub fn contract(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as Spanned<ContractArgs>);

    let span = args.span();
    expand(args.into_inner())
        .unwrap_or_else(|e| SynError::new(span, format!("{:?}", e)).to_compile_error())
        .into()
}

fn expand(args: ContractArgs) -> Result<TokenStream2, Box<dyn Error>> {
    Ok(args.into_builder()?.generate()?.into_tokens())
}

/// Contract procedural macro arguments.
#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
struct ContractArgs {
    visibility: Option<String>,
    artifact_path: String,
    parameters: Vec<Parameter>,
}

impl ContractArgs {
    fn into_builder(self) -> Result<Builder, Box<dyn Error>> {
        let mut builder = Builder::from_source_url(&self.artifact_path)?
            .with_visibility_modifier(self.visibility);

        for parameter in self.parameters.into_iter() {
            builder = match parameter {
                Parameter::Mod(name) => builder.with_contract_mod_override(Some(name)),
                Parameter::Contract(name) => builder.with_contract_name_override(Some(name)),
                Parameter::Crate(name) => builder.with_runtime_crate_name(name),
                Parameter::Deployments(deployments) => {
                    deployments.into_iter().fold(builder, |builder, d| {
                        builder.add_deployment(d.network_id, d.address)
                    })
                }
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
        let visibility = match input.parse::<Visibility>()? {
            Visibility::Inherited => None,
            token => Some(quote!(#token).to_string()),
        };

        // TODO(nlordell): Due to limitation with the proc-macro Span API, we
        //   can't currently get a path the the file where we were called from;
        //   therefore, the path will always be rooted on the cargo manifest
        //   directory. Eventually we can use the `Span::source_file` API to
        //   have a better experience.
        let (span, artifact_path) = {
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
                visibility,
                artifact_path,
                parameters,
            },
        ))
    }
}

/// A single procedural macro parameter.
#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
enum Parameter {
    Mod(String),
    Contract(String),
    Crate(String),
    Deployments(Vec<Deployment>),
    Methods(Vec<Method>),
    EventDerives(Vec<String>),
}

impl Parse for Parameter {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let name = input.call(Ident::parse_any)?;
        let param = match name.to_string().as_str() {
            "crate" => {
                input.parse::<Token![=]>()?;
                let name = input.call(Ident::parse_any)?.to_string();
                Parameter::Crate(name)
            }
            "mod" => {
                input.parse::<Token![=]>()?;
                let name = input.parse::<Ident>()?.to_string();
                Parameter::Mod(name)
            }
            "contract" => {
                input.parse::<Token![=]>()?;
                let name = input.parse::<Ident>()?.to_string();
                Parameter::Contract(name)
            }
            "deployments" => {
                let content;
                braced!(content in input);
                let deployments = {
                    let parsed =
                        content.parse_terminated::<_, Token![,]>(Spanned::<Deployment>::parse)?;

                    let mut deployments = Vec::with_capacity(parsed.len());
                    let mut networks = HashSet::new();
                    for deployment in parsed {
                        if !networks.insert(deployment.network_id) {
                            return Err(ParseError::new(
                                deployment.span(),
                                "duplicate network ID in `ethcontract::contract!` macro invocation",
                            ));
                        }
                        deployments.push(deployment.into_inner())
                    }

                    deployments
                };

                Parameter::Deployments(deployments)
            }
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
                                "duplicate method signature in `ethcontract::contract!` macro invocation",
                            ));
                        }
                        if !aliases.insert(method.alias.clone()) {
                            return Err(ParseError::new(
                                method.span(),
                                "duplicate method alias in `ethcontract::contract!` macro invocation",
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

/// A manually specified dependency.
#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
struct Deployment {
    network_id: u32,
    address: Address,
}

impl Parse for Deployment {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let network_id = input.parse::<LitInt>()?.base10_parse()?;
        input.parse::<Token![=>]>()?;
        let address = {
            let literal = input.parse::<LitStr>()?;
            parse_address(&literal.value()).map_err(|err| ParseError::new(literal.span(), err))?
        };

        Ok(Deployment {
            network_id,
            address,
        })
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
                    let kind = ParamType::from_str(&ident.to_string())
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
                constant: false,
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

    fn deployment(network_id: u32, address: &str) -> Deployment {
        Deployment {
            network_id,
            address: parse_address(address).expect("failed to parse deployment address"),
        }
    }

    fn method(signature: &str, alias: &str) -> Method {
        Method {
            signature: signature.into(),
            alias: alias.into(),
        }
    }

    #[test]
    fn parse_contract_args() {
        let args = contract_args!("path/to/artifact.json");
        assert_eq!(args.artifact_path, "path/to/artifact.json");
    }

    #[test]
    fn crate_parameter_accepts_keywords() {
        let args = contract_args!("artifact.json", crate = crate);
        assert_eq!(args.parameters, &[Parameter::Crate("crate".into())]);
    }

    #[test]
    fn parse_contract_args_with_defaults() {
        let args = contract_args!("artifact.json");
        assert_eq!(
            args,
            ContractArgs {
                visibility: None,
                artifact_path: "artifact.json".into(),
                parameters: vec![],
            },
        );
    }

    #[test]
    fn parse_contract_args_with_parameters() {
        let args = contract_args!(
            pub(crate) "artifact.json",
            crate = foobar,
            mod = contract,
            contract = Contract,
            deployments {
                1 => "0x000102030405060708090a0b0c0d0e0f10111213",
                4 => "0x0123456789012345678901234567890123456789",
            },
            methods {
                myMethod(uint256, bool) as my_renamed_method;
                myOtherMethod() as my_other_renamed_method;
            },
            event_derives (Asdf, a::B, a::b::c::D)
        );
        assert_eq!(
            args,
            ContractArgs {
                visibility: Some(quote!(pub(crate)).to_string()),
                artifact_path: "artifact.json".into(),
                parameters: vec![
                    Parameter::Crate("foobar".into()),
                    Parameter::Mod("contract".into()),
                    Parameter::Contract("Contract".into()),
                    Parameter::Deployments(vec![
                        deployment(1, "0x000102030405060708090a0b0c0d0e0f10111213"),
                        deployment(4, "0x0123456789012345678901234567890123456789"),
                    ]),
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
    fn duplicate_network_id_error() {
        contract_args_err!(
            "artifact.json",
            deployments {
                1 => "0x000102030405060708090a0b0c0d0e0f10111213",
                1 => "0x0123456789012345678901234567890123456789",
            }
        );
    }

    #[test]
    fn duplicate_method_rename_error() {
        contract_args_err!(
            "artifact.json",
            methods {
                myMethod(uint256) as my_method_1;
                myMethod(uint256) as my_method_2;
            }
        );
        contract_args_err!(
            "artifact.json",
            methods {
                myMethod1(uint256) as my_method;
                myMethod2(uint256) as my_method;
            }
        );
    }

    #[test]
    fn method_invalid_method_parameter_type() {
        contract_args_err!(
            "artifact.json",
            methods {
                myMethod(invalid) as my_method;
            }
        );
    }
}
