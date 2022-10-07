use super::*;

use full_moon::ast::Ast;
use regex::Regex;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
#[serde(default)]
pub struct ShadowingConfig {
    ignore_pattern: String,
}

impl Default for ShadowingConfig {
    fn default() -> Self {
        Self {
            ignore_pattern: "^_".to_owned(),
        }
    }
}

pub struct ShadowingLint {
    ignore_pattern: Regex,
}

impl Lint for ShadowingLint {
    type Config = ShadowingConfig;
    type Error = regex::Error;

    const SEVERITY: Severity = Severity::Warning;
    const LINT_TYPE: LintType = LintType::Style;

    fn new(config: Self::Config) -> Result<Self, Self::Error> {
        Ok(ShadowingLint {
            ignore_pattern: Regex::new(&config.ignore_pattern)?,
        })
    }

    fn pass(&self, _: &Ast, _: &Context, ast_context: &AstContext) -> Vec<Diagnostic> {
        let mut shadows = Vec::new();

        for (_, variable) in &ast_context.scope_manager.variables {
            if let Some(shadow_id) = variable.shadowed {
                let shadow = &ast_context.scope_manager.variables[shadow_id];
                let definition = shadow.identifiers[0];

                let name = variable.name.to_owned();

                if self.ignore_pattern.is_match(&name) || name == "..." {
                    continue;
                }

                shadows.push(Shadow {
                    first_defined: (definition.0 as u32, definition.1 as u32),
                    name,
                    range: variable.identifiers[0],
                });
            }
        }

        shadows
            .iter()
            .map(|shadow| {
                Diagnostic::new_complete(
                    "shadowing",
                    format!("shadowing variable `{}`", shadow.name),
                    Label::new(shadow.range),
                    Vec::new(),
                    vec![Label::new_with_message(
                        shadow.first_defined,
                        "previously defined here".to_owned(),
                    )],
                )
            })
            .collect()
    }
}

struct Shadow {
    first_defined: (u32, u32),
    name: String,
    range: (usize, usize),
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    fn test_shadowing() {
        test_lint(
            ShadowingLint::new(ShadowingConfig::default()).unwrap(),
            "shadowing",
            "shadowing",
        );
    }

    #[test]
    fn test_empty_else() {
        test_lint(
            ShadowingLint::new(ShadowingConfig::default()).unwrap(),
            "shadowing",
            "empty_else",
        );
    }
}
