//! Structs expansion

use super::{types, Context};
use crate::util;
use ethers_core::{
    abi::{
        struct_def::{FieldDeclaration, FieldType, StructFieldType, StructType},
        Component, HumanReadableParser, ParamType, RawAbi, SolStruct,
    },
    macros::ethers_contract_crate,
};
use eyre::{eyre, Result};
use inflector::Inflector;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::{HashMap, VecDeque};
use syn::Ident;

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

    /// In the event of type conflicts this allows for removing a specific struct type.
    pub fn remove_struct(&mut self, name: &str) {
        if self.human_readable {
            self.abi_parser.structs.remove(name);
        } else {
            self.internal_structs.structs.remove(name);
        }
    }

    /// Returns the type definition for the struct with the given name
    pub fn struct_definition(&mut self, name: &str) -> Result<TokenStream> {
        if self.human_readable {
            self.generate_human_readable_struct(name)
        } else {
            self.generate_internal_struct(name)
        }
    }

    /// Generates the type definition for the name that matches the given identifier
    fn generate_internal_struct(&self, id: &str) -> Result<TokenStream> {
        let sol_struct =
            self.internal_structs.structs.get(id).ok_or_else(|| eyre!("Struct not found"))?;
        let struct_name = self
            .internal_structs
            .rust_type_names
            .get(id)
            .ok_or_else(|| eyre!("No types found for {id}"))?;
        let tuple = self
            .internal_structs
            .struct_tuples
            .get(id)
            .ok_or_else(|| eyre!("No types found for {id}"))?;
        let types = if let ParamType::Tuple(types) = tuple { types } else { unreachable!() };
        self.expand_internal_struct(struct_name, sol_struct, types)
    }

    /// Returns the `TokenStream` with all the internal structs extracted form the JSON ABI
    fn gen_internal_structs(&self) -> Result<TokenStream> {
        let mut structs = TokenStream::new();
        let mut ids: Vec<_> = self.internal_structs.structs.keys().collect();
        ids.sort();

        for id in ids {
            structs.extend(self.generate_internal_struct(id)?);
        }
        Ok(structs)
    }

    /// Expand all structs parsed from the internal types of the JSON ABI
    fn expand_internal_struct(
        &self,
        name: &str,
        sol_struct: &SolStruct,
        types: &[ParamType],
    ) -> Result<TokenStream> {
        let mut fields = Vec::with_capacity(sol_struct.fields().len());

        // determines whether we have enough info to create named fields
        let is_tuple = sol_struct.has_nameless_field();

        for field in sol_struct.fields() {
            let ty = match field.r#type() {
                FieldType::Elementary(ty) => types::expand(ty)?,
                FieldType::Struct(struct_ty) => types::expand_struct_type(struct_ty),
                FieldType::Mapping(_) => {
                    eyre::bail!("Mapping types in struct `{name}` are not supported")
                }
            };

            let field_name = if is_tuple {
                TokenStream::new()
            } else {
                let field_name = util::safe_ident(&field.name().to_snake_case());
                quote!(#field_name)
            };
            fields.push((field_name, ty));
        }

        let name = util::ident(name);

        let struct_def = expand_struct(&name, &fields, is_tuple);

        let sig = util::abi_signature_types(types);
        let doc_str = format!("`{name}({sig})`");

        let mut derives = self.expand_extra_derives();
        util::derive_builtin_traits_struct(&self.internal_structs, sol_struct, types, &mut derives);

        let ethers_contract = ethers_contract_crate();

        Ok(quote! {
            #[doc = #doc_str]
            #[derive(Clone, #ethers_contract::EthAbiType, #ethers_contract::EthAbiCodec, #derives)]
            pub #struct_def
        })
    }

    fn generate_human_readable_struct(&self, name: &str) -> Result<TokenStream> {
        let sol_struct =
            self.abi_parser.structs.get(name).ok_or_else(|| eyre!("Struct `{name}` not found"))?;
        let mut fields = Vec::with_capacity(sol_struct.fields().len());
        let mut param_types = Vec::with_capacity(sol_struct.fields().len());
        for field in sol_struct.fields() {
            let field_name = util::safe_ident(&field.name().to_snake_case());
            match field.r#type() {
                FieldType::Elementary(ty) => {
                    param_types.push(ty.clone());
                    let ty = types::expand(ty)?;
                    fields.push(quote! { pub #field_name: #ty });
                }
                FieldType::Struct(struct_ty) => {
                    let ty = types::expand_struct_type(struct_ty);
                    fields.push(quote! { pub #field_name: #ty });

                    let name = struct_ty.name();
                    let tuple = self
                        .abi_parser
                        .struct_tuples
                        .get(name)
                        .ok_or_else(|| eyre!("No types found for {name}"))?
                        .clone();
                    let tuple = ParamType::Tuple(tuple);

                    param_types.push(struct_ty.as_param(tuple));
                }
                FieldType::Mapping(_) => {
                    eyre::bail!("Mapping types in struct `{name}` are not supported")
                }
            }
        }

        let abi_signature = util::abi_signature(name, &param_types);

        let name = util::ident(name);

        let mut derives = self.expand_extra_derives();
        util::derive_builtin_traits(&param_types, &mut derives, true, true);

        let ethers_contract = ethers_contract_crate();

        Ok(quote! {
            #[doc = #abi_signature]
            #[derive(Clone, #ethers_contract::EthAbiType, #ethers_contract::EthAbiCodec, #derives)]
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
            structs.extend(self.generate_human_readable_struct(name)?);
        }
        Ok(structs)
    }
}

/// Helper to match `ethabi::Param`s with structs and nested structs
///
/// This is currently used to get access to all the unique solidity structs used as function
/// in/output until `ethabi` supports it as well.
#[derive(Debug, Clone, Default)]
pub struct InternalStructs {
    /// (function name, param name) -> struct which are the identifying properties we get the name
    /// from ethabi.
    pub(crate) function_params: HashMap<(String, String), String>,

    /// (function name) -> `Vec<structs>` all structs the function returns
    pub(crate) outputs: HashMap<String, Vec<String>>,

    /// (event name, idx) -> struct which are the identifying properties we get the name
    /// from ethabi.
    ///
    /// Note: we need to map the index of the event here because events can contain nameless inputs
    pub(crate) event_params: HashMap<(String, usize), String>,

    /// All the structs extracted from the abi with their identifier as key
    pub(crate) structs: HashMap<String, SolStruct>,

    /// solidity structs as tuples
    pub(crate) struct_tuples: HashMap<String, ParamType>,

    /// Contains the names for the rust types (id -> rust type name)
    pub(crate) rust_type_names: HashMap<String, String>,
}

impl InternalStructs {
    /// Creates a new instance with a filled type mapping table based on the abi
    pub fn new(abi: RawAbi) -> Self {
        let mut top_level_internal_types = HashMap::new();
        let mut function_params = HashMap::new();
        let mut outputs = HashMap::new();
        let mut event_params = HashMap::new();

        for item in abi
            .into_iter()
            .filter(|item| matches!(item.type_field.as_str(), "constructor" | "function" | "event"))
        {
            let is_event = item.type_field == "event";

            if let Some(name) = item.name {
                for (idx, input) in item.inputs.into_iter().enumerate() {
                    if let Some(ty) = input
                        .internal_type
                        .as_deref()
                        .filter(|ty| ty.starts_with("struct "))
                        .map(struct_type_identifier)
                    {
                        if is_event {
                            event_params.insert((name.clone(), idx), ty.to_string());
                        } else {
                            function_params
                                .insert((name.clone(), input.name.clone()), ty.to_string());
                        }
                        top_level_internal_types.insert(ty.to_string(), input);
                    }
                }

                if is_event {
                    // no outputs in an event
                    continue
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
        let mut structs = HashMap::new();
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
            function_params,
            outputs,
            structs,
            event_params,
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
        self.get_function_input_struct_solidity_id(function, input)
            .and_then(|id| self.rust_type_names.get(id))
            .map(String::as_str)
    }

    /// Returns solidity type identifier as it's used in the ABI.
    pub fn get_function_input_struct_solidity_id(
        &self,
        function: &str,
        input: &str,
    ) -> Option<&str> {
        let key = (function.to_string(), input.to_string());
        self.function_params.get(&key).map(String::as_str)
    }

    /// Returns the name of the rust type that will be generated if the given input is a struct
    /// This takes the index of event's parameter instead of the parameter's name like
    /// [`Self::get_function_input_struct_type`] does because we can't rely on the name since events
    /// support nameless parameters NOTE: this does not account for arrays or fixed arrays
    pub fn get_event_input_struct_type(&self, event: &str, idx: usize) -> Option<&str> {
        self.get_event_input_struct_solidity_id(event, idx)
            .and_then(|id| self.rust_type_names.get(id))
            .map(String::as_str)
    }

    /// Returns the type identifier as it's used in the solidity ABI
    pub fn get_event_input_struct_solidity_id(&self, event: &str, idx: usize) -> Option<&str> {
        let key = (event.to_string(), idx);
        self.event_params.get(&key).map(String::as_str)
    }

    /// Returns the name of the rust type that will be generated if the given output is a struct
    /// NOTE: this does not account for arrays or fixed arrays
    pub fn get_function_output_struct_type(
        &self,
        function: &str,
        internal_type: &str,
    ) -> Option<&str> {
        self.get_function_output_struct_solidity_id(function, internal_type)
            .and_then(|id| self.rust_type_names.get(id))
            .map(String::as_str)
    }

    /// Returns the name of the rust type that will be generated if the given output is a struct
    /// NOTE: this does not account for arrays or fixed arrays
    pub fn get_function_output_struct_solidity_id(
        &self,
        function: &str,
        internal_type: &str,
    ) -> Option<&str> {
        self.outputs
            .get(function)
            .and_then(|outputs| {
                outputs.iter().find(|s| s.as_str() == struct_type_identifier(internal_type))
            })
            .map(String::as_str)
    }

    /// Returns the name of the rust type for the type
    pub fn get_struct_type(&self, internal_type: &str) -> Option<&str> {
        self.rust_type_names.get(struct_type_identifier(internal_type)).map(String::as_str)
    }

    /// Returns the mapping table of abi `internal type identifier -> rust type`
    pub fn rust_type_names(&self) -> &HashMap<String, String> {
        &self.rust_type_names
    }

    /// Returns all the solidity struct types
    ///
    /// These are grouped by their case-sensitive type identifiers extracted from the ABI.
    pub fn structs_types(&self) -> &HashMap<String, SolStruct> {
        &self.structs
    }
}

/// This will determine the name of the rust type and will make sure that possible collisions are
/// resolved by adjusting the actual Rust name of the structure, e.g. `LibraryA.Point` and
/// `LibraryB.Point` to `LibraryAPoint` and `LibraryBPoint`.
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
            other_name = format!("{}{other_name}", other_projections.remove(0).to_pascal_case());
        }
        insert_rust_type_name(type_names, other_name, other_projections, other_id);

        if !projections.is_empty() {
            name = format!("{}{name}", projections.remove(0).to_pascal_case());
        }
        insert_rust_type_name(type_names, name, projections, id);
    } else {
        type_names.insert(name, (id, projections));
    }
}

/// Tries to determine the `ParamType::Tuple` for every struct.
///
/// If a structure has nested structures, these must be determined first, essentially starting with
/// structures consisting of only elementary types before moving on to higher level structures, for
/// example `Proof {point: Point}, Point {x:int, y:int}` start by converting Point into a tuple of
/// `x` and `y` and then substituting `point` with this within `Proof`.
fn resolve_struct_tuples(all_structs: &HashMap<String, SolStruct>) -> HashMap<String, ParamType> {
    let mut params = HashMap::new();
    let mut structs: VecDeque<_> = all_structs.iter().collect();

    // keep track of how often we retried nested structs
    let mut sequential_retries = 0;
    'outer: while let Some((id, ty)) = structs.pop_front() {
        if sequential_retries > structs.len() {
            break
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
                            continue 'outer
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

/// turns the tuple component into a struct if it's still missing in the map, including all inner
/// structs
fn insert_structs(structs: &mut HashMap<String, SolStruct>, tuple: &Component) {
    if let Some(internal_ty) = tuple.internal_type.as_ref() {
        let ident = struct_type_identifier(internal_ty);
        if structs.contains_key(ident) {
            return
        }
        if let Some(fields) = tuple
            .components
            .iter()
            .map(|f| {
                HumanReadableParser::parse_type(&f.type_field)
                    .ok()
                    .and_then(|kind| field(structs, f, kind))
            })
            .collect::<Option<Vec<_>>>()
        {
            let s = SolStruct { name: ident.to_string(), fields };
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

/// `struct Pairing.G2Point[]` -> `Pairing.G2Point`
fn struct_type_identifier(name: &str) -> &str {
    name.trim_start_matches("struct ").split('[').next().unwrap()
}

/// `struct Pairing.Nested.G2Point[]` -> `[Pairing, Nested]`
fn struct_type_projections(name: &str) -> Vec<String> {
    let id = struct_type_identifier(name);
    let mut iter = id.rsplit('.');
    iter.next();
    iter.rev().map(str::to_string).collect()
}

pub(crate) fn expand_struct(
    name: &Ident,
    fields: &[(TokenStream, TokenStream)],
    is_tuple: bool,
) -> TokenStream {
    _expand_struct(name, fields.iter().map(|(a, b)| (a, b, false)), is_tuple)
}

pub(crate) fn expand_event_struct(
    name: &Ident,
    fields: &[(TokenStream, TokenStream, bool)],
    is_tuple: bool,
) -> TokenStream {
    _expand_struct(name, fields.iter().map(|(a, b, c)| (a, b, *c)), is_tuple)
}

fn _expand_struct<'a>(
    name: &Ident,
    fields: impl Iterator<Item = (&'a TokenStream, &'a TokenStream, bool)>,
    is_tuple: bool,
) -> TokenStream {
    let fields = fields.map(|(field, ty, indexed)| {
        (field, ty, if indexed { Some(quote!(#[ethevent(indexed)])) } else { None })
    });
    let fields = if let Some(0) = fields.size_hint().1 {
        // unit struct
        quote!(;)
    } else if is_tuple {
        // tuple struct
        let fields = fields.map(|(_, ty, indexed)| quote!(#indexed pub #ty));
        quote!(( #( #fields ),* );)
    } else {
        // struct
        let fields = fields.map(|(field, ty, indexed)| quote!(#indexed pub #field: #ty));
        quote!({ #( #fields, )* })
    };

    quote!(struct #name #fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_determine_structs() {
        const VERIFIER_ABI: &str =
            include_str!("../../../tests/solidity-contracts/verifier_abi.json");
        let abi = serde_json::from_str::<RawAbi>(VERIFIER_ABI).unwrap();

        let _internal = InternalStructs::new(abi);
    }
}
