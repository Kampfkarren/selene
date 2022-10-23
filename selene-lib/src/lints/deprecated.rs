use std::convert::Infallible;

use full_moon::{ast, visitors::Visitor};
use serde::Deserialize;

use crate::ast_util::{name_paths::*, scopes::ScopeManager};

use super::{super::standard_library::*, *};

#[derive(Clone, Default, Deserialize)]
#[serde(default)]
pub struct DeprecatedLintConfig {
    pub allow: Vec<String>,
}

pub struct DeprecatedLint {
    config: DeprecatedLintConfig,
}

impl Lint for DeprecatedLint {
    type Config = DeprecatedLintConfig;
    type Error = Infallible;

    const SEVERITY: Severity = Severity::Warning;
    const LINT_TYPE: LintType = LintType::Correctness;

    fn new(config: Self::Config) -> Result<Self, Self::Error> {
        Ok(DeprecatedLint { config })
    }

    fn pass(&self, ast: &Ast, context: &Context, ast_context: &AstContext) -> Vec<Diagnostic> {
        let mut visitor = DeprecatedVisitor::new(
            &self.config,
            &ast_context.scope_manager,
            &context.standard_library,
        );

        visitor.visit_ast(ast);

        visitor.diagnostics
    }
}

struct DeprecatedVisitor<'a> {
    allow: Vec<Vec<String>>,
    diagnostics: Vec<Diagnostic>,
    scope_manager: &'a ScopeManager,
    standard_library: &'a StandardLibrary,
}

impl<'a> DeprecatedVisitor<'a> {
    fn new(
        config: &DeprecatedLintConfig,
        scope_manager: &'a ScopeManager,
        standard_library: &'a StandardLibrary,
    ) -> Self {
        Self {
            diagnostics: Vec::new(),
            scope_manager,
            standard_library,

            allow: config
                .allow
                .iter()
                .map(|allow| allow.split('.').map(ToOwned::to_owned).collect())
                .collect(),
        }
    }

    fn allowed(&self, name_path: &[String]) -> bool {
        'next_allow_path: for allow_path in &self.allow {
            if allow_path.len() > name_path.len() {
                continue;
            }

            for (allow_word, name_word) in allow_path.iter().zip(name_path.iter()) {
                if allow_word == "*" {
                    continue;
                }

                if allow_word != name_word {
                    continue 'next_allow_path;
                }
            }

            return true;
        }

        false
    }

    fn check_name_path<N: Node>(
        &mut self,
        node: &N,
        what: &str,
        name_path: &[String],
        parameters: &[String],
    ) {
        assert!(!name_path.is_empty());

        if self.allowed(name_path) {
            return;
        }

        for bound in 1..=name_path.len() {
            profiling::scope!("DeprecatedVisitor::check_name_path check in bound");
            let deprecated = match self.standard_library.find_global(&name_path[0..bound]) {
                Some(Field {
                    deprecated: Some(deprecated),
                    ..
                }) => deprecated,

                _ => continue,
            };

            let mut notes = vec![deprecated.message.to_owned()];

            if let Some(replace_with) = deprecated.try_instead(parameters) {
                notes.push(format!("try: {replace_with}"));
            }

            self.diagnostics.push(Diagnostic::new_complete(
                "deprecated",
                format!(
                    "standard library {what} `{}` is deprecated",
                    name_path.join(".")
                ),
                Label::from_node(node, None),
                notes,
                Vec::new(),
            ));
        }
    }
}

impl Visitor for DeprecatedVisitor<'_> {
    fn visit_expression(&mut self, expression: &ast::Expression) {
        if let Some(reference) = self
            .scope_manager
            .reference_at_byte(expression.start_position().unwrap().bytes())
        {
            if reference.resolved.is_some() {
                return;
            }
        }

        let name_path = match name_path(expression) {
            Some(name_path) => name_path,
            None => return,
        };

        self.check_name_path(expression, "expression", &name_path, &[]);
    }

    fn visit_function_call(&mut self, call: &ast::FunctionCall) {
        if let Some(reference) = self
            .scope_manager
            .reference_at_byte(call.start_position().unwrap().bytes())
        {
            if reference.resolved.is_some() {
                return;
            }
        }

        let mut keep_going = true;
        let mut suffixes: Vec<&ast::Suffix> = call
            .suffixes()
            .take_while(|suffix| take_while_keep_going(suffix, &mut keep_going))
            .collect();

        let name_path = match name_path_from_prefix_suffix(call.prefix(), suffixes.iter().copied())
        {
            Some(name_path) => name_path,
            None => return,
        };

        let call_suffix = suffixes.pop().unwrap();

        let function_args = match call_suffix {
            #[cfg_attr(
                feature = "force_exhaustive_checks",
                deny(non_exhaustive_omitted_patterns)
            )]
            ast::Suffix::Call(call) => match call {
                ast::Call::AnonymousCall(args) => args,
                ast::Call::MethodCall(method_call) => method_call.args(),
                _ => return,
            },

            _ => unreachable!("function_call.call_suffix != ast::Suffix::Call"),
        };

        #[cfg_attr(
            feature = "force_exhaustive_checks",
            deny(non_exhaustive_omitted_patterns)
        )]
        let argument_displays = match function_args {
            ast::FunctionArgs::Parentheses { arguments, .. } => arguments
                .iter()
                .map(|argument| argument.to_string())
                .collect(),

            ast::FunctionArgs::String(token) => vec![token.to_string()],
            ast::FunctionArgs::TableConstructor(table_constructor) => {
                vec![table_constructor.to_string()]
            }

            _ => Vec::new(),
        };

        self.check_name_path(call, "function", &name_path, &argument_displays);
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::*, *};

    #[test]
    fn test_deprecated_fields() {
        test_lint(
            DeprecatedLint::new(DeprecatedLintConfig::default()).unwrap(),
            "deprecated",
            "deprecated_fields",
        );
    }

    #[test]
    fn test_deprecated_functions() {
        test_lint(
            DeprecatedLint::new(DeprecatedLintConfig::default()).unwrap(),
            "deprecated",
            "deprecated_functions",
        );
    }

    #[test]
    fn test_specific_allow() {
        test_lint(
            DeprecatedLint::new(DeprecatedLintConfig {
                allow: vec![
                    "deprecated_allowed".to_owned(),
                    "more.*".to_owned(),
                    "wow.*.deprecated_allowed".to_owned(),
                ],
            })
            .unwrap(),
            "deprecated",
            "specific_allow",
        );
    }

    #[test]
    fn test_toml_forwards_compatibility() {
        test_lint(
            DeprecatedLint::new(DeprecatedLintConfig::default()).unwrap(),
            "deprecated",
            "toml_forwards_compatibility",
        );
    }
}
