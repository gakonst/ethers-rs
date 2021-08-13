//! This is a basic representation of a contract ABI that does no post processing but contains the raw content of the ABI.
//!
//! This is currently used to get access to all the unique solidity structs used as function in/output until `ethabi` supports it as well.

#![allow(missing_docs)]

use ethers_core::abi::param_type::Reader;
use ethers_core::abi::struct_def::{FieldDeclaration, FieldType, StructFieldType, StructType};
use ethers_core::abi::{ParamType, SolStruct};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Helper to match `ethabi::Param`s with structs and nested structs
#[derive(Debug, Clone, Default)]
pub struct InternalStructs {
    /// All unique internal types that are function inputs or outputs
    top_level_internal_types: HashMap<String, Component>,

    /// (function name, param name) -> struct which are the identifying properties we get the name from ethabi.
    function_params: HashMap<(String, String), String>,

    /// (function name) -> Vec<structs> all structs the function returns
    outputs: HashMap<String, Vec<String>>,

    structs: HashMap<String, SolStruct>,
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
                        .as_ref()
                        .filter(|ty| ty.starts_with("struct "))
                        .map(|ty|
                            // the internal type can be an array of a struct: `struct Proof[]`
                            ty.split('[').next().unwrap().to_string())
                    {
                        function_params.insert((name.clone(), input.name.clone()), ty.clone());
                        top_level_internal_types.insert(ty, input);
                    }
                }
                let mut output_structs = Vec::new();
                for output in item.outputs {
                    if let Some(ty) = output
                        .internal_type
                        .as_ref()
                        .filter(|ty| ty.starts_with("struct "))
                        .map(|ty| ty.split('[').next().unwrap().to_string())
                    {
                        output_structs.push(ty.clone());
                        top_level_internal_types.insert(ty, output);
                    }
                }
                outputs.insert(name, output_structs);
            }
        }

        for component in top_level_internal_types.values() {
            insert_structs(&mut structs, component);
        }

        todo!()
    }
}

/// turns the tuple component into a struct if it's still missing, including all inner structs
fn insert_structs(structs: &mut HashMap<String, SolStruct>, tuple: &Component) {
    let internal_ty = tuple.internal_type.as_ref().unwrap();
    let mut fields = Vec::with_capacity(tuple.components.len());
    for field_component in &tuple.components {
        let kind = Reader::read(&field_component.type_field).unwrap();
        fields.push(field(structs, field_component, kind));
    }
    let s = SolStruct {
        name: struct_type_name(internal_ty.as_str()).to_string(),
        fields,
    };
}


/// Determines the type of the field component
fn field(
    structs: &mut HashMap<String, SolStruct>,
    field_component: &Component,
    kind: ParamType,
) -> FieldDeclaration {
    match kind {
        ParamType::Array(ty) => {}
        ParamType::FixedArray(ty, num) => {
            let FieldDeclaration { ty, .. } = field(structs, field_component, *ty);
            match ty {
                FieldType::Elementary(kind) => {
                    FieldDeclaration::new(
                        field_component.name.clone(),
                        FieldType::Elementary(ParamType::FixedArray(Box::new(kind), num)),
                    );
                }
                FieldType::Struct(_) => {}
                _ => {
                    unreachable!("no mappings in params")
                }
            }

            // match *ty {
            //     ParamType::Array(_) => {}
            //     ParamType::FixedArray(_, _) => {}
            //     ParamType::Tuple(_) => {}
            //     _  => {
            // }
            //
            // let field = field(structs, field_component, *ty);
        }
        ParamType::Tuple(_) => {
            insert_structs(structs, field_component);
            let internal_type = field_component.internal_type.as_ref().unwrap();
            let ty = struct_type_name(internal_type).to_string();
            FieldDeclaration::new(
                field_component.name.clone(),
                FieldType::Struct(StructFieldType::Type(StructType::new(ty, Vec::new()))),
            );
        }
        elementary => {
            FieldDeclaration::new(
                field_component.name.clone(),
                FieldType::Elementary(elementary),
            );
        }
    }

    todo!()
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

/// Contract ABI
pub type RawAbi = Vec<Item>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub inputs: Vec<Component>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_mutability: Option<String>,
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default)]
    pub outputs: Vec<Component>,
}

/// Either
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Component {
    #[serde(
        rename = "internalType",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub internal_type: Option<String>,
    pub name: String,
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(default)]
    pub components: Vec<Component>,
}
