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
    let complexity = starting_complexity;
    match expression {
        ast::Expression::Parentheses { expression, .. } => {
            return count_expression_complexity(expression, complexity)
        },
        ast::Expression::Value { value, .. } => match &**value {
            #[cfg(feature = "roblox")]
            ast::Value::IfExpression(if_expression) => {
                if let Some(else_if_expressions) = if_expression.else_if_expressions() {
                    for else_if_expression in else_if_expressions {
                        return count_expression_complexity(else_if_expression.expression(), complexity + 1)
                    }
                }
                return complexity + 1
            },
            ast::Value::ParenthesesExpression(paren_expression) => {
                return count_expression_complexity(paren_expression, complexity)
            },
            ast::Value::FunctionCall(_call) => {
                // if_chain::if_chain! {
                //     // Check that we're calling it with an argument
                //     if let ast::Suffix::Call(call) = call.suffixes().next().unwrap();
                //     if let ast::Call::AnonymousCall(ast::FunctionArgs::Parentheses { arguments, .. }) = call;
                //     // Check that the argument, if it's there, is in the form of x == y
                //     for argument in arguments {
                //         complexity += count_expression_complexity(argument, complexity)
                //     }
                // }
                return complexity;
            },
            _ => {
                return complexity;
            },
        },
        ast::Expression::BinaryOperator {
            binop, rhs, ..
        } => {
            match binop {
                #[cfg_attr(
                    feature = "force_exhaustive_checks",
                    allow(non_exhaustive_omitted_patterns)
                )]
                | ast::BinOp::And(_)
                | ast::BinOp::Or(_) =>
                {
                    return count_expression_complexity(rhs, complexity + 1)
                },
                _ => {
                    return complexity;
                }
            }
        },
        _ => {
            return complexity;
        }
    }
}

fn count_block_complexity(block: &ast::Block, starting_complexity: u16) -> u16 {
    let mut complexity = starting_complexity;
    for statement in block.stmts() {
        if let ast::Stmt::If(if_block) = statement {
            complexity += count_expression_complexity(if_block.condition(), complexity + 1);
            println!("if expression complexity: {}", complexity);
            complexity += count_block_complexity(if_block.block(), complexity);
            println!("if block expression complexity: {}", complexity);
        }
        if let ast::Stmt::While(while_block) = statement {
            complexity += count_expression_complexity(while_block.condition(), complexity + 1);
            println!("while expression complexity: {}", complexity);
            complexity += count_block_complexity(while_block.block(), complexity);
            println!("while block expression complexity: {}", complexity);
        }
        if let ast::Stmt::Repeat(repeat_block) = statement {
            complexity += count_expression_complexity(repeat_block.until(), complexity + 1);
            println!("repeat until expression complexity: {}", complexity);
            complexity += count_block_complexity(repeat_block.block(), complexity);
            println!("repeat block expression complexity: {}", complexity);
        }
        if let ast::Stmt::NumericFor(numeric_for) = statement {
            complexity += count_expression_complexity(numeric_for.start(), complexity + 1);
            println!("numeric for start expression complexity: {}", complexity);
            complexity += count_expression_complexity(numeric_for.end(), complexity + 1);
            println!("numeric for end expression complexity: {}", complexity);
            complexity += count_block_complexity(numeric_for.block(), complexity);
            println!("numeric for block complexity: {}", complexity);
        }
        if let ast::Stmt::GenericFor(generic_for) = statement {
            for expression in generic_for.expressions() {
                complexity += count_expression_complexity(expression, complexity + 1);
                println!("generic for expression complexity: {}", complexity);
                complexity += count_block_complexity(generic_for.block(), complexity);
                println!("generic for block complexity: {}", complexity);
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
        // TODO: need to handle `return if throw then mama .. 'from' else train()`
        // if let ast::LastStmt::Return(return_) = statement {
        //     complexity = count_expression_complexity(return_.returns(), complexity)
        // }
    }


    return complexity
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
