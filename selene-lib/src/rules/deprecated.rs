use std::convert::Infallible;

use full_moon::{ast, visitors::Visitor};

use crate::ast_util::{name_paths::*, scopes::ScopeManager};

use super::{super::standard_library::*, *};

pub struct DeprecatedLint;

impl Rule for DeprecatedLint {
    type Config = ();
    type Error = Infallible;

    fn new(_: Self::Config) -> Result<Self, Self::Error> {
        Ok(DeprecatedLint)
    }

    fn pass(&self, ast: &Ast, context: &Context, ast_context: &AstContext) -> Vec<Diagnostic> {
        let mut visitor = DeprecatedVisitor {
            diagnostics: Vec::new(),
            scope_manager: &ast_context.scope_manager,
            standard_library: &context.standard_library,
        };

        visitor.visit_ast(ast);

        visitor.diagnostics
    }

    fn rule_type(&self) -> RuleType {
        RuleType::Correctness
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }
}

struct DeprecatedVisitor<'a> {
    diagnostics: Vec<Diagnostic>,
    scope_manager: &'a ScopeManager,
    standard_library: &'a StandardLibrary,
}

impl DeprecatedVisitor<'_> {
    fn check_name_path<N: Node>(
        &mut self,
        node: &N,
        what: &str,
        name_path: &[String],
        parameters: &[String],
    ) {
        assert!(!name_path.is_empty());

        for bound in 1..=name_path.len() {
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
            DeprecatedLint::new(()).unwrap(),
            "deprecated",
            "deprecated_fields",
        );
    }

    #[test]
    fn test_deprecated_functions() {
        test_lint(
            DeprecatedLint::new(()).unwrap(),
            "deprecated",
            "deprecated_functions",
        );
    }

    #[test]
    fn test_toml_forwards_compatibility() {
        test_lint(
            DeprecatedLint::new(()).unwrap(),
            "deprecated",
            "toml_forwards_compatibility",
        );
    }
}
