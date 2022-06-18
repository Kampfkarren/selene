use super::*;

use full_moon::ast::Ast;
use regex::Regex;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
#[serde(default)]
pub struct UnusedVariableConfig {
    allow_unused_self: bool,
    ignore_pattern: String,
}

impl Default for UnusedVariableConfig {
    fn default() -> Self {
        Self {
            allow_unused_self: true,
            ignore_pattern: "^_".to_owned(),
        }
    }
}

pub struct UnusedVariableLint {
    allow_unused_self: bool,
    ignore_pattern: Regex,
}

impl Rule for UnusedVariableLint {
    type Config = UnusedVariableConfig;
    type Error = regex::Error;

    fn new(config: Self::Config) -> Result<Self, Self::Error> {
        Ok(Self {
            allow_unused_self: config.allow_unused_self,
            ignore_pattern: Regex::new(&config.ignore_pattern)?,
        })
    }

    fn pass(&self, _: &Ast, _: &Context, ast_context: &AstContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (_, variable) in ast_context
            .scope_manager
            .variables
            .iter()
            .filter(|(_, variable)| !self.ignore_pattern.is_match(&variable.name))
        {
            let mut references = variable
                .references
                .iter()
                .copied()
                .map(|id| &ast_context.scope_manager.references[id]);

            if !references.clone().any(|reference| reference.read) {
                let mut notes = Vec::new();

                if variable.is_self {
                    if self.allow_unused_self {
                        continue;
                    }

                    notes.push("`self` is implicitly defined when defining a method".to_owned());
                    notes
                        .push("if you don't need it, consider using `.` instead of `:`".to_owned());
                }

                diagnostics.push(Diagnostic::new_complete(
                    "unused_variable",
                    if references.any(|reference| reference.write) {
                        format!("{} is assigned a value, but never used", variable.name)
                    } else {
                        format!("{} is defined, but never used", variable.name)
                    },
                    Label::new(variable.identifiers[0]),
                    notes,
                    Vec::new(),
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
    fn test_explicit_self() {
        test_lint(
            UnusedVariableLint::new(UnusedVariableConfig::default()).unwrap(),
            "unused_variable",
            "explicit_self",
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
    fn test_invalid_regex() {
        assert!(UnusedVariableLint::new(UnusedVariableConfig {
            ignore_pattern: "(".to_owned(),
            ..UnusedVariableConfig::default()
        })
        .is_err());
    }

    #[test]
    fn test_mutating_functions() {
        test_lint(
            UnusedVariableLint::new(UnusedVariableConfig::default()).unwrap(),
            "unused_variable",
            "mutating_functions",
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
    fn test_self() {
        test_lint(
            UnusedVariableLint::new(UnusedVariableConfig {
                allow_unused_self: false,
                ..UnusedVariableConfig::default()
            })
            .unwrap(),
            "unused_variable",
            "self",
        );
    }

    #[test]
    fn test_self_ignored() {
        test_lint(
            UnusedVariableLint::new(UnusedVariableConfig::default()).unwrap(),
            "unused_variable",
            "self_ignored",
        );
    }

    #[test]
    fn test_shadowing() {
        test_lint(
            UnusedVariableLint::new(UnusedVariableConfig::default()).unwrap(),
            "unused_variable",
            "shadowing",
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

    #[cfg(feature = "roblox")]
    #[test]
    fn test_types() {
        test_lint(
            UnusedVariableLint::new(UnusedVariableConfig::default()).unwrap(),
            "unused_variable",
            "types",
        );
    }
}
