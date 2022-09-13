use super::*;
use crate::ast_util::range;
use std::convert::Infallible;


use full_moon::{
    ast::{self, Ast},
    // node::Node,
    visitors::Visitor,
};

use serde::Deserialize;

#[derive(Clone, Copy, Deserialize)]
pub struct LimitFunctionComplexityConfig {
    maximum_complexity: u16,
}

impl Default for LimitFunctionComplexityConfig {
    fn default() -> Self {
        Self {
            // eslint defaults to 20, but testing on OSS Lua shows that 20 is too aggressive
            maximum_complexity: 40.to_owned(),
        }
    }
}


#[derive(Default)]
pub struct LimitFunctionComplexityLint {
    config: LimitFunctionComplexityConfig
}

impl Rule for LimitFunctionComplexityLint {
    type Config = LimitFunctionComplexityConfig;
    type Error = Infallible;

    const SEVERITY: Severity = Severity::Warning;
    const RULE_TYPE: RuleType = RuleType::Style;

    fn new(config: Self::Config) -> Result<Self, Self::Error> {
        Ok(LimitFunctionComplexityLint { config })
    }

    fn pass(&self, ast: &Ast, _: &Context, _: &AstContext) -> Vec<Diagnostic> {
        let mut visitor = LimitFunctionComplexityVisitor {
            positions: Vec::new(),
            config: self.config,
        };

        visitor.visit_ast(ast);

        visitor
            .positions
            .into_iter()
            .map(|position| {
                Diagnostic::new(
                    "limit_function_complexity",
                    format!(
                        "function has cyclomatic complexity of {}, exceeding configured limit of {}",
                        position.1,
                        self.config.maximum_complexity
                    ).to_owned(),
                    Label::new(position.0),
                )
            })
            .collect()
    }
}

struct LimitFunctionComplexityVisitor {
    positions: Vec<((u32, u32), u16)>,
    config: LimitFunctionComplexityConfig,
}

fn count_expression_complexity(expression: &ast::Expression, starting_complexity: u16) -> u16 {
    let mut complexity = starting_complexity;

    match expression {
        ast::Expression::Parentheses { expression, .. } => {
            count_expression_complexity(expression, complexity)
        },
        ast::Expression::Value { value, .. } => match &**value {
            #[cfg(feature = "roblox")]
            ast::Value::IfExpression(if_expression) => {
                complexity += 1;
                if let Some(else_if_expressions) = if_expression.else_if_expressions() {
                    for else_if_expression in else_if_expressions {
                        complexity += 1;
                        complexity = count_expression_complexity(else_if_expression.expression(), complexity);
                    }
                }
                complexity
            },
            ast::Value::ParenthesesExpression(paren_expression) => {
                return count_expression_complexity(paren_expression, complexity)
            },
            ast::Value::FunctionCall(call) => {
                for suffix in call.suffixes() {
                    if let ast::Suffix::Call(ast::Call::AnonymousCall(
                        ast::FunctionArgs::Parentheses { arguments, .. }
                    )) = suffix {
                        for argument in arguments {
                            complexity = count_expression_complexity(&argument, complexity)
                        }
                    }
                }
                return complexity;
            },
            ast::Value::TableConstructor(table) => {
                for field in table.fields() {
                    match field {
                        ast::Field::ExpressionKey { key, value, .. } => {
                            complexity = count_expression_complexity(key, complexity);
                            complexity = count_expression_complexity(value, complexity);
                        },

                        ast::Field::NameKey { value, .. } => {
                            complexity = count_expression_complexity(value, complexity);
                        },

                        ast::Field::NoKey(expression) => {
                            complexity = count_expression_complexity(expression, complexity);
                        },

                        _ => {
                            return complexity;
                        }
                    }
                }
                return complexity;
            },
            _ => {
                return complexity;
            },
        },
        ast::Expression::BinaryOperator {
            lhs, binop, rhs, ..
        } => {
            match binop {
                #[cfg_attr(
                    feature = "force_exhaustive_checks",
                    allow(non_exhaustive_omitted_patterns)
                )]
                | ast::BinOp::And(_)
                | ast::BinOp::Or(_) =>
                {
                    complexity += 1;
                    complexity = count_expression_complexity(lhs, complexity);
                    complexity = count_expression_complexity(rhs, complexity);
                    return complexity;
                },
                _ => {
                    return complexity;
                },
            }
        }
        _ => {
            return complexity;
        }
    }
}

fn count_block_complexity(block: &ast::Block, starting_complexity: u16) -> u16 {
    let mut complexity = starting_complexity;
    for statement in block.stmts() {

        if let ast::Stmt::If(if_block) = statement {
            complexity += 1;
            complexity = count_expression_complexity(if_block.condition(), complexity);
            complexity = count_block_complexity(if_block.block(), complexity);

            if let Some(else_if_statements) = if_block.else_if() {
                for else_if in else_if_statements {
                    complexity += 1;
                    complexity = count_expression_complexity(else_if.condition(), complexity);
                    complexity = count_block_complexity(else_if.block(), complexity);
                }
            }
        }

        if let ast::Stmt::While(while_block) = statement {
            complexity = count_expression_complexity(while_block.condition(), complexity + 1);
            complexity = count_block_complexity(while_block.block(), complexity);
        }

        if let ast::Stmt::Repeat(repeat_block) = statement {
            complexity = count_expression_complexity(repeat_block.until(), complexity + 1);
            complexity = count_block_complexity(repeat_block.block(), complexity);
        }

        if let ast::Stmt::NumericFor(numeric_for) = statement {
            complexity += 1;
            complexity = count_expression_complexity(numeric_for.start(), complexity);
            complexity = count_expression_complexity(numeric_for.end(), complexity);

            if let Some(step_expression) = numeric_for.step() {
                complexity = count_expression_complexity(step_expression, complexity);
            }

            complexity = count_block_complexity(numeric_for.block(), complexity);
        }

        if let ast::Stmt::GenericFor(generic_for) = statement {
            complexity += 1;
            for expression in generic_for.expressions() {
                complexity = count_expression_complexity(expression, complexity);
                complexity = count_block_complexity(generic_for.block(), complexity);
            }
        }

        if let ast::Stmt::Assignment(assignment) = statement {
            for expression in assignment.expressions() {
                complexity = count_expression_complexity(expression, complexity);
            }
        }

        if let ast::Stmt::LocalAssignment(local_assignment) = statement {
            for expression in local_assignment.expressions() {
                complexity = count_expression_complexity(expression, complexity);
            }
        }

        if let ast::Stmt::FunctionCall(call) = statement {
            for suffix in call.suffixes() {
                if let ast::Suffix::Call(ast::Call::AnonymousCall(
                    ast::FunctionArgs::Parentheses { arguments, .. }
                )) = suffix {
                    for argument in arguments {
                        complexity = count_expression_complexity(&argument, complexity)
                    }
                }
            }
            return complexity;
        }
    };

    if let Some(last_statement) = block.last_stmt() {
        if let ast::LastStmt::Return(return_) = last_statement {
            for return_expression in return_.returns() {
                complexity = count_expression_complexity(return_expression, complexity);
            }
        }
    }

    return complexity;
}

impl Visitor for LimitFunctionComplexityVisitor {
    fn visit_function_body(&mut self, function_body: &ast::FunctionBody) {
        let complexity = count_block_complexity(function_body.block(), 1);
        if complexity > self.config.maximum_complexity {
            self.positions.push((range(function_body.parameters_parentheses()), complexity));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super
        ::test_util::test_lint, *};

    #[test]
    fn test_limit_function_complexity() {
        test_lint(
            LimitFunctionComplexityLint::new(LimitFunctionComplexityConfig::default()).unwrap(),
            "limit_function_complexity",
            "limit_function_complexity",
        );
    }
}
