use super::*;
use std::convert::Infallible;

use full_moon::{
    ast::{self, punctuated::Punctuated, Ast},
    node::Node,
    tokenizer::{Symbol, TokenType},
    visitors::Visitor,
};

pub struct UnbalancedAssignmentsLint;

impl Lint for UnbalancedAssignmentsLint {
    type Config = ();
    type Error = Infallible;

    const SEVERITY: Severity = Severity::Warning;
    const LINT_TYPE: LintType = LintType::Complexity;

    fn new(_: Self::Config) -> Result<Self, Self::Error> {
        Ok(UnbalancedAssignmentsLint)
    }

    fn pass(&self, ast: &Ast, _: &Context, _: &AstContext) -> Vec<Diagnostic> {
        let mut visitor = UnbalancedAssignmentsVisitor {
            assignments: Vec::new(),
        };

        visitor.visit_ast(ast);

        visitor
            .assignments
            .drain(..)
            .map(|assignment| {
                if assignment.more {
                    Diagnostic::new(
                        "unbalanced_assignments",
                        "too many values on the right side of the assignment".to_owned(),
                        Label::new(assignment.range),
                    )
                } else {
                    let secondary_labels = match assignment.first_call {
                        Some(range) => vec![Label::new_with_message(
                            range,
                            "help: if this function returns more than one value, \
                             the only first return value is actually used"
                                .to_owned(),
                        )],
                        None => Vec::new(),
                    };

                    Diagnostic::new_complete(
                        "unbalanced_assignments",
                        "values on right side don't match up to the left side of the assignment"
                            .to_owned(),
                        Label::new(assignment.range),
                        Vec::new(),
                        secondary_labels,
                    )
                }
            })
            .collect()
    }
}

struct UnbalancedAssignmentsVisitor {
    assignments: Vec<UnbalancedAssignment>,
}

fn expression_is_call(expression: &ast::Expression) -> bool {
    match expression {
        ast::Expression::Parentheses { expression, .. } => expression_is_call(expression),
        ast::Expression::Value { value, .. } => {
            matches!(&**value, ast::Value::FunctionCall(_))
        }

        _ => false,
    }
}

fn expression_is_nil(expression: &ast::Expression) -> bool {
    match expression {
        ast::Expression::Parentheses { expression, .. } => expression_is_call(expression),
        ast::Expression::Value { value, .. } => {
            if let ast::Value::Symbol(symbol) = &**value {
                *symbol.token_type()
                    == TokenType::Symbol {
                        symbol: Symbol::Nil,
                    }
            } else {
                false
            }
        }

        _ => false,
    }
}

fn expression_is_ellipsis(expression: &ast::Expression) -> bool {
    if let ast::Expression::Value { value, .. } = expression {
        if let ast::Value::Symbol(symbol) = &**value {
            return *symbol.token_type()
                == TokenType::Symbol {
                    symbol: Symbol::Ellipse,
                };
        }
    }

    false
}

fn range<N: Node>(node: N) -> (u32, u32) {
    let (start, end) = node.range().unwrap();
    (start.bytes() as u32, end.bytes() as u32)
}

impl UnbalancedAssignmentsVisitor {
    fn lint_assignment(&mut self, lhs: usize, rhs: &Punctuated<ast::Expression>) {
        if rhs.is_empty() {
            return;
        }

        let last_rhs = rhs.last().unwrap().value();

        if rhs.len() > lhs {
            self.assignments.push(UnbalancedAssignment {
                more: true,
                range: (
                    // TODO: Implement Index and get() on Punctuated
                    rhs.iter()
                        .nth(lhs)
                        .unwrap()
                        .start_position()
                        .unwrap()
                        .bytes() as u32,
                    last_rhs.end_position().unwrap().bytes() as u32,
                ),
                ..UnbalancedAssignment::default()
            });
        } else if rhs.len() < lhs
            && !expression_is_ellipsis(last_rhs)
            && !expression_is_call(last_rhs)
            && !expression_is_nil(last_rhs)
        {
            self.assignments.push(UnbalancedAssignment {
                first_call: rhs.iter().find(|e| expression_is_call(e)).map(range),
                range: range(rhs),
                ..UnbalancedAssignment::default()
            });
        }
    }
}

impl Visitor for UnbalancedAssignmentsVisitor {
    fn visit_assignment(&mut self, assignment: &ast::Assignment) {
        self.lint_assignment(assignment.variables().len(), assignment.expressions());
    }

    fn visit_local_assignment(&mut self, assignment: &ast::LocalAssignment) {
        self.lint_assignment(assignment.names().len(), assignment.expressions());
    }
}

#[derive(Clone, Copy, Default)]
struct UnbalancedAssignment {
    first_call: Option<(u32, u32)>,
    more: bool,
    range: (u32, u32),
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    fn test_unbalanced_assignments() {
        test_lint(
            UnbalancedAssignmentsLint::new(()).unwrap(),
            "unbalanced_assignments",
            "unbalanced_assignments",
        );
    }
}
