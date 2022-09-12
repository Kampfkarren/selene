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
            maximum_complexity: 100.to_owned(),
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
            .iter()
            .map(|position| {
                Diagnostic::new_complete(
                    "limit_function_complexity",
                    "this has highly complex branch states".to_owned(),
                    Label::new((position.0, position.1)),
                    Vec::new(),
                    Vec::new(),
                )
            })
            .collect()
    }
}

struct LimitFunctionComplexityVisitor {
    positions: Vec<(u32, u32)>,
    config: LimitFunctionComplexityConfig,
}

fn count_expression_complexity(expression: &ast::Expression, starting_complexity: u16) -> u16 {
    let mut complexity = starting_complexity;
    println!("** starting expression complexity: {}", complexity);

    match expression {
        ast::Expression::Parentheses { expression, .. } => {
            complexity = count_expression_complexity(expression, complexity);
            println!("** parenthesis expression complexity: {}", complexity);
            return complexity;
        },
        ast::Expression::Value { value, .. } => match &**value {
            #[cfg(feature = "roblox")]
            ast::Value::IfExpression(if_expression) => {
                complexity += 1;
                println!("** if-expression complexity: {}", complexity);
                if let Some(else_if_expressions) = if_expression.else_if_expressions() {
                    for else_if_expression in else_if_expressions {
                        complexity += 1;
                        complexity = count_expression_complexity(else_if_expression.expression(), complexity);
                        println!("** elseif-expression complexity: {}", complexity);
                    }
                }
                return complexity;
            },
            ast::Value::ParenthesesExpression(paren_expression) => {
                return count_expression_complexity(paren_expression, complexity)
            },
            ast::Value::FunctionCall(call) => {
                println!("** FunctionCall complexity: {}", complexity);
                // Check that we're calling it with an argument
                if let Some(ast::Suffix::Call(call)) = call.suffixes().next() {
                    println!("** suffix::Call complexity: {}", complexity);

                    if let ast::Call::AnonymousCall(ast::FunctionArgs::Parentheses { arguments, .. }) = call {
                        println!("** AnonymousCall complexity: {}", complexity);

                        // Check that the argument, if it's there, is in the form of x == y
                        for argument in arguments {
                            complexity += count_expression_complexity(argument, complexity)
                        }
                    } else {
                        println!("** OTHER CALL!!! complexity: {}", complexity);
                    }
                    println!("** functionCall expression complexity: {}", complexity);
                }
                return complexity;
            },
            ast::Value::TableConstructor(table) => {
                println!("** table ctor complexity: {}", complexity);
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
                            println!("** table ctor field default: {}", complexity);
                            return complexity;
                        }
                    }
                }
                return complexity;
            },
            _ => {
                println!("** default value expression complexity: {}", complexity);
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
                    println!("** and/or expression complexity: {}", complexity);
                    return complexity;
                },
                _ => {
                    println!("** default binary operator expression complexity: {}", complexity);
                    return complexity;
                },
            }
        }
        _ => {
            println!("** default expression complexity: {}", complexity);
            return complexity;
        }
    }
}

fn count_block_complexity(block: &ast::Block, starting_complexity: u16) -> u16 {
    let mut complexity = starting_complexity;
    for statement in block.stmts() {
        println!("starting statement complexity: {}", complexity);

        if let ast::Stmt::If(if_block) = statement {
            complexity += 1;
            complexity = count_expression_complexity(if_block.condition(), complexity);
            println!("if expression complexity: {}", complexity);
            complexity = count_block_complexity(if_block.block(), complexity);
            println!("if block expression complexity: {}", complexity);
            if let Some(else_if_statements) = if_block.else_if() {
                for else_if in else_if_statements {
                    complexity += 1;
                    complexity = count_expression_complexity(else_if.condition(), complexity);
                    println!("** elseif condition complexity: {}", complexity);
                    complexity = count_block_complexity(else_if.block(), complexity);
                }
            }

        }
        if let ast::Stmt::While(while_block) = statement {
            complexity = count_expression_complexity(while_block.condition(), complexity + 1);
            println!("while expression complexity: {}", complexity);
            complexity = count_block_complexity(while_block.block(), complexity);
            println!("while block expression complexity: {}", complexity);
        }
        if let ast::Stmt::Repeat(repeat_block) = statement {
            complexity = count_expression_complexity(repeat_block.until(), complexity + 1);
            println!("repeat until expression complexity: {}", complexity);
            complexity = count_block_complexity(repeat_block.block(), complexity);
            println!("repeat block expression complexity: {}", complexity);
        }
        if let ast::Stmt::NumericFor(numeric_for) = statement {
            complexity += 1;
            complexity = count_expression_complexity(numeric_for.start(), complexity);
            println!("numeric for start expression complexity: {}", complexity);
            complexity = count_expression_complexity(numeric_for.end(), complexity);
            println!("numeric for end expression complexity: {}", complexity);
            if let Some(step_expression) = numeric_for.step() {
                complexity = count_expression_complexity(step_expression, complexity);
            }

            complexity = count_block_complexity(numeric_for.block(), complexity);
            println!("numeric for block complexity: {}", complexity);
        }
        if let ast::Stmt::GenericFor(generic_for) = statement {
            complexity += 1;
            for expression in generic_for.expressions() {
                complexity = count_expression_complexity(expression, complexity);
                println!("generic for expression complexity: {}", complexity);
                complexity = count_block_complexity(generic_for.block(), complexity);
                println!("generic for block complexity: {}", complexity);
            }
        }
        if let ast::Stmt::Assignment(assignment) = statement {
            println!("assignment statement complexity: {}", complexity);

            for expression in assignment.expressions() {
                complexity = count_expression_complexity(expression, complexity);
                println!("assignment expression complexity: {}", complexity);

            }
        }
        if let ast::Stmt::LocalAssignment(local_assignment) = statement {
            println!("local assignment statement complexity: {}", complexity);

            for expression in local_assignment.expressions() {
                complexity = count_expression_complexity(expression, complexity);
                println!("local assignment expression complexity: {}", complexity);
            }
        }

        if let ast::Stmt::FunctionCall(_call) = statement {
            // if_chain::if_chain! {
            //     // Check that we're calling it with an argument
            //     if let ast::Suffix::Call(call) = call.suffixes().next().unwrap();
            //     if let ast::Call::AnonymousCall(ast::FunctionArgs::Parentheses { arguments, .. }) = call;
            //     // Check that the argument, if it's there, is in the form of x == y
            //     for argument in arguments {
            //         complexity += count_expression_complexity(argument, complexity)
            //         println!("function call argument expression complexity: {}", complexity);
            //     }
            // }
        }
    };
    if let Some(last_statement) = block.last_stmt() {
        println!("last statement complexity: {}", complexity);
        if let ast::LastStmt::Return(return_) = last_statement {
            println!("return statement complexity: {}", complexity);

            for return_expression in return_.returns() {
                complexity = count_expression_complexity(return_expression, complexity);
                println!("return expressions complexity: {}", complexity);
            }
        }
    }

    return complexity;
}

impl Visitor for LimitFunctionComplexityVisitor {
    fn visit_function_body(&mut self, function_body: &ast::FunctionBody) {
        let complexity = count_block_complexity(function_body.block(), 1);
        println!("function complexity: {}", complexity);
        if complexity > self.config.maximum_complexity {
            self.positions.push(range(function_body));
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
