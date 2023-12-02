use super::*;
use std::convert::Infallible;

use full_moon::{ast::Ast, tokenizer::TokenType, visitors::Visitor};

pub struct ApproxConstantLint;

impl Lint for ApproxConstantLint {
    type Config = ();
    type Error = Infallible;

    const SEVERITY: Severity = Severity::Warning;
    const LINT_TYPE: LintType = LintType::Correctness;

    fn new((): Self::Config) -> Result<Self, Self::Error> {
        Ok(ApproxConstantLint)
    }

    fn pass(&self, ast: &Ast, _: &Context, _: &AstContext) -> Vec<Diagnostic> {
        let mut visitor = ApproxConstantVisitor {
            approx_constants: Vec::new(),
        };

        visitor.visit_ast(ast);

        visitor
            .approx_constants
            .iter()
            .map(|constant| {
                Diagnostic::new(
                    "approx_constant",
                    format!("`{}` is more precise", constant.constant),
                    Label::new(constant.range),
                )
            })
            .collect()
    }
}

struct ApproxConstantVisitor {
    approx_constants: Vec<ApproximatedConstant>,
}

struct ApproximatedConstant {
    range: (usize, usize),
    constant: String,
}

impl Visitor for ApproxConstantVisitor {
    fn visit_number(&mut self, token: &full_moon::tokenizer::Token) {
        if let TokenType::Number { text } = token.token_type() {
            if is_approx_const(std::f64::consts::PI, text, 3) {
                self.approx_constants.push(ApproximatedConstant {
                    range: (token.start_position().bytes(), token.end_position().bytes()),
                    constant: "math.pi".to_string(),
                });
            }
        }
    }
}

#[must_use]
fn is_approx_const(constant: f64, value: &str, min_digits: usize) -> bool {
    if value.len() <= min_digits {
        false
    } else if constant.to_string().starts_with(value) {
        // The value is a truncated constant
        true
    } else {
        let round_const = format!("{constant:.*}", value.len() - 2);
        value == round_const
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    fn test_approx_constant() {
        test_lint(
            ApproxConstantLint::new(()).unwrap(),
            "approx_constant",
            "approx_constant",
        );
    }
}
