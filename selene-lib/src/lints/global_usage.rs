use super::*;
use std::collections::HashSet;

use full_moon::ast::Ast;
use regex::Regex;
use serde::Deserialize;

fn is_global(name: &str, roblox: bool) -> bool {
    (roblox && name == "shared") || name == "_G"
}

#[derive(Clone, Deserialize)]
#[serde(default)]
pub struct GlobalConfig {
    ignore_pattern: String,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            ignore_pattern: ".^".to_owned(),
        }
    }
}

pub struct GlobalLint {
    ignore_pattern: Regex,
}

impl Lint for GlobalLint {
    type Config = GlobalConfig;
    type Error = regex::Error;

    const SEVERITY: Severity = Severity::Warning;
    const LINT_TYPE: LintType = LintType::Complexity;

    fn new(config: Self::Config) -> Result<Self, Self::Error> {
        Ok(GlobalLint {
            ignore_pattern: Regex::new(&config.ignore_pattern)?,
        })
    }

    fn pass(&self, _: &Ast, context: &Context, ast_context: &AstContext) -> Vec<Diagnostic> {
        let mut checked = HashSet::new(); // TODO: Fix ScopeManager having duplicate references

        ast_context
            .scope_manager
            .references
            .iter()
            .filter(|(_, reference)| {
                if !checked.contains(&reference.identifier) {
                    checked.insert(reference.identifier);

                    let matches_ignore_pattern = match reference
                        .indexing
                        .as_ref()
                        .and_then(|indexing| indexing.first())
                        .and_then(|index_entry| index_entry.static_name.as_ref())
                    {
                        // Trim whitespace at the end as `_G.a  = 1` yields `a  `
                        Some(name) => self
                            .ignore_pattern
                            .is_match(&name.to_string().trim_end_matches(char::is_whitespace)),
                        None => false,
                    };

                    is_global(&reference.name, context.is_roblox())
                        && !matches_ignore_pattern
                        && reference.resolved.is_none()
                } else {
                    false
                }
            })
            .map(|(_, reference)| {
                Diagnostic::new(
                    "global_usage",
                    format!(
                        "use of `{}` is not allowed, structure your code in a more idiomatic way",
                        reference.name
                    ),
                    Label::new(reference.identifier),
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    fn test_global_usage() {
        test_lint(
            GlobalLint::new(GlobalConfig::default()).unwrap(),
            "global_usage",
            "global_usage",
        );
    }

    #[test]
    fn test_global_usage_ignore() {
        test_lint(
            GlobalLint::new(GlobalConfig {
                ignore_pattern: "^_.*_$".to_owned(),
            })
            .unwrap(),
            "global_usage",
            "global_usage_ignore",
        );
    }
}
