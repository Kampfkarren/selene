use super::*;
use crate::ast_util::scopes;

use full_moon::ast::Ast;
use regex::Regex;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
#[serde(default)]
pub struct UnusedVariableConfig {
    ignore_pattern: String,
}

impl Default for UnusedVariableConfig {
    fn default() -> Self {
        Self {
            ignore_pattern: "^_".to_owned(),
        }
    }
}

pub struct UnusedVariableLint {
    ignore_pattern: Regex,
}

impl Rule for UnusedVariableLint {
    type Config = UnusedVariableConfig;
    type Error = regex::Error;

    fn new(config: Self::Config) -> Result<Self, Self::Error> {
        Ok(Self {
            ignore_pattern: Regex::new(&config.ignore_pattern)?,
        })
    }

    fn pass(&self, ast: &Ast) -> Vec<Diagnostic> {
        let scope_manager = scopes::ScopeManager::new(ast);
        let mut diagnostics = Vec::new();

        for (_, variable) in scope_manager
            .variables
            .iter()
            .filter(|(_, variable)| !self.ignore_pattern.is_match(&variable.name))
        {
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
    fn test_blocks() {
        test_lint(
            UnusedVariableLint::new(UnusedVariableConfig::default()).unwrap(),
            "unused_variable",
            "blocks",
        );
    }

    #[test]
    fn test_locals() {
        test_lint(
            UnusedVariableLint::new(UnusedVariableConfig::default()).unwrap(),
            "unused_variable",
            "locals",
        );
    }

    #[test]
    fn test_edge_cases() {
        test_lint(
            UnusedVariableLint::new(UnusedVariableConfig::default()).unwrap(),
            "unused_variable",
            "edge_cases",
        );
    }

    #[test]
    fn test_generic_for_shadowing() {
        test_lint(
            UnusedVariableLint::new(UnusedVariableConfig::default()).unwrap(),
            "unused_variable",
            "generic_for_shadowing",
        );
    }

    #[test]
    fn test_if() {
        test_lint(
            UnusedVariableLint::new(UnusedVariableConfig::default()).unwrap(),
            "unused_variable",
            "if",
        );
    }

    #[test]
    fn test_ignore() {
        test_lint(
            UnusedVariableLint::new(UnusedVariableConfig::default()).unwrap(),
            "unused_variable",
            "ignore",
        );
    }

    #[test]
    fn test_objects() {
        test_lint(
            UnusedVariableLint::new(UnusedVariableConfig::default()).unwrap(),
            "unused_variable",
            "objects",
        );
    }

    #[test]
    fn test_overriding() {
        test_lint(
            UnusedVariableLint::new(UnusedVariableConfig::default()).unwrap(),
            "unused_variable",
            "overriding",
        );
    }

    #[test]
    fn test_varargs() {
        test_lint(
            UnusedVariableLint::new(UnusedVariableConfig::default()).unwrap(),
            "unused_variable",
            "varargs",
        );
    }

    #[test]
    fn test_invalid_regex() {
        assert!(UnusedVariableLint::new(UnusedVariableConfig {
            ignore_pattern: "(".to_owned(),
        })
        .is_err());
    }
}
