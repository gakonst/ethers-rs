//! Bindings for solc's `ast` output field

use crate::artifacts::serde_helpers;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt, fmt::Write, str::FromStr};

/// Represents the AST field in the solc output
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ast {
    #[serde(rename = "absolutePath")]
    pub absolute_path: String,
    pub id: usize,
    #[serde(default, rename = "exportedSymbols")]
    pub exported_symbols: BTreeMap<String, Vec<usize>>,
    #[serde(rename = "nodeType")]
    pub node_type: NodeType,
    #[serde(with = "serde_helpers::display_from_str")]
    pub src: SourceLocation,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nodes: Vec<Node>,
    #[serde(flatten)]
    pub other: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
    pub id: usize,
    #[serde(rename = "nodeType")]
    pub node_type: NodeType,
    #[serde(with = "serde_helpers::display_from_str")]
    pub src: SourceLocation,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nodes: Vec<Node>,
    #[serde(flatten)]
    pub other: BTreeMap<String, serde_json::Value>,
}

/// Represents the source location of a node : `<start>:<length>:<index>`
///
/// The `length` and `index` can be -1 which is represented as `None`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLocation {
    pub start: usize,
    pub length: Option<usize>,
    pub index: Option<usize>,
}

impl FromStr for SourceLocation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let invalid_location = move || format!("{} invalid source location", s);

        let mut split = s.split(':');
        let start = split
            .next()
            .ok_or_else(invalid_location)?
            .parse::<usize>()
            .map_err(|_| invalid_location())?;
        let length = split
            .next()
            .ok_or_else(invalid_location)?
            .parse::<isize>()
            .map_err(|_| invalid_location())?;
        let index = split
            .next()
            .ok_or_else(invalid_location)?
            .parse::<isize>()
            .map_err(|_| invalid_location())?;

        let length = if length < 0 { None } else { Some(length as usize) };
        let index = if index < 0 { None } else { Some(index as usize) };

        Ok(Self { start, length, index })
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.start.fmt(f)?;
        f.write_char(':')?;
        if let Some(length) = self.length {
            length.fmt(f)?;
        } else {
            f.write_str("-1")?;
        }
        f.write_char(':')?;
        if let Some(index) = self.index {
            index.fmt(f)?;
        } else {
            f.write_str("-1")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NodeType {
    YulAssignment,
    YulBlock,
    YulExpressionStatement,
    YulForLoop,
    YulIf,
    YulVariableDeclaration,
    YulFunctionDefinition,
    SourceUnit,
    PragmaDirective,
    ContractDefinition,
    EventDefinition,
    ErrorDefinition,
    Other(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_ast() {
        let ast = r#"
        {
  "absolutePath": "input.sol",
  "exportedSymbols":
  {
    "Ballot":
    [
      2
    ],
    "Ballot2":
    [
      3
    ],
    "Ballot3":
    [
      4
    ]
  },
  "id": 5,
  "nodeType": "SourceUnit",
  "nodes":
  [
    {
      "id": 1,
      "literals":
      [
        "solidity",
        ">=",
        "0.4",
        ".0"
      ],
      "nodeType": "PragmaDirective",
      "src": "1:24:0"
    },
    {
      "abstract": false,
      "baseContracts": [],
      "canonicalName": "Ballot",
      "contractDependencies": [],
      "contractKind": "contract",
      "fullyImplemented": true,
      "id": 2,
      "linearizedBaseContracts":
      [
        2
      ],
      "name": "Ballot",
      "nameLocation": "36:6:0",
      "nodeType": "ContractDefinition",
      "nodes": [],
      "scope": 5,
      "src": "27:20:0",
      "usedErrors": []
    },
    {
      "abstract": false,
      "baseContracts": [],
      "canonicalName": "Ballot2",
      "contractDependencies": [],
      "contractKind": "contract",
      "fullyImplemented": true,
      "id": 3,
      "linearizedBaseContracts":
      [
        3
      ],
      "name": "Ballot2",
      "nameLocation": "58:7:0",
      "nodeType": "ContractDefinition",
      "nodes": [],
      "scope": 5,
      "src": "49:21:0",
      "usedErrors": []
    },
    {
      "abstract": false,
      "baseContracts": [],
      "canonicalName": "Ballot3",
      "contractDependencies": [],
      "contractKind": "contract",
      "fullyImplemented": true,
      "id": 4,
      "linearizedBaseContracts":
      [
        4
      ],
      "name": "Ballot3",
      "nameLocation": "81:7:0",
      "nodeType": "ContractDefinition",
      "nodes": [],
      "scope": 5,
      "src": "72:21:0",
      "usedErrors": []
    }
  ],
  "src": "1:92:0"
}
        "#;
        let _ast: Ast = serde_json::from_str(ast).unwrap();

        dbg!(serde_json::from_str::<serde_json::Value>("{}").unwrap());
    }
}
