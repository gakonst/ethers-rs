//! Bindings for solc's `ast` output field
mod macros;
mod misc;

pub use misc::*;
pub mod util;
pub mod visitor;

use crate::{artifacts::serde_helpers, EvmVersion};
use macros::{ast_node, expr_node, stmt_node};
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
    struct Identifier {
        argument_types: Vec<TypeDescriptions>,
        name: String,
        overloaded_declarations: Vec<usize>,
        referenced_declaration: Option<usize>,
        type_descriptions: TypeDescriptions,
    }
);

expr_node!(
    struct IndexAccess {
        base_expression: Expression,
        index_expression: Expression,
    }
);

expr_node!(
    struct IndexRangeAccess {
        base_expression: Expression,
        start_expression: Option<Expression>,
        end_expression: Option<Expression>,
    }
);

expr_node!(
    struct Literal {
        // TODO
        hex_value: String,
        kind: LiteralKind,
        subdenomination: Option<String>, // TODO
        value: Option<String>,           // TODO
    }
);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LiteralKind {
    Bool,
    Number,
    String,
    HexString,
    UnicodeString,
}

expr_node!(
    struct MemberAccess {
        expression: Expression,
        member_name: String,
        referenced_declaration: Option<usize>,
    }
);

expr_node!(
    struct NewExpression {
        type_name: TypeName,
    }
);

ast_node!(
    struct ArrayTypeName {
        type_descriptions: TypeDescriptions,
        base_type: TypeName,
        length: Option<Expression>,
    }
);

ast_node!(
    struct FunctionTypeName {
        type_descriptions: TypeDescriptions,
        parameter_types: ParameterList,
        return_parameter_types: ParameterList,
        state_mutability: StateMutability,
        visibility: Visibility,
    }
);

ast_node!(
    struct ParameterList {
        parameters: Vec<VariableDeclaration>,
    }
);

ast_node!(
    struct VariableDeclaration {
        name: String,
        name_location: Option<String>, // TODO
        base_functions: Vec<usize>,
        constant: bool,
        documentation: Option<StructuredDocumentation>,
        function_selector: Option<String>, // TODO
        indexed: bool,
        mutability: Mutability,
        overrides: Option<OverrideSpecifier>,
        scope: usize,
        state_variable: bool,
        storage_location: StorageLocation,
        type_descriptions: TypeDescriptions,
        type_name: Option<TypeName>,
        value: Option<Expression>,
        visibility: Visibility,
    }
);

ast_node!(
    struct StructuredDocumentation {
        text: String,
    }
);

ast_node!(
    struct OverrideSpecifier {
        overrides: Vec<UserDefinedTypeNameOrIdentifierPath>,
    }
);

ast_node!(
    struct UserDefinedTypeName {
        type_descriptions: TypeDescriptions,
        contract_scope: Option<String>, // TODO
        name: Option<String>,
        path_node: Option<IdentifierPath>,
        referenced_declaration: usize,
    }
);

ast_node!(
    struct IdentifierPath {
        name: String,
        referenced_declaration: usize,
    }
);

ast_node!(
    struct Mapping {
        type_descriptions: TypeDescriptions,
        key_type: TypeName,
        value_type: TypeName,
    }
);

expr_node!(
    struct TupleExpression {
        components: Vec<Expression>,
        is_inline_array: bool,
    }
);

expr_node!(
    struct UnaryOperation {
        operator: UnaryOperator,
        prefix: bool,
        sub_expression: Expression,
    }
);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOperator {
    /// ++
    Increment,
    /// --
    Decrement,
    /// -
    Negate,
    /// !
    Not,
    /// delete
    Delete,
}

ast_node!(
    struct EnumDefinition {
        name: String,
        name_location: Option<String>, // TODO
        canonical_name: String,
        members: Vec<EnumValue>,
    }
);

ast_node!(
    struct EnumValue {
        name: String,
        name_location: Option<String>, // TODO
    }
);

ast_node!(
    struct ErrorDefinition {
        name: String,
        name_location: String, // TODO
        documentation: Option<StructuredDocumentation>,
        error_selector: Option<String>, // TODO
        parameters: ParameterList,
    }
);

ast_node!(
    struct EventDefinition {
        name: String,
        name_location: Option<String>, // TODO
        anonymous: bool,
        event_selector: Option<String>, // TODO
        documentation: Option<StructuredDocumentation>,
        parameters: ParameterList,
    }
);

ast_node!(
    struct FunctionDefinition {
        name: String,
        name_location: Option<String>, // TODO
        base_functions: Vec<usize>,
        body: Option<Block>,
        documentation: Option<StructuredDocumentation>,
        function_selector: Option<String>, // TODO
        implemented: bool,
        kind: FunctionKind,
        modifiers: Vec<ModifierInvocation>,
        overrides: Option<OverrideSpecifier>,
        parameters: ParameterList,
        return_parameters: ParameterList,
        scope: usize,
        state_mutability: StateMutability,
        is_virtual: bool,
        visibility: Visibility,
    }
);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FunctionKind {
    Function,
    Receive,
    Constructor,
    Fallback,
    FreeFunction,
}

ast_node!(
    struct Block {
        documentation: Option<String>, // TODO
        statements: Vec<Statement>,
    }
);

stmt_node!(
    struct Break {}
);

stmt_node!(
    struct Continue {}
);

stmt_node!(
    struct DoWhileStatement {
        block: Block,
        condition: Expression,
    }
);

stmt_node!(
    struct EmitStatement {
        event_call: FunctionCall,
    }
);

stmt_node!(
    struct ExpressionStatement {
        expression: Expression,
    }
);

stmt_node!(
    struct ForStatement {
        body: BlockOrStatement,
        condition: Option<Expression>,
        initialization_expression: Option<ExpressionOrVariableDeclarationStatement>,
        loop_expression: Option<ExpressionStatement>,
    }
);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockOrStatement {
    Block,
    Statement,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpressionOrVariableDeclarationStatement {
    ExpressionStatement,
    VariableDeclarationStatement,
}

stmt_node!(
    struct VariableDeclarationStatement {
        assignments: Vec<usize>,
        declarations: Vec<VariableDeclaration>,
        initial_value: Option<Expression>,
    }
);

stmt_node!(
    struct IfStatement {
        condition: Expression,
        false_body: Option<BlockOrStatement>,
        true_body: BlockOrStatement,
    }
);

ast_node!(
    struct InlineAssembly {
        documentation: Option<String>, // TODO
        ast: YulBlock,
        evm_version: EvmVersion,
        external_references: ExternalInlineAssemblyReference,
        flags: Vec<InlineAssemblyFlag>,
    }
);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalInlineAssemblyReference {
    #[serde(with = "serde_helpers::display_from_str")]
    pub src: SourceLocation,
    pub declaration: usize,
    pub is_offset: bool,
    pub is_slot: bool,
    pub value_size: usize,
    pub suffix: AssemblyReferenceSuffix,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssemblyReferenceSuffix {
    Slot,
    Offset,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InlineAssemblyFlag {
    MemorySafe,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YulBlock {
    #[serde(with = "serde_helpers::display_from_str")]
    pub src: SourceLocation,
    pub statements: Vec<YulStatement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YulAssignment {
    #[serde(with = "serde_helpers::display_from_str")]
    pub src: SourceLocation,
    pub value: YulExpression,
    pub variable_names: Vec<YulIdentifier>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YulFunctionCall {
    #[serde(with = "serde_helpers::display_from_str")]
    pub src: SourceLocation,
    pub arguments: Vec<YulExpression>,
    pub variable_names: Vec<YulIdentifier>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YulIdentifier {
    #[serde(with = "serde_helpers::display_from_str")]
    pub src: SourceLocation,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YulLiteralValue {
    #[serde(with = "serde_helpers::display_from_str")]
    pub src: SourceLocation,
    pub value: String, // TODO
    pub kind: YulLiteralValueKind,
    pub type_name: String, // TODO
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum YulLiteralValueKind {
    Number,
    String,
    Bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YulLiteralHexValue {
    #[serde(with = "serde_helpers::display_from_str")]
    pub src: SourceLocation,
    pub hex_value: String,     // TODO
    pub value: Option<String>, // TODO
    pub kind: YulLiteralValueKind,
    pub type_name: String, // TODO
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YulKeyword {
    #[serde(with = "serde_helpers::display_from_str")]
    pub src: SourceLocation,
}

pub type YulBreak = YulKeyword;
pub type YulContinue = YulKeyword;
pub type YulLeave = YulKeyword;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YulExpressionStatement {
    #[serde(with = "serde_helpers::display_from_str")]
    pub src: SourceLocation,
    pub expression: YulExpression,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YulForLoop {
    #[serde(with = "serde_helpers::display_from_str")]
    pub src: SourceLocation,
    pub body: YulBlock,
    pub condition: YulExpression,
    pub post: YulBlock,
    pub pre: YulBlock,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YulFunctionDefinition {
    #[serde(with = "serde_helpers::display_from_str")]
    pub src: SourceLocation,
    pub body: YulBlock,
    pub name: String,
    pub parameters: Vec<YulTypedName>,
    pub return_variables: Vec<YulTypedName>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YulTypedName {
    #[serde(with = "serde_helpers::display_from_str")]
    pub src: SourceLocation,
    pub name: String,
    pub type_name: String, // TODO
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YulIf {
    #[serde(with = "serde_helpers::display_from_str")]
    pub src: SourceLocation,
    pub body: YulBlock,
    pub condition: YulExpression,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YulSwitch {
    #[serde(with = "serde_helpers::display_from_str")]
    pub src: SourceLocation,
    pub cases: Vec<YulCase>,
    pub expression: YulExpression,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YulCase {
    #[serde(with = "serde_helpers::display_from_str")]
    pub src: SourceLocation,
    pub body: YulBlock,
    pub value: YulCaseValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum YulCaseValue {
    Default,
    YulLiteral,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YulVariableDeclaration {
    #[serde(with = "serde_helpers::display_from_str")]
    pub src: SourceLocation,
    pub value: Option<YulExpression>,
    pub variables: Vec<YulTypedName>,
}

stmt_node!(
    struct PlaceholderStatement {}
);

stmt_node!(
    struct Return {
        expression: Option<Expression>,
        function_return_parameters: usize,
    }
);

stmt_node!(
    struct RevertStatement {
        error_call: FunctionCall,
    }
);

stmt_node!(
    struct TryStatement {
        clauses: Vec<TryCatchClause>,
        external_call: FunctionCall,
    }
);

ast_node!(
    struct TryCatchClause {
        block: Block,
        error_name: String,
        parameters: Vec<ParameterList>,
    }
);

stmt_node!(
    struct UncheckedBlock {
        statements: Vec<Statement>,
    }
);

stmt_node!(
    struct WhileStatement {
        body: BlockOrStatement,
        condition: Expression,
    }
);

ast_node!(
    struct ModifierInvocation {
        arguments: Vec<Expression>,
        kind: ModifierInvocationKind,
        modifier_name: Option<IdentifierOrIdentifierPath>,
    }
);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModifierInvocationKind {
    ModifierInvocation,
    BaseConstructorSpecifier,
}

// TODO: Better name
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdentifierOrIdentifierPath {
    Identifier,
    IdentifierPath,
}

ast_node!(
    struct ModifierDefinition {
        name: String,
        name_location: Option<String>, // TODO
        base_modifiers: Vec<usize>,
        body: Block,
        documentation: Option<StructuredDocumentation>,
        overrides: Option<OverrideSpecifier>,
        parameters: ParameterList,
        is_virtual: bool,
        visibility: Visibility,
    }
);

ast_node!(
    struct StructDefinition {
        name: String,
        name_location: Option<String>, // TODO
        canonical_name: String,
        members: Vec<VariableDeclaration>,
        scope: usize,
        visibility: Visibility,
    }
);

ast_node!(
    struct UserDefinedValueTypeDefinition {
        name: String,
        name_location: Option<String>, // TODO
        canonical_name: Option<String>,
        underlying_type: TypeName,
    }
);

ast_node!(
    struct UsingForDirective {
        function_list: Vec<FunctionIdentifierPath>,
        global: bool,
        library_name: Option<UserDefinedTypeNameOrIdentifierPath>,
        type_name: Option<TypeName>,
    }
);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionIdentifierPath {
    pub function: IdentifierPath,
}

ast_node!(
    struct ImportDirective {
        absolute_path: String,
        file: String,
        name_location: Option<String>, // TODO
        scope: usize,
        source_unit: usize,
        symbol_aliases: Vec<SymbolAlias>,
        unit_alias: String,
    }
);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolAlias {
    pub foreign: Identifier,
    pub local: Option<String>,
    pub name_location: Option<String>, // TODO
}

ast_node!(
    struct PragmaDirective {
        literals: Vec<String>,
    }
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_ast() {
        let ast = include_str!("../../../test-data/ast/ast-erc4626.json");
        let _ast: SourceUnit = serde_json::from_str(ast).unwrap();
    }
}
