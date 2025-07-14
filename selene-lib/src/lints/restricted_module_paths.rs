use super::*;
use crate::ast_util::name_paths::name_path;
use std::collections::HashMap;
use std::convert::Infallible;

use full_moon::{
    ast::{self, Ast},
    visitors::Visitor,
};
use serde::Deserialize;

#[derive(Clone, Default, Deserialize)]
#[serde(default)]
pub struct RestrictedModulePathsConfig {
    pub restricted_paths: HashMap<String, String>,
}

pub struct RestrictedModulePathsLint {
    config: RestrictedModulePathsConfig,
}

impl Lint for RestrictedModulePathsLint {
    type Config = RestrictedModulePathsConfig;
    type Error = Infallible;

    const SEVERITY: Severity = Severity::Error;
    const LINT_TYPE: LintType = LintType::Correctness;

    fn new(config: Self::Config) -> Result<Self, Self::Error> {
        Ok(RestrictedModulePathsLint { config })
    }

    fn pass(&self, ast: &Ast, _: &Context, _: &AstContext) -> Vec<Diagnostic> {
        if self.config.restricted_paths.is_empty() {
            return Vec::new();
        }

        let mut visitor = RestrictedModulePathsVisitor {
            restricted_paths: &self.config.restricted_paths,
            violations: Vec::new(),
        };

        visitor.visit_ast(ast);

        visitor.violations
    }
}

struct RestrictedModulePathsVisitor<'a> {
    restricted_paths: &'a HashMap<String, String>,
    violations: Vec<Diagnostic>,
}

impl<'a> Visitor for RestrictedModulePathsVisitor<'a> {
    fn visit_local_assignment(&mut self, node: &ast::LocalAssignment) {
        // Check each assignment in the local statement
        for expression in node.expressions() {
            if let Some(path) = name_path(expression) {
                let full_path = path.join(".");

                // Check if this path is restricted
                if let Some(message) = self.restricted_paths.get(&full_path) {
                    let range = expression.range().unwrap();

                    self.violations.push(Diagnostic::new_complete(
                        "restricted_module_paths",
                        format!("Module path `{}` is restricted", full_path),
                        Label::new((range.0.bytes() as u32, range.1.bytes() as u32)),
                        vec![message.clone()],
                        Vec::new(),
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};
    use std::collections::HashMap;

    #[test]
    fn test_restricted_module_paths() {
        let mut restricted_paths = HashMap::new();
        restricted_paths.insert(
            "OldLibrary.Utils.deprecatedFunction".to_string(),
            "OldLibrary.Utils.deprecatedFunction has been deprecated. Use NewLibrary.Utils.modernFunction instead.".to_string(),
        );

        test_lint(
            RestrictedModulePathsLint::new(RestrictedModulePathsConfig { restricted_paths })
                .unwrap(),
            "restricted_module_paths",
            "restricted_module_paths",
        );
    }
}
