use std::collections::{btree_map::Entry, BTreeMap, HashMap, HashSet};

use super::{types, util, Context};
use crate::{
    contract::common::{expand_data_struct, expand_data_tuple, expand_param_type, expand_params},
    util::can_derive_defaults,
};
use ethers_core::{
    abi::{Function, FunctionExt, Param, ParamType},
    macros::{ethers_contract_crate, ethers_core_crate},
    types::Selector,
};
use eyre::{Context as _, Result};
use inflector::Inflector;
use proc_macro2::{Literal, TokenStream};
use quote::quote;
use syn::Ident;

/// The maximum amount of overloaded functions that are attempted to auto aliased with their param
/// name. If there is a function that with `NAME_ALIASING_OVERLOADED_FUNCTIONS_CAP` overloads then
/// all functions are aliased with their index, like `log0, log1, log2,....`
const NAME_ALIASING_OVERLOADED_FUNCTIONS_CAP: usize = 3;

/// Expands a context into a method struct containing all the generated bindings
/// to the Solidity contract methods.
impl Context {
    /// Expands all method implementations
    pub(crate) fn methods_and_call_structs(&self) -> Result<(TokenStream, TokenStream)> {
        let aliases = self.get_method_aliases()?;
        let sorted_functions: BTreeMap<_, _> = self.abi.functions.iter().collect();
        let functions = sorted_functions
            .values()
            .flat_map(std::ops::Deref::deref)
            .map(|function| {
                let signature = function.abi_signature();
                self.expand_function(function, aliases.get(&signature).cloned())
                    .with_context(|| format!("error expanding function '{signature}'"))
            })
            .collect::<Result<Vec<_>>>()?;

        let function_impls = quote! { #( #functions )* };
        let call_structs = self.expand_call_structs(aliases.clone())?;
        let return_structs = self.expand_return_structs(aliases)?;

        let all_structs = quote! {
            #call_structs
            #return_structs
        };

        Ok((function_impls, all_structs))
    }

    /// Returns all deploy (constructor) implementations
    pub(crate) fn deployment_methods(&self) -> TokenStream {
        if self.contract_bytecode.is_none() {
            // don't generate deploy if no bytecode
            return quote! {}
        }
        let ethers_core = ethers_core_crate();
        let ethers_contract = ethers_contract_crate();

        let abi_name = self.inline_abi_ident();
        let get_abi = quote! {
            #abi_name.clone()
        };

        let bytecode_name = self.inline_bytecode_ident();
        let get_bytecode = quote! {
            #bytecode_name.clone().into()
        };

        let deploy = quote! {
            /// Constructs the general purpose `Deployer` instance based on the provided constructor arguments and sends it.
            /// Returns a new instance of a deployer that returns an instance of this contract after sending the transaction
            ///
            /// Notes:
            /// 1. If there are no constructor arguments, you should pass `()` as the argument.
            /// 1. The default poll duration is 7 seconds.
            /// 1. The default number of confirmations is 1 block.
            ///
            ///
            /// # Example
            ///
            /// Generate contract bindings with `abigen!` and deploy a new contract instance.
            ///
            /// *Note*: this requires a `bytecode` and `abi` object in the `greeter.json` artifact.
            ///
            /// ```ignore
            /// # async fn deploy<M: ethers::providers::Middleware>(client: ::std::sync::Arc<M>) {
            ///     abigen!(Greeter,"../greeter.json");
            ///
            ///    let greeter_contract = Greeter::deploy(client, "Hello world!".to_string()).unwrap().send().await.unwrap();
            ///    let msg = greeter_contract.greet().call().await.unwrap();
            /// # }
            /// ```
            pub fn deploy<T: #ethers_core::abi::Tokenize >(client: ::std::sync::Arc<M>, constructor_args: T) -> ::std::result::Result<#ethers_contract::builders::ContractDeployer<M, Self>, #ethers_contract::ContractError<M>> {
               let factory = #ethers_contract::ContractFactory::new(#get_abi, #get_bytecode, client);
               let deployer = factory.deploy(constructor_args)?;
               let deployer = #ethers_contract::ContractDeployer::new(deployer);
               Ok(deployer)
            }

        };

        deploy
    }

    /// Expands to the corresponding struct type based on the inputs of the given function
    fn expand_call_struct(
        &self,
        function: &Function,
        alias: Option<&MethodAlias>,
    ) -> Result<TokenStream> {
        let call_name = expand_call_struct_name(function, alias);
        let fields = self.expand_input_params(function)?;
        // expand as a tuple if all fields are anonymous
        let all_anonymous_fields = function.inputs.iter().all(|input| input.name.is_empty());
        let call_type_definition = if all_anonymous_fields {
            // expand to a tuple struct
            expand_data_tuple(&call_name, &fields)
        } else {
            // expand to a struct
            expand_data_struct(&call_name, &fields)
        };
        let function_name = &function.name;
        let abi_signature = function.abi_signature();
        let doc = format!(
            "Container type for all input parameters for the `{}` function with signature `{}` and selector `{:?}`",
            function.name,
            abi_signature,
            function.selector()
        );
        let abi_signature_doc = util::expand_doc(&doc);
        let ethers_contract = ethers_contract_crate();
        // use the same derives as for events
        let derives = util::expand_derives(&self.event_derives);

        // rust-std only derives default automatically for arrays len <= 32
        // for large array types we skip derive(Default) <https://github.com/gakonst/ethers-rs/issues/1640>
        let derive_default = if can_derive_defaults(&function.inputs) {
            quote! {
                #[derive(Default)]
            }
        } else {
            quote! {}
        };

        Ok(quote! {
            #abi_signature_doc
            #[derive(Clone, Debug, Eq, PartialEq, #ethers_contract::EthCall, #ethers_contract::EthDisplay, #derives)]
            #derive_default
            #[ethcall( name = #function_name, abi = #abi_signature )]
            pub #call_type_definition
        })
    }

    /// Expands to the corresponding struct type based on the inputs of the given function
    pub fn expand_return_struct(
        &self,
        function: &Function,
        alias: Option<&MethodAlias>,
    ) -> Result<TokenStream> {
        let struct_name = expand_return_struct_name(function, alias);
        let fields = self.expand_output_params(function)?;
        // no point in having structs when there is no data returned
        if function.outputs.is_empty() {
            return Ok(TokenStream::new())
        }
        // expand as a tuple if all fields are anonymous
        let all_anonymous_fields = function.outputs.iter().all(|output| output.name.is_empty());
        let return_type_definition = if all_anonymous_fields {
            // expand to a tuple struct
            expand_data_tuple(&struct_name, &fields)
        } else {
            // expand to a struct
            expand_data_struct(&struct_name, &fields)
        };
        let abi_signature = function.abi_signature();
        let doc = format!(
            "Container type for all return fields from the `{}` function with signature `{}` and selector `{:?}`",
            function.name,
            abi_signature,
            function.selector()
        );
        let abi_signature_doc = util::expand_doc(&doc);
        let ethers_contract = ethers_contract_crate();
        // use the same derives as for events
        let derives = util::expand_derives(&self.event_derives);

        // rust-std only derives default automatically for arrays len <= 32
        // for large array types we skip derive(Default) <https://github.com/gakonst/ethers-rs/issues/1640>
        let derive_default = if can_derive_defaults(&function.outputs) {
            quote! {
                #[derive(Default)]
            }
        } else {
            quote! {}
        };

        Ok(quote! {
            #abi_signature_doc
            #[derive(Clone, Debug,Eq, PartialEq, #ethers_contract::EthAbiType, #ethers_contract::EthAbiCodec, #derives)]
             #derive_default
            pub #return_type_definition
        })
    }

    /// Expands all call structs
    fn expand_call_structs(&self, aliases: BTreeMap<String, MethodAlias>) -> Result<TokenStream> {
        let mut struct_defs = Vec::new();
        let mut struct_names = Vec::new();
        let mut variant_names = Vec::new();
        for function in self.abi.functions.values().flatten() {
            let signature = function.abi_signature();
            let alias = aliases.get(&signature);
            struct_defs.push(self.expand_call_struct(function, alias)?);
            struct_names.push(expand_call_struct_name(function, alias));
            variant_names.push(expand_call_struct_variant_name(function, alias));
        }

        let struct_def_tokens = quote! {
            #(#struct_defs)*
        };

        if struct_defs.len() <= 1 {
            // no need for an enum
            return Ok(struct_def_tokens)
        }

        let ethers_core = ethers_core_crate();
        let ethers_contract = ethers_contract_crate();

        // use the same derives as for events
        let derives = util::expand_derives(&self.event_derives);
        let enum_name = self.expand_calls_enum_name();

        Ok(quote! {
            #struct_def_tokens

           #[derive(Debug, Clone, PartialEq, Eq, #ethers_contract::EthAbiType, #derives)]
            pub enum #enum_name {
                #(#variant_names(#struct_names)),*
            }

        impl  #ethers_core::abi::AbiDecode for #enum_name {
            fn decode(data: impl AsRef<[u8]>) -> ::std::result::Result<Self, #ethers_core::abi::AbiError> {
                 #(
                    if let Ok(decoded) = <#struct_names as #ethers_core::abi::AbiDecode>::decode(data.as_ref()) {
                        return Ok(#enum_name::#variant_names(decoded))
                    }
                )*
                Err(#ethers_core::abi::Error::InvalidData.into())
            }
        }

         impl  #ethers_core::abi::AbiEncode for #enum_name {
            fn encode(self) -> Vec<u8> {
                match self {
                    #(
                        #enum_name::#variant_names(element) => element.encode()
                    ),*
                }
            }
        }

        impl ::std::fmt::Display for #enum_name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    #(
                        #enum_name::#variant_names(element) => element.fmt(f)
                    ),*
                }
            }
        }

        #(
            impl ::std::convert::From<#struct_names> for #enum_name {
                fn from(var: #struct_names) -> Self {
                    #enum_name::#variant_names(var)
                }
            }
        )*

        })
    }

    /// Expands all return structs
    fn expand_return_structs(&self, aliases: BTreeMap<String, MethodAlias>) -> Result<TokenStream> {
        let mut struct_defs = Vec::new();
        for function in self.abi.functions.values().flatten() {
            let signature = function.abi_signature();
            let alias = aliases.get(&signature);
            struct_defs.push(self.expand_return_struct(function, alias)?);
        }

        let struct_def_tokens = quote! {
            #(#struct_defs)*
        };

        Ok(struct_def_tokens)
    }

    /// The name ident of the calls enum
    fn expand_calls_enum_name(&self) -> Ident {
        util::ident(&format!("{}Calls", self.contract_ident))
    }

    /// Expands to the `name : type` pairs of the function's inputs
    fn expand_input_params(&self, fun: &Function) -> Result<Vec<(TokenStream, TokenStream)>> {
        fun.inputs
            .iter()
            .enumerate()
            .map(|(idx, param)| {
                let name = util::expand_input_name(idx, &param.name);
                let ty = self.expand_input_param_type(fun, &param.name, &param.kind)?;
                Ok((name, ty))
            })
            .collect()
    }

    /// Expands to the `name : type` pairs of the function's outputs
    fn expand_output_params(&self, fun: &Function) -> Result<Vec<(TokenStream, TokenStream)>> {
        expand_params(&fun.outputs, |s| {
            self.internal_structs.get_function_output_struct_type(&fun.name, s)
        })
    }

    /// Expands to the return type of a function
    fn expand_outputs(&self, fun: &Function) -> Result<TokenStream> {
        let mut outputs = Vec::with_capacity(fun.outputs.len());
        for param in fun.outputs.iter() {
            let ty = self.expand_output_param_type(fun, param, &param.kind)?;
            outputs.push(ty);
        }

        let return_ty = match outputs.len() {
            0 => quote! { () },
            1 => outputs[0].clone(),
            _ => {
                quote! { (#( #outputs ),*) }
            }
        };
        Ok(return_ty)
    }

    /// Expands the arguments for the call that eventually calls the contract
    fn expand_contract_call_args(&self, fun: &Function) -> Result<TokenStream> {
        let mut call_args = Vec::with_capacity(fun.inputs.len());
        for (idx, param) in fun.inputs.iter().enumerate() {
            let name = util::expand_input_name(idx, &param.name);
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
        let call_args = match call_args.len() {
            0 => quote! { () },
            1 => quote! { #( #call_args )* },
            _ => quote! { ( #(#call_args, )* ) },
        };

        Ok(call_args)
    }

    /// returns the Tokenstream for the corresponding rust type of the param
    fn expand_input_param_type(
        &self,
        fun: &Function,
        param: &str,
        kind: &ParamType,
    ) -> Result<TokenStream> {
        let ethers_core = ethers_core_crate();
        match kind {
            ParamType::Array(ty) => {
                let ty = self.expand_input_param_type(fun, param, ty)?;
                Ok(quote! {
                    ::std::vec::Vec<#ty>
                })
            }
            ParamType::FixedArray(ty, size) => {
                let ty = match **ty {
                    ParamType::Uint(size) => {
                        if size / 8 == 1 {
                            // this prevents type ambiguity with `FixedBytes`
                            quote! { #ethers_core::types::Uint8}
                        } else {
                            self.expand_input_param_type(fun, param, ty)?
                        }
                    }
                    _ => self.expand_input_param_type(fun, param, ty)?,
                };

                let size = *size;
                Ok(quote! {[#ty; #size]})
            }
            ParamType::Tuple(_) => {
                let ty = if let Some(rust_struct_name) =
                    self.internal_structs.get_function_input_struct_type(&fun.name, param)
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

    /// returns the TokenStream for the corresponding rust type of the output param
    fn expand_output_param_type(
        &self,
        fun: &Function,
        param: &Param,
        kind: &ParamType,
    ) -> Result<TokenStream> {
        expand_param_type(param, kind, |s| {
            self.internal_structs.get_function_output_struct_type(&fun.name, s)
        })
    }

    /// Expands a single function with the given alias
    fn expand_function(
        &self,
        function: &Function,
        alias: Option<MethodAlias>,
    ) -> Result<TokenStream> {
        let ethers_contract = ethers_contract_crate();

        let name = expand_function_name(function, alias.as_ref());
        let selector = expand_selector(function.selector());

        let contract_args = self.expand_contract_call_args(function)?;
        let function_params =
            self.expand_input_params(function)?.into_iter().map(|(name, ty)| quote! { #name: #ty });
        let function_params = quote! { #( , #function_params )* };

        let outputs = self.expand_outputs(function)?;

        let result = quote! { #ethers_contract::builders::ContractCall<M, #outputs> };

        let doc = util::expand_doc(&format!(
            "Calls the contract's `{}` (0x{}) function",
            function.name,
            hex::encode(function.selector())
        ));
        Ok(quote! {

            #doc
            pub fn #name(&self #function_params) -> #result {
                self.0.method_hash(#selector, #contract_args)
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
    fn get_method_aliases(&self) -> Result<BTreeMap<String, MethodAlias>> {
        let mut aliases = self.method_aliases.clone();

        // it might be the case that there are functions with different capitalization so we sort
        // them all by lc name first
        let mut all_functions = HashMap::new();
        for function in self.abi.functions() {
            all_functions
                .entry(util::safe_snake_case_ident(&function.name))
                .or_insert_with(Vec::new)
                .push(function);
        }

        // find all duplicates, where no aliases where provided
        for functions in all_functions.values() {
            if functions.iter().filter(|f| !aliases.contains_key(&f.abi_signature())).count() <= 1 {
                // no overloads, hence no conflicts
                continue
            }

            let num_functions = functions.len();
            // sort functions by number of inputs asc
            let mut functions = functions.iter().enumerate().collect::<Vec<_>>();
            functions.sort_by(|(_, f1), (_, f2)| f1.inputs.len().cmp(&f2.inputs.len()));

            // the `functions` are now mapped with their index as defined in the ABI, but
            // we always want the zero arg function (`log()`) to be `log0`
            for (idx, (f_idx, _)) in functions.iter_mut().enumerate() {
                *f_idx = idx;
            }

            // the first function will be the function with the least amount of inputs, like log()
            // and is the baseline for the diff
            let (first_fun_idx, first_fun) = functions[0];

            // assuming here that if there is an overloaded function with nameless params like
            // `log;, log(string); log(string, string)` `log()` it should also be
            // aliased as well with its index to `log0`
            let mut needs_alias_for_first_fun_using_idx = false;

            // all the overloaded functions together with their diffs compare to the `first_fun`
            let mut diffs = Vec::new();

            /// helper function that checks if there are any conflicts due to parameter names
            fn name_conflicts(idx: usize, diffs: &[(usize, Vec<&Param>, &&Function)]) -> bool {
                let diff = &diffs.iter().find(|(i, _, _)| *i == idx).expect("diff exists").1;

                for (_, other, _) in diffs.iter().filter(|(i, _, _)| *i != idx) {
                    let (a, b) =
                        if other.len() > diff.len() { (other, diff) } else { (diff, other) };

                    if a.iter()
                        .all(|d| b.iter().any(|o| o.name.to_snake_case() == d.name.to_snake_case()))
                    {
                        return true
                    }
                }
                false
            }
            // compare each overloaded function with the `first_fun`
            for (idx, overloaded_fun) in functions.into_iter().skip(1) {
                // keep track of matched params
                let mut already_matched_param_diff = HashSet::new();
                // attempt to find diff in the input arguments
                let mut diff = Vec::new();
                let mut same_params = true;
                for (idx, i1) in overloaded_fun.inputs.iter().enumerate() {
                    // Find the first param that differs and hasn't already been matched as diff
                    if let Some((pos, _)) = first_fun
                        .inputs
                        .iter()
                        .enumerate()
                        .filter(|(pos, _)| !already_matched_param_diff.contains(pos))
                        .find(|(_, i2)| i1 != *i2)
                    {
                        already_matched_param_diff.insert(pos);
                        diff.push(i1);
                        same_params = false;
                    } else {
                        // check for cases like `log(string); log(string, string)` by keep track of
                        // same order
                        if same_params && idx + 1 > first_fun.inputs.len() {
                            diff.push(i1);
                        }
                    }
                }
                diffs.push((idx, diff, overloaded_fun));
            }

            for (idx, diff, overloaded_fun) in &diffs {
                let alias = match diff.len() {
                    0 => {
                        // this may happen if there are functions with different casing,
                        // like `INDEX`and `index`

                        // this should not happen since functions with same
                        // name and inputs are illegal
                        eyre::ensure!(
                            overloaded_fun.name != first_fun.name,
                            "Function with same name and parameter types defined twice: {}",
                            overloaded_fun.name
                        );

                        let overloaded_id = overloaded_fun.name.to_snake_case();
                        let first_fun_id = first_fun.name.to_snake_case();
                        if first_fun_id != overloaded_id {
                            // no conflict
                            overloaded_id
                        } else {
                            let overloaded_alias = MethodAlias {
                                function_name: util::safe_ident(&overloaded_fun.name),
                                struct_name: util::safe_ident(&overloaded_fun.name),
                            };
                            aliases.insert(overloaded_fun.abi_signature(), overloaded_alias);

                            let first_fun_alias = MethodAlias {
                                function_name: util::safe_ident(&first_fun.name),
                                struct_name: util::safe_ident(&first_fun.name),
                            };
                            aliases.insert(first_fun.abi_signature(), first_fun_alias);
                            continue
                        }
                    }
                    1 => {
                        // single additional input params
                        if diff[0].name.is_empty() ||
                            num_functions > NAME_ALIASING_OVERLOADED_FUNCTIONS_CAP ||
                            name_conflicts(*idx, &diffs)
                        {
                            needs_alias_for_first_fun_using_idx = true;
                            format!("{}{idx}", overloaded_fun.name.to_snake_case())
                        } else {
                            format!(
                                "{}_with_{}",
                                overloaded_fun.name.to_snake_case(),
                                diff[0].name.to_snake_case()
                            )
                        }
                    }
                    _ => {
                        if diff.iter().any(|d| d.name.is_empty()) ||
                            num_functions > NAME_ALIASING_OVERLOADED_FUNCTIONS_CAP ||
                            name_conflicts(*idx, &diffs)
                        {
                            needs_alias_for_first_fun_using_idx = true;
                            format!("{}{idx}", overloaded_fun.name.to_snake_case())
                        } else {
                            // 1 + n additional input params
                            let and = diff
                                .iter()
                                .skip(1)
                                .map(|i| i.name.to_snake_case())
                                .collect::<Vec<_>>()
                                .join("_and_");
                            format!(
                                "{}_with_{}_and_{}",
                                overloaded_fun.name.to_snake_case(),
                                diff[0].name.to_snake_case(),
                                and
                            )
                        }
                    }
                };
                let alias = MethodAlias::new(&alias);
                aliases.insert(overloaded_fun.abi_signature(), alias);
            }

            if needs_alias_for_first_fun_using_idx {
                // insert an alias for the root duplicated call
                let prev_alias = format!("{}{first_fun_idx}", first_fun.name.to_snake_case());

                let alias = MethodAlias::new(&prev_alias);

                aliases.insert(first_fun.abi_signature(), alias);
            }
        }

        // we have to handle the edge cases with underscore prefix and suffix that would get
        // stripped by Inflector::to_snake_case/pascalCase if there is another function that
        // would collide we manually add an alias for it eg. abi = ["_a(), a(), a_(),
        // _a_()"] will generate identical rust functions
        for (name, functions) in self.abi.functions.iter() {
            if name.starts_with('_') || name.ends_with('_') {
                let ident = name.trim_matches('_').trim_end_matches('_');
                // check for possible collisions after Inflector would remove the underscores
                if self.abi.functions.contains_key(ident) {
                    for function in functions {
                        if let Entry::Vacant(entry) = aliases.entry(function.abi_signature()) {
                            // use the full name as alias
                            entry.insert(MethodAlias::new(name.as_str()));
                        }
                    }
                }
            }
        }
        Ok(aliases)
    }
}

fn expand_selector(selector: Selector) -> TokenStream {
    let bytes = selector.iter().copied().map(Literal::u8_unsuffixed);
    quote! { [#( #bytes ),*] }
}

/// Represents the aliases to use when generating method related elements
#[derive(Debug, Clone)]
pub struct MethodAlias {
    pub function_name: Ident,
    pub struct_name: Ident,
}

impl MethodAlias {
    pub fn new(alias: &str) -> Self {
        MethodAlias {
            function_name: util::safe_snake_case_ident(alias),
            struct_name: util::safe_pascal_case_ident(alias),
        }
    }
}

fn expand_function_name(function: &Function, alias: Option<&MethodAlias>) -> Ident {
    if let Some(alias) = alias {
        alias.function_name.clone()
    } else {
        util::safe_ident(&util::safe_snake_case(&function.name))
    }
}

/// Expands the name of a struct by a postfix
fn expand_struct_name_postfix(
    function: &Function,
    alias: Option<&MethodAlias>,
    postfix: &str,
) -> Ident {
    let name = if let Some(alias) = alias {
        format!("{}{postfix}", alias.struct_name)
    } else {
        format!("{}{postfix}", util::safe_pascal_case(&function.name))
    };
    util::ident(&name)
}

/// Expands to the name of the call struct
fn expand_call_struct_name(function: &Function, alias: Option<&MethodAlias>) -> Ident {
    expand_struct_name_postfix(function, alias, "Call")
}

/// Expands to the name of the return struct
fn expand_return_struct_name(function: &Function, alias: Option<&MethodAlias>) -> Ident {
    expand_struct_name_postfix(function, alias, "Return")
}

/// Expands to the name of the call struct
fn expand_call_struct_variant_name(function: &Function, alias: Option<&MethodAlias>) -> Ident {
    if let Some(alias) = alias {
        alias.struct_name.clone()
    } else {
        util::safe_ident(&util::safe_pascal_case(&function.name))
    }
}

#[cfg(test)]
mod tests {
    use ethers_core::abi::ParamType;

    use super::*;

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
            Param { name: "arg_a".to_string(), kind: ParamType::Address, internal_type: None },
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
            Param { name: "arg_a".to_string(), kind: ParamType::Address, internal_type: None },
            Param {
                name: "arg_b".to_string(),
                kind: ParamType::Uint(128usize),
                internal_type: None,
            },
            Param { name: "arg_c".to_string(), kind: ParamType::Bool, internal_type: None },
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
                Param { name: "a".to_string(), kind: ParamType::Bool, internal_type: None },
                Param { name: "b".to_string(), kind: ParamType::Address, internal_type: None },
            ],)
            .unwrap(),
            { (bool, ethers_core::types::Address) },
        );
    }
}
