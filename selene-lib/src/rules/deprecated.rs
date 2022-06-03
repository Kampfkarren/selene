use super::{super::standard_library::*, *};
use crate::{
    ast_util::{
        name_paths::{name_path_from_prefix_suffix, take_while_keep_going},
        scopes::ScopeManager,
    },
    standard_library,
};
use std::convert::Infallible;

use full_moon::{
    ast::{self, Ast},
    node::Node,
    visitors::Visitor,
};

pub struct DeprecatedLint;

impl Rule for DeprecatedLint {
    type Config = ();
    type Error = Infallible;

    fn new(_: Self::Config) -> Result<Self, Self::Error> {
        Ok(DeprecatedLint)
    }

    fn pass(&self, ast: &Ast, context: &Context) -> Vec<Diagnostic> {
        let mut visitor = DeprecatedVisitor {
            diagnostics: Vec::new(),
            scope_manager: ScopeManager::new(ast),
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
    scope_manager: ScopeManager,
    standard_library: &'a StandardLibrary,
}

impl Visitor for DeprecatedVisitor<'_> {
    fn visit_function_call(&mut self, call: &ast::FunctionCall) {
        if let Some(reference) = self
            .scope_manager
            .reference_at_byte(call.start_position().unwrap().bytes())
        {
            if reference.resolved.is_some() {
                return;
            }

            let mut keep_going = true;
            let mut suffixes: Vec<&ast::Suffix> = call
                .suffixes()
                .take_while(|suffix| take_while_keep_going(suffix, &mut keep_going))
                .collect();

            let name_path =
                match name_path_from_prefix_suffix(call.prefix(), suffixes.iter().copied()) {
                    Some(name_path) => name_path,
                    None => return,
                };

            let call_suffix = suffixes.pop().unwrap();

            let field = match self.standard_library.find_global(&name_path) {
                Some(field) => field,
                None => return,
            };

            let deprecated = match &field {
                standard_library::Field::Complex {
                    function:
                        Some(FunctionBehavior {
                            deprecated: Some(deprecated),
                            ..
                        }),
                    ..
                } => deprecated,

                _ => {
                    return;
                }
            };

            let function_args = match call_suffix {
                ast::Suffix::Call(call) =>
                {
                    #[cfg_attr(
                        feature = "force_exhaustive_checks",
                        deny(non_exhaustive_omitted_patterns)
                    )]
                    match call {
                        ast::Call::AnonymousCall(args) => args,
                        ast::Call::MethodCall(method_call) => method_call.args(),
                        _ => return,
                    }
                }

                _ => unreachable!("function_call.call_suffix != ast::Suffix::Call"),
            };

            #[cfg_attr(
                feature = "force_exhaustive_checks",
                deny(non_exhaustive_omitted_patterns)
            )]
            let parameters = match function_args {
                ast::FunctionArgs::Parentheses { arguments, .. } => arguments
                    .iter()
                    .map(|argument| argument.to_string())
                    .collect(),
                ast::FunctionArgs::String(string) => vec![string.to_string()],
                ast::FunctionArgs::TableConstructor(table) => vec![table.to_string()],
                _ => Vec::new(),
            };

            let mut notes = vec![deprecated.message.to_owned()];

            if let Some(replace_with) = deprecated.try_instead(&parameters) {
                notes.push(format!("try: {replace_with}"));
            }

            self.diagnostics.push(Diagnostic::new_complete(
                "deprecated",
                format!(
                    "standard library function `{}` is deprecated",
                    name_path.join(".")
                ),
                Label::from_node(call, None),
                notes,
                Vec::new(),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::*, *};

    #[test]
    fn test_deprecated_functions() {
        test_lint(
            DeprecatedLint::new(()).unwrap(),
            "deprecated",
            "deprecated_functions",
        );
    }
}
