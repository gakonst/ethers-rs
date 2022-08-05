use super::{macros::node_group, misc::SourceLocation};
use crate::artifacts::serde_helpers;
use serde::{Deserialize, Serialize};

node_group! {
    YulStatement;

    YulAssignment,
    YulBlock,
    YulBreak,
    YulContinue,
    YulExpressionStatement,
    YulLeave,
    YulForLoop,
    YulFunctionDefinition,
    YulIf,
    YulSwitch,
    YulVariableDeclaration,
}

node_group! {
    YulExpression;

    YulFunctionCall,
    YulIdentifier,
    YulLiteral,
}

node_group! {
    YulLiteral;

    YulLiteralValue,
    YulLiteralHexValue,
}

/// A Yul block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YulBlock {
    #[serde(with = "serde_helpers::display_from_str")]
    pub src: SourceLocation,
    pub statements: Vec<YulStatement>,
}

/// A Yul assignment statement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YulAssignment {
    #[serde(with = "serde_helpers::display_from_str")]
    pub src: SourceLocation,
    pub value: YulExpression,
    pub variable_names: Vec<YulIdentifier>,
}

/// A Yul function call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YulFunctionCall {
    #[serde(with = "serde_helpers::display_from_str")]
    pub src: SourceLocation,
    pub arguments: Vec<YulExpression>,
    pub variable_names: Vec<YulIdentifier>,
}

/// A Yul identifier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YulIdentifier {
    #[serde(with = "serde_helpers::display_from_str")]
    pub src: SourceLocation,
    pub name: String,
}

/// A literal Yul value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YulLiteralValue {
    #[serde(with = "serde_helpers::display_from_str")]
    pub src: SourceLocation,
    pub value: String, // TODO
    pub kind: YulLiteralValueKind,
    pub type_name: String, // TODO
}

/// Yul literal value kinds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum YulLiteralValueKind {
    /// A number literal.
    Number,
    /// A string literal.
    String,
    /// A boolean literal.
    Bool,
}

/// A literal Yul hex value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YulLiteralHexValue {
    #[serde(with = "serde_helpers::display_from_str")]
    pub src: SourceLocation,
    pub hex_value: String,     // TODO
    pub value: Option<String>, // TODO
    pub kind: YulLiteralValueKind,
    pub type_name: String, // TODO
}

/// A Yul keyword.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YulKeyword {
    #[serde(with = "serde_helpers::display_from_str")]
    pub src: SourceLocation,
}

/// The Yul break keyword.
pub type YulBreak = YulKeyword;
/// The Yul continue keyword.
pub type YulContinue = YulKeyword;
/// The Yul leave keyword.
pub type YulLeave = YulKeyword;

/// A Yul expression statement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YulExpressionStatement {
    #[serde(with = "serde_helpers::display_from_str")]
    pub src: SourceLocation,
    pub expression: YulExpression,
}

/// A Yul for loop.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YulForLoop {
    #[serde(with = "serde_helpers::display_from_str")]
    pub src: SourceLocation,
    pub body: YulBlock,
    pub condition: YulExpression,
    pub post: YulBlock,
    pub pre: YulBlock,
}

/// A Yul function definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YulFunctionDefinition {
    #[serde(with = "serde_helpers::display_from_str")]
    pub src: SourceLocation,
    pub body: YulBlock,
    pub name: String,
    pub parameters: Vec<YulTypedName>,
    pub return_variables: Vec<YulTypedName>,
}

/// A Yul type name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YulTypedName {
    #[serde(with = "serde_helpers::display_from_str")]
    pub src: SourceLocation,
    pub name: String,
    pub type_name: String, // TODO
}

/// A Yul if statement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YulIf {
    #[serde(with = "serde_helpers::display_from_str")]
    pub src: SourceLocation,
    pub body: YulBlock,
    pub condition: YulExpression,
}

/// A Yul switch statement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YulSwitch {
    #[serde(with = "serde_helpers::display_from_str")]
    pub src: SourceLocation,
    pub cases: Vec<YulCase>,
    pub expression: YulExpression,
}

/// A Yul switch statement case.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YulCase {
    #[serde(with = "serde_helpers::display_from_str")]
    pub src: SourceLocation,
    pub body: YulBlock,
    pub value: YulCaseValue,
}

/// A Yul switch case value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum YulCaseValue {
    /// The default case.
    Default,
    /// A case defined by a literal value.
    YulLiteral(YulLiteral),
}

/// A Yul variable declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YulVariableDeclaration {
    #[serde(with = "serde_helpers::display_from_str")]
    pub src: SourceLocation,
    pub value: Option<YulExpression>,
    pub variables: Vec<YulTypedName>,
}
