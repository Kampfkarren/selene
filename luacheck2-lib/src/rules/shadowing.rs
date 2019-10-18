use super::*;
use crate::ast_util::scopes::ScopeManager;
use std::convert::Infallible;

use full_moon::ast::Ast;

pub struct ShadowingLint;

impl Rule for ShadowingLint {
    type Config = ();
    type Error = Infallible;

    fn new(_: Self::Config) -> Result<Self, Self::Error> {
        Ok(ShadowingLint)
    }

    fn pass(&self, ast: &Ast, _: &Context) -> Vec<Diagnostic> {
        let scope_manager = ScopeManager::new(ast);
        let mut shadows = Vec::new();

        for (_, variable) in &scope_manager.variables {
            if let Some(shadow_id) = variable.shadowed {
                let shadow = &scope_manager.variables[shadow_id];
                let definition = shadow.definitions[0];

                shadows.push(Shadow {
                    first_defined: (definition.0 as u32, definition.1 as u32),
                    name: variable.name.to_owned(),
                    range: variable.definitions[0],
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

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn rule_type(&self) -> RuleType {
        RuleType::Style
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
        test_lint(ShadowingLint::new(()).unwrap(), "shadowing", "shadowing");
    }
}
