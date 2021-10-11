use std::collections::BTreeMap;

use anyhow::{Context as _, Result};
use inflector::Inflector;
use proc_macro2::{Literal, TokenStream};
use quote::quote;
use syn::Ident;

use ethers_core::abi::ParamType;
use ethers_core::{
    abi::{Function, FunctionExt, Param},
    types::Selector,
};

use super::{types, util, Context};

/// Expands a context into a method struct containing all the generated bindings
/// to the Solidity contract methods.
impl Context {
    /// Expands all method implementations
    pub(crate) fn methods(&self) -> Result<TokenStream> {
        let mut aliases = self.get_method_aliases()?;
        let sorted_functions: BTreeMap<_, _> = self.abi.functions.clone().into_iter().collect();
        let functions = sorted_functions
            .values()
            .flatten()
            .map(|function| {
                let signature = function.abi_signature();
                self.expand_function(function, aliases.remove(&signature))
                    .with_context(|| format!("error expanding function '{}'", signature))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(quote! { #( #functions )* })
    }

    fn expand_inputs_call_arg_with_structs(
        &self,
        fun: &Function,
    ) -> Result<(TokenStream, TokenStream)> {
        let mut args = Vec::with_capacity(fun.inputs.len());
        let mut call_args = Vec::with_capacity(fun.inputs.len());
        for (i, param) in fun.inputs.iter().enumerate() {
            let name = util::expand_input_name(i, &param.name);
            let ty = self.expand_input_param(fun, &param.name, &param.kind)?;
            args.push(quote! { #name: #ty });
            let call_arg = match param.kind {
                // this is awkward edge case where the function inputs are a single struct
                // we need to force this argument into a tuple so it gets expanded to `((#name,))`
                // this is currently necessary because internally `flatten_tokens` is called which
                // removes the outermost `tuple` level and since `((#name))` is not
                // a rust tuple it doesn't get wrapped into another tuple that will be peeled off by
                // `flatten_tokens`
                ParamType::Tuple(_) if fun.inputs.len() == 1 => {
                    // make sure the tuple gets converted to `Token::Tuple`
                    quote! {(#name,)}
                }
                _ => name,
            };
            call_args.push(call_arg);
        }
        let args = quote! { #( , #args )* };
        let call_args = match call_args.len() {
            0 => quote! { () },
            1 => quote! { #( #call_args )* },
            _ => quote! { ( #(#call_args, )* ) },
        };

        Ok((args, call_args))
    }

    fn expand_input_param(
        &self,
        fun: &Function,
        param: &str,
        kind: &ParamType,
    ) -> Result<TokenStream> {
        match kind {
            ParamType::Array(ty) => {
                let ty = self.expand_input_param(fun, param, ty)?;
                Ok(quote! {
                    ::std::vec::Vec<#ty>
                })
            }
            ParamType::FixedArray(ty, size) => {
                let ty = self.expand_input_param(fun, param, ty)?;
                let size = *size;
                Ok(quote! {[#ty; #size]})
            }
            ParamType::Tuple(_) => {
                let ty = if let Some(rust_struct_name) = self
                    .internal_structs
                    .get_function_input_struct_type(&fun.name, param)
                {
                    let ident = util::ident(rust_struct_name);
                    quote! {#ident}
                } else {
                    types::expand(kind)?
                };
                Ok(ty)
            }
            _ => types::expand(kind),
        }
    }

    /// Expands a single function with the given alias
    fn expand_function(&self, function: &Function, alias: Option<Ident>) -> Result<TokenStream> {
        let name = alias.unwrap_or_else(|| util::safe_ident(&function.name.to_snake_case()));
        let selector = expand_selector(function.selector());

        // TODO use structs
        let outputs = expand_fn_outputs(&function.outputs)?;

        let _ethers_core = util::ethers_core_crate();
        let _ethers_providers = util::ethers_providers_crate();
        let ethers_contract = util::ethers_contract_crate();

        let result = quote! { #ethers_contract::builders::ContractCall<M, #outputs> };

        let (input, arg) = self.expand_inputs_call_arg_with_structs(function)?;

        let doc = util::expand_doc(&format!(
            "Calls the contract's `{}` (0x{}) function",
            function.name,
            hex::encode(function.selector())
        ));
        Ok(quote! {

            #doc
            pub fn #name(&self #input) -> #result {
                self.0.method_hash(#selector, #arg)
                    .expect("method not found (this should never happen)")
            }
        })
    }

    /// Returns the method aliases, either configured by the user or determined
    /// based on overloaded functions.
    ///
    /// In case of overloaded functions we would follow rust's general
    /// convention of suffixing the function name with _with
    // The first function or the function with the least amount of arguments should
    // be named as in the ABI, the following functions suffixed with _with_ +
    // additional_params[0].name + (_and_(additional_params[1+i].name))*
    fn get_method_aliases(&self) -> Result<BTreeMap<String, Ident>> {
        let mut aliases = self.method_aliases.clone();
        // find all duplicates, where no aliases where provided
        for functions in self.abi.functions.values() {
            if functions
                .iter()
                .filter(|f| !aliases.contains_key(&f.abi_signature()))
                .count()
                <= 1
            {
                // no conflicts
                continue;
            }

            // sort functions by number of inputs asc
            let mut functions = functions.iter().collect::<Vec<_>>();
            functions.sort_by(|f1, f2| f1.inputs.len().cmp(&f2.inputs.len()));
            let prev = functions[0];
            for duplicate in functions.into_iter().skip(1) {
                // attempt to find diff in the input arguments
                let diff = duplicate
                    .inputs
                    .iter()
                    .filter(|i1| prev.inputs.iter().all(|i2| *i1 != i2))
                    .collect::<Vec<_>>();

                let alias = match diff.len() {
                    0 => {
                        // this should not happen since functions with same name and input are
                        // illegal
                        anyhow::bail!(
                            "Function with same name and parameter types defined twice: {}",
                            duplicate.name
                        );
                    }
                    1 => {
                        // single additional input params
                        format!(
                            "{}_with_{}",
                            duplicate.name.to_snake_case(),
                            diff[0].name.to_snake_case()
                        )
                    }
                    _ => {
                        // 1 + n additional input params
                        let and = diff
                            .iter()
                            .skip(1)
                            .map(|i| i.name.to_snake_case())
                            .collect::<Vec<_>>()
                            .join("_and_");
                        format!(
                            "{}_with_{}_and_{}",
                            duplicate.name.to_snake_case(),
                            diff[0].name.to_snake_case(),
                            and
                        )
                    }
                };
                aliases.insert(duplicate.abi_signature(), util::safe_ident(&alias));
            }
        }
        Ok(aliases)
    }
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
    use ethers_core::abi::ParamType;

    use super::*;

    // packs the argument in a tuple to be used for the contract call
    fn expand_inputs_call_arg(inputs: &[Param]) -> TokenStream {
        let names = inputs
            .iter()
            .enumerate()
            .map(|(i, param)| {
                let name = util::expand_input_name(i, &param.name);
                match param.kind {
                    // this is awkward edge case where the function inputs are a single struct
                    // we need to force this argument into a tuple so it gets expanded to
                    // `((#name,))` this is currently necessary because
                    // internally `flatten_tokens` is called which removes the outermost `tuple`
                    // level and since `((#name))` is not a rust tuple it
                    // doesn't get wrapped into another tuple that will be peeled off by
                    // `flatten_tokens`
                    ParamType::Tuple(_) if inputs.len() == 1 => {
                        // make sure the tuple gets converted to `Token::Tuple`
                        quote! {(#name,)}
                    }
                    _ => name,
                }
            })
            .collect::<Vec<TokenStream>>();
        match names.len() {
            0 => quote! { () },
            1 => quote! { #( #names )* },
            _ => quote! { ( #(#names, )* ) },
        }
    }

    // converts the function params to name/type pairs
    fn expand_inputs(inputs: &[Param]) -> Result<TokenStream> {
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

    #[test]
    fn test_expand_inputs_call_arg() {
        // no inputs
        let params = vec![];
        let token_stream = expand_inputs_call_arg(&params);
        assert_eq!(token_stream.to_string(), "()");

        // single input
        let params = vec![Param {
            name: "arg_a".to_string(),
            kind: ParamType::Address,
            internal_type: None,
        }];
        let token_stream = expand_inputs_call_arg(&params);
        assert_eq!(token_stream.to_string(), "arg_a");

        // two inputs
        let params = vec![
            Param {
                name: "arg_a".to_string(),
                kind: ParamType::Address,
                internal_type: None,
            },
            Param {
                name: "arg_b".to_string(),
                kind: ParamType::Uint(256usize),
                internal_type: None,
            },
        ];
        let token_stream = expand_inputs_call_arg(&params);
        assert_eq!(token_stream.to_string(), "(arg_a , arg_b ,)");

        // three inputs
        let params = vec![
            Param {
                name: "arg_a".to_string(),
                kind: ParamType::Address,
                internal_type: None,
            },
            Param {
                name: "arg_b".to_string(),
                kind: ParamType::Uint(128usize),
                internal_type: None,
            },
            Param {
                name: "arg_c".to_string(),
                kind: ParamType::Bool,
                internal_type: None,
            },
        ];
        let token_stream = expand_inputs_call_arg(&params);
        assert_eq!(token_stream.to_string(), "(arg_a , arg_b , arg_c ,)");
    }

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
                        internal_type: None,
                    },
                    Param {
                        name: "b".to_string(),
                        kind: ParamType::Address,
                        internal_type: None,
                    },
                ],
            )
            .unwrap(),
            { , a: bool, b: ethers_core::types::Address },
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
                internal_type: None,
            }])
            .unwrap(),
            { bool },
        );
    }

    #[test]
    fn expand_fn_outputs_multiple() {
        assert_quote!(
            expand_fn_outputs(&[
                Param {
                    name: "a".to_string(),
                    kind: ParamType::Bool,
                    internal_type: None,
                },
                Param {
                    name: "b".to_string(),
                    kind: ParamType::Address,
                    internal_type: None,
                },
            ],)
            .unwrap(),
            { (bool, ethers_core::types::Address) },
        );
    }
}
