use super::*;
use crate::ast_util::scopes;
use std::convert::Infallible;

use full_moon::ast::Ast;

pub struct UnusedVariableLint;

impl Rule for UnusedVariableLint {
    type Config = ();
    type Error = Infallible;

    fn new(_: Self::Config) -> Result<Self, Self::Error> {
        Ok(UnusedVariableLint)
    }

    fn pass(&self, ast: &Ast) -> Vec<Diagnostic> {
        let scope_manager = scopes::ScopeManager::new(ast);
        let mut diagnostics = Vec::new();

        for (_, variable) in &scope_manager.variables {
            let mut references = variable
                .references
                .iter()
                .copied()
                .map(|id| &scope_manager.references[id]);

            if !references.clone().any(|reference| reference.read) {
                diagnostics.push(Diagnostic::new(
                    "unused_variable",
                    if references.any(|reference| reference.write) {
                        format!("{} is assigned a value, but never used", variable.name)
                    } else {
                        format!("{} is defined, but never used", variable.name)
                    },
                    Label::new(variable.identifiers[0]),
                ));
            };
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

    #[test]
    fn test_if() {
        test_lint(
            UnusedVariableLint::new(()).unwrap(),
            "unused_variable",
            "if",
        );
    }

    #[test]
    fn test_objects() {
        test_lint(
            UnusedVariableLint::new(()).unwrap(),
            "unused_variable",
            "objects",
        );
    }

    #[test]
    fn test_overriding() {
        test_lint(
            UnusedVariableLint::new(()).unwrap(),
            "unused_variable",
            "overriding",
        );
    }

    #[test]
    fn test_varargs() {
        test_lint(
            UnusedVariableLint::new(()).unwrap(),
            "unused_variable",
            "varargs",
        );
    }
}
