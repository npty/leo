// Copyright (C) 2019-2021 Aleo Systems Inc.
// This file is part of the Leo library.

// The Leo library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The Leo library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the Leo library. If not, see <https://www.gnu.org/licenses/>.

//! This module contains the reducer which iterates through ast nodes - converting them into
//! asg nodes and saving relevant information.

use crate::*;
use indexmap::IndexMap;

pub struct ReconstructingDirector<R: ReconstructingReducer> {
    reducer: R,
    in_circuit: bool,
}

impl<R: ReconstructingReducer> ReconstructingDirector<R> {
    pub fn new(reducer: R) -> Self {
        Self {
            reducer,
            in_circuit: false,
        }
    }

    pub fn reduce_type(&mut self, type_: &Type, span: &Span) -> Result<Type, CanonicalizeError> {
        let new = match type_ {
            Type::Array(type_, dimensions) => Type::Array(Box::new(self.reduce_type(type_, span)?), dimensions.clone()),
            Type::Tuple(types) => {
                let mut reduced_types = vec![];
                for type_ in types.iter() {
                    reduced_types.push(self.reduce_type(type_, span)?);
                }

                Type::Tuple(reduced_types)
            }
            Type::Circuit(identifier) => Type::Circuit(self.reduce_identifier(identifier)?),
            _ => type_.clone(),
        };

        self.reducer.reduce_type(type_, new, self.in_circuit, span)
    }

    // Expressions
    pub fn reduce_expression(&mut self, expression: &Expression) -> Result<Expression, CanonicalizeError> {
        let new = match expression {
            Expression::Identifier(identifier) => Expression::Identifier(self.reduce_identifier(&identifier)?),
            Expression::Value(value) => Expression::Value(self.reduce_value(&value)?),
            Expression::Binary(binary) => Expression::Binary(self.reduce_binary(&binary)?),
            Expression::Unary(unary) => Expression::Unary(self.reduce_unary(&unary)?),
            Expression::Ternary(ternary) => Expression::Ternary(self.reduce_ternary(&ternary)?),
            Expression::Cast(cast) => Expression::Cast(self.reduce_cast(&cast)?),

            Expression::ArrayInline(array_inline) => Expression::ArrayInline(self.reduce_array_inline(&array_inline)?),
            Expression::ArrayInit(array_init) => Expression::ArrayInit(self.reduce_array_init(&array_init)?),
            Expression::ArrayAccess(array_access) => Expression::ArrayAccess(self.reduce_array_access(&array_access)?),
            Expression::ArrayRangeAccess(array_range_access) => {
                Expression::ArrayRangeAccess(self.reduce_array_range_access(&array_range_access)?)
            }

            Expression::TupleInit(tuple_init) => Expression::TupleInit(self.reduce_tuple_init(&tuple_init)?),
            Expression::TupleAccess(tuple_access) => Expression::TupleAccess(self.reduce_tuple_access(&tuple_access)?),

            Expression::CircuitInit(circuit_init) => Expression::CircuitInit(self.reduce_circuit_init(&circuit_init)?),
            Expression::CircuitMemberAccess(circuit_member_access) => {
                Expression::CircuitMemberAccess(self.reduce_circuit_member_access(&circuit_member_access)?)
            }
            Expression::CircuitStaticFunctionAccess(circuit_static_fn_access) => {
                Expression::CircuitStaticFunctionAccess(
                    self.reduce_circuit_static_fn_access(&circuit_static_fn_access)?,
                )
            }

            Expression::Call(call) => Expression::Call(self.reduce_call(&call)?),
        };

        self.reducer.reduce_expression(expression, new, self.in_circuit)
    }

    pub fn reduce_identifier(&mut self, identifier: &Identifier) -> Result<Identifier, CanonicalizeError> {
        self.reducer.reduce_identifier(identifier)
    }

    pub fn reduce_group_tuple(&mut self, group_tuple: &GroupTuple) -> Result<GroupTuple, CanonicalizeError> {
        self.reducer.reduce_group_tuple(group_tuple)
    }

    pub fn reduce_group_value(&mut self, group_value: &GroupValue) -> Result<GroupValue, CanonicalizeError> {
        let new = match group_value {
            GroupValue::Tuple(group_tuple) => GroupValue::Tuple(self.reduce_group_tuple(&group_tuple)?),
            _ => group_value.clone(),
        };

        self.reducer.reduce_group_value(group_value, new)
    }

    pub fn reduce_value(&mut self, value: &ValueExpression) -> Result<ValueExpression, CanonicalizeError> {
        let new = match value {
            ValueExpression::Group(group_value) => {
                ValueExpression::Group(Box::new(self.reduce_group_value(&group_value)?))
            }
            _ => value.clone(),
        };

        self.reducer.reduce_value(value, new)
    }

    pub fn reduce_binary(&mut self, binary: &BinaryExpression) -> Result<BinaryExpression, CanonicalizeError> {
        let left = self.reduce_expression(&binary.left)?;
        let right = self.reduce_expression(&binary.right)?;

        self.reducer
            .reduce_binary(binary, left, right, binary.op.clone(), self.in_circuit)
    }

    pub fn reduce_unary(&mut self, unary: &UnaryExpression) -> Result<UnaryExpression, CanonicalizeError> {
        let inner = self.reduce_expression(&unary.inner)?;

        self.reducer
            .reduce_unary(unary, inner, unary.op.clone(), self.in_circuit)
    }

    pub fn reduce_ternary(&mut self, ternary: &TernaryExpression) -> Result<TernaryExpression, CanonicalizeError> {
        let condition = self.reduce_expression(&ternary.condition)?;
        let if_true = self.reduce_expression(&ternary.if_true)?;
        let if_false = self.reduce_expression(&ternary.if_false)?;

        self.reducer
            .reduce_ternary(ternary, condition, if_true, if_false, self.in_circuit)
    }

    pub fn reduce_cast(&mut self, cast: &CastExpression) -> Result<CastExpression, CanonicalizeError> {
        let inner = self.reduce_expression(&cast.inner)?;
        let target_type = self.reduce_type(&cast.target_type, &cast.span)?;

        self.reducer.reduce_cast(cast, inner, target_type, self.in_circuit)
    }

    pub fn reduce_array_inline(
        &mut self,
        array_inline: &ArrayInlineExpression,
    ) -> Result<ArrayInlineExpression, CanonicalizeError> {
        let mut elements = vec![];
        for element in array_inline.elements.iter() {
            let reduced_element = match element {
                SpreadOrExpression::Expression(expression) => {
                    SpreadOrExpression::Expression(self.reduce_expression(expression)?)
                }
                SpreadOrExpression::Spread(expression) => {
                    SpreadOrExpression::Spread(self.reduce_expression(expression)?)
                }
            };

            elements.push(reduced_element);
        }

        self.reducer
            .reduce_array_inline(array_inline, elements, self.in_circuit)
    }

    pub fn reduce_array_init(
        &mut self,
        array_init: &ArrayInitExpression,
    ) -> Result<ArrayInitExpression, CanonicalizeError> {
        let element = self.reduce_expression(&array_init.element)?;

        self.reducer.reduce_array_init(array_init, element, self.in_circuit)
    }

    pub fn reduce_array_access(
        &mut self,
        array_access: &ArrayAccessExpression,
    ) -> Result<ArrayAccessExpression, CanonicalizeError> {
        let array = self.reduce_expression(&array_access.array)?;
        let index = self.reduce_expression(&array_access.index)?;

        self.reducer
            .reduce_array_access(array_access, array, index, self.in_circuit)
    }

    pub fn reduce_array_range_access(
        &mut self,
        array_range_access: &ArrayRangeAccessExpression,
    ) -> Result<ArrayRangeAccessExpression, CanonicalizeError> {
        let array = self.reduce_expression(&array_range_access.array)?;
        let left = array_range_access
            .left
            .as_ref()
            .map(|left| self.reduce_expression(left))
            .transpose()?;
        let right = array_range_access
            .right
            .as_ref()
            .map(|right| self.reduce_expression(right))
            .transpose()?;

        self.reducer
            .reduce_array_range_access(array_range_access, array, left, right, self.in_circuit)
    }

    pub fn reduce_tuple_init(
        &mut self,
        tuple_init: &TupleInitExpression,
    ) -> Result<TupleInitExpression, CanonicalizeError> {
        let mut elements = vec![];
        for element in tuple_init.elements.iter() {
            elements.push(self.reduce_expression(element)?);
        }

        self.reducer.reduce_tuple_init(tuple_init, elements, self.in_circuit)
    }

    pub fn reduce_tuple_access(
        &mut self,
        tuple_access: &TupleAccessExpression,
    ) -> Result<TupleAccessExpression, CanonicalizeError> {
        let tuple = self.reduce_expression(&tuple_access.tuple)?;

        self.reducer.reduce_tuple_access(tuple_access, tuple, self.in_circuit)
    }

    pub fn reduce_circuit_implied_variable_definition(
        &mut self,
        variable: &CircuitImpliedVariableDefinition,
    ) -> Result<CircuitImpliedVariableDefinition, CanonicalizeError> {
        let identifier = self.reduce_identifier(&variable.identifier)?;
        let expression = variable
            .expression
            .as_ref()
            .map(|expr| self.reduce_expression(expr))
            .transpose()?;

        self.reducer
            .reduce_circuit_implied_variable_definition(variable, identifier, expression, self.in_circuit)
    }

    pub fn reduce_circuit_init(
        &mut self,
        circuit_init: &CircuitInitExpression,
    ) -> Result<CircuitInitExpression, CanonicalizeError> {
        let name = self.reduce_identifier(&circuit_init.name)?;

        let mut members = vec![];
        for member in circuit_init.members.iter() {
            members.push(self.reduce_circuit_implied_variable_definition(member)?);
        }

        self.reducer
            .reduce_circuit_init(circuit_init, name, members, self.in_circuit)
    }

    pub fn reduce_circuit_member_access(
        &mut self,
        circuit_member_access: &CircuitMemberAccessExpression,
    ) -> Result<CircuitMemberAccessExpression, CanonicalizeError> {
        let circuit = self.reduce_expression(&circuit_member_access.circuit)?;
        let name = self.reduce_identifier(&circuit_member_access.name)?;

        self.reducer
            .reduce_circuit_member_access(circuit_member_access, circuit, name, self.in_circuit)
    }

    pub fn reduce_circuit_static_fn_access(
        &mut self,
        circuit_static_fn_access: &CircuitStaticFunctionAccessExpression,
    ) -> Result<CircuitStaticFunctionAccessExpression, CanonicalizeError> {
        let circuit = self.reduce_expression(&circuit_static_fn_access.circuit)?;
        let name = self.reduce_identifier(&circuit_static_fn_access.name)?;

        self.reducer
            .reduce_circuit_static_fn_access(circuit_static_fn_access, circuit, name, self.in_circuit)
    }

    pub fn reduce_call(&mut self, call: &CallExpression) -> Result<CallExpression, CanonicalizeError> {
        let function = self.reduce_expression(&call.function)?;

        let mut arguments = vec![];
        for argument in call.arguments.iter() {
            arguments.push(self.reduce_expression(argument)?);
        }

        self.reducer.reduce_call(call, function, arguments, self.in_circuit)
    }

    // Statements
    pub fn reduce_statement(&mut self, statement: &Statement) -> Result<Statement, CanonicalizeError> {
        let new = match statement {
            Statement::Return(return_statement) => Statement::Return(self.reduce_return(&return_statement)?),
            Statement::Definition(definition) => Statement::Definition(self.reduce_definition(&definition)?),
            Statement::Assign(assign) => Statement::Assign(self.reduce_assign(&assign)?),
            Statement::Conditional(conditional) => Statement::Conditional(self.reduce_conditional(&conditional)?),
            Statement::Iteration(iteration) => Statement::Iteration(self.reduce_iteration(&iteration)?),
            Statement::Console(console) => Statement::Console(self.reduce_console(&console)?),
            Statement::Expression(expression) => Statement::Expression(self.reduce_expression_statement(&expression)?),
            Statement::Block(block) => Statement::Block(self.reduce_block(&block)?),
        };

        self.reducer.reduce_statement(statement, new, self.in_circuit)
    }

    pub fn reduce_return(&mut self, return_statement: &ReturnStatement) -> Result<ReturnStatement, CanonicalizeError> {
        let expression = self.reduce_expression(&return_statement.expression)?;

        self.reducer
            .reduce_return(return_statement, expression, self.in_circuit)
    }

    pub fn reduce_variable_name(&mut self, variable_name: &VariableName) -> Result<VariableName, CanonicalizeError> {
        let identifier = self.reduce_identifier(&variable_name.identifier)?;

        self.reducer.reduce_variable_name(variable_name, identifier)
    }

    pub fn reduce_definition(
        &mut self,
        definition: &DefinitionStatement,
    ) -> Result<DefinitionStatement, CanonicalizeError> {
        let mut variable_names = vec![];
        for variable_name in definition.variable_names.iter() {
            variable_names.push(self.reduce_variable_name(variable_name)?);
        }

        let type_ = definition
            .type_
            .as_ref()
            .map(|type_| self.reduce_type(type_, &definition.span))
            .transpose()?;

        let value = self.reduce_expression(&definition.value)?;

        self.reducer
            .reduce_definition(definition, variable_names, type_, value, self.in_circuit)
    }

    pub fn reduce_assignee_access(&mut self, access: &AssigneeAccess) -> Result<AssigneeAccess, CanonicalizeError> {
        let new = match access {
            AssigneeAccess::ArrayRange(left, right) => {
                let left = left.as_ref().map(|left| self.reduce_expression(left)).transpose()?;
                let right = right.as_ref().map(|right| self.reduce_expression(right)).transpose()?;

                AssigneeAccess::ArrayRange(left, right)
            }
            AssigneeAccess::ArrayIndex(index) => AssigneeAccess::ArrayIndex(self.reduce_expression(&index)?),
            AssigneeAccess::Member(identifier) => AssigneeAccess::Member(self.reduce_identifier(&identifier)?),
            _ => access.clone(),
        };

        self.reducer.reduce_assignee_access(access, new, self.in_circuit)
    }

    pub fn reduce_assignee(&mut self, assignee: &Assignee) -> Result<Assignee, CanonicalizeError> {
        let identifier = self.reduce_identifier(&assignee.identifier)?;

        let mut accesses = vec![];
        for access in assignee.accesses.iter() {
            accesses.push(self.reduce_assignee_access(access)?);
        }

        self.reducer
            .reduce_assignee(assignee, identifier, accesses, self.in_circuit)
    }

    pub fn reduce_assign(&mut self, assign: &AssignStatement) -> Result<AssignStatement, CanonicalizeError> {
        let assignee = self.reduce_assignee(&assign.assignee)?;
        let value = self.reduce_expression(&assign.value)?;

        self.reducer.reduce_assign(assign, assignee, value, self.in_circuit)
    }

    pub fn reduce_conditional(
        &mut self,
        conditional: &ConditionalStatement,
    ) -> Result<ConditionalStatement, CanonicalizeError> {
        let condition = self.reduce_expression(&conditional.condition)?;
        let block = self.reduce_block(&conditional.block)?;
        let next = conditional
            .next
            .as_ref()
            .map(|condition| self.reduce_statement(condition))
            .transpose()?;

        self.reducer
            .reduce_conditional(conditional, condition, block, next, self.in_circuit)
    }

    pub fn reduce_iteration(
        &mut self,
        iteration: &IterationStatement,
    ) -> Result<IterationStatement, CanonicalizeError> {
        let variable = self.reduce_identifier(&iteration.variable)?;
        let start = self.reduce_expression(&iteration.start)?;
        let stop = self.reduce_expression(&iteration.stop)?;
        let block = self.reduce_block(&iteration.block)?;

        self.reducer
            .reduce_iteration(iteration, variable, start, stop, block, self.in_circuit)
    }

    pub fn reduce_console(
        &mut self,
        console_function_call: &ConsoleStatement,
    ) -> Result<ConsoleStatement, CanonicalizeError> {
        let function = match &console_function_call.function {
            ConsoleFunction::Assert(expression) => ConsoleFunction::Assert(self.reduce_expression(expression)?),
            ConsoleFunction::Debug(format) | ConsoleFunction::Error(format) | ConsoleFunction::Log(format) => {
                let mut parameters = vec![];
                for parameter in format.parameters.iter() {
                    parameters.push(self.reduce_expression(parameter)?);
                }

                let formatted = FormatString {
                    parts: format.parts.clone(),
                    parameters,
                    span: format.span.clone(),
                };

                match &console_function_call.function {
                    ConsoleFunction::Debug(_) => ConsoleFunction::Debug(formatted),
                    ConsoleFunction::Error(_) => ConsoleFunction::Error(formatted),
                    ConsoleFunction::Log(_) => ConsoleFunction::Log(formatted),
                    _ => unimplemented!(), // impossible
                }
            }
        };

        self.reducer
            .reduce_console(console_function_call, function, self.in_circuit)
    }

    pub fn reduce_expression_statement(
        &mut self,
        expression: &ExpressionStatement,
    ) -> Result<ExpressionStatement, CanonicalizeError> {
        let inner_expression = self.reduce_expression(&expression.expression)?;
        self.reducer
            .reduce_expression_statement(expression, inner_expression, self.in_circuit)
    }

    pub fn reduce_block(&mut self, block: &Block) -> Result<Block, CanonicalizeError> {
        let mut statements = vec![];
        for statement in block.statements.iter() {
            statements.push(self.reduce_statement(statement)?);
        }

        self.reducer.reduce_block(block, statements, self.in_circuit)
    }

    // Program
    pub fn reduce_program(&mut self, program: &Program) -> Result<Program, CanonicalizeError> {
        let mut inputs = vec![];
        for input in program.expected_input.iter() {
            inputs.push(self.reduce_function_input(input)?);
        }

        let mut imports = vec![];
        for import in program.imports.iter() {
            imports.push(self.reduce_import(import)?);
        }

        let mut circuits = IndexMap::new();
        for (identifier, circuit) in program.circuits.iter() {
            circuits.insert(self.reduce_identifier(identifier)?, self.reduce_circuit(circuit)?);
        }

        let mut functions = IndexMap::new();
        for (identifier, function) in program.functions.iter() {
            functions.insert(self.reduce_identifier(identifier)?, self.reduce_function(function)?);
        }

        self.reducer
            .reduce_program(program, inputs, imports, circuits, functions)
    }

    pub fn reduce_function_input_variable(
        &mut self,
        variable: &FunctionInputVariable,
    ) -> Result<FunctionInputVariable, CanonicalizeError> {
        let identifier = self.reduce_identifier(&variable.identifier)?;
        let type_ = self.reduce_type(&variable.type_, &variable.span)?;

        self.reducer
            .reduce_function_input_variable(variable, identifier, type_, self.in_circuit)
    }

    pub fn reduce_function_input(&mut self, input: &FunctionInput) -> Result<FunctionInput, CanonicalizeError> {
        let new = match input {
            FunctionInput::Variable(function_input_variable) => {
                FunctionInput::Variable(self.reduce_function_input_variable(function_input_variable)?)
            }
            _ => input.clone(),
        };

        self.reducer.reduce_function_input(input, new, self.in_circuit)
    }

    pub fn reduce_package_or_packages(
        &mut self,
        package_or_packages: &PackageOrPackages,
    ) -> Result<PackageOrPackages, CanonicalizeError> {
        let new = match package_or_packages {
            PackageOrPackages::Package(package) => PackageOrPackages::Package(Package {
                name: self.reduce_identifier(&package.name)?,
                access: package.access.clone(),
                span: package.span.clone(),
            }),
            PackageOrPackages::Packages(packages) => PackageOrPackages::Packages(Packages {
                name: self.reduce_identifier(&packages.name)?,
                accesses: packages.accesses.clone(),
                span: packages.span.clone(),
            }),
        };

        self.reducer.reduce_package_or_packages(package_or_packages, new)
    }

    pub fn reduce_import(&mut self, import: &ImportStatement) -> Result<ImportStatement, CanonicalizeError> {
        let package_or_packages = self.reduce_package_or_packages(&import.package_or_packages)?;

        self.reducer.reduce_import(import, package_or_packages)
    }

    pub fn reduce_circuit_member(
        &mut self,
        circuit_member: &CircuitMember,
    ) -> Result<CircuitMember, CanonicalizeError> {
        self.in_circuit = !self.in_circuit;
        let new = match circuit_member {
            CircuitMember::CircuitVariable(identifier, type_) => CircuitMember::CircuitVariable(
                self.reduce_identifier(&identifier)?,
                self.reduce_type(&type_, &identifier.span)?,
            ),
            CircuitMember::CircuitFunction(function) => {
                CircuitMember::CircuitFunction(self.reduce_function(&function)?)
            }
        };
        self.in_circuit = !self.in_circuit;

        self.reducer.reduce_circuit_member(circuit_member, new)
    }

    pub fn reduce_circuit(&mut self, circuit: &Circuit) -> Result<Circuit, CanonicalizeError> {
        let circuit_name = self.reduce_identifier(&circuit.circuit_name)?;

        let mut members = vec![];
        for member in circuit.members.iter() {
            members.push(self.reduce_circuit_member(member)?);
        }

        self.reducer.reduce_circuit(circuit, circuit_name, members)
    }

    fn reduce_annotation(&mut self, annotation: &Annotation) -> Result<Annotation, CanonicalizeError> {
        let name = self.reduce_identifier(&annotation.name)?;

        self.reducer.reduce_annotation(annotation, name)
    }

    pub fn reduce_function(&mut self, function: &Function) -> Result<Function, CanonicalizeError> {
        let identifier = self.reduce_identifier(&function.identifier)?;

        let mut annotations = vec![];
        for annotation in function.annotations.iter() {
            annotations.push(self.reduce_annotation(annotation)?);
        }

        let mut inputs = vec![];
        for input in function.input.iter() {
            inputs.push(self.reduce_function_input(input)?);
        }

        let output = function
            .output
            .as_ref()
            .map(|type_| self.reduce_type(type_, &function.span))
            .transpose()?;

        let block = self.reduce_block(&function.block)?;

        self.reducer.reduce_function(
            function,
            identifier,
            annotations,
            inputs,
            output,
            block,
            self.in_circuit,
        )
    }
}
