use super::*;
use crate::ast_util::name_paths::{name_path, name_path_from_prefix_suffix, take_while_keep_going};
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
    fn visit_expression(&mut self, expression: &ast::Expression) {
        self.check_expression(expression);
    }

    fn visit_function_call(&mut self, call: &ast::FunctionCall) {
        // Handle function call statements (standalone function calls)
        let mut keep_going = true;
        let suffixes: Vec<&ast::Suffix> = call
            .suffixes()
            .take_while(|suffix| take_while_keep_going(suffix, &mut keep_going))
            .collect();

        if let Some(path) = name_path_from_prefix_suffix(call.prefix(), suffixes.iter().copied()) {
            let full_path = path.join(".");

            // Calculate range from prefix start to last suffix end
            let start_pos = call.prefix().start_position().unwrap();
            let end_pos = if let Some(last_suffix) = suffixes.last() {
                last_suffix.end_position().unwrap()
            } else {
                call.prefix().end_position().unwrap()
            };

            self.check_path_restriction(&full_path, (start_pos.bytes(), end_pos.bytes()));
        }
    }
}

impl<'a> RestrictedModulePathsVisitor<'a> {
    fn check_expression(&mut self, expression: &ast::Expression) {
        // Only handle variable expressions here, function calls are handled by visit_function_call
        if let ast::Expression::Var(_) = expression {
            if let Some(path) = name_path(expression) {
                let full_path = path.join(".");
                let range = expression.range().unwrap();
                self.check_path_restriction(&full_path, (range.0.bytes(), range.1.bytes()));
            }
        }
    }

    fn check_path_restriction(&mut self, full_path: &str, range: (usize, usize)) {
        // Check if this path is restricted
        if let Some(message) = self.restricted_paths.get(full_path) {
            self.violations.push(Diagnostic::new_complete(
                "restricted_module_paths",
                format!("Module path `{}` is restricted", full_path),
                Label::new((range.0 as u32, range.1 as u32)),
                vec![message.clone()],
                Vec::new(),
            ));
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
