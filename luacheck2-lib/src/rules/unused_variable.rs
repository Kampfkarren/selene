use super::*;
use std::{cmp::Ordering, collections::BinaryHeap, convert::Infallible};

use full_moon::{
    ast::{self, Ast},
    node::Node,
    tokenizer::{self, Token, TokenKind, TokenReference},
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
            position: 0,
            instruction: InstructionType::ScopeBegin,
        });

        instructions.push(Instruction {
            position: match block.last_stmts() {
                Some(last) => last.end_position().unwrap().bytes(),
                None => block
                    .iter_stmts()
                    .last()
                    .and_then(|stmt| stmt.end_position())
                    .map(tokenizer::Position::bytes)
                    .unwrap_or(0),
            },
            instruction: InstructionType::ScopeEnd,
        });

        let mut visitor = UnusedVariableVisitor { instructions };
        visitor.visit_ast(ast);

        let instructions = visitor.instructions.into_sorted_vec();

        println!("{:#?}", instructions);
        unimplemented!();
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn rule_type(&self) -> RuleType {
        RuleType::Style
    }
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
            position: Token::start_position(&*name).bytes(),
            instruction: InstructionType::ReadVariable(name.to_string()),
        });
    }

    fn read_prefix(&mut self, prefix: &ast::Prefix) {
        if let ast::Prefix::Name(name) = prefix {
            self.read_name(name);
        }
    }

    // TODO: Unused fields
    fn read_suffix(&mut self, suffix: &ast::Suffix) {
        // if let ast::Suffix::Index(index) = suffix {
        //     if let ast::Index::Dot { name, .. } = index {
        //         self.read_name(name);
        //     }
        // }
    }

    fn declare_name(&mut self, name: &TokenReference) {
        self.instructions.push(Instruction {
            position: Token::start_position(&*name).bytes(),
            instruction: InstructionType::DeclareVariable(name.to_string()),
        });
    }

    fn mutate_name(&mut self, name: &TokenReference) {
        self.instructions.push(Instruction {
            position: Token::start_position(&*name).bytes(),
            instruction: InstructionType::MutateVariable(name.to_string()),
        });
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
    }

    fn visit_call(&mut self, call: &ast::Call) {
        if let ast::Call::MethodCall(method_call) = call {
            self.read_name(method_call.name());
        }
    }

    fn visit_else_if(&mut self, else_if: &ast::ElseIf) {
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
    }

    fn visit_function_call(&mut self, call: &ast::FunctionCall) {
        self.read_prefix(call.prefix());

        for suffix in call.iter_suffixes() {
            self.read_suffix(suffix);
        }
    }

    fn visit_function_declaration(&mut self, declaration: &ast::FunctionDeclaration) {
        let name = declaration.name();
        let name = name.names().iter().next().unwrap();
        self.declare_name(name);
    }

    fn visit_generic_for(&mut self, generic_for: &ast::GenericFor) {
        for name in generic_for.names() {
            self.declare_name(name);
        }

        for expression in generic_for.expr_list().iter() {
            self.read_expression(expression);
        }
    }

    fn visit_if(&mut self, if_block: &ast::If) {
        self.read_expression(if_block.condition());
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
    }

    fn visit_numeric_for(&mut self, numeric_for: &ast::NumericFor) {
        self.declare_name(numeric_for.index_variable());
        self.read_expression(numeric_for.start());
        self.read_expression(numeric_for.end());

        if let Some(step) = numeric_for.step() {
            self.read_expression(step);
        }
    }

    fn visit_repeat(&mut self, repeat: &ast::Repeat) {
        self.read_expression(repeat.until());
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
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Instruction {
    position: usize,
    instruction: InstructionType,
}

// We don't use #[derive(PartialOrd, Ord)] because then InstructionType would dictate order
impl PartialOrd for Instruction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for Instruction {
    fn cmp(&self, other: &Self) -> Ordering {
        self.position.cmp(&other.position)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum InstructionType {
    DeclareVariable(String),
    MutateVariable(String),
    ReadVariable(String),

    ScopeBegin,
    ScopeEnd,
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    fn test_unused_locals() {
        test_lint(
            UnusedVariableLint::new(()).unwrap(),
            "unused_variable",
            "locals",
        );
    }
}
