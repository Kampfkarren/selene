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
pub struct RestrictedImportsConfig {
    pub restricted_paths: HashMap<String, String>,
}

pub struct RestrictedImportsLint {
    config: RestrictedImportsConfig,
}

impl Lint for RestrictedImportsLint {
    type Config = RestrictedImportsConfig;
    type Error = Infallible;

    const SEVERITY: Severity = Severity::Error;
    const LINT_TYPE: LintType = LintType::Correctness;

    fn new(config: Self::Config) -> Result<Self, Self::Error> {
        Ok(RestrictedImportsLint { config })
    }

    fn pass(&self, ast: &Ast, _: &Context, _: &AstContext) -> Vec<Diagnostic> {
        if self.config.restricted_paths.is_empty() {
            return Vec::new();
        }

        let mut visitor = RestrictedImportsVisitor {
            restricted_paths: &self.config.restricted_paths,
            violations: Vec::new(),
        };

        visitor.visit_ast(ast);

        visitor.violations
    }
}

struct RestrictedImportsVisitor<'a> {
    restricted_paths: &'a HashMap<String, String>,
    violations: Vec<Diagnostic>,
}

impl<'a> Visitor for RestrictedImportsVisitor<'a> {
    fn visit_local_assignment(&mut self, node: &ast::LocalAssignment) {
        // Check each assignment in the local statement
        for expression in node.expressions() {
            if let Some(path) = name_path(expression) {
                let full_path = path.join(".");

                // Check if this path is restricted
                if self.restricted_paths.contains_key(&full_path) {
                    let range = expression.range().unwrap();

                    self.violations.push(Diagnostic::new(
                        "restricted_imports",
                        format!("import path `{full_path}` is restricted"),
                        Label::new((range.0.bytes() as u32, range.1.bytes() as u32)),
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
    fn test_restricted_imports() {
        let mut restricted_paths = HashMap::new();
        restricted_paths.insert(
            "OldLibrary.Utils.deprecatedFunction".to_string(),
            "OldLibrary.Utils.deprecatedFunction has been deprecated. Use NewLibrary.Utils.modernFunction instead.".to_string(),
        );

        test_lint(
            RestrictedImportsLint::new(RestrictedImportsConfig { restricted_paths }).unwrap(),
            "restricted_imports",
            "restricted_imports",
        );
    }
}
