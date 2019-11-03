use super::*;
use std::{collections::HashSet, convert::Infallible};
use crate::ast_util::scopes::ScopeManager;

use full_moon::ast::Ast;

fn is_global(name: &str) -> bool {
    name == "_G"
}

pub struct GlobalLint;

impl Rule for GlobalLint {
    type Config = ();
    type Error = Infallible;

    fn new(_: Self::Config) -> Result<Self, Self::Error> {
        Ok(GlobalLint)
    }

    fn pass(&self, ast: &Ast, _: &Context) -> Vec<Diagnostic> {
        let scope_manager = ScopeManager::new(ast);
        let mut checked = HashSet::new(); // TODO: Fix ScopeManager having duplicate references

        scope_manager.references.iter().filter(|(_, reference)| {
            if !checked.contains(&reference.identifier) {
                checked.insert(reference.identifier);
                is_global(&reference.name) && reference.resolved.is_none()
            } else {
                false
            }
        }).map(|(_, reference)| {
            Diagnostic::new(
                "global_usage",
                format!("use of `{}` is not allowed, structure your code in a more idiomatic way", reference.name),
                Label::new(reference.identifier),
            )
        }).collect()
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn rule_type(&self) -> RuleType {
        RuleType::Complexity
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    fn test_global_usage() {
        test_lint(
            GlobalLint::new(()).unwrap(),
            "global_usage",
            "global_usage",
        );
    }
}
