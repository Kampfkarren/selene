use super::*;
use crate::ast_util::range;
use std::{
    collections::{HashMap, HashSet},
    convert::Infallible,
};

use full_moon::{
    ast::{self, Ast},
    tokenizer::{TokenReference, TokenType},
    visitors::Visitor,
};

pub struct RoactExhaustiveDepsLint;

impl Lint for RoactExhaustiveDepsLint {
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

        let mut visitor = RoactMissingDependencyVisitor {
            missing_dependencies: Vec::new(),
            upvalue_start_bytes_to_depth: HashMap::new(),
            current_depth: 0,
        };

        visitor.visit_ast(ast);

        let mut diagnostics = Vec::new();

        for invalid_event in visitor.missing_dependencies {
            let missing_dependencies = invalid_event
                .missing_dependencies
                .iter()
                .filter(|upvalue| {
                    !context
                        .standard_library
                        .global_has_fields(&upvalue.identifier)
                })
                .collect::<Vec<_>>();

            if !missing_dependencies.is_empty() {
                diagnostics.push(Diagnostic::new(
                    "roblox_roact_exhaustive_deps",
                    get_formatted_error_message(&missing_dependencies),
                    Label::new(invalid_event.range),
                ));
            }
        }

        diagnostics
    }
}

fn get_formatted_error_message(missing_dependencies: &Vec<&Upvalue>) -> String {
    format!(
        "React hook useEffect has {}: {}. Either include {} or remove the dependency array.",
        if missing_dependencies.len() == 1 {
            "a missing dependency"
        } else {
            "missing dependencies"
        },
        match missing_dependencies.len() {
            1 => format!("'{}'", missing_dependencies[0].identifier),
            2 => format!(
                "'{}' and '{}'",
                missing_dependencies[0].identifier, missing_dependencies[1].identifier
            ),
            _ => {
                let all_but_last = missing_dependencies[..missing_dependencies.len() - 1]
                    .iter()
                    .map(|upvalue| format!("'{}'", &upvalue.identifier))
                    .collect::<Vec<String>>()
                    .join(", ");
                format!(
                    "{}, and '{}'",
                    all_but_last,
                    missing_dependencies.last().unwrap().identifier
                )
            }
        },
        if missing_dependencies.len() == 1 {
            "it"
        } else {
            "them"
        },
    )
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

enum NodeType<'a> {
    Expression(&'a ast::Expression),
    FunctionCall(&'a ast::FunctionCall),
    VarExpression(&'a ast::VarExpression),
}

fn get_token_identifier(token: &TokenReference) -> String {
    match token.token_type() {
        TokenType::Identifier { identifier } => identifier.to_string(),
        _ => "".to_string(),
    }
}

fn add_referenced_vars(
    referenced_vars: &mut Vec<Upvalue>,
    fn_defined_vars: &HashSet<String>,
    new_vars: &[Upvalue],
) {
    referenced_vars.extend(
        new_vars
            .iter()
            // Filter out variables defined in the function so far as they are no longer upvalues
            .filter(|var| !fn_defined_vars.contains(var.identifier.as_str()))
            .cloned()
            .collect::<Vec<_>>(),
    );
}

// local a = b + c -> [b, c]
// d = e(f) -> [e, f]
// { g, h.i, j[k], l["m"] } -> [g, [h, i], [j, k], l]
fn get_referenced_upvalues(expression_type: &NodeType) -> Vec<Upvalue> {
    let mut referenced_vars = Vec::new();
    let mut fn_defined_vars: HashSet<String> = HashSet::new();

    match expression_type {
        NodeType::Expression(expression) => {
            match expression {
                ast::Expression::Value { value, .. } => match &**value {
                    ast::Value::Var(var) => {
                        if let ast::Var::Name(token) = var {
                            referenced_vars.push(Upvalue {
                                identifier: get_token_identifier(token),
                            });
                        } else if let ast::Var::Expression(value) = var {
                            add_referenced_vars(
                                &mut referenced_vars,
                                &fn_defined_vars,
                                &get_referenced_upvalues(&NodeType::VarExpression(value)),
                            );
                        }
                    }
                    ast::Value::TableConstructor(table) => {
                        for field in table.fields() {
                            if let ast::Field::NoKey(value) = field {
                                // TODO: Store this somewhere else so we know which are from one dependency
                                add_referenced_vars(
                                    &mut referenced_vars,
                                    &fn_defined_vars,
                                    &get_referenced_upvalues(&NodeType::Expression(value)),
                                );
                            }
                        }
                    }
                    ast::Value::Function((_, function_body)) => {
                        for stmt in function_body.block().stmts() {
                            if let ast::Stmt::Assignment(assignment) = stmt {
                                for variable in assignment.variables() {
                                    if let ast::Var::Name(name) = variable {
                                        // FIXME: this works well with assigning single variables,
                                        // but would false positive with `a.b = c; d = a.b.somethingelse`
                                        fn_defined_vars.insert(get_token_identifier(name));
                                    }
                                }

                                for expr in assignment.expressions() {
                                    add_referenced_vars(
                                        &mut referenced_vars,
                                        &fn_defined_vars,
                                        &get_referenced_upvalues(&NodeType::Expression(expr)),
                                    );
                                }
                            } else if let ast::Stmt::LocalAssignment(assignment) = stmt {
                                for variable in assignment.names() {
                                    fn_defined_vars.insert(get_token_identifier(variable));
                                }

                                for expr in assignment.expressions() {
                                    add_referenced_vars(
                                        &mut referenced_vars,
                                        &fn_defined_vars,
                                        &get_referenced_upvalues(&NodeType::Expression(expr)),
                                    );
                                }
                            } else if let ast::Stmt::FunctionCall(call) = stmt {
                                add_referenced_vars(
                                    &mut referenced_vars,
                                    &fn_defined_vars,
                                    &get_referenced_upvalues(&NodeType::FunctionCall(call)),
                                );
                            }
                        }
                    }
                    ast::Value::FunctionCall(call) => {
                        add_referenced_vars(
                            &mut referenced_vars,
                            &fn_defined_vars,
                            &get_referenced_upvalues(&NodeType::FunctionCall(call)),
                        );
                    }
                    ast::Value::InterpolatedString(interpolated_string) => {
                        for expression in interpolated_string.expressions() {
                            add_referenced_vars(
                                &mut referenced_vars,
                                &fn_defined_vars,
                                &get_referenced_upvalues(&NodeType::Expression(expression)),
                            );
                        }
                    }
                    _ => {}
                },
                ast::Expression::BinaryOperator { lhs, rhs, .. } => {
                    add_referenced_vars(
                        &mut referenced_vars,
                        &fn_defined_vars,
                        &get_referenced_upvalues(&NodeType::Expression(lhs)),
                    );
                    add_referenced_vars(
                        &mut referenced_vars,
                        &fn_defined_vars,
                        &get_referenced_upvalues(&NodeType::Expression(rhs)),
                    );
                }
                ast::Expression::UnaryOperator { expression, .. } => {
                    add_referenced_vars(
                        &mut referenced_vars,
                        &fn_defined_vars,
                        &get_referenced_upvalues(&NodeType::Expression(expression)),
                    );
                }
                ast::Expression::Parentheses { expression, .. } => {
                    add_referenced_vars(
                        &mut referenced_vars,
                        &fn_defined_vars,
                        &get_referenced_upvalues(&NodeType::Expression(expression)),
                    );
                }
                _ => {}
            }
        }
        NodeType::FunctionCall(call) => {
            for suffix in call.suffixes() {
                if let ast::Suffix::Call(ast::Call::AnonymousCall(
                    ast::FunctionArgs::Parentheses { arguments, .. },
                )) = suffix
                {
                    for arg in arguments.pairs() {
                        let expr = match arg {
                            ast::punctuated::Pair::Punctuated(expr, _)
                            | ast::punctuated::Pair::End(expr) => expr,
                        };
                        add_referenced_vars(
                            &mut referenced_vars,
                            &fn_defined_vars,
                            &get_referenced_upvalues(&NodeType::Expression(expr)),
                        );
                    }
                }
            }

            if let ast::Prefix::Name(prefix) = call.prefix() {
                add_referenced_vars(
                    &mut referenced_vars,
                    &fn_defined_vars,
                    &[Upvalue {
                        identifier: get_token_identifier(prefix),
                    }],
                );
            }
        }
        NodeType::VarExpression(expression) => {
            match &expression.prefix() {
                ast::Prefix::Expression(expr) => {
                    referenced_vars.extend(
                        expr.tokens()
                            .map(|token| Upvalue {
                                identifier: get_token_identifier(token),
                            })
                            .collect::<Vec<_>>(),
                    );
                }
                ast::Prefix::Name(token) => {
                    referenced_vars.push(Upvalue {
                        identifier: get_token_identifier(token),
                    });
                }
                _ => {}
            };

            for suffix in expression.suffixes() {
                if let ast::Suffix::Index(index) = suffix {
                    if let ast::Index::Dot { name, .. } = index {
                        referenced_vars.push(Upvalue {
                            identifier: get_token_identifier(name),
                        });
                    } else if let ast::Index::Brackets { expression, .. } = index {
                        referenced_vars.push(Upvalue {
                            identifier: expression.to_string(),
                        });
                    }
                }
            }
        }
    }

    referenced_vars
}

#[derive(Debug)]
struct RoactMissingDependencyVisitor {
    missing_dependencies: Vec<MissingDependency>,
    upvalue_start_bytes_to_depth: HashMap<usize, usize>,
    current_depth: usize,
}

#[derive(Clone, Debug)]
struct Upvalue {
    identifier: String,
}

#[derive(Debug)]
struct MissingDependency {
    missing_dependencies: Vec<Upvalue>,
    range: (usize, usize),
}

impl Visitor for RoactMissingDependencyVisitor {
    fn visit_function_call(&mut self, call: &ast::FunctionCall) {
        let last_suffix =
            get_last_function_call_suffix(call.prefix(), &call.suffixes().collect::<Vec<_>>());

        let function_args = match call.suffixes().last() {
            Some(ast::Suffix::Call(ast::Call::AnonymousCall(args))) => args,
            _ => return,
        };

        if last_suffix.as_str() == "useEffect" {
            if let ast::FunctionArgs::Parentheses { arguments, .. } = function_args {
                let referenced_upvalues =
                    if let Some(ast::punctuated::Pair::Punctuated(expression, ..)) =
                        arguments.first()
                    {
                        get_referenced_upvalues(&NodeType::Expression(expression))
                    } else {
                        return;
                    };

                if let Some(dependency_array_expr) = arguments.iter().nth(1) {
                    let dependencies_list: HashMap<String, Upvalue> =
                        get_referenced_upvalues(&NodeType::Expression(dependency_array_expr))
                            .into_iter()
                            .map(|upvalue| (upvalue.identifier.clone(), upvalue))
                            .collect();

                    let missing_dependencies: Vec<_> = referenced_upvalues
                        .iter()
                        .filter(|upvalue| !dependencies_list.contains_key(&upvalue.identifier))
                        .cloned()
                        .collect();

                    if !missing_dependencies.is_empty() {
                        self.missing_dependencies.push(MissingDependency {
                            missing_dependencies,
                            range: range(dependency_array_expr),
                        });
                    }
                } else {
                }
            }
        }
    }

    fn visit_assignment(&mut self, _node: &ast::Assignment) {}

    fn visit_stmt(&mut self, stmt: &ast::Stmt) {
        self.current_depth += 1;
        self.upvalue_start_bytes_to_depth
            .insert(range(stmt).0, self.current_depth);
    }

    fn visit_stmt_end(&mut self, _: &ast::Stmt) {
        self.current_depth -= 1;

        self.upvalue_start_bytes_to_depth
            .retain(|_, depth| *depth < self.current_depth);
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    fn test_ignore_globals() {
        test_lint(
            RoactExhaustiveDepsLint::new(()).unwrap(),
            "roblox_roact_exhaustive_deps",
            "ignore_globals",
        );
    }

    #[test]
    fn test_no_roact() {
        test_lint(
            RoactExhaustiveDepsLint::new(()).unwrap(),
            "roblox_roact_exhaustive_deps",
            "no_roact",
        );
    }

    #[test]
    fn test_roblox_roact_dangling_connection() {
        test_lint(
            RoactExhaustiveDepsLint::new(()).unwrap(),
            "roblox_roact_exhaustive_deps",
            "roblox_roact_exhaustive_deps",
        );
    }
}
