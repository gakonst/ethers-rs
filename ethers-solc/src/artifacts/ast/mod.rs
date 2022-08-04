//! Bindings for solc's `ast` output field
mod macros;
mod misc;

pub use misc::*;
pub mod util;
pub mod visitor;

use crate::artifacts::serde_helpers;
use macros::{ast_node, expr_node};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// TODO: Node ID

ast_node!(
    /// The root node of a Solidity AST.
    struct SourceUnit {
        #[serde(rename = "absolutePath")]
        absolute_path: String,
        #[serde(default, rename = "exportedSymbols")]
        exported_symbols: BTreeMap<String, Vec<usize>>,
        license: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        nodes: Vec<SourceUnitPart>,
    }
);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceUnitPart {
    PragmaDirective,
    ImportDirective,
    UsingForDirective,
    VariableDeclaration,
    EnumDefinition,
    ErrorDefinition,
    FunctionDefinition,
    StructDefinition,
    UserDefinedValueTypeDefinition,
    ContractDefinition,
}

ast_node!(
    struct ContractDefinition {
        name: String,
        name_location: Option<String>,
        is_abstract: bool,
        base_contracts: Vec<InheritanceSpecifier>,
        canonical_name: Option<String>,
        contract_dependencies: Vec<usize>,
        kind: ContractKind,
        documentation: Option<StructuredDocumentation>,
        fully_implemented: bool,
        linearized_base_contracts: Vec<usize>,
        nodes: Vec<ContractDefinitionPart>,
        scope: usize,
        used_errors: Vec<usize>,
    }
);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContractKind {
    Contract,
    Interface,
    Library,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContractDefinitionPart {
    EnumDefinition,
    ErrorDefinition,
    EventDefinition,
    FunctionDefinition,
    ModifierDefinition,
    StructDefinition,
    UserDefinedValueTypeDefinition,
    UsingForDirective,
    VariableDeclaration,
}

ast_node!(
    struct InheritanceSpecifier {
        arguments: Vec<Expression>,
        base_name: UserDefinedTypeNameOrIdentifierPath,
    }
);

// TODO: Better name
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserDefinedTypeNameOrIdentifierPath {
    UserDefinedTypeName,
    IdentifierPath,
}

expr_node!(
    struct Assignment {
        lhs: Expression,
        operator: AssignmentOperator,
        rhs: Expression,
    }
);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssignmentOperator {
    /// =
    Assign,
    /// +=
    AddAssign,
    /// -=
    SubAssign,
    /// *=
    MulAssign,
    /// /=
    DivAssign,
    /// %=
    ModAssign,
    /// |=
    OrAssign,
    /// &=
    AndAssign,
    /// ^=
    XorAssign,
    /// >>=
    ShrAssign,
    /// <<=
    ShlAssign,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeDescriptions {
    pub type_identifier: Option<String>,
    pub type_string: Option<String>,
}

ast_node!(
    struct BinaryOperation {
        common_type: TypeDescriptions,
        lhs: Expression,
        operator: BinaryOperator,
        rhs: Expression,
    }
);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOperator {
    /// +
    Add,
    /// -
    Sub,
    /// *
    Mul,
    /// /
    Div,
    /// %
    Mod,
    /// **
    Pow,
    /// &&
    And,
    /// ||
    Or,
    /// !=
    NotEqual,
    /// ==
    Equal,
    /// <
    LessThan,
    /// <=
    LessThanOrEqual,
    /// >
    GreaterThan,
    /// >=
    GreaterThanOrEqual,
    /// ^
    Xor,
    /// &
    BitAnd,
    /// |
    BitOr,
    /// <<
    Shl,
    /// >>
    Shr,
}

expr_node!(
    struct Conditional {
        condition: Expression,
        false_expression: Expression,
        true_expression: Expression,
    }
);

expr_node!(
    struct ElementaryTypeNameExpression {
        type_name: ElementaryTypeName,
    }
);

ast_node!(
    struct ElementaryTypeName {
        type_descriptions: TypeDescriptions,
        name: String,
        state_mutability: Option<StateMutability>,
    }
);

expr_node!(
    struct FunctionCall {
        arguments: Vec<Expression>,
        expression: Expression,
        kind: FunctionCallKind,
        names: Vec<String>,
        try_call: bool,
    }
);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FunctionCallKind {
    FunctionCall,
    TypeConversion,
    StructConstructorCall,
}

expr_node!(
    struct FunctionCallOptions {
        expression: Expression,
        names: Vec<String>,
        options: Vec<Expression>,
    }
);

ast_node!(
    struct StructuredDocumentation {
        text: String,
    }
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_ast() {
        let ast = include_str!("../../test-data/ast/ast-erc4626.json");
        let _ast: SourceUnit = serde_json::from_str(ast).unwrap();
    }
}
