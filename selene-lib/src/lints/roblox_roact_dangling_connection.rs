use super::{AstContext, Context, Diagnostic, Label, Lint, LintType, Node, Severity};
use crate::ast_util::range;
use std::{collections::HashSet, convert::Infallible};

use full_moon::{
    ast::{self, Ast},
    visitors::Visitor,
};

pub struct RoactDanglingConnectionLint;

impl Lint for RoactDanglingConnectionLint {
    type Config = ();
    type Error = Infallible;

    const SEVERITY: Severity = Severity::Error;
    const LINT_TYPE: LintType = LintType::Correctness;

    fn new((): Self::Config) -> Result<Self, Self::Error> {
        Ok(Self)
    }

    fn pass(
        &self,
        ast: &Ast,
        context: &Context,
        AstContext { scope_manager, .. }: &AstContext,
    ) -> Vec<Diagnostic> {
        if !context.is_roblox() {
            return Vec::new();
        }

        if scope_manager.variables.iter().all(|(_, variable)| {
            !["Roact", "React"].contains(&variable.name.trim_end_matches(char::is_whitespace))
        }) {
            return Vec::new();
        }

        let mut visitor = RoactDanglingConnectionVisitor {
            dangling_connections: Vec::new(),
            dangling_connection_start_ranges: scope_manager
                .function_calls
                .iter()
                .filter_map(|(_, function_call_stmt)| {
                    function_call_stmt
                        .call_name_path
                        .last()
                        .and_then(|last_name| {
                            ["Connect", "connect", "ConnectParallel", "Once"]
                                .contains(&last_name.as_str())
                                .then_some(function_call_stmt.call_prefix_range.0)
                        })
                })
                .collect(),
            function_contexts: Vec::new(),
            definitions_of_roact_functions: HashSet::new(),
        };

        visitor.visit_ast(ast);

        let mut diagnostics = Vec::new();

        for invalid_event in visitor.dangling_connections {
            if let ConnectionContext::UseEffect = invalid_event.function_context {
                diagnostics.push(Diagnostic::new(
                    "roblox_roact_dangling_connection",
                    "disconnect the connection in the useEffect cleanup function".to_owned(),
                    Label::new(invalid_event.range),
                ));
            } else {
                diagnostics.push(Diagnostic::new(
                    "roblox_roact_dangling_connection",
                    "disconnect the connection where appropriate".to_owned(),
                    Label::new(invalid_event.range),
                ));
            }
        }

        diagnostics
    }
}

fn get_last_function_call_suffix(prefix: &ast::Prefix, suffixes: &[&ast::Suffix]) -> String {
    let last_suffix = match suffixes.last() {
        Some(ast::Suffix::Call(ast::Call::MethodCall(_))) => suffixes.last(),
        Some(ast::Suffix::Call(ast::Call::AnonymousCall(_))) => {
            if suffixes.len() == 1 {
                // a()
                return if let ast::Prefix::Name(name) = prefix {
                    name.token().to_string()
                } else {
                    String::new()
                };
            } else {
                // In a.b(), b is the suffix before the last one
                Some(&suffixes[suffixes.len() - 2])
            }
        }
        _ => return String::new(),
    };

    last_suffix
        .map(|suffix| match suffix {
            ast::Suffix::Index(ast::Index::Dot { name, .. }) => name.token().to_string(),
            ast::Suffix::Call(ast::Call::MethodCall(method_call)) => {
                method_call.name().token().to_string()
            }
            ast::Suffix::Call(ast::Call::AnonymousCall(anonymous_call)) => {
                anonymous_call.to_string()
            }
            _ => String::new(),
        })
        .unwrap_or_default()
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum ConnectionContext {
    Unknown,
    UseEffect,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum ConnectionContextType {
    FunctionBody,
    FunctionCall,
}

#[derive(Debug)]
struct RoactDanglingConnectionVisitor {
    dangling_connections: Vec<DanglingConnection>,
    dangling_connection_start_ranges: HashSet<usize>,
    function_contexts: Vec<(ConnectionContextType, ConnectionContext)>,
    definitions_of_roact_functions: HashSet<String>,
}

#[derive(Debug)]
struct DanglingConnection {
    range: (usize, usize),
    function_context: ConnectionContext,
}

fn get_last_known_context(
    function_contexts: &[(ConnectionContextType, ConnectionContext)],
) -> ConnectionContext {
    match function_contexts
        .iter()
        .rev()
        .find(|(_, connection_type)| *connection_type != ConnectionContext::Unknown)
    {
        Some(context) => context.1,
        None => ConnectionContext::Unknown,
    }
}

fn is_roact_function(prefix: &ast::Prefix) -> bool {
    if let ast::Prefix::Name(prefix_token) = prefix {
        ["roact", "react", "hooks"]
            .contains(&prefix_token.token().to_string().to_lowercase().as_str())
    } else {
        false
    }
}

impl Visitor for RoactDanglingConnectionVisitor {
    fn visit_function_call(&mut self, call: &ast::FunctionCall) {
        let last_suffix =
            get_last_function_call_suffix(call.prefix(), &call.suffixes().collect::<Vec<_>>());

        if !self.function_contexts.is_empty() {
            if let Some(call_range) = call.range() {
                if self
                    .dangling_connection_start_ranges
                    .contains(&call_range.0.bytes())
                {
                    self.dangling_connections.push(DanglingConnection {
                        range: range(call),
                        function_context: get_last_known_context(&self.function_contexts),
                    });
                }
            }
        }

        // Check if caller is Roact.<function> or a variable defined to it
        let mut suffixes = call.suffixes().collect::<Vec<_>>();
        suffixes.pop();

        let mut is_this_roact_function = false;

        if suffixes.is_empty() {
            // Call is foo(), not foo.bar()
            if let ast::Prefix::Name(name) = call.prefix() {
                is_this_roact_function = self
                    .definitions_of_roact_functions
                    .get(&name.token().to_string())
                    .is_some();
            }
        } else if suffixes.len() == 1 {
            // Call is foo.bar()
            is_this_roact_function = is_roact_function(call.prefix());
        }

        self.function_contexts.push((
            ConnectionContextType::FunctionCall,
            match last_suffix.as_str() {
                "useEffect" if is_this_roact_function => ConnectionContext::UseEffect,
                _ => ConnectionContext::Unknown,
            },
        ));
    }

    fn visit_function_call_end(&mut self, _: &ast::FunctionCall) {
        self.function_contexts.pop();
    }

    fn visit_function_body(&mut self, _: &ast::FunctionBody) {
        self.function_contexts.push((
            ConnectionContextType::FunctionBody,
            ConnectionContext::Unknown,
        ));
    }

    fn visit_function_body_end(&mut self, _: &ast::FunctionBody) {
        self.function_contexts.pop();
    }

    fn visit_local_assignment(&mut self, node: &ast::LocalAssignment) {
        for (name, expr) in node.names().iter().zip(node.expressions().iter()) {
            if let ast::Expression::Var(ast::Var::Expression(var_expr)) = expr {
                if is_roact_function(var_expr.prefix()) {
                    self.definitions_of_roact_functions
                        .insert(name.token().to_string());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    fn test_no_roact() {
        test_lint(
            RoactDanglingConnectionLint::new(()).unwrap(),
            "roblox_roact_dangling_connection",
            "no_roact",
        );
    }

    #[test]
    fn test_roblox_roact_dangling_connection() {
        test_lint(
            RoactDanglingConnectionLint::new(()).unwrap(),
            "roblox_roact_dangling_connection",
            "roblox_roact_dangling_connection",
        );
    }

    #[test]
    fn test_with_roact() {
        test_lint(
            RoactDanglingConnectionLint::new(()).unwrap(),
            "roblox_roact_dangling_connection",
            "with_roact",
        );
    }
}
