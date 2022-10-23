use crate::ast_util::{purge_trivia, range, strip_parentheses};

use super::*;
use std::convert::Infallible;

use full_moon::{
    ast::{self, Ast, BinOp},
    visitors::Visitor,
};

pub struct ConstantTableComparisonLint;

impl Lint for ConstantTableComparisonLint {
    type Config = ();
    type Error = Infallible;

    const SEVERITY: Severity = Severity::Error;
    const LINT_TYPE: LintType = LintType::Correctness;

    fn new(_: Self::Config) -> Result<Self, Self::Error> {
        Ok(ConstantTableComparisonLint)
    }

    fn pass(&self, ast: &Ast, _: &Context, _: &AstContext) -> Vec<Diagnostic> {
        let mut visitor = ConstantTableComparisonVisitor {
            comparisons: Vec::new(),
        };

        visitor.visit_ast(ast);

        visitor
            .comparisons
            .iter()
            .map(|comparison| {
                Diagnostic::new_complete(
                    "constant_table_comparison",
                    "comparing to a constant table will always fail".to_owned(),
                    Label::new(comparison.range),
                    if let Some(empty_side) = comparison.empty_side {
                        vec![format!(
                            "try: `next({}) {} nil`",
                            match empty_side {
                                EmptyComparison::CheckEmpty(side) => match side {
                                    EmptyComparisonSide::Left => &comparison.rhs,
                                    EmptyComparisonSide::Right => &comparison.lhs,
                                },

                                EmptyComparison::CheckNotEmpty(side) => match side {
                                    EmptyComparisonSide::Left => &comparison.rhs,
                                    EmptyComparisonSide::Right => &comparison.lhs,
                                },
                            },
                            match empty_side {
                                EmptyComparison::CheckEmpty(_) => "==",
                                EmptyComparison::CheckNotEmpty(_) => "~=",
                            }
                        )]
                    } else {
                        Vec::new()
                    },
                    Vec::new(),
                )
            })
            .collect()
    }
}

struct ConstantTableComparisonVisitor {
    comparisons: Vec<Comparison>,
}

#[derive(Clone, Copy)]
enum EmptyComparisonSide {
    Left,
    Right,
}

#[derive(Clone, Copy)]
enum EmptyComparison {
    CheckEmpty(EmptyComparisonSide),
    CheckNotEmpty(EmptyComparisonSide),
}

struct Comparison {
    lhs: String,
    rhs: String,
    empty_side: Option<EmptyComparison>,
    range: (usize, usize),
}

enum ConstantTableMatch {
    Empty,
    NotEmpty,
}

fn constant_table_match(expression: &ast::Expression) -> Option<ConstantTableMatch> {
    if let ast::Expression::Value { value, .. } = strip_parentheses(expression) {
        if let ast::Value::TableConstructor(table_constructor) = &**value {
            return if table_constructor.fields().is_empty() {
                Some(ConstantTableMatch::Empty)
            } else {
                Some(ConstantTableMatch::NotEmpty)
            };
        }
    }

    None
}

impl Visitor for ConstantTableComparisonVisitor {
    fn visit_expression(&mut self, node: &ast::Expression) {
        if let ast::Expression::BinaryOperator {
            lhs,
            binop:
                binop @ (BinOp::TwoEqual(_)
                | BinOp::TildeEqual(_)
                | BinOp::GreaterThan(_)
                | BinOp::LessThan(_)
                | BinOp::GreaterThanEqual(_)
                | BinOp::LessThanEqual(_)),
            rhs,
        } = node
        {
            match (constant_table_match(lhs), constant_table_match(rhs)) {
                // The (Some(_), Some(_)) case is rare, but also blatantly useless.
                // `{} == {}` translating to `next({}) == nil` is clearly silly.
                (Some(_), Some(_))
                | (Some(ConstantTableMatch::NotEmpty), _)
                | (_, Some(ConstantTableMatch::NotEmpty)) => {
                    self.comparisons.push(Comparison {
                        lhs: purge_trivia(lhs).to_string(),
                        rhs: purge_trivia(rhs).to_string(),
                        empty_side: None,
                        range: range(node),
                    });
                }

                empty_checks @ ((Some(ConstantTableMatch::Empty), None)
                | (None, Some(ConstantTableMatch::Empty))) => {
                    let side = match empty_checks.0.is_some() {
                        true => EmptyComparisonSide::Left,
                        false => EmptyComparisonSide::Right,
                    };

                    self.comparisons.push(Comparison {
                        lhs: purge_trivia(lhs).to_string(),
                        rhs: purge_trivia(rhs).to_string(),
                        empty_side: match binop {
                            ast::BinOp::TwoEqual(_) => Some(EmptyComparison::CheckEmpty(side)),
                            ast::BinOp::TildeEqual(_) => Some(EmptyComparison::CheckNotEmpty(side)),
                            _ => None,
                        },
                        range: range(node),
                    });
                }

                (None, None) => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    fn test_constant_table_comparison() {
        test_lint(
            ConstantTableComparisonLint::new(()).unwrap(),
            "constant_table_comparison",
            "constant_table_comparison",
        );
    }
}
