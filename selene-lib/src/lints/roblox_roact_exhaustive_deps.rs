use super::*;
use crate::ast_util::range;
use std::{
    collections::{HashMap, HashSet},
    convert::Infallible,
    hash::Hash,
};

use full_moon::{
    ast::{self, Ast, FunctionCall},
    visitors::Visitor,
};
use if_chain::if_chain;

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
            scope_manager,
            missing_dependencies: Vec::new(),
            non_reactive_upvalue_starts: HashSet::new(),
        };

        visitor.visit_ast(ast);

        let mut diagnostics = Vec::new();

        for invalid_event in visitor.missing_dependencies {
            let missing_dependencies = invalid_event
                .missing_dependencies
                .iter()
                .filter(|upvalue| !context.standard_library.global_has_fields(&upvalue.name))
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
            1 => format!("'{}'", missing_dependencies[0].name),
            2 => format!(
                "'{}' and '{}'",
                missing_dependencies[0].name, missing_dependencies[1].name
            ),
            _ => {
                let all_but_last = missing_dependencies[..missing_dependencies.len() - 1]
                    .iter()
                    .map(|upvalue| format!("'{}'", &upvalue.name))
                    .collect::<Vec<String>>()
                    .join(", ");
                format!(
                    "{}, and '{}'",
                    all_but_last,
                    missing_dependencies.last().unwrap().name
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

fn is_roact_function(call: &FunctionCall) -> bool {
    if let ast::Prefix::Name(name) = call.prefix() {
        return name.token().to_string() == "Roact"
            || name.token().to_string() == "React"
            || name.token().to_string() == "hooks";
    }
    false
}

#[derive(Debug)]
struct RoactMissingDependencyVisitor<'a> {
    scope_manager: &'a ScopeManager,
    missing_dependencies: Vec<MissingDependency>,

    // Some variables are safe to omit from the dependency array, such as setState
    non_reactive_upvalue_starts: HashSet<usize>,
}

#[derive(Clone, Debug, Eq)]
struct Upvalue {
    name: String,
    identifier_start_range: usize,

    // Knowing where referenced variable was initialized lets us narrow down whether it's a reactive variable
    resolved_start_range: Option<usize>,
}

// Ensures we don't report a variable more than once if it's used multiple times in an effect
impl Hash for Upvalue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for Upvalue {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

#[derive(Debug)]
struct MissingDependency {
    missing_dependencies: Vec<Upvalue>,
    range: (usize, usize),
}

impl Visitor for RoactMissingDependencyVisitor<'_> {
    fn visit_function_call(&mut self, call: &ast::FunctionCall) {
        let last_suffix =
            get_last_function_call_suffix(call.prefix(), &call.suffixes().collect::<Vec<_>>());

        let function_args = match call.suffixes().last() {
            Some(ast::Suffix::Call(ast::Call::AnonymousCall(args))) => args,
            _ => return,
        };

        if last_suffix.as_str() == "useEffect" && is_roact_function(call) {
            if let ast::FunctionArgs::Parentheses { arguments, .. } = function_args {
                if let Some(dependency_array_expr) = arguments.iter().nth(1) {
                    let referenced_upvalues =
                        if let Some(ast::punctuated::Pair::Punctuated(effect_callback, ..)) =
                            arguments.first()
                        {
                            self.scope_manager
                                .references
                                .iter()
                                .filter_map(|(_, reference)| {
                                    if reference.identifier.0 > range(effect_callback).0
                                        && reference.identifier.1 < range(effect_callback).1
                                        && reference.read
                                    {
                                        let resolved_start_range = if let Some(resolved) =
                                            reference.resolved
                                        {
                                            let variable = &self.scope_manager.variables[resolved];

                                            // FIXME: We need the start range where the variable was last set. Otherwise
                                            // a variable can be first set outside but set again inside a component, and it
                                            // identifies as non-reactive. However, this seems to only capture when user
                                            // does `local` again. Is there an alternative to also capture var = without local?
                                            // This is low priority as this only matters if user does something weird, like
                                            // writing to an outside variable within a component
                                            variable.identifiers.last().map(|(start, _)| *start)
                                        } else {
                                            None
                                        };

                                        Some(Upvalue {
                                            name: reference.name.clone(),
                                            identifier_start_range: reference.identifier.0,
                                            resolved_start_range,
                                        })
                                    } else {
                                        None
                                    }
                                })
                                .collect::<HashSet<_>>()
                        } else {
                            return;
                        };

                    let dependencies_list = self
                        .scope_manager
                        .references
                        .iter()
                        .filter_map(|(_, reference)| {
                            if reference.identifier.0 > range(dependency_array_expr).0
                                && reference.identifier.1 < range(dependency_array_expr).1
                                && reference.read
                            {
                                let upvalue = Upvalue {
                                    name: reference.name.clone(),
                                    identifier_start_range: reference.identifier.0,
                                    resolved_start_range: None,
                                };
                                Some((upvalue.name.clone(), upvalue))
                            } else {
                                None
                            }
                        })
                        .collect::<HashMap<_, _>>();

                    let mut missing_dependencies: Vec<_> = referenced_upvalues
                        .iter()
                        .filter(|upvalue| {
                            let is_non_reactive =
                                upvalue.resolved_start_range.map_or(false, |start_range| {
                                    self.non_reactive_upvalue_starts.contains(&start_range)
                                });

                            !dependencies_list.contains_key(&upvalue.name) && !is_non_reactive
                        })
                        .cloned()
                        .collect();

                    if !missing_dependencies.is_empty() {
                        missing_dependencies.sort_by_key(|upvalue| upvalue.identifier_start_range);

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

    fn visit_local_assignment(&mut self, assignment: &ast::LocalAssignment) {
        if_chain! {
            if let Some(ast::punctuated::Pair::End(expression)) = assignment.expressions().first();
            if let ast::Expression::Value { value, .. } = expression;
            if let ast::Value::FunctionCall(call) = &**value;
            if is_roact_function(call);
            then {
                let function_suffix = get_last_function_call_suffix(
                    call.prefix(),
                    &call.suffixes().collect::<Vec<_>>(),
                );

                // Setter functions are stable and can be omitted from dependency array
                if function_suffix == "useState" || function_suffix == "useBinding" {
                    if let Some(second_var) = assignment.names().iter().nth(1) {
                        self.non_reactive_upvalue_starts
                            .insert(range(second_var).0);
                    }
                }
            }
        }
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
