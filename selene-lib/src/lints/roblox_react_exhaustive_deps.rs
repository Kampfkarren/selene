use super::*;
use crate::ast_util::{range, strip_parentheses};
use std::{
    collections::{HashMap, HashSet},
    convert::Infallible,
};

use full_moon::{
    ast::{self, Ast},
    visitors::Visitor,
};
use if_chain::if_chain;

pub struct ReactExhaustiveDepsLint;

impl Lint for ReactExhaustiveDepsLint {
    type Config = ();
    type Error = Infallible;

    const SEVERITY: Severity = Severity::Warning;
    const LINT_TYPE: LintType = LintType::Correctness;

    fn new(_: Self::Config) -> Result<Self, Self::Error> {
        Ok(ReactExhaustiveDepsLint)
    }

    fn pass(&self, ast: &Ast, context: &Context, ast_context: &AstContext) -> Vec<Diagnostic> {
        if !context.is_roblox() {
            return Vec::new();
        }

        let mut visitor = ReactExhaustiveDepsVisitor {
            scope_manager: &ast_context.scope_manager,
            definitions_of_hooks: HashMap::new(),
            issues: Vec::new(),
            current_scope_id: None,
        };

        visitor.visit_ast(ast);

        visitor.issues
    }
}

struct ReactExhaustiveDepsVisitor<'a> {
    scope_manager: &'a crate::ast_util::scopes::ScopeManager,
    definitions_of_hooks: HashMap<String, HookType>,
    issues: Vec<Diagnostic>,
    current_scope_id: Option<id_arena::Id<crate::ast_util::scopes::Scope>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HookType {
    UseEffect,
    UseCallback,
    UseMemo,
    UseLayoutEffect,
    UseImperativeHandle,
}

impl HookType {
    fn name(&self) -> &'static str {
        match self {
            HookType::UseEffect => "useEffect",
            HookType::UseCallback => "useCallback",
            HookType::UseMemo => "useMemo",
            HookType::UseLayoutEffect => "useLayoutEffect",
            HookType::UseImperativeHandle => "useImperativeHandle",
        }
    }

    fn deps_position(&self) -> usize {
        match self {
            HookType::UseEffect
            | HookType::UseCallback
            | HookType::UseMemo
            | HookType::UseLayoutEffect => 1,
            HookType::UseImperativeHandle => 2,
        }
    }

    fn callback_position(&self) -> usize {
        match self {
            HookType::UseEffect
            | HookType::UseCallback
            | HookType::UseMemo
            | HookType::UseLayoutEffect => 0,
            HookType::UseImperativeHandle => 1,
        }
    }
}

fn is_react_hook_call(prefix: &ast::Prefix, suffixes: &[&ast::Suffix]) -> Option<HookType> {
    if_chain! {
        if let ast::Prefix::Name(prefix_token) = prefix;
        if ["React", "Roact"].contains(&&*prefix_token.token().to_string());
        if suffixes.len() == 1;
        if let ast::Suffix::Index(ast::Index::Dot { name, .. }) = suffixes[0];
        then {
            match &*name.token().to_string() {
                "useEffect" => Some(HookType::UseEffect),
                "useCallback" => Some(HookType::UseCallback),
                "useMemo" => Some(HookType::UseMemo),
                "useLayoutEffect" => Some(HookType::UseLayoutEffect),
                "useImperativeHandle" => Some(HookType::UseImperativeHandle),
                _ => None,
            }
        } else {
            None
        }
    }
}

fn extract_dependencies_from_array(
    table: &ast::TableConstructor,
) -> Option<Vec<(String, (usize, usize))>> {
    let mut deps = Vec::new();

    for field in table.fields() {
        match field {
            ast::Field::NoKey(expr) => {
                if let Some(dep_name) = extract_dependency_name(expr) {
                    deps.push((dep_name, range(expr)));
                } else {
                    // Complex expression in deps array - we can't analyze it
                    return None;
                }
            }
            _ => {
                // Key-value pairs in deps array are invalid
                return None;
            }
        }
    }

    Some(deps)
}

fn extract_dependency_name(expr: &ast::Expression) -> Option<String> {
    let expr = strip_parentheses(expr);

    match expr {
        ast::Expression::Var(var) => {
            if let ast::Var::Name(name) = var {
                Some(name.token().to_string())
            } else if let ast::Var::Expression(var_expr) = var {
                // Handle property access like props.value or obj.field
                extract_property_chain(var_expr)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn extract_property_chain(var_expr: &ast::VarExpression) -> Option<String> {
    let mut parts = Vec::new();

    // Get the base name
    if let ast::Prefix::Name(name) = var_expr.prefix() {
        parts.push(name.token().to_string());
    } else {
        return None;
    }

    // Get all property accesses
    for suffix in var_expr.suffixes() {
        if let ast::Suffix::Index(ast::Index::Dot { name, .. }) = suffix {
            parts.push(name.token().to_string());
        } else {
            // Complex access like brackets, we can't analyze simply
            return None;
        }
    }

    Some(parts.join("."))
}

fn collect_referenced_variables(
    expr: &ast::Expression,
    scope_id: id_arena::Id<crate::ast_util::scopes::Scope>,
    scope_manager: &crate::ast_util::scopes::ScopeManager,
) -> HashSet<String> {
    let mut collector = VariableCollector {
        variables: HashSet::new(),
        scope_id,
        scope_manager,
    };

    collector.visit_expression(expr);
    collector.variables
}

struct VariableCollector<'a> {
    variables: HashSet<String>,
    #[allow(dead_code)]
    scope_id: id_arena::Id<crate::ast_util::scopes::Scope>,
    #[allow(dead_code)]
    scope_manager: &'a crate::ast_util::scopes::ScopeManager,
}

impl<'a> VariableCollector<'a> {
    fn visit_expression(&mut self, expr: &ast::Expression) {
        let expr = strip_parentheses(expr);

        match expr {
            ast::Expression::Var(var) => {
                self.visit_var(var);
            }
            ast::Expression::BinaryOperator { lhs, rhs, .. } => {
                self.visit_expression(lhs);
                self.visit_expression(rhs);
            }
            ast::Expression::UnaryOperator { expression, .. } => {
                self.visit_expression(expression);
            }
            ast::Expression::Function(_) => {
                // Don't traverse into nested functions - they have their own scope
            }
            ast::Expression::FunctionCall(call) => {
                self.visit_function_call(call);
            }
            ast::Expression::TableConstructor(table) => {
                self.visit_table_constructor(table);
            }
            // Note: We skip Luau-specific expressions like IfExpression and InterpolatedString
            // as they require more complex handling
            _ => {}
        }
    }

    fn visit_var(&mut self, var: &ast::Var) {
        match var {
            ast::Var::Name(name) => {
                let var_name = name.token().to_string();

                // Check if this variable is defined in the current scope or parent scopes
                if self.is_external_variable(&var_name) {
                    self.variables.insert(var_name);
                }
            }
            ast::Var::Expression(var_expr) => {
                // Handle property access like props.value
                if let Some(chain) = extract_property_chain(var_expr) {
                    // Get the base variable name
                    if let Some(base) = chain.split('.').next() {
                        if self.is_external_variable(base) {
                            self.variables.insert(chain);
                        }
                    }
                } else {
                    // Fallback to visiting the prefix and suffixes
                    self.visit_prefix(var_expr.prefix());
                    for suffix in var_expr.suffixes() {
                        self.visit_suffix(suffix);
                    }
                }
            }
            _ => {}
        }
    }

    fn visit_prefix(&mut self, prefix: &ast::Prefix) {
        match prefix {
            ast::Prefix::Name(name) => {
                let var_name = name.token().to_string();
                if self.is_external_variable(&var_name) {
                    self.variables.insert(var_name);
                }
            }
            ast::Prefix::Expression(expr) => {
                self.visit_expression(expr);
            }
            _ => {}
        }
    }

    fn visit_suffix(&mut self, suffix: &ast::Suffix) {
        match suffix {
            ast::Suffix::Index(index) => match index {
                ast::Index::Brackets { expression, .. } => {
                    self.visit_expression(expression);
                }
                ast::Index::Dot { .. } => {
                    // Property access, handled in extract_property_chain
                }
                _ => {}
            },
            ast::Suffix::Call(call) => match call {
                ast::Call::AnonymousCall(args) => match args {
                    ast::FunctionArgs::Parentheses { arguments, .. } => {
                        for arg in arguments {
                            self.visit_expression(arg);
                        }
                    }
                    ast::FunctionArgs::String(_) => {}
                    ast::FunctionArgs::TableConstructor(table) => {
                        self.visit_table_constructor(table);
                    }
                    _ => {}
                },
                ast::Call::MethodCall(method_call) => {
                    if let ast::FunctionArgs::Parentheses { arguments, .. } = method_call.args() {
                        for arg in arguments {
                            self.visit_expression(arg);
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn visit_function_call(&mut self, call: &ast::FunctionCall) {
        self.visit_prefix(call.prefix());
        for suffix in call.suffixes() {
            self.visit_suffix(suffix);
        }
    }

    fn visit_table_constructor(&mut self, table: &ast::TableConstructor) {
        for field in table.fields() {
            match field {
                ast::Field::ExpressionKey { key, value, .. } => {
                    self.visit_expression(key);
                    self.visit_expression(value);
                }
                ast::Field::NameKey { value, .. } => {
                    self.visit_expression(value);
                }
                ast::Field::NoKey(expr) => {
                    self.visit_expression(expr);
                }
                _ => {}
            }
        }
    }

    fn is_external_variable(&self, name: &str) -> bool {
        // Skip built-in globals and common constants
        if ["true", "false", "nil", "self"].contains(&name) {
            return false;
        }

        // Check if this variable is defined in the current function scope
        // For now, we'll use a simple heuristic: if it starts with lowercase and
        // is a simple identifier, it's likely an external variable
        // TODO: Use scope_manager to properly check if variable is external
        true
    }
}

fn visit_block_for_variables(
    block: &ast::Block,
    scope_id: id_arena::Id<crate::ast_util::scopes::Scope>,
    scope_manager: &crate::ast_util::scopes::ScopeManager,
) -> HashSet<String> {
    let mut variables = HashSet::new();

    for stmt in block.stmts() {
        variables.extend(collect_variables_from_stmt(stmt, scope_id, scope_manager));
    }

    if let Some(last_stmt) = block.last_stmt() {
        match last_stmt {
            ast::LastStmt::Return(return_stmt) => {
                for expr in return_stmt.returns() {
                    variables.extend(collect_referenced_variables(expr, scope_id, scope_manager));
                }
            }
            _ => {}
        }
    }

    variables
}

fn collect_variables_from_stmt(
    stmt: &ast::Stmt,
    scope_id: id_arena::Id<crate::ast_util::scopes::Scope>,
    scope_manager: &crate::ast_util::scopes::ScopeManager,
) -> HashSet<String> {
    let mut variables = HashSet::new();

    match stmt {
        ast::Stmt::Assignment(assignment) => {
            for expr in assignment.expressions() {
                variables.extend(collect_referenced_variables(expr, scope_id, scope_manager));
            }
            // Don't traverse into the variables being assigned to
        }
        ast::Stmt::LocalAssignment(assignment) => {
            for expr in assignment.expressions() {
                variables.extend(collect_referenced_variables(expr, scope_id, scope_manager));
            }
        }
        ast::Stmt::FunctionCall(call) => {
            variables.extend(collect_variables_from_function_call(
                call,
                scope_id,
                scope_manager,
            ));
        }
        ast::Stmt::If(if_stmt) => {
            variables.extend(collect_referenced_variables(
                if_stmt.condition(),
                scope_id,
                scope_manager,
            ));
            variables.extend(visit_block_for_variables(
                if_stmt.block(),
                scope_id,
                scope_manager,
            ));

            if let Some(else_ifs) = if_stmt.else_if() {
                for else_if in else_ifs {
                    variables.extend(collect_referenced_variables(
                        else_if.condition(),
                        scope_id,
                        scope_manager,
                    ));
                    variables.extend(visit_block_for_variables(
                        else_if.block(),
                        scope_id,
                        scope_manager,
                    ));
                }
            }

            if let Some(else_block) = if_stmt.else_block() {
                variables.extend(visit_block_for_variables(
                    else_block,
                    scope_id,
                    scope_manager,
                ));
            }
        }
        ast::Stmt::While(while_stmt) => {
            variables.extend(collect_referenced_variables(
                while_stmt.condition(),
                scope_id,
                scope_manager,
            ));
            variables.extend(visit_block_for_variables(
                while_stmt.block(),
                scope_id,
                scope_manager,
            ));
        }
        ast::Stmt::Repeat(repeat_stmt) => {
            variables.extend(visit_block_for_variables(
                repeat_stmt.block(),
                scope_id,
                scope_manager,
            ));
            variables.extend(collect_referenced_variables(
                repeat_stmt.until(),
                scope_id,
                scope_manager,
            ));
        }
        ast::Stmt::Do(do_stmt) => {
            variables.extend(visit_block_for_variables(
                do_stmt.block(),
                scope_id,
                scope_manager,
            ));
        }
        ast::Stmt::GenericFor(for_stmt) => {
            for expr in for_stmt.expressions() {
                variables.extend(collect_referenced_variables(expr, scope_id, scope_manager));
            }
            variables.extend(visit_block_for_variables(
                for_stmt.block(),
                scope_id,
                scope_manager,
            ));
        }
        ast::Stmt::NumericFor(for_stmt) => {
            variables.extend(collect_referenced_variables(
                for_stmt.start(),
                scope_id,
                scope_manager,
            ));
            variables.extend(collect_referenced_variables(
                for_stmt.end(),
                scope_id,
                scope_manager,
            ));
            if let Some(step) = for_stmt.step() {
                variables.extend(collect_referenced_variables(step, scope_id, scope_manager));
            }
            variables.extend(visit_block_for_variables(
                for_stmt.block(),
                scope_id,
                scope_manager,
            ));
        }
        _ => {}
    }

    variables
}

fn collect_variables_from_function_call(
    call: &ast::FunctionCall,
    scope_id: id_arena::Id<crate::ast_util::scopes::Scope>,
    scope_manager: &crate::ast_util::scopes::ScopeManager,
) -> HashSet<String> {
    let mut variables = HashSet::new();

    // Visit the prefix
    match call.prefix() {
        ast::Prefix::Name(name) => {
            let var_name = name.token().to_string();
            // Check if it's an external variable
            if !["true", "false", "nil", "self"].contains(&&*var_name) {
                variables.insert(var_name);
            }
        }
        ast::Prefix::Expression(expr) => {
            variables.extend(collect_referenced_variables(expr, scope_id, scope_manager));
        }
        _ => {}
    }

    // Visit suffixes
    for suffix in call.suffixes() {
        match suffix {
            ast::Suffix::Call(call) => match call {
                ast::Call::AnonymousCall(args) => match args {
                    ast::FunctionArgs::Parentheses { arguments, .. } => {
                        for arg in arguments {
                            variables.extend(collect_referenced_variables(
                                arg,
                                scope_id,
                                scope_manager,
                            ));
                        }
                    }
                    ast::FunctionArgs::TableConstructor(table) => {
                        for field in table.fields() {
                            match field {
                                ast::Field::ExpressionKey { key, value, .. } => {
                                    variables.extend(collect_referenced_variables(
                                        key,
                                        scope_id,
                                        scope_manager,
                                    ));
                                    variables.extend(collect_referenced_variables(
                                        value,
                                        scope_id,
                                        scope_manager,
                                    ));
                                }
                                ast::Field::NameKey { value, .. } => {
                                    variables.extend(collect_referenced_variables(
                                        value,
                                        scope_id,
                                        scope_manager,
                                    ));
                                }
                                ast::Field::NoKey(expr) => {
                                    variables.extend(collect_referenced_variables(
                                        expr,
                                        scope_id,
                                        scope_manager,
                                    ));
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                },
                ast::Call::MethodCall(method) => {
                    if let ast::FunctionArgs::Parentheses { arguments, .. } = method.args() {
                        for arg in arguments {
                            variables.extend(collect_referenced_variables(
                                arg,
                                scope_id,
                                scope_manager,
                            ));
                        }
                    }
                }
                _ => {}
            },
            ast::Suffix::Index(index) => match index {
                ast::Index::Brackets { expression, .. } => {
                    variables.extend(collect_referenced_variables(
                        expression,
                        scope_id,
                        scope_manager,
                    ));
                }
                _ => {}
            },
            _ => {}
        }
    }

    variables
}

impl<'a> Visitor for ReactExhaustiveDepsVisitor<'a> {
    fn visit_function_call(&mut self, call: &ast::FunctionCall) {
        // Check if this is a React hook call
        let mut suffixes = call.suffixes().collect::<Vec<_>>();
        let call_suffix = suffixes.pop();

        let mut hook_type = None;

        if suffixes.is_empty() {
            // Call is foo(), not foo.bar()
            // Check if foo is a variable for a React hook
            if let ast::Prefix::Name(name) = call.prefix() {
                let name_str = name.token().to_string();
                if let Some(stored_hook) = self.definitions_of_hooks.get(&name_str) {
                    hook_type = Some(*stored_hook);
                }
            }
        } else if suffixes.len() == 1 {
            // Call is foo.bar()
            // Check if foo.bar is a React hook
            if let Some(detected_hook) = is_react_hook_call(call.prefix(), &suffixes) {
                hook_type = Some(detected_hook);
            }
        }

        let hook_type = match hook_type {
            Some(ht) => ht,
            None => return,
        };

        // Extract arguments
        if_chain! {
            if let Some(ast::Suffix::Call(ast::Call::AnonymousCall(
                ast::FunctionArgs::Parentheses { arguments, .. }
            ))) = call_suffix;
            then {
                let args: Vec<_> = arguments.iter().collect();

                // Get callback and deps based on hook type
                let callback_pos = hook_type.callback_position();
                let deps_pos = hook_type.deps_position();

                if args.len() <= callback_pos {
                    return;
                }

                let callback = args[callback_pos];
                let deps_arg = if args.len() > deps_pos {
                    Some(args[deps_pos])
                } else {
                    None
                };

                // Extract the function body to analyze
                let callback_body = match strip_parentheses(callback) {
                    ast::Expression::Function(func) => func.1.block(),
                    _ => return, // Not a function literal
                };

                // Collect variables referenced in the callback
                let callback_range: (usize, usize) = range(callback);

                // We need a valid scope ID - if we don't have one, skip this analysis
                let scope_id = match self.current_scope_id {
                    Some(id) => id,
                    None => {
                        // Try to use the initial scope if available
                        match self.scope_manager.initial_scope {
                            Some(id) => id,
                            None => return,
                        }
                    }
                };

                let referenced_vars = visit_block_for_variables(
                    callback_body,
                    scope_id,
                    self.scope_manager,
                );

                // Extract declared dependencies
                let declared_deps = if let Some(deps_expr) = deps_arg {
                    match strip_parentheses(deps_expr) {
                        ast::Expression::TableConstructor(table) => {
                            extract_dependencies_from_array(table)
                        }
                        _ => None, // Not a table, can't analyze
                    }
                } else {
                    Some(Vec::new()) // No deps array means empty dependencies
                };

                if let Some(declared_deps) = declared_deps {
                    let declared_set: HashSet<String> = declared_deps
                        .iter()
                        .map(|(name, _)| name.clone())
                        .collect();

                    // Filter out variables that should be ignored
                    let filtered_referenced: HashSet<String> = referenced_vars
                        .into_iter()
                        .filter(|var| {
                            // Ignore built-in globals
                            !["print", "warn", "error", "assert", "type", "typeof",
                              "pairs", "ipairs", "next", "tonumber", "tostring",
                              "table", "string", "math", "coroutine", "debug",
                              "require", "game", "workspace", "script", "task",
                              "_G", "_VERSION"].contains(&&**var)
                        })
                        .collect();

                    // Find missing dependencies
                    let mut missing: Vec<String> = filtered_referenced
                        .difference(&declared_set)
                        .cloned()
                        .collect();
                    missing.sort();

                    // Find unnecessary dependencies
                    let mut unnecessary: Vec<String> = declared_set
                        .difference(&filtered_referenced)
                        .cloned()
                        .collect();
                    unnecessary.sort();

                    // Report if there are issues
                    if !missing.is_empty() || !unnecessary.is_empty() {
                        let deps_range = deps_arg.map(|expr| range(expr));

                        let mut message = format!("React Hook {} has", hook_type.name());
                        if !missing.is_empty() {
                            message.push_str(&format!(
                                " missing dependenc{}: {}",
                                if missing.len() == 1 { "y" } else { "ies" },
                                missing.join(", ")
                            ));
                        }
                        if !missing.is_empty() && !unnecessary.is_empty() {
                            message.push_str(", and");
                        }
                        if !unnecessary.is_empty() {
                            message.push_str(&format!(
                                " unnecessary dependenc{}: {}",
                                if unnecessary.len() == 1 { "y" } else { "ies" },
                                unnecessary.join(", ")
                            ));
                        }

                        let label_range = deps_range.unwrap_or(callback_range);

                        let mut notes = Vec::new();
                        if !missing.is_empty() {
                            notes.push(format!(
                                "Either include {} in the dependency array or remove the dependency",
                                if missing.len() == 1 {
                                    format!("'{}'", missing[0])
                                } else {
                                    format!("them ({})", missing.join(", "))
                                }
                            ));
                        }
                        if !unnecessary.is_empty() {
                            notes.push(format!(
                                "Remove {} from the dependency array",
                                if unnecessary.len() == 1 {
                                    format!("'{}'", unnecessary[0])
                                } else {
                                    format!("them ({})", unnecessary.join(", "))
                                }
                            ));
                        }

                        self.issues.push(Diagnostic::new_complete(
                            "roblox_react_exhaustive_deps",
                            message,
                            Label::new(label_range),
                            notes,
                            Vec::new(),
                        ));
                    }
                }
            }
        }
    }

    fn visit_local_assignment(&mut self, node: &ast::LocalAssignment) {
        for (name, expr) in node.names().iter().zip(node.expressions().iter()) {
            if_chain! {
                if let ast::Expression::Var(ast::Var::Expression(var_expr)) = expr;
                if let Some(hook) = is_react_hook_call(var_expr.prefix(), &var_expr.suffixes().collect::<Vec<_>>());
                then {
                    self.definitions_of_hooks.insert(name.token().to_string(), hook);
                }
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    fn test_missing_dependencies() {
        test_lint(
            ReactExhaustiveDepsLint::new(()).unwrap(),
            "roblox_react_exhaustive_deps",
            "missing_dependencies",
        );
    }

    #[test]
    fn test_unnecessary_dependencies() {
        test_lint(
            ReactExhaustiveDepsLint::new(()).unwrap(),
            "roblox_react_exhaustive_deps",
            "unnecessary_dependencies",
        );
    }

    #[test]
    fn test_correct_dependencies() {
        test_lint(
            ReactExhaustiveDepsLint::new(()).unwrap(),
            "roblox_react_exhaustive_deps",
            "correct_dependencies",
        );
    }

    #[test]
    fn test_property_access() {
        test_lint(
            ReactExhaustiveDepsLint::new(()).unwrap(),
            "roblox_react_exhaustive_deps",
            "property_access",
        );
    }

    #[test]
    fn test_use_callback() {
        test_lint(
            ReactExhaustiveDepsLint::new(()).unwrap(),
            "roblox_react_exhaustive_deps",
            "use_callback",
        );
    }

    #[test]
    fn test_use_memo() {
        test_lint(
            ReactExhaustiveDepsLint::new(()).unwrap(),
            "roblox_react_exhaustive_deps",
            "use_memo",
        );
    }
}
