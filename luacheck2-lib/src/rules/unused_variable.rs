use super::*;
use std::{collections::BinaryHeap, convert::Infallible};

use full_moon::{
    ast::{self, Ast},
    node::Node,
    tokenizer::{self, Token, TokenReference},
    visitors::Visitor,
};

pub struct UnusedVariableLint;

impl Rule for UnusedVariableLint {
    type Config = ();
    type Error = Infallible;

    fn new(_: Self::Config) -> Result<Self, Self::Error> {
        Ok(UnusedVariableLint)
    }

    fn pass(&self, ast: &Ast) -> Vec<Diagnostic> {
        let mut instructions = BinaryHeap::new();
        let block = ast.nodes();

        instructions.push(Instruction {
            position: (0, 0),
            instruction: InstructionType::ScopeBegin,
        });

        instructions.push(Instruction {
            position: match block.last_stmts() {
                Some(last) => (last.end_position().unwrap().bytes() as u32, 0),
                None => (
                    block
                        .iter_stmts()
                        .last()
                        .and_then(|stmt| stmt.end_position())
                        .map(tokenizer::Position::bytes)
                        .unwrap_or(0) as u32,
                    0,
                ),
            },
            instruction: InstructionType::ScopeEnd,
        });

        let mut visitor = UnusedVariableVisitor { instructions };
        visitor.visit_ast(ast);

        let mut diagnostics = Vec::new();
        let mut scopes: Vec<Vec<Variable>> = Vec::new();

        for instruction in visitor.instructions.into_sorted_vec() {
            match instruction.instruction {
                InstructionType::ScopeBegin => {
                    scopes.push(Vec::new());
                }

                InstructionType::ScopeEnd => {
                    for variable in scopes.pop().unwrap() {
                        match (variable.read, variable.mutated) {
                            (Some(read), Some(mutated)) => {
                                if mutated > read {
                                    // We mutated it after the last time we read it
                                    diagnostics.push(Diagnostic::new(
                                        "unused_variable",
                                        format!(
                                            "assignment to {} is overriden before it can be read",
                                            variable.name
                                        ),
                                        Label::new(mutated),
                                    ));
                                }
                            }

                            (None, Some(_)) => {
                                diagnostics.push(Diagnostic::new(
                                    "unused_variable",
                                    format!("{} is mutated, but never used", variable.name),
                                    Label::new(variable.position),
                                ));
                            }

                            // Constant variable
                            (Some(_), None) => {}

                            (None, None) => {
                                diagnostics.push(Diagnostic::new(
                                    "unused_variable",
                                    format!("{} is unused", variable.name),
                                    Label::new(variable.position),
                                ));
                            }
                        };
                    }
                }

                InstructionType::DeclareVariable(name) => {
                    scopes.last_mut().unwrap().push(Variable {
                        position: instruction.position,
                        name,
                        mutated: None,
                        read: None,
                    });
                }

                InstructionType::MutateVariable(name) => {
                    // TODO: Undefined variable
                    if let Some(variable) = get_variable_from_scopes(&mut scopes, &name) {
                        variable.mutated = Some(instruction.position);
                    }
                }

                InstructionType::ReadVariable(name) => {
                    // TODO: Undefined variable
                    if let Some(variable) = get_variable_from_scopes(&mut scopes, &name) {
                        variable.read = Some(instruction.position);
                    }
                }
            }
        }

        diagnostics
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn rule_type(&self) -> RuleType {
        RuleType::Style
    }
}

struct Variable {
    name: String,
    position: (u32, u32),
    mutated: Option<(u32, u32)>,
    read: Option<(u32, u32)>,
}

fn get_variable_from_scopes<'a>(
    scopes: &'a mut Vec<Vec<Variable>>,
    name: &str,
) -> Option<&'a mut Variable> {
    for scope in scopes.iter_mut().rev() {
        for variable in scope.iter_mut() {
            if variable.name == name {
                return Some(variable);
            }
        }
    }

    None
}

fn range(token: &TokenReference) -> (u32, u32) {
    (
        Token::start_position(&*token).bytes() as u32,
        Token::end_position(&*token).bytes() as u32,
    )
}

struct UnusedVariableVisitor {
    instructions: BinaryHeap<Instruction>,
}

impl UnusedVariableVisitor {
    fn read_expression(&mut self, expression: &ast::Expression) {
        match expression {
            ast::Expression::Parentheses { expression, .. }
            | ast::Expression::UnaryOperator { expression, .. } => {
                self.read_expression(expression);
            }

            ast::Expression::Value { value, binop } => {
                if let Some(binop) = binop {
                    self.read_expression(binop.rhs());
                }

                match &**value {
                    ast::Value::ParseExpression(expression) => {
                        self.read_expression(expression);
                    }

                    ast::Value::Var(var) => match var {
                        ast::Var::Expression(expression) => {
                            self.read_prefix(expression.prefix());

                            for suffix in expression.iter_suffixes() {
                                self.read_suffix(suffix);
                            }
                        }

                        ast::Var::Name(name) => {
                            self.read_name(name);
                        }
                    },

                    _ => {}
                }
            }
        }
    }

    fn read_name(&mut self, name: &TokenReference) {
        self.instructions.push(Instruction {
            position: range(name),
            instruction: InstructionType::ReadVariable(name.to_string()),
        });
    }

    fn read_prefix(&mut self, prefix: &ast::Prefix) {
        if let ast::Prefix::Name(name) = prefix {
            self.read_name(name);
        }
    }

    // TODO: Unused fields
    fn read_suffix(&mut self, _suffix: &ast::Suffix) {
        // if let ast::Suffix::Index(index) = suffix {
        //     if let ast::Index::Dot { name, .. } = index {
        //         self.read_name(name);
        //     }
        // }
    }

    fn declare_name(&mut self, name: &TokenReference) {
        self.instructions.push(Instruction {
            position: range(name),
            instruction: InstructionType::DeclareVariable(name.to_string()),
        });
    }

    fn mutate_name(&mut self, name: &TokenReference) {
        self.instructions.push(Instruction {
            position: range(name),
            instruction: InstructionType::MutateVariable(name.to_string()),
        });
    }

    fn read_block<N: Node>(&mut self, node: N) {
        if let Some((start, end)) = node.range() {
            self.instructions.push(Instruction {
                position: (start.bytes() as u32, 0),
                instruction: InstructionType::ScopeBegin,
            });

            self.instructions.push(Instruction {
                position: (end.bytes() as u32, 0),
                instruction: InstructionType::ScopeEnd,
            });
        }
    }
}

impl Visitor<'_> for UnusedVariableVisitor {
    fn visit_assignment(&mut self, assignment: &ast::Assignment) {
        for var in assignment.var_list() {
            match var {
                ast::Var::Expression(expression) => {
                    self.read_prefix(expression.prefix());
                }

                ast::Var::Name(name) => {
                    self.mutate_name(name);
                }
            };
        }

        for expression in assignment.expr_list() {
            self.read_expression(expression);
        }
    }

    fn visit_call(&mut self, call: &ast::Call) {
        if let ast::Call::MethodCall(method_call) = call {
            self.read_name(method_call.name());
        }
    }

    fn visit_do(&mut self, do_block: &ast::Do) {
        self.read_block(do_block.block());
    }

    fn visit_else_if(&mut self, else_if: &ast::ElseIf) {
        self.read_block(else_if.block());
        self.read_expression(else_if.condition());
    }

    fn visit_function_args(&mut self, args: &ast::FunctionArgs) {
        if let ast::FunctionArgs::Parentheses { arguments, .. } = args {
            for argument in arguments {
                self.read_expression(argument);
            }
        }
    }

    fn visit_function_body(&mut self, body: &ast::FunctionBody) {
        for parameter in body.iter_parameters() {
            match parameter {
                ast::Parameter::Ellipse(token) | ast::Parameter::Name(token) => {
                    self.declare_name(token);
                }
            }
        }

        self.read_block(body.block());
    }

    fn visit_function_call(&mut self, call: &ast::FunctionCall) {
        self.read_prefix(call.prefix());

        for suffix in call.iter_suffixes() {
            self.read_suffix(suffix);
        }
    }

    fn visit_function_declaration(&mut self, declaration: &ast::FunctionDeclaration) {
        let name = declaration.name();

        let mut names = name.names().iter();
        let base = names.next().unwrap();

        if names.next().is_some() {
            self.mutate_name(base);
        } else {
            self.read_name(base);
        }
    }

    fn visit_generic_for(&mut self, generic_for: &ast::GenericFor) {
        for name in generic_for.names() {
            self.declare_name(name);
        }

        for expression in generic_for.expr_list().iter() {
            self.read_expression(expression);
        }

        self.read_block(generic_for.block());
    }

    fn visit_if(&mut self, if_block: &ast::If) {
        self.read_expression(if_block.condition());
        self.read_block(if_block.block());

        if let Some(block) = if_block.else_block() {
            self.read_block(block);
        }
    }

    fn visit_local_assignment(&mut self, node: &ast::LocalAssignment) {
        for name in node.name_list() {
            self.declare_name(name);
        }

        for expression in node.expr_list() {
            self.read_expression(expression);
        }
    }

    fn visit_local_function(&mut self, local_function: &ast::LocalFunction) {
        self.declare_name(local_function.name());
        self.read_block(local_function.func_body().block());
    }

    fn visit_numeric_for(&mut self, numeric_for: &ast::NumericFor) {
        self.declare_name(numeric_for.index_variable());
        self.read_expression(numeric_for.start());
        self.read_expression(numeric_for.end());

        if let Some(step) = numeric_for.step() {
            self.read_expression(step);
        }

        self.read_block(numeric_for.block());
    }

    fn visit_repeat(&mut self, repeat: &ast::Repeat) {
        self.read_expression(repeat.until());

        // Variables inside the read block are accessible in the until
        // So we read the entire statement, not just repeat.block()
        self.read_block(repeat);
    }

    fn visit_return(&mut self, return_stmt: &ast::Return) {
        for value in return_stmt.returns() {
            self.read_expression(value);
        }
    }

    fn visit_table_constructor(&mut self, table: &ast::TableConstructor) {
        for (field, _) in table.iter_fields() {
            match field {
                ast::Field::ExpressionKey { key, value, .. } => {
                    self.read_expression(key);
                    self.read_expression(value);
                }

                ast::Field::NameKey { value, .. } | ast::Field::NoKey(value) => {
                    self.read_expression(value);
                }
            }
        }
    }

    fn visit_while(&mut self, while_loop: &ast::While) {
        self.read_expression(while_loop.condition());
        self.read_block(while_loop.block());
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Instruction {
    position: (u32, u32),
    instruction: InstructionType,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum InstructionType {
    ScopeBegin,
    ScopeEnd,

    DeclareVariable(String),
    MutateVariable(String),
    ReadVariable(String),
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    fn test_unused_blocks() {
        test_lint(
            UnusedVariableLint::new(()).unwrap(),
            "unused_variable",
            "blocks",
        );
    }

    #[test]
    fn test_unused_locals() {
        test_lint(
            UnusedVariableLint::new(()).unwrap(),
            "unused_variable",
            "locals",
        );
    }

    #[test]
    fn test_edge_cases() {
        test_lint(
            UnusedVariableLint::new(()).unwrap(),
            "unused_variable",
            "edge_cases",
        );
    }
}
