use super::*;
use crate::ast_util::range;
use std::convert::Infallible;

use full_moon::{
    ast::{self, Ast},
    visitors::Visitor,
};

pub struct ParentheseConditionsLint;

impl Lint for ParentheseConditionsLint {
    type Config = ();
    type Error = Infallible;

    const SEVERITY: Severity = Severity::Warning;
    const LINT_TYPE: LintType = LintType::Style;

    fn new(_: Self::Config) -> Result<Self, Self::Error> {
        Ok(ParentheseConditionsLint)
    }

    fn pass(&self, ast: &Ast, _: &Context, _: &AstContext) -> Vec<Diagnostic> {
        let mut visitor = ParentheseConditionsVisitor {
            positions: Vec::new(),
        };

        visitor.visit_ast(ast);

        visitor
            .positions
            .iter()
            .map(|position| {
                Diagnostic::new(
                    "parenthese_conditions",
                    "lua does not require parentheses around conditions".to_owned(),
                    Label::new(*position),
                )
            })
            .collect()
    }
}

struct ParentheseConditionsVisitor {
    positions: Vec<(usize, usize)>,
}

impl ParentheseConditionsVisitor {
    fn lint_condition(&mut self, condition: &ast::Expression) {
        let is_parentheses = match condition {
            ast::Expression::Parentheses { .. } => true,
            ast::Expression::Value { value, .. } => {
                matches!(&**value, ast::Value::ParenthesesExpression(_))
            }
            _ => false,
        };

        if is_parentheses {
            self.positions.push(range(condition));
        }
    }
}

impl Visitor for ParentheseConditionsVisitor {
    fn visit_if(&mut self, node: &ast::If) {
        self.lint_condition(node.condition());

        if let Some(else_ifs) = node.else_if() {
            for else_if in else_ifs {
                self.lint_condition(else_if.condition());
            }
        }
    }

    fn visit_repeat(&mut self, node: &ast::Repeat) {
        self.lint_condition(node.until());
    }

    fn visit_while(&mut self, node: &ast::While) {
        self.lint_condition(node.condition());
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    fn test_parenthese_conditions() {
        test_lint(
            ParentheseConditionsLint::new(()).unwrap(),
            "parenthese_conditions",
            "parenthese_conditions",
        );
    }
}
