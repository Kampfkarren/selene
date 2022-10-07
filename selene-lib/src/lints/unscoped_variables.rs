use crate::ast_util::scopes::ReferenceWrite;

use super::*;
use std::collections::HashSet;

use full_moon::ast::Ast;
use regex::Regex;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
#[serde(default)]
pub struct UnscopedVariablesConfig {
    ignore_pattern: String,
}

impl Default for UnscopedVariablesConfig {
    fn default() -> Self {
        Self {
            ignore_pattern: "^_".to_owned(),
        }
    }
}

pub struct UnscopedVariablesLint {
    ignore_pattern: Regex,
}

impl Lint for UnscopedVariablesLint {
    type Config = UnscopedVariablesConfig;
    type Error = regex::Error;

    const SEVERITY: Severity = Severity::Warning;
    const LINT_TYPE: LintType = LintType::Complexity;

    fn new(config: Self::Config) -> Result<Self, Self::Error> {
        Ok(UnscopedVariablesLint {
            ignore_pattern: Regex::new(&config.ignore_pattern)?,
        })
    }

    fn pass(&self, _: &Ast, context: &Context, ast_context: &AstContext) -> Vec<Diagnostic> {
        // ScopeManager repeats references, and I just don't want to fix it right now
        let mut read = HashSet::new();

        let mut diagnostics = Vec::new();

        for (_, reference) in &ast_context.scope_manager.references {
            if reference.resolved.is_none()
                && reference.write == Some(ReferenceWrite::Assign)
                && !read.contains(&reference.identifier)
                && !self.ignore_pattern.is_match(&reference.name)
                && !context.standard_library.global_has_fields(&reference.name)
            {
                read.insert(reference.identifier);

                diagnostics.push(Diagnostic::new(
                    "unscoped_variables",
                    format!(
                        "`{}` is not declared locally, and will be available in every scope",
                        reference.name
                    ),
                    Label::new(reference.identifier),
                ));
            }
        }

        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::*, *};

    #[test]
    fn test_function_overriding() {
        test_lint(
            UnscopedVariablesLint::new(UnscopedVariablesConfig::default()).unwrap(),
            "unscoped_variables",
            "function_overriding",
        );
    }

    #[test]
    fn test_unscoped_variables() {
        test_lint(
            UnscopedVariablesLint::new(UnscopedVariablesConfig::default()).unwrap(),
            "unscoped_variables",
            "unscoped_variables",
        );
    }
}
