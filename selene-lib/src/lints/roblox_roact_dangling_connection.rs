use super::*;
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

    fn new(_: Self::Config) -> Result<Self, Self::Error> {
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
                            if ["Connect", "connect", "ConnectParallel", "Once"]
                                .contains(&last_name.as_str())
                            {
                                Some(function_call_stmt.call_prefix_range.0)
                            } else {
                                None
                            }
                        })
                })
                .collect(),
            function_contexts: Vec::new(),
        };

        visitor.visit_ast(ast);

        let mut diagnostics = Vec::new();

        for invalid_event in visitor.dangling_connections {
            match invalid_event.function_context {
                ConnectionContext::UseEffect => {
                    diagnostics.push(Diagnostic::new(
                        "roblox_roact_dangling_connection",
                        "disconnect the connection in the useEffect cleanup function".to_owned(),
                        Label::new(invalid_event.range),
                    ));
                }
                _ => {
                    diagnostics.push(Diagnostic::new(
                        "roblox_roact_dangling_connection",
                        "disconnect the connection where appropriate".to_owned(),
                        Label::new(invalid_event.range),
                    ));
                }
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
                    "".to_owned()
                };
            } else {
                // In a.b(), b is the suffix before the last one
                Some(&suffixes[suffixes.len() - 2])
            }
        }
        _ => return "".to_owned(),
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
            _ => "".to_string(),
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

        self.function_contexts.push((
            ConnectionContextType::FunctionCall,
            match last_suffix.as_str() {
                "useEffect" => ConnectionContext::UseEffect,
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
