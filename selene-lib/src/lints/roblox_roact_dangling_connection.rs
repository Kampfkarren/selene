use super::*;
use crate::ast_util::range;
use std::{collections::HashMap, convert::Infallible};

use full_moon::{
    ast::{self, Ast, Expression},
    tokenizer::TokenReference,
    visitors::Visitor,
};

pub struct RoactDanglingConnectionLint;

impl Lint for RoactDanglingConnectionLint {
    type Config = ();
    type Error = Infallible;

    const SEVERITY: Severity = Severity::Error;
    const LINT_TYPE: LintType = LintType::Correctness;

    fn new(_: Self::Config) -> Result<Self, Self::Error> {
        Ok(RoactDanglingConnectionLint)
    }

    fn pass(&self, ast: &Ast, context: &Context, _: &AstContext) -> Vec<Diagnostic> {
        if !context.is_roblox() {
            return Vec::new();
        }

        let mut visitor = RoactDanglingConnectionVisitor {
            dangling_connections: Vec::new(),
            has_roact_in_file: false,
            assignments: HashMap::new(),
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
        .unwrap_or_else(|| "".to_owned())
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
    has_roact_in_file: bool,
    assignments: HashMap<(usize, usize), String>,
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

impl RoactDanglingConnectionVisitor {
    fn process_assignment(&mut self, name: &TokenReference, expression: &Expression) {
        if ["Roact", "React"].contains(
            &name
                .token()
                .to_string()
                .trim_end_matches(char::is_whitespace),
        ) {
            self.has_roact_in_file = true;
        }

        if let ast::Expression::Value { value, .. } = expression {
            if let ast::Value::FunctionCall(_) = &**value {
                self.assignments
                    .insert(range(value), name.token().to_string());
            }
        }
    }
}

impl Visitor for RoactDanglingConnectionVisitor {
    fn visit_function_call(&mut self, call: &ast::FunctionCall) {
        let last_suffix =
            get_last_function_call_suffix(call.prefix(), &call.suffixes().collect::<Vec<_>>());

        // Ignore cases like a(b:Connect()) where connection is passed as an argument
        let is_immediately_in_function_call =
            self.function_contexts.last().map_or(false, |context| {
                context.0 == ConnectionContextType::FunctionCall
            });

        let is_call_assigned_to_variable = self.assignments.contains_key(&range(call));

        if self.has_roact_in_file
            && !is_immediately_in_function_call
            && !is_call_assigned_to_variable
            // Ignore connections on the top level as they are not in a Roact component
            && !self.function_contexts.is_empty()
            && ["Connect", "connect", "ConnectParallel", "Once"].contains(&last_suffix.as_str())
        {
            self.dangling_connections.push(DanglingConnection {
                range: range(call),
                function_context: get_last_known_context(&self.function_contexts),
            });
        }

        self.function_contexts.push((
            ConnectionContextType::FunctionCall,
            match last_suffix.as_str() {
                "useEffect" => ConnectionContext::UseEffect,
                _ => ConnectionContext::Unknown,
            },
        ));
    }

    fn visit_function_call_end(&mut self, _node: &ast::FunctionCall) {
        self.function_contexts.pop();
    }

    fn visit_assignment(&mut self, assignment: &ast::Assignment) {
        for (var, expression) in assignment
            .variables()
            .iter()
            .zip(assignment.expressions().iter())
        {
            if let ast::Var::Name(name) = var {
                self.process_assignment(name, expression);
            }
        }
    }

    fn visit_local_assignment(&mut self, node: &ast::LocalAssignment) {
        for (name, expression) in node.names().iter().zip(node.expressions().iter()) {
            self.process_assignment(name, expression)
        }
    }

    fn visit_function_body(&mut self, _node: &ast::FunctionBody) {
        self.function_contexts.push((
            ConnectionContextType::FunctionBody,
            ConnectionContext::Unknown,
        ));
    }

    fn visit_function_body_end(&mut self, _node: &ast::FunctionBody) {
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
