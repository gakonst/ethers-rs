use super::{types, util, Context};

use ethers_abi::{Function, FunctionExt, Param};
use ethers_types::Selector;

use anyhow::{anyhow, Context as _, Result};
use inflector::Inflector;
use proc_macro2::{Literal, TokenStream};
use quote::quote;
use rustc_hex::ToHex;
use syn::Ident;

/// Expands a context into a method struct containing all the generated bindings
/// to the Solidity contract methods.
impl Context {
    pub(crate) fn methods(&self) -> Result<TokenStream> {
        let mut aliases = self.method_aliases.clone();

        let functions = self
            .abi
            .functions()
            .map(|function| {
                let signature = function.abi_signature();
                expand_function(function, aliases.remove(&signature))
                    .with_context(|| format!("error expanding function '{}'", signature))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(quote! { #( #functions )* })
    }
}

#[allow(unused)]
fn expand_function(function: &Function, alias: Option<Ident>) -> Result<TokenStream> {
    let name = alias.unwrap_or_else(|| util::safe_ident(&function.name.to_snake_case()));
    let selector = expand_selector(function.selector());

    let input = expand_inputs(&function.inputs)?;

    let outputs = expand_fn_outputs(&function.outputs)?;

    let result = if function.constant {
        quote! { ContractCall<'a, P, N, S, #outputs> }
    } else {
        quote! { ContractCall<'a, P, N, S, H256> }
    };

    let arg = expand_inputs_call_arg(&function.inputs);
    let doc = util::expand_doc(&format!(
        "Calls the contract's `{}` (0x{}) function",
        function.name,
        function.selector().to_hex::<String>()
    ));
    Ok(quote! {

        #doc
        pub fn #name(&self #input) -> #result {
            self.0.method_hash(#selector, #arg)
                .expect("method not found (this should never happen)")
        }
    })
}

// converts the function params to name/type pairs
pub(crate) fn expand_inputs(inputs: &[Param]) -> Result<TokenStream> {
    let params = inputs
        .iter()
        .enumerate()
        .map(|(i, param)| {
            let name = util::expand_input_name(i, &param.name);
            let kind = types::expand(&param.kind)?;
            Ok(quote! { #name: #kind })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(quote! { #( , #params )* })
}

// packs the argument in a tuple to be used for the contract call
pub(crate) fn expand_inputs_call_arg(inputs: &[Param]) -> TokenStream {
    let names = inputs
        .iter()
        .enumerate()
        .map(|(i, param)| util::expand_input_name(i, &param.name));
    quote! { ( #( #names ,)* ) }
}

fn expand_fn_outputs(outputs: &[Param]) -> Result<TokenStream> {
    match outputs.len() {
        0 => Ok(quote! { () }),
        1 => types::expand(&outputs[0].kind),
        _ => {
            let types = outputs
                .iter()
                .map(|param| types::expand(&param.kind))
                .collect::<Result<Vec<_>>>()?;
            Ok(quote! { (#( #types ),*) })
        }
    }
}

fn expand_selector(selector: Selector) -> TokenStream {
    let bytes = selector.iter().copied().map(Literal::u8_unsuffixed);
    quote! { [#( #bytes ),*] }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethers_abi::ParamType;

    #[test]
    fn expand_inputs_empty() {
        assert_quote!(expand_inputs(&[]).unwrap().to_string(), {},);
    }

    #[test]
    fn expand_inputs_() {
        assert_quote!(
            expand_inputs(

                &[
                    Param {
                        name: "a".to_string(),
                        kind: ParamType::Bool,
                    },
                    Param {
                        name: "b".to_string(),
                        kind: ParamType::Address,
                    },
                ],
            )
            .unwrap(),
            { , a: bool, b: Address },
        );
    }

    #[test]
    fn expand_fn_outputs_empty() {
        assert_quote!(expand_fn_outputs(&[],).unwrap(), { () });
    }

    #[test]
    fn expand_fn_outputs_single() {
        assert_quote!(
            expand_fn_outputs(&[Param {
                name: "a".to_string(),
                kind: ParamType::Bool,
            }])
            .unwrap(),
            { bool },
        );
    }

    #[test]
    fn expand_fn_outputs_muliple() {
        assert_quote!(
            expand_fn_outputs(&[
                Param {
                    name: "a".to_string(),
                    kind: ParamType::Bool,
                },
                Param {
                    name: "b".to_string(),
                    kind: ParamType::Address,
                },
            ],)
            .unwrap(),
            { (bool, Address) },
        );
    }
}
