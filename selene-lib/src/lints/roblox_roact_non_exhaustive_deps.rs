use super::*;
use crate::ast_util::{range, scopes::Reference};
use std::{
    collections::{HashMap, HashSet},
    convert::Infallible,
    hash::Hash,
    vec,
};

use full_moon::{
    ast::{self, Ast, Expression, FunctionCall},
    tokenizer::TokenType,
    visitors::Visitor,
};
use if_chain::if_chain;

pub struct RoactNonExhaustiveDepsLint;

impl Lint for RoactNonExhaustiveDepsLint {
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
            fn_declaration_starts_stack: Vec::new(),
            missing_dependencies: Vec::new(),
            unnecessary_dependencies: Vec::new(),
            complex_dependencies: Vec::new(),
            non_reactive_upvalue_starts: HashSet::new(),
        };

        visitor.visit_ast(ast);

        let mut diagnostics = Vec::new();

        for invalid_event in visitor.missing_dependencies {
            if !invalid_event.missing_dependencies.is_empty() {
                diagnostics.push(Diagnostic::new_complete(
                    "roblox_roact_non_exhaustive_deps",
                    get_formatted_error_message(
                        &invalid_event.hook_name,
                        &invalid_event.missing_dependencies,
                        "missing",
                    ),
                    Label::new(invalid_event.range),
                    vec![format!(
                        "help: either include {} or remove the dependency array",
                        if invalid_event.missing_dependencies.len() == 1 {
                            "it"
                        } else {
                            "them"
                        },
                    )],
                    Vec::new(),
                ));
            }
        }

        for invalid_event in visitor.unnecessary_dependencies {
            if let Some(first_unnecessary_dependency) =
                invalid_event.unnecessary_dependencies.first()
            {
                diagnostics.push(Diagnostic::new_complete(
                    "roblox_roact_non_exhaustive_deps",
                    get_formatted_error_message(
                        &invalid_event.hook_name,
                        &invalid_event.unnecessary_dependencies,
                        "unnecessary",
                    ),
                    Label::new(invalid_event.range),
                    vec![format!(
                        "help: either exclude {} or remove the dependency array",
                        if invalid_event.unnecessary_dependencies.len() == 1 {
                            "it"
                        } else {
                            "them"
                        },
                    ), format!(
                        "outer scope variables like '{}' aren't valid dependencies because mutating them doesn't re-render the component",
                        first_unnecessary_dependency.prefix,
                    )],
                    Vec::new(),
                ));
            }
        }

        for invalid_event in visitor.complex_dependencies {
            diagnostics.push(Diagnostic::new_complete(
                "roblox_roact_non_exhaustive_deps",
                format!(
                    "react hook {} has a complex expression in the dependency array",
                    invalid_event.hook_name
                ),
                Label::new(invalid_event.range),
                vec![
                    "help: extract it to a separate variable so it can be statically checked"
                        .to_string(),
                ],
                Vec::new(),
            ));
        }

        diagnostics
    }
}

fn get_formatted_error_message(
    hook_name: &String,
    missing_dependencies: &Vec<Upvalue>,
    missing_or_unnecessary: &str,
) -> String {
    format!(
        "react hook {} has {}: {}",
        hook_name,
        if missing_dependencies.len() == 1 {
            format!("{} dependency", missing_or_unnecessary)
        } else {
            format!("{} dependencies", missing_or_unnecessary)
        },
        match missing_dependencies.len() {
            1 => format!("'{}'", missing_dependencies[0].indexing_expression_name()),
            2 => format!(
                "'{}' and '{}'",
                missing_dependencies[0].indexing_expression_name(),
                missing_dependencies[1].indexing_expression_name()
            ),
            _ => {
                let all_but_last = missing_dependencies[..missing_dependencies.len() - 1]
                    .iter()
                    .map(|upvalue| format!("'{}'", &upvalue.indexing_expression_name()))
                    .collect::<Vec<String>>()
                    .join(", ");
                format!(
                    "{}, and '{}'",
                    all_but_last,
                    missing_dependencies
                        .last()
                        .unwrap()
                        .indexing_expression_name()
                )
            }
        }
    )
}

fn is_lua_valid_identifier(string: &str) -> bool {
    // Valid identifier cannot start with numbers
    let first_char = string.chars().next().unwrap();
    if !first_char.is_alphabetic() && first_char != '_' {
        return false;
    }

    string.chars().all(|c| c.is_alphanumeric() || c == '_')
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
    unnecessary_dependencies: Vec<UnnecessaryDependency>,
    complex_dependencies: Vec<ComplexDependency>,
    fn_declaration_starts_stack: Vec<usize>,

    // Some variables are safe to omit from the dependency array, such as setState
    non_reactive_upvalue_starts: HashSet<usize>,
}

#[derive(Clone, Debug, Eq)]
struct Upvalue {
    /// `a.b` yields `a`
    prefix: String,

    /// `a.b["c d"].e` yields ["a", "b", "c d", "e"]`
    ///
    /// `a.b.[c].d` yields `["a", "b"]`
    ///
    /// `a.b.c().d` yields `["a", "b", "c"]`
    // eslint requires passing in `a.b` for `a.b.c()` since js implicitly passes `a.b`
    // as `this` to `c`. Lua doesn't do this, so we can allow `a.b.c` as deps
    indexing_identifiers: Vec<String>,

    /// True if there's any dynamic indexing or function calls such as `a[b]` or `a.b()`
    /// FIXME: This false negatives for function calls without indexing such as `a()`
    is_complex_expression: bool,

    /// Knowing where referenced variable was initialized lets us narrow down whether it's a reactive variable
    resolved_start_range: Option<usize>,
}

impl Upvalue {
    fn new(reference: &Reference, resolved_start_range: &Option<usize>) -> Self {
        let default_indexing = Vec::new();
        let indexing = reference.indexing.as_ref().unwrap_or(&default_indexing);

        let indexing_identifiers = indexing
            .iter()
            .enumerate()
            .map_while(|(i, index_entry)| {
                if i > 0 && indexing[i - 1].is_function_call {
                    return None;
                }

                index_entry.static_name.as_ref().and_then(|static_name| {
                    match static_name.token().token_type() {
                        TokenType::Identifier { identifier } => Some(identifier.to_string()),
                        TokenType::StringLiteral { literal, .. } => Some(literal.to_string()),
                        _ => None,
                    }
                })
            })
            .collect::<Vec<_>>();

        let is_complex_expression = indexing
            .iter()
            .any(|index_entry| index_entry.static_name.is_none() || index_entry.is_function_call);

        Upvalue {
            prefix: reference.name.clone(),
            indexing_identifiers,
            is_complex_expression,
            resolved_start_range: *resolved_start_range,
        }
    }

    /// `a.b["c"]["d e"]` yields `a.b.c["d e"]`
    ///
    /// `a.b.c().d` yields `a.b.c`
    ///
    /// `a` just yields `a`
    fn indexing_expression_name(&self) -> String {
        self.indexing_prefixes()
            .last()
            .unwrap_or(&"".to_string())
            .to_string()
    }

    /// `a.b.c` yields `["a", "a.b", "a.b.c"]`
    fn indexing_prefixes(&self) -> Vec<String> {
        let mut prefixes = vec![self.prefix.clone()];
        let mut current_name = self.prefix.clone();
        for index_name in &self.indexing_identifiers {
            if is_lua_valid_identifier(index_name) {
                current_name.push_str(format!(".{}", index_name).as_str());
            } else {
                current_name.push_str(format!("[\"{}\"]", index_name).as_str());
            }
            prefixes.push(current_name.clone());
        }
        prefixes
    }
}

impl Hash for Upvalue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.indexing_expression_name().hash(state);
    }
}

impl PartialEq for Upvalue {
    fn eq(&self, other: &Self) -> bool {
        self.prefix == other.prefix
    }
}

#[derive(Debug)]
struct MissingDependency {
    missing_dependencies: Vec<Upvalue>,
    range: (usize, usize),
    hook_name: String,
}

#[derive(Debug)]
struct UnnecessaryDependency {
    unnecessary_dependencies: Vec<Upvalue>,
    range: (usize, usize),
    hook_name: String,
}

#[derive(Debug)]
struct ComplexDependency {
    range: (usize, usize),
    hook_name: String,
}

impl RoactMissingDependencyVisitor<'_> {
    fn get_upvalues_in_expression(&self, expression: &Expression) -> HashSet<Upvalue> {
        self.scope_manager
            .references
            .iter()
            .filter_map(|(_, reference)| {
                if reference.identifier.0 > range(expression).0
                    && reference.identifier.1 < range(expression).1
                    && reference.read
                {
                    let resolved_start_range = reference.resolved.and_then(|resolved| {
                        let variable = &self.scope_manager.variables[resolved];

                        // FIXME: We need the start range where the variable was last set. Otherwise
                        // a variable can be first set outside but set again inside a component, and it
                        // identifies as non-reactive. However, this seems to only capture when user
                        // does `local` again. This is low priority as this only matters if user does something
                        // weird like writing to an outside variable within a component, which breaks a different rule
                        variable.identifiers.last().map(|(start, _)| *start)
                    });

                    Some(Upvalue::new(reference, &resolved_start_range))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Useful for determining whether a byte is outside the component when called from the context of a hook
    ///
    /// Uses current position of AST traversal. Calling this at different points will yield different results
    /// ```lua
    ///  local var1
    ///  local function component()
    ///      local var2
    ///      -- Called with var1 - TRUE
    ///      -- Called with var2 - FALSE
    ///      useEffect(function()
    ///         -- Called with var1 - TRUE
    ///         -- Called with var2 - FALSE
    ///      end)
    ///  end
    /// ```
    fn is_byte_outside_enclosing_named_fn(&self, byte: usize) -> bool {
        self.fn_declaration_starts_stack
            .last()
            .map_or(false, |&last_fn_start| byte < last_fn_start)
    }
}

impl Visitor for RoactMissingDependencyVisitor<'_> {
    fn visit_function_call(&mut self, call: &ast::FunctionCall) {
        let last_suffix =
            get_last_function_call_suffix(call.prefix(), &call.suffixes().collect::<Vec<_>>());

        let function_args = match call.suffixes().last() {
            Some(ast::Suffix::Call(ast::Call::AnonymousCall(args))) => args,
            _ => return,
        };

        if !["useEffect", "useMemo", "useCallback", "useLayoutEffect"]
            .contains(&last_suffix.as_str())
            || !is_roact_function(call)
        {
            return;
        }

        let arguments = match function_args {
            ast::FunctionArgs::Parentheses { arguments, .. } => arguments,
            _ => return,
        };

        let dependency_array_expr = match arguments.iter().nth(1) {
            Some(expr) => expr,
            _ => return,
        };

        let referenced_upvalues = match arguments.first() {
            Some(ast::punctuated::Pair::Punctuated(effect_callback, ..)) => {
                self.get_upvalues_in_expression(effect_callback)
            }
            _ => return,
        };

        let dependencies = self
            .get_upvalues_in_expression(dependency_array_expr)
            .into_iter()
            .map(|upvalue| (upvalue.indexing_expression_name(), upvalue))
            .collect::<HashMap<_, _>>();

        if dependencies
            .iter()
            .any(|(_, dependency)| dependency.is_complex_expression)
        {
            self.complex_dependencies.push(ComplexDependency {
                range: range(dependency_array_expr),
                hook_name: last_suffix.to_string(),
            });
            return;
        }

        let mut missing_dependencies = referenced_upvalues
            .iter()
            .filter(|upvalue| {
                // A reference of `a.b.c` can have a dep of either `a`, `a.b`, or `a.b.c` to satisfy
                for indexing_prefix in upvalue.indexing_prefixes() {
                    if dependencies.contains_key(&indexing_prefix) {
                        return false;
                    }
                }

                upvalue
                    .resolved_start_range
                    // Treat unresolved variables as globals, which are not reactive
                    .map_or(false, |resolved_start| {
                        // Ignore variables declared inside the hook callback
                        if upvalue.resolved_start_range >= range(call).0 {
                            return false;
                        }

                        if self.is_byte_outside_enclosing_named_fn(resolved_start) {
                            return false;
                        }

                        !self.non_reactive_upvalue_starts.contains(&resolved_start)
                    })
            })
            .cloned()
            .collect::<Vec<_>>();

        if !missing_dependencies.is_empty() {
            // Without sorting, error message will be non-deterministic
            missing_dependencies.sort_by_key(|upvalue| upvalue.indexing_expression_name());

            let mut reported_indexing_prefixes = HashSet::new();

            // If `a` is already reported missing, no need to report `a.b` as well
            // This algorithm only works because this is sorted, so `a` will always come before `a.<anything>`
            missing_dependencies.retain(|upvalue| {
                let already_reported = upvalue
                    .indexing_prefixes()
                    .iter()
                    .any(|indexing_prefix| reported_indexing_prefixes.contains(indexing_prefix));

                if !already_reported {
                    reported_indexing_prefixes.insert(upvalue.indexing_expression_name());
                }

                !already_reported
            });

            self.missing_dependencies.push(MissingDependency {
                missing_dependencies,
                range: range(dependency_array_expr),
                hook_name: last_suffix.to_string(),
            });
        }

        // Non-reactive variables should not be put in the dependency array
        let mut unnecessary_dependencies = dependencies
            .iter()
            .filter_map(|(_, dependency)| {
                if let Some(resolved_start) = dependency.resolved_start_range {
                    if self.is_byte_outside_enclosing_named_fn(resolved_start) {
                        Some(dependency.clone())
                    } else {
                        None
                    }
                } else {
                    // Assume unresolved variables are globals and should not be included in deps
                    Some(dependency.clone())
                }
            })
            .collect::<Vec<Upvalue>>();

        if !unnecessary_dependencies.is_empty() {
            // Without sorting, error message will be non-deterministic
            unnecessary_dependencies.sort_by_key(|upvalue| upvalue.prefix.to_string());

            self.unnecessary_dependencies.push(UnnecessaryDependency {
                unnecessary_dependencies,
                range: range(dependency_array_expr),
                hook_name: last_suffix.to_string(),
            });
        }
    }

    fn visit_local_assignment(&mut self, assignment: &ast::LocalAssignment) {
        if_chain! {
            if let Some(ast::punctuated::Pair::End(expression)) = assignment.expressions().first();
            if let ast::Expression::FunctionCall(call) = expression;
            if is_roact_function(call);
            then {
                let function_suffix = get_last_function_call_suffix(
                    call.prefix(),
                    &call.suffixes().collect::<Vec<_>>(),
                );

                if ["useRef", "useBinding"].contains(&function_suffix.as_str()) {
                    if let Some(first_var) = assignment.names().first() {
                        self.non_reactive_upvalue_starts
                            .insert(range(first_var).0);
                    }
                }

                if ["useState", "useBinding", "useDispatch", "useTransition"].contains(&function_suffix.as_str()) {
                    if let Some(second_var) = assignment.names().iter().nth(1) {
                        self.non_reactive_upvalue_starts
                            .insert(range(second_var).0);
                    }
                }
            }
        }
    }

    fn visit_function_declaration(&mut self, function_declaration: &ast::FunctionDeclaration) {
        self.fn_declaration_starts_stack
            .push(range(function_declaration).0);
    }

    fn visit_function_declaration_end(&mut self, _: &ast::FunctionDeclaration) {
        self.fn_declaration_starts_stack.pop();
    }

    fn visit_local_function(&mut self, local_function: &ast::LocalFunction) {
        self.fn_declaration_starts_stack
            .push(range(local_function).0);
    }

    fn visit_local_function_end(&mut self, _: &ast::LocalFunction) {
        self.fn_declaration_starts_stack.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    fn test_complex_deps() {
        test_lint(
            RoactNonExhaustiveDepsLint::new(()).unwrap(),
            "roblox_roact_non_exhaustive_deps",
            "complex_deps",
        );
    }

    #[test]
    fn test_known_stable_vars() {
        test_lint(
            RoactNonExhaustiveDepsLint::new(()).unwrap(),
            "roblox_roact_non_exhaustive_deps",
            "known_stable_vars",
        );
    }

    #[test]
    fn test_no_deps() {
        test_lint(
            RoactNonExhaustiveDepsLint::new(()).unwrap(),
            "roblox_roact_non_exhaustive_deps",
            "no_deps",
        );
    }

    #[test]
    fn test_no_roact() {
        test_lint(
            RoactNonExhaustiveDepsLint::new(()).unwrap(),
            "roblox_roact_non_exhaustive_deps",
            "no_roact",
        );
    }

    #[test]
    fn test_roblox_roact_non_exhaustive_deps() {
        test_lint(
            RoactNonExhaustiveDepsLint::new(()).unwrap(),
            "roblox_roact_non_exhaustive_deps",
            "roblox_roact_non_exhaustive_deps",
        );
    }

    #[test]
    fn test_too_complex_deps() {
        test_lint(
            RoactNonExhaustiveDepsLint::new(()).unwrap(),
            "roblox_roact_non_exhaustive_deps",
            "too_complex_deps",
        );
    }

    #[test]
    fn test_use_memo() {
        test_lint(
            RoactNonExhaustiveDepsLint::new(()).unwrap(),
            "roblox_roact_non_exhaustive_deps",
            "use_memo",
        );
    }
}
