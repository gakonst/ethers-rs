use super::{lowfidelity::TypedAst, yul::*, *};
use as_any::AsAny;
use eyre::Result;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum VisitError {
    #[error("{0}")]
    MsgError(String),
    #[error("")]
    Unknown,
}

macro_rules! impl_visitor {
    ($type:ty) => {
        paste::paste! {
            fn [<visit_ $type:snake>](&mut self, node_type: &mut $type) -> Result<(), VisitError> {
                node_type.visit(self)
            }
        }
    };
}

pub trait Visitor<D>: AsAny {
    fn shared_data(&mut self) -> &D;

    impl_visitor!(SourceUnit);
    impl_visitor!(PragmaDirective);
    impl_visitor!(ImportDirective);
    impl_visitor!(SourceUnitPart);
    impl_visitor!(UsingForDirective);
    impl_visitor!(FunctionIdentifierPath);
    impl_visitor!(VariableDeclaration);
    impl_visitor!(BinaryOperation);
    impl_visitor!(Conditional);
    impl_visitor!(ElementaryTypeName);
    impl_visitor!(ElementaryTypeNameExpression);
    impl_visitor!(FunctionCall);
    impl_visitor!(UnaryOperation);
    impl_visitor!(ParameterList);
    impl_visitor!(EnumValue);
    impl_visitor!(OverrideSpecifier);
    impl_visitor!(TupleExpression);
    impl_visitor!(NewExpression);
    impl_visitor!(MemberAccess);
    impl_visitor!(TypeDescriptions);
    impl_visitor!(Literal);
    impl_visitor!(BlockOrStatement);
    impl_visitor!(IndexRangeAccess);
    impl_visitor!(IndexAccess);
    impl_visitor!(Identifier);
    impl_visitor!(FunctionCallOptions);
    impl_visitor!(EnumDefinition);
    impl_visitor!(EventDefinition);
    impl_visitor!(ModifierDefinition);
    impl_visitor!(ModifierInvocation);
    impl_visitor!(ErrorDefinition);
    impl_visitor!(FunctionDefinition);
    impl_visitor!(StructDefinition);
    impl_visitor!(Expression);
    impl_visitor!(Statement);
    impl_visitor!(ContractDefinition);
    impl_visitor!(ContractDefinitionPart);
    impl_visitor!(TypeName);
    impl_visitor!(UserDefinedValueTypeDefinition);
    impl_visitor!(UserDefinedTypeNameOrIdentifierPath);
    impl_visitor!(ExpressionOrVariableDeclarationStatement);
    impl_visitor!(IdentifierOrIdentifierPath);
    impl_visitor!(InheritanceSpecifier);
    impl_visitor!(UserDefinedTypeName);
    impl_visitor!(Mapping);
    impl_visitor!(FunctionTypeName);
    impl_visitor!(ArrayTypeName);
    impl_visitor!(Assignment);
    impl_visitor!(AssignmentOperator);
    impl_visitor!(BinaryOperator);
    impl_visitor!(PlaceholderStatement);
    impl_visitor!(InlineAssembly);
    impl_visitor!(IfStatement);
    impl_visitor!(ForStatement);
    impl_visitor!(ExpressionStatement);
    impl_visitor!(EmitStatement);
    impl_visitor!(DoWhileStatement);
    impl_visitor!(Continue);
    impl_visitor!(Break);
    impl_visitor!(Block);
    impl_visitor!(Return);
    impl_visitor!(RevertStatement);
    impl_visitor!(TryStatement);
    impl_visitor!(UncheckedBlock);
    impl_visitor!(VariableDeclarationStatement);
    impl_visitor!(WhileStatement);
    impl_visitor!(IdentifierPath);
    impl_visitor!(StructuredDocumentation);
    impl_visitor!(FunctionCallKind);
    impl_visitor!(UnaryOperator);
    impl_visitor!(ExternalInlineAssemblyReference);
    impl_visitor!(AssemblyReferenceSuffix);
    impl_visitor!(InlineAssemblyFlag);
    impl_visitor!(TryCatchClause);
    impl_visitor!(YulAssignment);
    impl_visitor!(YulBlock);
    impl_visitor!(YulBreak);
    impl_visitor!(YulContinue);
    impl_visitor!(YulStatement);
    impl_visitor!(YulExpression);
    impl_visitor!(YulExpressionStatement);
    impl_visitor!(YulFunctionDefinition);
    impl_visitor!(YulForLoop);
    impl_visitor!(YulIf);
    impl_visitor!(YulSwitch);
    impl_visitor!(YulVariableDeclaration);
    impl_visitor!(YulFunctionCall);
    impl_visitor!(YulIdentifier);
    impl_visitor!(YulLiteral);
    impl_visitor!(YulLeave);
}

pub trait Visitable {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized;
}

/// Helper for nodes that don't need much implementation to traverse the childrens
macro_rules! empty_visitable {
    ($type:ty) => {
        impl Visitable for $type {
            fn visit<V, D>(&mut self, _: &mut V) -> Result<(), VisitError>
            where
                V: Visitor<D> + ?Sized,
            {
                Ok(())
            }
        }
    };
}

empty_visitable!(PragmaDirective);
empty_visitable!(ElementaryTypeName);
empty_visitable!(ElementaryTypeNameExpression);
empty_visitable!(EnumValue);
empty_visitable!(Literal);
empty_visitable!(Identifier);
empty_visitable!(EventDefinition);
empty_visitable!(ModifierDefinition);
empty_visitable!(UserDefinedTypeName);
empty_visitable!(Mapping);
empty_visitable!(FunctionTypeName);
empty_visitable!(AssignmentOperator);
empty_visitable!(TypeDescriptions);
empty_visitable!(BinaryOperator);
empty_visitable!(PlaceholderStatement);
empty_visitable!(Continue);
empty_visitable!(Break);
empty_visitable!(IdentifierPath);
empty_visitable!(StructuredDocumentation);
empty_visitable!(FunctionCallKind);
empty_visitable!(UnaryOperator);
empty_visitable!(AssemblyReferenceSuffix);
empty_visitable!(InlineAssemblyFlag);
empty_visitable!(YulBreak);
empty_visitable!(YulExpressionStatement);
empty_visitable!(YulFunctionDefinition);
empty_visitable!(YulIf);
empty_visitable!(YulSwitch);
empty_visitable!(YulVariableDeclaration);
empty_visitable!(YulIdentifier);
empty_visitable!(YulLiteral);

impl<T> Visitable for Vec<T>
where
    T: Visitable,
{
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        for item in self.iter_mut() {
            item.visit(v)?;
        }
        Ok(())
    }
}

/// Main entry point of the ast
impl Visitable for TypedAst {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        v.visit_source_unit(&mut self.source_unit)
    }
}

impl Visitable for SourceUnit {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        self.nodes.visit(v)
    }
}

impl Visitable for SourceUnitPart {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        match self {
            SourceUnitPart::PragmaDirective(e) => v.visit_pragma_directive(e),
            SourceUnitPart::ImportDirective(e) => v.visit_import_directive(e),
            SourceUnitPart::UsingForDirective(e) => v.visit_using_for_directive(e),
            SourceUnitPart::VariableDeclaration(e) => v.visit_variable_declaration(e),
            SourceUnitPart::EnumDefinition(e) => v.visit_enum_definition(e),
            SourceUnitPart::ErrorDefinition(e) => v.visit_error_definition(e),
            SourceUnitPart::FunctionDefinition(e) => v.visit_function_definition(e),
            SourceUnitPart::StructDefinition(e) => v.visit_struct_definition(e),
            SourceUnitPart::UserDefinedValueTypeDefinition(e) => {
                v.visit_user_defined_value_type_definition(e)
            }
            SourceUnitPart::ContractDefinition(e) => v.visit_contract_definition(e),
        }
    }
}

impl Visitable for Expression {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        match self {
            Expression::Assignment(e) => v.visit_assignment(e),
            Expression::BinaryOperation(e) => v.visit_binary_operation(e),
            Expression::Conditional(e) => v.visit_conditional(e),
            Expression::ElementaryTypeNameExpression(e) => {
                v.visit_elementary_type_name_expression(e)
            }
            Expression::FunctionCall(e) => v.visit_function_call(e),
            Expression::FunctionCallOptions(e) => v.visit_function_call_options(e),
            Expression::Identifier(e) => v.visit_identifier(e),
            Expression::IndexAccess(e) => v.visit_index_access(e),
            Expression::IndexRangeAccess(e) => v.visit_index_range_access(e),
            Expression::Literal(e) => v.visit_literal(e),
            Expression::MemberAccess(e) => v.visit_member_access(e),
            Expression::NewExpression(e) => v.visit_new_expression(e),
            Expression::TupleExpression(e) => v.visit_tuple_expression(e),
            Expression::UnaryOperation(e) => v.visit_unary_operation(e),
        }
    }
}

impl Visitable for Statement {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        match self {
            Statement::Block(e) => v.visit_block(e),
            Statement::Break(e) => v.visit_break(e),
            Statement::Continue(e) => v.visit_continue(e),
            Statement::DoWhileStatement(e) => v.visit_do_while_statement(e),
            Statement::EmitStatement(e) => v.visit_emit_statement(e),
            Statement::ExpressionStatement(e) => v.visit_expression_statement(e),
            Statement::ForStatement(e) => v.visit_for_statement(e),
            Statement::IfStatement(e) => v.visit_if_statement(e),
            Statement::InlineAssembly(e) => v.visit_inline_assembly(e),
            Statement::PlaceholderStatement(e) => v.visit_placeholder_statement(e),
            Statement::Return(e) => v.visit_return(e),
            Statement::RevertStatement(e) => v.visit_revert_statement(e),
            Statement::TryStatement(e) => v.visit_try_statement(e),
            Statement::UncheckedBlock(e) => v.visit_unchecked_block(e),
            Statement::VariableDeclarationStatement(e) => v.visit_variable_declaration_statement(e),
            Statement::WhileStatement(e) => v.visit_while_statement(e),
        }
    }
}

impl Visitable for ContractDefinitionPart {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        match self {
            ContractDefinitionPart::EnumDefinition(e) => v.visit_enum_definition(e),
            ContractDefinitionPart::ErrorDefinition(e) => v.visit_error_definition(e),
            ContractDefinitionPart::EventDefinition(e) => v.visit_event_definition(e),
            ContractDefinitionPart::FunctionDefinition(e) => v.visit_function_definition(e),
            ContractDefinitionPart::ModifierDefinition(e) => v.visit_modifier_definition(e),
            ContractDefinitionPart::StructDefinition(e) => v.visit_struct_definition(e),
            ContractDefinitionPart::UserDefinedValueTypeDefinition(e) => {
                v.visit_user_defined_value_type_definition(e)
            }
            ContractDefinitionPart::UsingForDirective(e) => v.visit_using_for_directive(e),
            ContractDefinitionPart::VariableDeclaration(e) => v.visit_variable_declaration(e),
        }
    }
}

impl Visitable for TypeName {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        match self {
            TypeName::ArrayTypeName(e) => v.visit_array_type_name(e),
            TypeName::ElementaryTypeName(e) => v.visit_elementary_type_name(e),
            TypeName::FunctionTypeName(e) => v.visit_function_type_name(e),
            TypeName::Mapping(e) => v.visit_mapping(e),
            TypeName::UserDefinedTypeName(e) => v.visit_user_defined_type_name(e),
        }
    }
}

impl Visitable for UserDefinedTypeNameOrIdentifierPath {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        match self {
            UserDefinedTypeNameOrIdentifierPath::UserDefinedTypeName(e) => {
                v.visit_user_defined_type_name(e)
            }
            UserDefinedTypeNameOrIdentifierPath::IdentifierPath(e) => v.visit_identifier_path(e),
        }
    }
}

impl Visitable for ExpressionOrVariableDeclarationStatement {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        match self {
            ExpressionOrVariableDeclarationStatement::ExpressionStatement(e) => {
                v.visit_expression_statement(e)
            }
            ExpressionOrVariableDeclarationStatement::VariableDeclarationStatement(e) => {
                v.visit_variable_declaration_statement(e)
            }
        }
    }
}

impl Visitable for IdentifierOrIdentifierPath {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        match self {
            IdentifierOrIdentifierPath::Identifier(e) => v.visit_identifier(e),
            IdentifierOrIdentifierPath::IdentifierPath(e) => v.visit_identifier_path(e),
        }
    }
}

impl Visitable for YulStatement {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        match self {
            YulStatement::YulAssignment(e) => v.visit_yul_assignment(e),
            YulStatement::YulBlock(e) => v.visit_yul_block(e),
            YulStatement::YulBreak(e) => v.visit_yul_break(e),
            YulStatement::YulContinue(e) => v.visit_yul_continue(e),
            YulStatement::YulExpressionStatement(e) => v.visit_yul_expression_statement(e),
            YulStatement::YulLeave(e) => v.visit_yul_leave(e),
            YulStatement::YulForLoop(e) => v.visit_yul_for_loop(e),
            YulStatement::YulFunctionDefinition(e) => v.visit_yul_function_definition(e),
            YulStatement::YulIf(e) => v.visit_yul_if(e),
            YulStatement::YulSwitch(e) => v.visit_yul_switch(e),
            YulStatement::YulVariableDeclaration(e) => v.visit_yul_variable_declaration(e),
        }
    }
}

impl Visitable for YulExpression {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        match self {
            YulExpression::YulFunctionCall(e) => v.visit_yul_function_call(e),
            YulExpression::YulIdentifier(e) => v.visit_yul_identifier(e),
            YulExpression::YulLiteral(e) => v.visit_yul_literal(e),
        }
    }
}

/// Implement nodes that may have sub nodes
impl Visitable for ImportDirective {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        self.symbol_aliases.iter_mut().try_for_each(|sa| {
            let foreign = &mut sa.foreign;
            v.visit_identifier(foreign)
        })
    }
}

impl Visitable for SymbolAlias {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        v.visit_identifier(&mut self.foreign)
    }
}

impl Visitable for UsingForDirective {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        self.function_list.iter_mut().try_for_each(|fd| v.visit_function_identifier_path(fd))?;
        self.library_name
            .as_mut()
            .map_or_else(|| Ok(()), |e| v.visit_user_defined_type_name_or_identifier_path(e))?;
        self.type_name.as_mut().map_or_else(|| Ok(()), |e| v.visit_type_name(e))
    }
}

impl Visitable for FunctionIdentifierPath {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        v.visit_identifier_path(&mut self.function)
    }
}

impl Visitable for VariableDeclaration {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        self.overrides.as_mut().map_or_else(|| Ok(()), |e| v.visit_override_specifier(e))?;
        self.type_name.as_mut().map_or_else(|| Ok(()), |e| v.visit_type_name(e))?;
        self.value.as_mut().map_or_else(|| Ok(()), |e| v.visit_expression(e))
    }
}

impl Visitable for EnumDefinition {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        self.members.iter_mut().try_for_each(|e| v.visit_enum_value(e))
    }
}

impl Visitable for ErrorDefinition {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        v.visit_parameter_list(&mut self.parameters)
    }
}

impl Visitable for ModifierInvocation {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        self.arguments.iter_mut().try_for_each(|a| v.visit_expression(a))?;
        v.visit_identifier_or_identifier_path(&mut self.modifier_name)
    }
}

impl Visitable for Block {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        self.statements.iter_mut().try_for_each(|s| v.visit_statement(s))
    }
}

impl Visitable for OverrideSpecifier {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        self.overrides
            .iter_mut()
            .try_for_each(|o| v.visit_user_defined_type_name_or_identifier_path(o))
    }
}

impl Visitable for ParameterList {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        self.parameters.iter_mut().try_for_each(|p| v.visit_variable_declaration(p))
    }
}

impl Visitable for FunctionDefinition {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        self.body.as_mut().map_or_else(|| Ok(()), |n| v.visit_block(n))?;
        self.modifiers.iter_mut().try_for_each(|s| v.visit_modifier_invocation(s))?;
        self.overrides.as_mut().map_or_else(|| Ok(()), |n| v.visit_override_specifier(n))?;
        v.visit_parameter_list(&mut self.parameters)?;
        v.visit_parameter_list(&mut self.return_parameters)
    }
}

impl Visitable for StructDefinition {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        self.members.iter_mut().try_for_each(|s| v.visit_variable_declaration(s))
    }
}

impl Visitable for UserDefinedValueTypeDefinition {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        v.visit_type_name(&mut self.underlying_type)
    }
}

impl Visitable for ArrayTypeName {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        v.visit_type_descriptions(&mut self.type_descriptions)?;
        v.visit_type_name(&mut self.base_type)?;
        self.length.as_mut().map_or_else(|| Ok(()), |n| v.visit_expression(n))
    }
}

impl Visitable for InheritanceSpecifier {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        self.arguments.iter_mut().try_for_each(|s| v.visit_expression(s))?;
        v.visit_user_defined_type_name_or_identifier_path(&mut self.base_name)
    }
}

impl Visitable for ContractDefinition {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        self.base_contracts.iter_mut().try_for_each(|s| v.visit_inheritance_specifier(s))?;
        self.documentation
            .as_mut()
            .map_or_else(|| Ok(()), |n| v.visit_structured_documentation(n))?;
        self.nodes.iter_mut().try_for_each(|s| v.visit_contract_definition_part(s))
    }
}

impl Visitable for Assignment {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        v.visit_expression(&mut self.lhs)?;
        v.visit_assignment_operator(&mut self.operator)?;
        v.visit_expression(&mut self.rhs)
    }
}

impl Visitable for BinaryOperation {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        v.visit_type_descriptions(&mut self.common_type)?;
        v.visit_expression(&mut self.lhs)?;
        v.visit_binary_operator(&mut self.operator)?;
        v.visit_expression(&mut self.rhs)
    }
}

impl Visitable for Conditional {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        v.visit_expression(&mut self.condition)?;
        v.visit_expression(&mut self.false_expression)?;
        v.visit_expression(&mut self.true_expression)
    }
}

impl Visitable for FunctionCall {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        self.arguments.iter_mut().try_for_each(|s| v.visit_expression(s))?;
        v.visit_expression(&mut self.expression)?;
        v.visit_function_call_kind(&mut self.kind)
    }
}

impl Visitable for FunctionCallOptions {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        v.visit_expression(&mut self.expression)?;
        self.options.iter_mut().try_for_each(|s| v.visit_expression(s))
    }
}

impl Visitable for IndexAccess {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        v.visit_expression(&mut self.base_expression)?;
        self.index_expression.as_mut().map_or_else(|| Ok(()), |n| v.visit_expression(n))
    }
}

impl Visitable for IndexRangeAccess {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        v.visit_expression(&mut self.base_expression)?;
        self.start_expression.as_mut().map_or_else(|| Ok(()), |n| v.visit_expression(n))?;
        self.end_expression.as_mut().map_or_else(|| Ok(()), |n| v.visit_expression(n))
    }
}

impl Visitable for MemberAccess {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        v.visit_expression(&mut self.expression)
    }
}

impl Visitable for NewExpression {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        v.visit_type_name(&mut self.type_name)
    }
}

impl Visitable for TupleExpression {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        self.components
            .iter_mut()
            .try_for_each(|s| s.as_mut().map_or_else(|| Ok(()), |n| v.visit_expression(n)))
    }
}

impl Visitable for UnaryOperation {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        v.visit_unary_operator(&mut self.operator)?;
        v.visit_expression(&mut self.sub_expression)
    }
}

impl Visitable for DoWhileStatement {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        v.visit_block(&mut self.block)?;
        v.visit_expression(&mut self.condition)
    }
}

impl Visitable for EmitStatement {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        v.visit_function_call(&mut self.event_call)
    }
}

impl Visitable for ExpressionStatement {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        v.visit_expression(&mut self.expression)
    }
}

impl Visitable for BlockOrStatement {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        match self {
            BlockOrStatement::Block(e) => e.visit(v),
            BlockOrStatement::Statement(e) => e.visit(v),
        }
    }
}

impl Visitable for ForStatement {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        v.visit_block_or_statement(&mut self.body)?;
        self.condition.as_mut().map_or_else(|| Ok(()), |n| v.visit_expression(n))?;
        self.initialization_expression
            .as_mut()
            .map_or_else(|| Ok(()), |n| v.visit_expression_or_variable_declaration_statement(n))?;
        self.loop_expression.as_mut().map_or_else(|| Ok(()), |n| v.visit_expression_statement(n))
    }
}

impl Visitable for VariableDeclarationStatement {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        self.declarations.iter_mut().try_for_each(|s| {
            s.as_mut().map_or_else(|| Ok(()), |n| v.visit_variable_declaration(n))
        })?;
        self.initial_value.as_mut().map_or_else(|| Ok(()), |n| v.visit_expression(n))
    }
}

impl Visitable for IfStatement {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        v.visit_expression(&mut self.condition)?;
        self.false_body.as_mut().map_or_else(|| Ok(()), |n| v.visit_block_or_statement(n))?;
        v.visit_block_or_statement(&mut self.true_body)
    }
}

impl Visitable for YulBlock {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        self.statements.iter_mut().try_for_each(|s| v.visit_yul_statement(s))
    }
}

impl Visitable for YulAssignment {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        v.visit_yul_expression(&mut self.value)?;
        self.variable_names.iter_mut().try_for_each(|s| v.visit_yul_identifier(s))
    }
}

impl Visitable for YulForLoop {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        v.visit_yul_block(&mut self.body)?;
        v.visit_yul_expression(&mut self.condition)?;
        v.visit_yul_block(&mut self.post)?;
        v.visit_yul_block(&mut self.pre)
    }
}

impl Visitable for YulFunctionCall {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        self.arguments.iter_mut().try_for_each(|s| v.visit_yul_expression(s))?;
        v.visit_yul_identifier(&mut self.function_name)
    }
}

impl Visitable for InlineAssembly {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        v.visit_yul_block(&mut self.ast)?;
        self.external_references
            .iter_mut()
            .try_for_each(|s| v.visit_external_inline_assembly_reference(s))?;
        self.flags.iter_mut().try_for_each(|s| v.visit_inline_assembly_flag(s))
    }
}

impl Visitable for Return {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        self.expression.as_mut().map_or_else(|| Ok(()), |n| v.visit_expression(n))
    }
}

impl Visitable for RevertStatement {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        v.visit_function_call(&mut self.error_call)
    }
}

impl Visitable for TryCatchClause {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        v.visit_block(&mut self.block)?;
        self.parameters.as_mut().map_or_else(|| Ok(()), |n| v.visit_parameter_list(n))
    }
}

impl Visitable for TryStatement {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        self.clauses.iter_mut().try_for_each(|s| v.visit_try_catch_clause(s))?;
        v.visit_function_call(&mut self.external_call)
    }
}

impl Visitable for UncheckedBlock {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        self.statements.iter_mut().try_for_each(|s| v.visit_statement(s))
    }
}

impl Visitable for WhileStatement {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        v.visit_block_or_statement(&mut self.body)?;
        v.visit_expression(&mut self.condition)
    }
}

impl Visitable for ExternalInlineAssemblyReference {
    fn visit<V, D>(&mut self, v: &mut V) -> Result<(), VisitError>
    where
        V: Visitor<D> + ?Sized,
    {
        self.suffix.as_mut().map_or_else(|| Ok(()), |n| v.visit_assembly_reference_suffix(n))
    }
}
