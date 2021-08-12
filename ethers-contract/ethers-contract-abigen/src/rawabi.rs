//! This is a basic representation of a contract ABI that does no post processing but contains the raw content of the ABI.
//!
//! This is currently used to get access to all the unique solidity structs used as function in/output until `ethabi` supports it as well.

#![allow(missing_docs)]

use serde::{
    Deserialize, Serialize,
};
use std::collections::HashMap;


#[derive(Debug, Clone)]
pub struct InternalStructs {
    internal_types: HashMap<String, Component>,
}

impl InternalStructs {

    pub fn new(abi: RawAbi) -> Self {

        abi.into_iter().flat_map(|item|item.inputs)
            

            .flat_map(|input|.inputs).map(||)
        todo!()
    }

}

pub type RawAbi = Vec<Item>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub inputs: Vec<Input>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_mutability:Option<String>,
    #[serde(rename = "type")]
    pub type_field: String,
    pub name: Option<String>,
    #[serde(default)]
    pub outputs: Vec<Output>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Input {
    #[serde(rename = "internalType")]
    pub internal_type: Option<String>,
    pub name: String,
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(default)]
    pub components: Vec<Component>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Component {
    pub components: Vec<Component>,
    pub internal_type: String,
    pub name: String,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    pub internal_type: String,
    pub name: String,
    #[serde(rename = "type")]
    pub type_field: String,
}
