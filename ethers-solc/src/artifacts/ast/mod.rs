//! Bindings for the Solidity and Yul ASTs.
//!
//! The Yul AST bindings are available in the [yul] module.
mod macros;
mod misc;
pub use misc::*;
pub mod util;
pub mod visitor;

/// Types for the Yul AST.
///
/// The Yul AST is embedded into the Solidity AST for inline assembly blocks.
pub mod yul;

use crate::artifacts::serde_helpers;
use macros::{ast_node, expr_node, node_group, stmt_node};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use yul::YulBlock;

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

node_group! {
    SourceUnitPart;

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

node_group! {
    Expression;

    Assignment,
    BinaryOperation,
    Conditional,
    ElementaryTypeNameExpression,
    FunctionCall,
    FunctionCallOptions,
    Identifier,
    IndexAccess,
    IndexRangeAccess,
    Literal,
    MemberAccess,
    NewExpression,
    TupleExpression,
    UnaryOperation,
}

node_group! {
    Statement;

    Block,
    Break,
    Continue,
    DoWhileStatement,
    EmitStatement,
    ExpressionStatement,
    ForStatement,
    IfStatement,
    InlineAssembly,
    PlaceholderStatement,
    Return,
    RevertStatement,
    TryStatement,
    UncheckedBlock,
    VariableDeclarationStatement,
    WhileStatement,

}

node_group! {
    ContractDefinitionPart;

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

node_group! {
    TypeName;

    ArrayTypeName,
    ElementaryTypeName,
    FunctionTypeName,
    Mapping,
    UserDefinedTypeName,
}

// TODO: Better name
node_group! {
    UserDefinedTypeNameOrIdentifierPath;

    UserDefinedTypeName,
    IdentifierPath,
}

// TODO: Better name
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BlockOrStatement {
    Statement(Statement),
    Block(Block),
}

// TODO: Better name
node_group! {
    ExpressionOrVariableDeclarationStatement;

    ExpressionStatement,
    VariableDeclarationStatement
}

// TODO: Better name
node_group! {
    IdentifierOrIdentifierPath;

    Identifier,
    IdentifierPath
}

ast_node!(
    /// A contract definition.
    struct ContractDefinition {
        name: String,
        #[serde(with = "serde_helpers::display_from_str_opt")]
        name_location: Option<SourceLocation>,
        #[serde(rename = "abstract")]
        is_abstract: bool,
        base_contracts: Vec<InheritanceSpecifier>,
        canonical_name: Option<String>,
        contract_dependencies: Vec<usize>,
        #[serde(rename = "contractKind")]
        kind: ContractKind,
        documentation: Option<StructuredDocumentation>,
        fully_implemented: bool,
        linearized_base_contracts: Vec<usize>,
        nodes: Vec<ContractDefinitionPart>,
        scope: usize,
        used_errors: Vec<usize>,
    }
);

/// All Solidity contract kinds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ContractKind {
    /// A normal contract.
    Contract,
    /// An interface.
    Interface,
    /// A library.
    Library,
}

ast_node!(
    /// An inheritance specifier.
    struct InheritanceSpecifier {
        #[serde(default)]
        arguments: Vec<Expression>,
        base_name: UserDefinedTypeNameOrIdentifierPath,
    }
);

expr_node!(
    /// An assignment expression.
    struct Assignment {
        #[serde(rename = "leftHandSide")]
        lhs: Expression,
        operator: AssignmentOperator,
        #[serde(rename = "rightHandSide")]
        rhs: Expression,
    }
);

/// Assignment operators.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssignmentOperator {
    /// Simple assignment (`=`)
    #[serde(rename = "=")]
    Assign,
    /// Add and assign (`+=`)
    #[serde(rename = "+=")]
    AddAssign,
    /// Subtract and assign (`-=`)
    #[serde(rename = "-=")]
    SubAssign,
    /// Multiply and assign (`*=`)
    #[serde(rename = "*=")]
    MulAssign,
    /// Divide and assign (`/=`)
    #[serde(rename = "/=")]
    DivAssign,
    /// Modulo and assign (`%=`)
    #[serde(rename = "%=")]
    ModAssign,
    /// Bitwise or and assign (`|=`)
    #[serde(rename = "|=")]
    OrAssign,
    /// Bitwise and and assign (`&=`)
    #[serde(rename = "&=")]
    AndAssign,
    /// Bitwise xor and assign (`^=`)
    #[serde(rename = "^=")]
    XorAssign,
    /// Right shift and assign (`>>=`)
    #[serde(rename = ">>=")]
    ShrAssign,
    /// Left shift and assign (`<<=`)
    #[serde(rename = "<<=")]
    ShlAssign,
}

ast_node!(
    /// A binary operation.
    struct BinaryOperation {
        common_type: TypeDescriptions,
        #[serde(rename = "leftExpression")]
        lhs: Expression,
        operator: BinaryOperator,
        #[serde(rename = "rightExpression")]
        rhs: Expression,
    }
);

/// Binary operators.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOperator {
    /// Addition (`+`)
    #[serde(rename = "+")]
    Add,
    /// Subtraction (`-`)
    #[serde(rename = "-")]
    Sub,
    /// Multiplication (`*`)
    #[serde(rename = "*")]
    Mul,
    /// Division (`/`)
    #[serde(rename = "/")]
    Div,
    /// Modulo (`%`)
    #[serde(rename = "%")]
    Mod,
    /// Exponentiation (`**`)
    #[serde(rename = "**")]
    Pow,
    /// Logical and (`&&`)
    #[serde(rename = "&&")]
    And,
    /// Logical or (`||`)
    #[serde(rename = "||")]
    Or,
    /// Not equals (`!=`)
    #[serde(rename = "!=")]
    NotEqual,
    /// Equals (`==`)
    #[serde(rename = "==")]
    Equal,
    /// Less than (`<`)
    #[serde(rename = "<")]
    LessThan,
    /// Less than or equal (`<=`)
    #[serde(rename = "<=")]
    LessThanOrEqual,
    /// Greater than (`>`)
    #[serde(rename = ">")]
    GreaterThan,
    /// Greater than or equal (`>=`)
    #[serde(rename = ">=")]
    GreaterThanOrEqual,
    /// Bitwise xor (`^`)
    #[serde(rename = "^")]
    Xor,
    /// Bitwise and (`&`)
    #[serde(rename = "&")]
    BitAnd,
    /// Bitwise or (`|`)
    #[serde(rename = "|")]
    BitOr,
    /// Shift left (`<<`)
    #[serde(rename = "<<")]
    Shl,
    /// Shift right (`>>`)
    #[serde(rename = ">>")]
    Shr,
}

expr_node!(
    /// A conditional expression.
    struct Conditional {
        /// The condition.
        condition: Expression,
        /// The expression to evaluate if falsy.
        false_expression: Expression,
        /// The expression to evaluate if truthy.
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
    /// A function call expression.
    struct FunctionCall {
        arguments: Vec<Expression>,
        expression: Expression,
        kind: FunctionCallKind,
        names: Vec<String>,
        try_call: bool,
    }
);

/// Function call kinds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FunctionCallKind {
    /// A regular function call.
    FunctionCall,
    /// A type conversion (e.g. `bytes(x)`).
    TypeConversion,
    /// A struct constructor call (e.g. `MyStruct({ ... })`).
    StructConstructorCall,
}

expr_node!(
    /// A function call options expression (e.g. `x.f{gas: 1}`).
    struct FunctionCallOptions {
        expression: Expression,
        names: Vec<String>,
        options: Vec<Expression>,
    }
);

ast_node!(
    /// An identifier.
    struct Identifier {
        #[serde(default)]
        argument_types: Vec<TypeDescriptions>,
        name: String,
        overloaded_declarations: Vec<isize>,
        referenced_declaration: Option<isize>,
        type_descriptions: TypeDescriptions,
    }
);

expr_node!(
    /// An index access.
    struct IndexAccess {
        base_expression: Expression,
        index_expression: Expression,
    }
);

expr_node!(
    /// An index range access.
    struct IndexRangeAccess {
        base_expression: Expression,
        start_expression: Option<Expression>,
        end_expression: Option<Expression>,
    }
);

expr_node!(
    /// A literal value.
    struct Literal {
        // TODO
        hex_value: String,
        kind: LiteralKind,
        subdenomination: Option<String>, // TODO
        value: Option<String>,           // TODO
    }
);

/// Literal kinds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LiteralKind {
    /// A boolean.
    Bool,
    /// A number.
    Number,
    /// A string.
    String,
    /// A hexadecimal string.
    HexString,
    /// A unicode string.
    UnicodeString,
}

expr_node!(
    /// Member access.
    struct MemberAccess {
        expression: Expression,
        member_name: String,
        referenced_declaration: Option<isize>,
    }
);

expr_node!(
    /// A `new` expression.
    struct NewExpression {
        type_name: TypeName,
    }
);

ast_node!(
    /// An array type name.
    struct ArrayTypeName {
        type_descriptions: TypeDescriptions,
        base_type: TypeName,
        length: Option<Expression>,
    }
);

ast_node!(
    /// A function type name.
    struct FunctionTypeName {
        type_descriptions: TypeDescriptions,
        parameter_types: ParameterList,
        return_parameter_types: ParameterList,
        state_mutability: StateMutability,
        visibility: Visibility,
    }
);

ast_node!(
    /// A parameter list.
    struct ParameterList {
        parameters: Vec<VariableDeclaration>,
    }
);

ast_node!(
    /// A variable declaration.
    struct VariableDeclaration {
        name: String,
        #[serde(with = "serde_helpers::display_from_str_opt")]
        name_location: Option<SourceLocation>,
        #[serde(default)]
        base_functions: Vec<usize>,
        constant: bool,
        documentation: Option<StructuredDocumentation>,
        function_selector: Option<String>, // TODO
        #[serde(default)]
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
    /// Structured documentation (NatSpec).
    struct StructuredDocumentation {
        text: String,
    }
);

ast_node!(
    /// An override specifier.
    struct OverrideSpecifier {
        overrides: Vec<UserDefinedTypeNameOrIdentifierPath>,
    }
);

ast_node!(
    /// A user defined type name.
    struct UserDefinedTypeName {
        type_descriptions: TypeDescriptions,
        contract_scope: Option<String>, // TODO
        name: Option<String>,
        path_node: Option<IdentifierPath>,
        referenced_declaration: isize,
    }
);

ast_node!(
    /// An identifier path.
    struct IdentifierPath {
        name: String,
        referenced_declaration: isize,
    }
);

ast_node!(
    /// A mapping type.
    struct Mapping {
        type_descriptions: TypeDescriptions,
        key_type: TypeName,
        value_type: TypeName,
    }
);

expr_node!(
    /// A tuple expression.
    struct TupleExpression {
        components: Vec<Expression>,
        is_inline_array: bool,
    }
);

expr_node!(
    /// A unary operation.
    struct UnaryOperation {
        operator: UnaryOperator,
        /// Whether the unary operator is before or after the expression (e.g. `x++` vs. `++x`)
        prefix: bool,
        sub_expression: Expression,
    }
);

/// Unary operators.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOperator {
    /// Increment (`++`)
    #[serde(rename = "++")]
    Increment,
    /// Decrement (`--`)
    #[serde(rename = "--")]
    Decrement,
    /// Negate (`-`)
    #[serde(rename = "-")]
    Negate,
    /// Not (`!`)
    #[serde(rename = "!")]
    Not,
    /// `delete`
    #[serde(rename = "delete")]
    Delete,
}

ast_node!(
    /// An enum definition.
    struct EnumDefinition {
        name: String,
        #[serde(with = "serde_helpers::display_from_str_opt")]
        name_location: Option<SourceLocation>,
        canonical_name: String,
        members: Vec<EnumValue>,
    }
);

ast_node!(
    /// An enum value.
    struct EnumValue {
        name: String,
        #[serde(with = "serde_helpers::display_from_str_opt")]
        name_location: Option<SourceLocation>,
    }
);

ast_node!(
    /// A custom error definition.
    struct ErrorDefinition {
        name: String,
        #[serde(with = "serde_helpers::display_from_str")]
        name_location: SourceLocation,
        documentation: Option<StructuredDocumentation>,
        error_selector: Option<String>, // TODO
        parameters: ParameterList,
    }
);

ast_node!(
    /// An event definition.
    struct EventDefinition {
        name: String,
        #[serde(with = "serde_helpers::display_from_str_opt")]
        name_location: Option<SourceLocation>,
        anonymous: bool,
        event_selector: Option<String>, // TODO
        documentation: Option<StructuredDocumentation>,
        parameters: ParameterList,
    }
);

ast_node!(
    /// A function definition.
    struct FunctionDefinition {
        name: String,
        #[serde(with = "serde_helpers::display_from_str_opt")]
        name_location: Option<SourceLocation>,
        #[serde(default)]
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
        #[serde(rename = "virtual")]
        is_virtual: bool,
        visibility: Visibility,
    }
);

/// Function kinds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FunctionKind {
    /// A contract function.
    Function,
    /// A receive function.
    Receive,
    /// A constructor.
    Constructor,
    /// A fallback function.
    Fallback,
    /// A free-standing function.
    FreeFunction,
}

ast_node!(
    /// A block of statements.
    struct Block {
        documentation: Option<String>, // TODO
        statements: Vec<Statement>,
    }
);

stmt_node!(
    /// The break keyword.
    struct Break {}
);

stmt_node!(
    /// The continue keyword.
    struct Continue {}
);

stmt_node!(
    /// A do while statement.
    struct DoWhileStatement {
        block: Block,
        condition: Expression,
    }
);

stmt_node!(
    /// An emit statement.
    struct EmitStatement {
        event_call: FunctionCall,
    }
);

stmt_node!(
    /// An expression statement.
    struct ExpressionStatement {
        expression: Expression,
    }
);

stmt_node!(
    /// A for statement.
    struct ForStatement {
        body: BlockOrStatement,
        condition: Option<Expression>,
        initialization_expression: Option<ExpressionOrVariableDeclarationStatement>,
        loop_expression: Option<ExpressionStatement>,
    }
);

stmt_node!(
    /// A variable declaration statement.
    struct VariableDeclarationStatement {
        assignments: Vec<usize>,
        declarations: Vec<VariableDeclaration>,
        initial_value: Option<Expression>,
    }
);

stmt_node!(
    /// An if statement.
    struct IfStatement {
        condition: Expression,
        false_body: Option<BlockOrStatement>,
        true_body: BlockOrStatement,
    }
);

ast_node!(
    /// A block of inline assembly.
    ///
    /// Refer to the [yul] module for Yul AST nodes.
    struct InlineAssembly {
        documentation: Option<String>, // TODO
        #[serde(rename = "AST")]
        ast: YulBlock,
        // TODO: We need this camel case for the AST, but pascal case other places in ethers-solc
        //evm_version: EvmVersion,
        external_references: Vec<ExternalInlineAssemblyReference>,
        #[serde(default)]
        flags: Vec<InlineAssemblyFlag>,
    }
);

/// A reference to an external variable or slot in an inline assembly block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalInlineAssemblyReference {
    #[serde(with = "serde_helpers::display_from_str")]
    pub src: SourceLocation,
    pub declaration: usize,
    #[serde(default)]
    pub offset: bool,
    #[serde(default)]
    pub slot: bool,
    pub value_size: usize,
    pub suffix: Option<AssemblyReferenceSuffix>,
}

/// An assembly reference suffix.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AssemblyReferenceSuffix {
    /// The reference refers to a storage slot.
    Slot,
    /// The reference refers to an offset.
    Offset,
}

/// Inline assembly flags.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InlineAssemblyFlag {
    MemorySafe,
}

stmt_node!(
    /// A placeholder statement (`_`)
    struct PlaceholderStatement {}
);

stmt_node!(
    /// A return statement.
    struct Return {
        expression: Option<Expression>,
        function_return_parameters: usize,
    }
);

stmt_node!(
    /// A revert statement.
    struct RevertStatement {
        error_call: FunctionCall,
    }
);

stmt_node!(
    /// A try/catch statement.
    struct TryStatement {
        clauses: Vec<TryCatchClause>,
        external_call: FunctionCall,
    }
);

ast_node!(
    /// A try/catch clause.
    struct TryCatchClause {
        block: Block,
        error_name: String,
        #[serde(default)]
        parameters: Vec<ParameterList>,
    }
);

stmt_node!(
    /// An unchecked block.
    struct UncheckedBlock {
        statements: Vec<Statement>,
    }
);

stmt_node!(
    /// A while statement.
    struct WhileStatement {
        body: BlockOrStatement,
        condition: Expression,
    }
);

ast_node!(
    /// A modifier or base constructor invocation.
    struct ModifierInvocation {
        #[serde(default)]
        arguments: Vec<Expression>,
        kind: Option<ModifierInvocationKind>,
        modifier_name: IdentifierOrIdentifierPath,
    }
);

/// Modifier invocation kinds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ModifierInvocationKind {
    /// A regular modifier invocation.
    ModifierInvocation,
    /// A base constructor invocation.
    BaseConstructorSpecifier,
}

ast_node!(
    /// A modifier definition.
    struct ModifierDefinition {
        name: String,
        #[serde(with = "serde_helpers::display_from_str_opt")]
        name_location: Option<SourceLocation>,
        #[serde(default)]
        base_modifiers: Vec<usize>,
        body: Block,
        documentation: Option<StructuredDocumentation>,
        overrides: Option<OverrideSpecifier>,
        parameters: ParameterList,
        #[serde(rename = "virtual")]
        is_virtual: bool,
        visibility: Visibility,
    }
);

ast_node!(
    /// A struct definition.
    struct StructDefinition {
        name: String,
        #[serde(with = "serde_helpers::display_from_str_opt")]
        name_location: Option<SourceLocation>,
        canonical_name: String,
        members: Vec<VariableDeclaration>,
        scope: usize,
        visibility: Visibility,
    }
);

ast_node!(
    /// A user defined value type definition.
    struct UserDefinedValueTypeDefinition {
        name: String,
        #[serde(with = "serde_helpers::display_from_str_opt")]
        name_location: Option<SourceLocation>,
        canonical_name: Option<String>,
        underlying_type: TypeName,
    }
);

ast_node!(
    /// A using for directive.
    struct UsingForDirective {
        #[serde(default)]
        function_list: Vec<FunctionIdentifierPath>,
        #[serde(default)]
        global: bool,
        library_name: Option<UserDefinedTypeNameOrIdentifierPath>,
        type_name: Option<TypeName>,
    }
);

/// A wrapper around [IdentifierPath] for the [UsingForDirective].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionIdentifierPath {
    pub function: IdentifierPath,
}

ast_node!(
    /// An import directive.
    struct ImportDirective {
        absolute_path: String,
        file: String,
        #[serde(with = "serde_helpers::display_from_str_opt")]
        name_location: Option<SourceLocation>,
        scope: usize,
        source_unit: usize,
        symbol_aliases: Vec<SymbolAlias>,
        unit_alias: String,
    }
);

/// A symbol alias.
///
/// Symbol aliases can be defined using the [ImportDirective].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolAlias {
    pub foreign: Identifier,
    pub local: Option<String>,
    #[serde(with = "serde_helpers::display_from_str_opt")]
    pub name_location: Option<SourceLocation>,
}

ast_node!(
    /// A pragma directive.
    struct PragmaDirective {
        literals: Vec<String>,
    }
);

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf};

    #[test]
    fn can_parse_ast() {
        fs::read_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data").join("ast"))
            .unwrap()
            .for_each(|path| {
                let path = path.unwrap().path();
                let path_str = path.to_string_lossy();

                // TODO: Support legacy AST. In legacy AST, "nodeType" is "name" and many
                // properties are lifted from the AST nodes themselves into an "attributes" map
                if path_str.contains("legacy") {
                    println!("... {} skipped", path.to_string_lossy());
                    return
                }

                let input = fs::read_to_string(&path).unwrap();
                let deserializer = &mut serde_json::Deserializer::from_str(&input);
                let result: Result<SourceUnit, _> = serde_path_to_error::deserialize(deserializer);
                match result {
                    Err(e) => {
                        println!("... {} fail: {e}", path_str);
                        panic!();
                    }
                    Ok(_) => {
                        println!("... {} ok", path_str);
                    }
                }
            })
    }
}
