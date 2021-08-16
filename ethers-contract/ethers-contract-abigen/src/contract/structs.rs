//! Methods for expanding structs
use std::collections::{HashMap, VecDeque};

use anyhow::{Context as _, Result};
use inflector::Inflector;
use proc_macro2::{Literal, TokenStream};
use quote::quote;

use ethers_core::abi::{
    param_type::Reader,
    struct_def::{FieldDeclaration, FieldType, StructFieldType, StructType},
    ParamType, SolStruct,
};

use crate::contract::{types, Context};
use crate::rawabi::{Component, RawAbi};
use crate::util;
use std::any::Any;

impl Context {
    /// Generate corresponding types for structs parsed from a human readable ABI
    ///
    /// NOTE: This assumes that all structs that are potentially used as type for variable are
    /// in fact present in the `AbiParser`, this is sound because `AbiParser::parse` would have
    /// failed already
    pub fn abi_structs(&self) -> Result<TokenStream> {
        if self.human_readable {
            self.gen_human_readable_structs()
        } else {
            self.gen_internal_structs()
        }
    }

    /// Returns the `TokenStream` with all the internal structs extracted form the JSON ABI
    fn gen_internal_structs(&self) -> Result<TokenStream> {
        let mut structs = TokenStream::new();
        let mut ids: Vec<_> = self.internal_structs.structs.keys().collect();
        ids.sort();

        for id in ids {
            let sol_struct = &self.internal_structs.structs[id];
            let struct_name = self
                .internal_structs
                .rust_type_names
                .get(id)
                .context(format!("No types found for {}", id))?;
            let tuple = self
                .internal_structs
                .struct_tuples
                .get(id)
                .context(format!("No types found for {}", id))?
                .clone();
            structs.extend(self.expand_internal_struct(struct_name, sol_struct, tuple)?);
        }
        Ok(structs)
    }

    /// Expand all structs parsed from the internal types of the JSON ABI
    fn expand_internal_struct(
        &self,
        name: &str,
        sol_struct: &SolStruct,
        tuple: ParamType,
    ) -> Result<TokenStream> {
        let mut fields = Vec::with_capacity(sol_struct.fields().len());
        for field in sol_struct.fields() {
            let field_name = util::ident(&field.name().to_snake_case());
            match field.r#type() {
                FieldType::Elementary(ty) => {
                    let ty = types::expand(ty)?;
                    fields.push(quote! { pub #field_name: #ty });
                }
                FieldType::Struct(struct_ty) => {
                    let ty = expand_struct_type(struct_ty);
                    fields.push(quote! { pub #field_name: #ty });
                }
                FieldType::Mapping(_) => {
                    return Err(anyhow::anyhow!(
                        "Mapping types in struct `{}` are not supported {:?}",
                        name,
                        field
                    ));
                }
            }
        }

        let sig = if let ParamType::Tuple(ref tokens) = tuple {
            tokens
                .iter()
                .map(|kind| kind.to_string())
                .collect::<Vec<_>>()
                .join(",")
        } else {
            "".to_string()
        };

        let abi_signature = format!("{}({})", name, sig,);

        let abi_signature_doc = util::expand_doc(&format!("`{}`", abi_signature));

        let name = util::ident(name);

        // use the same derives as for events
        let derives = &self.event_derives;
        let derives = quote! {#(#derives),*};

        Ok(quote! {
            #abi_signature_doc
            #[derive(Clone, Debug, Default, Eq, PartialEq, ethers::contract::EthAbiType, #derives)]
            pub struct #name {
                #( #fields ),*
            }
        })
    }

    /// Expand all structs parsed from the human readable ABI
    fn gen_human_readable_structs(&self) -> Result<TokenStream> {
        let mut structs = TokenStream::new();
        let mut names: Vec<_> = self.abi_parser.structs.keys().collect();
        names.sort();
        for name in names {
            let sol_struct = &self.abi_parser.structs[name];
            let mut fields = Vec::with_capacity(sol_struct.fields().len());
            let mut param_types = Vec::with_capacity(sol_struct.fields().len());
            for field in sol_struct.fields() {
                let field_name = util::ident(&field.name().to_snake_case());
                match field.r#type() {
                    FieldType::Elementary(ty) => {
                        param_types.push(ty.clone());
                        let ty = types::expand(ty)?;
                        fields.push(quote! { pub #field_name: #ty });
                    }
                    FieldType::Struct(struct_ty) => {
                        let ty = expand_struct_type(struct_ty);
                        fields.push(quote! { pub #field_name: #ty });

                        let name = struct_ty.name();
                        let tuple = self
                            .abi_parser
                            .struct_tuples
                            .get(name)
                            .context(format!("No types found for {}", name))?
                            .clone();
                        let tuple = ParamType::Tuple(tuple);

                        param_types.push(struct_ty.as_param(tuple));
                    }
                    FieldType::Mapping(_) => {
                        return Err(anyhow::anyhow!(
                            "Mapping types in struct `{}` are not supported {:?}",
                            name,
                            field
                        ));
                    }
                }
            }

            let abi_signature = format!(
                "{}({})",
                name,
                param_types
                    .iter()
                    .map(|kind| kind.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            );

            let abi_signature_doc = util::expand_doc(&format!("`{}`", abi_signature));

            let name = util::ident(name);

            // use the same derives as for events
            let derives = &self.event_derives;
            let derives = quote! {#(#derives),*};

            structs.extend(quote! {
            #abi_signature_doc
            #[derive(Clone, Debug, Default, Eq, PartialEq, ethers::contract::EthAbiType, #derives)]
            pub struct #name {
                #( #fields ),*
            }
        });
        }
        Ok(structs)
    }
}

/// Helper to match `ethabi::Param`s with structs and nested structs
///
/// This is currently used to get access to all the unique solidity structs used as function in/output until `ethabi` supports it as well.
#[derive(Debug, Clone, Default)]
pub struct InternalStructs {
    /// All unique internal types that are function inputs or outputs
    top_level_internal_types: HashMap<String, Component>,

    /// (function name, param name) -> struct which are the identifying properties we get the name from ethabi.
    function_params: HashMap<(String, String), String>,

    /// (function name) -> Vec<structs> all structs the function returns
    outputs: HashMap<String, Vec<String>>,

    /// All the structs extracted from the abi with their identifier as key
    structs: HashMap<String, SolStruct>,

    /// solidity structs as tuples
    struct_tuples: HashMap<String, ParamType>,

    /// Contains the names for the rust types (id -> rust type name)
    rust_type_names: HashMap<String, String>,
}

impl InternalStructs {
    pub fn new(abi: RawAbi) -> Self {
        let mut top_level_internal_types = HashMap::new();
        let mut function_params = HashMap::new();
        let mut outputs = HashMap::new();
        let mut structs = HashMap::new();
        for item in abi
            .into_iter()
            .filter(|item| item.type_field == "constructor" || item.type_field == "function")
        {
            if let Some(name) = item.name {
                for input in item.inputs {
                    if let Some(ty) = input
                        .internal_type
                        .as_deref()
                        .filter(|ty| ty.starts_with("struct "))
                        .map(struct_type_identifier)
                    {
                        function_params.insert((name.clone(), input.name.clone()), ty.to_string());
                        top_level_internal_types.insert(ty.to_string(), input);
                    }
                }
                let mut output_structs = Vec::new();
                for output in item.outputs {
                    if let Some(ty) = output
                        .internal_type
                        .as_deref()
                        .filter(|ty| ty.starts_with("struct "))
                        .map(struct_type_identifier)
                    {
                        output_structs.push(ty.to_string());
                        top_level_internal_types.insert(ty.to_string(), output);
                    }
                }
                outputs.insert(name, output_structs);
            }
        }

        // turn each top level internal type (function input/output) and their nested types
        // into a struct will create all structs
        for component in top_level_internal_types.values() {
            insert_structs(&mut structs, component);
        }

        // determine the `ParamType` representation of each struct
        let struct_tuples = resolve_struct_tuples(&structs);

        // name -> (id, projections)
        let mut type_names: HashMap<String, (String, Vec<String>)> =
            HashMap::with_capacity(structs.len());
        for id in structs.keys() {
            let name = struct_type_name(id).to_pascal_case();
            let projections = struct_type_projections(id);
            insert_rust_type_name(&mut type_names, name, projections, id.clone());
        }

        Self {
            top_level_internal_types,
            function_params,
            outputs,
            structs,
            struct_tuples,
            rust_type_names: type_names
                .into_iter()
                .map(|(rust_name, (id, _))| (id, rust_name))
                .collect(),
        }
    }

    /// Returns the name of the rust type that will be generated if the given input is a struct
    /// NOTE: this does not account for arrays or fixed arrays
    pub fn get_function_input_struct_type(&self, function: &str, input: &str) -> Option<&str> {
        let key = (function.to_string(), input.to_string());
        self.function_params
            .get(&key)
            .and_then(|id| self.rust_type_names.get(id))
            .map(String::as_str)
    }
}

/// This will determine the name of the rust type and will make sure that possible collisions are resolved by adjusting the actual Rust name of the structure, e.g. `LibraryA.Point` and `LibraryB.Point` to `LibraryAPoint` and `LibraryBPoint`.
fn insert_rust_type_name(
    type_names: &mut HashMap<String, (String, Vec<String>)>,
    mut name: String,
    mut projections: Vec<String>,
    id: String,
) {
    if let Some((other_id, mut other_projections)) = type_names.remove(&name) {
        let mut other_name = name.clone();
        // name collision `A.name` `B.name`, rename to `AName`, `BName`
        if !other_projections.is_empty() {
            other_name = format!(
                "{}{}",
                other_projections.remove(0).to_pascal_case(),
                other_name
            );
        }
        insert_rust_type_name(type_names, other_name, other_projections, other_id);

        if !projections.is_empty() {
            name = format!("{}{}", projections.remove(0).to_pascal_case(), name);
        }
        insert_rust_type_name(type_names, name, projections, id);
    } else {
        type_names.insert(name, (id, projections));
    }
}

/// Tries to determine the `ParamType::Tuple` for every struct.
///
/// If a structure has nested structures, these must be determined first, essentially starting with structures consisting of only elementary types before moving on to higher level structures, for example `Proof {point: Point}, Point {x:int, y:int}` start by converting Point into a tuple of `x` and `y` and then substituting `point` with this within `Proof`.
fn resolve_struct_tuples(all_structs: &HashMap<String, SolStruct>) -> HashMap<String, ParamType> {
    let mut params = HashMap::new();
    let mut structs: VecDeque<_> = all_structs.iter().collect();

    // keep track of how often we retried nested structs
    let mut sequential_retries = 0;
    'outer: while let Some((id, ty)) = structs.pop_front() {
        if sequential_retries > structs.len() {
            break;
        }
        if let Some(tuple) = ty.as_tuple() {
            params.insert(id.to_string(), tuple);
        } else {
            // try to substitute all nested struct types with their `ParamTypes`
            let mut struct_params = Vec::with_capacity(ty.fields.len());
            for field in ty.fields() {
                match field.ty {
                    FieldType::Elementary(ref param) => {
                        struct_params.push(param.clone());
                    }
                    FieldType::Struct(ref field_ty) => {
                        // nested struct
                        let ty_id = field_ty.identifier();
                        if let Some(nested) = params.get(&ty_id).cloned() {
                            match field_ty {
                                StructFieldType::Type(_) => struct_params.push(nested),
                                StructFieldType::Array(_) => {
                                    struct_params.push(ParamType::Array(Box::new(nested)));
                                }
                                StructFieldType::FixedArray(_, size) => {
                                    struct_params
                                        .push(ParamType::FixedArray(Box::new(nested), *size));
                                }
                            }
                        } else {
                            // struct field needs to be resolved first
                            structs.push_back((id, ty));
                            sequential_retries += 1;
                            continue 'outer;
                        }
                    }
                    _ => {
                        unreachable!("mapping types are unsupported")
                    }
                }
            }
            params.insert(id.to_string(), ParamType::Tuple(struct_params));
        }

        // we resolved a new param, so we can try all again
        sequential_retries = 0;
    }
    params
}

/// turns the tuple component into a struct if it's still missing in the map, including all inner structs
fn insert_structs(structs: &mut HashMap<String, SolStruct>, tuple: &Component) {
    if let Some(internal_ty) = tuple.internal_type.as_ref() {
        let ident = struct_type_identifier(internal_ty);
        if structs.contains_key(ident) {
            return;
        }
        if let Some(fields) = tuple
            .components
            .iter()
            .map(|f| {
                Reader::read(&f.type_field)
                    .ok()
                    .and_then(|kind| field(structs, f, kind))
            })
            .collect::<Option<Vec<_>>>()
        {
            let s = SolStruct {
                name: ident.to_string(),
                fields,
            };
            structs.insert(ident.to_string(), s);
        }
    }
}

/// Determines the type of the field component
fn field(
    structs: &mut HashMap<String, SolStruct>,
    field_component: &Component,
    kind: ParamType,
) -> Option<FieldDeclaration> {
    match kind {
        ParamType::Array(ty) => {
            let FieldDeclaration { ty, .. } = field(structs, field_component, *ty)?;
            match ty {
                FieldType::Elementary(kind) => {
                    // this arm represents all the elementary types like address, uint...
                    Some(FieldDeclaration::new(
                        field_component.name.clone(),
                        FieldType::Elementary(ParamType::Array(Box::new(kind))),
                    ))
                }
                FieldType::Struct(ty) => Some(FieldDeclaration::new(
                    field_component.name.clone(),
                    FieldType::Struct(StructFieldType::Array(Box::new(ty))),
                )),
                _ => {
                    unreachable!("no mappings types support as function inputs or outputs")
                }
            }
        }
        ParamType::FixedArray(ty, size) => {
            let FieldDeclaration { ty, .. } = field(structs, field_component, *ty)?;
            match ty {
                FieldType::Elementary(kind) => {
                    // this arm represents all the elementary types like address, uint...
                    Some(FieldDeclaration::new(
                        field_component.name.clone(),
                        FieldType::Elementary(ParamType::FixedArray(Box::new(kind), size)),
                    ))
                }
                FieldType::Struct(ty) => Some(FieldDeclaration::new(
                    field_component.name.clone(),
                    FieldType::Struct(StructFieldType::FixedArray(Box::new(ty), size)),
                )),
                _ => {
                    unreachable!("no mappings types support as function inputs or outputs")
                }
            }
        }
        ParamType::Tuple(_) => {
            insert_structs(structs, field_component);
            let internal_type = field_component.internal_type.as_ref()?;
            let ty = struct_type_identifier(internal_type);
            // split the identifier into the name and all projections:
            // `A.B.C.name` -> name, [A,B,C]
            let mut idents = ty.rsplit('.');
            let name = idents.next().unwrap().to_string();
            let projections = idents.rev().map(str::to_string).collect();
            Some(FieldDeclaration::new(
                field_component.name.clone(),
                FieldType::Struct(StructFieldType::Type(StructType::new(name, projections))),
            ))
        }
        elementary => Some(FieldDeclaration::new(
            field_component.name.clone(),
            FieldType::Elementary(elementary),
        )),
    }
}

/// `struct Pairing.G2Point[]` -> `G2Point`
fn struct_type_name(name: &str) -> &str {
    struct_type_identifier(name).rsplit('.').next().unwrap()
}

/// `Pairing.G2Point` -> `Pairing.G2Point`
fn struct_type_identifier(name: &str) -> &str {
    name.trim_start_matches("struct ")
        .split('[')
        .next()
        .unwrap()
}

/// `struct Pairing.Nested.G2Point[]` -> `[Pairing, Nested]`
fn struct_type_projections(name: &str) -> Vec<String> {
    let id = struct_type_identifier(name);
    let mut iter = id.rsplit('.');
    iter.next();
    iter.rev().map(str::to_string).collect()
}

/// Expands to the rust struct type
fn expand_struct_type(struct_ty: &StructFieldType) -> TokenStream {
    match struct_ty {
        StructFieldType::Type(ty) => {
            let ty = util::ident(ty.name());
            quote! {#ty}
        }
        StructFieldType::Array(ty) => {
            let ty = expand_struct_type(&*ty);
            quote! {::std::vec::Vec<#ty>}
        }
        StructFieldType::FixedArray(ty, size) => {
            let ty = expand_struct_type(&*ty);
            quote! { [#ty; #size]}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn can_determine_structs() {
        const VERIFIER_ABI: &str =
            include_str!("../../../tests/solidity-contracts/verifier_abi.json");
        let abi = serde_json::from_str::<RawAbi>(VERIFIER_ABI).unwrap();

        let internal = InternalStructs::new(abi);
        dbg!(internal.rust_type_names);
    }
}
