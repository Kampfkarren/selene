use full_moon::{
    ast::{self, punctuated::Punctuated, Expression},
    visitors::Visitor,
};

use crate::ast_util::{
    expression_to_ident, range, scopes::AssignedValue, strip_parentheses, LoopTracker,
};

use super::*;
use std::convert::Infallible;

pub struct ManualTableCloneLint;

impl Lint for ManualTableCloneLint {
    type Config = ();
    type Error = Infallible;

    const SEVERITY: Severity = Severity::Warning;
    const LINT_TYPE: LintType = LintType::Complexity;

    fn new(_: Self::Config) -> Result<Self, Self::Error> {
        Ok(ManualTableCloneLint)
    }

    fn pass(&self, ast: &Ast, context: &Context, ast_context: &AstContext) -> Vec<Diagnostic> {
        if context
            .standard_library
            .find_global(&["table", "clone"])
            .is_none()
        {
            return Vec::new();
        }

        let mut visitor = ManualTableCloneVisitor {
            matches: Vec::new(),
            loop_tracker: LoopTracker::new(ast),
            scope_manager: &ast_context.scope_manager,
            stmt_begins: Vec::new(),
        };

        visitor.visit_ast(ast);

        visitor
            .matches
            .into_iter()
            .map(ManualTableCloneMatch::into_diagnostic)
            .collect()
    }
}

#[derive(Debug)]
struct ManualTableCloneMatch {
    range: (usize, usize),
    assigning_into: String,
    looping_over: String,
    loop_type: LoopType,
    replaces_definition_range: Option<(usize, usize)>,
}

impl ManualTableCloneMatch {
    fn into_diagnostic(self) -> Diagnostic {
        Diagnostic::new_complete(
            "manual_table_clone",
            "manual implementation of table.clone".to_owned(),
            Label::new(self.range),
            {
                let mut notes = vec![format!(
                    "try `local {} = table.clone({})`",
                    self.assigning_into.trim(),
                    self.looping_over.trim()
                )];

                if matches!(self.loop_type, LoopType::Ipairs) {
                    notes.push("if this is a mixed table, then table.clone is not equivalent, as ipairs only goes over the array portion.\n\
                        ignore this lint with `-- selene: allow(manual_table_clone)` if this is the case.".to_owned());
                }

                notes
            },
            if let Some(definition_range) = self.replaces_definition_range {
                vec![Label::new_with_message(
                    definition_range,
                    "remove this definition".to_owned(),
                )]
            } else {
                Vec::new()
            },
        )
    }
}

struct ManualTableCloneVisitor<'ast> {
    matches: Vec<ManualTableCloneMatch>,
    loop_tracker: LoopTracker,
    scope_manager: &'ast ScopeManager,
    stmt_begins: Vec<usize>,
}

#[derive(Debug)]
enum LoopType {
    Ipairs,
    Other,
}

impl ManualTableCloneVisitor<'_> {
    // Not technically true, but we can worry about that later.
    // Can't just check standard library because name "roblox" will override "luau".
    fn is_luau(&self) -> bool {
        true
    }

    fn loop_expression<'ast>(
        &self,
        expressions: &'ast Punctuated<Expression>,
    ) -> Option<(LoopType, &'ast Expression)> {
        match expressions.len() {
            1 => {
                let loop_expression = expressions.iter().next().unwrap();

                let function_call = match strip_parentheses(loop_expression) {
                    Expression::Value { value, .. } => match &**value {
                        ast::Value::FunctionCall(function_call) => function_call,
                        _ if self.is_luau() => return Some((LoopType::Other, loop_expression)),
                        _ => return None,
                    },

                    _ if self.is_luau() => return Some((LoopType::Other, loop_expression)),

                    _ => return None,
                };

                #[cfg_attr(
                    feature = "force_exhaustive_checks",
                    deny(non_exhaustive_omitted_patterns)
                )]
                let function_token = match function_call.prefix() {
                    ast::Prefix::Name(name) => name,
                    ast::Prefix::Expression(expression) => match expression_to_ident(expression) {
                        Some(name) => name,
                        None => return None,
                    },
                    _ => return None,
                };

                let function_name = function_token.token().to_string();
                let looping_expression;

                if function_name == "ipairs" || function_name == "pairs" {
                    let suffixes = function_call.suffixes().collect::<Vec<_>>();
                    if suffixes.len() != 1 {
                        return None;
                    }

                    let suffix = suffixes[0];

                    let inner_expression = match suffix {
                        ast::Suffix::Call(ast::Call::AnonymousCall(
                            ast::FunctionArgs::Parentheses { arguments, .. },
                        )) => {
                            if arguments.len() != 1 {
                                return None;
                            }

                            arguments.iter().next().unwrap()
                        }

                        _ => return None,
                    };

                    looping_expression = inner_expression;
                } else if self.is_luau() {
                    // `for i, v in what(x) do` is valid in Luau, but shouldn't be treated as a pairs equivalent.
                    looping_expression = loop_expression;
                } else {
                    return None;
                }

                Some((
                    if function_name == "ipairs" {
                        LoopType::Ipairs
                    } else {
                        LoopType::Other
                    },
                    looping_expression,
                ))
            }

            2 => {
                let mut expressions = expressions.iter();
                let (first, second) = (expressions.next().unwrap(), expressions.next().unwrap());

                match expression_to_ident(first) {
                    Some(ident) => {
                        if ident.token().to_string() != "next" {
                            return None;
                        }
                    }

                    _ => return None,
                }

                Some((LoopType::Other, second))
            }

            _ => None,
        }
    }

    fn statement_in_way_of_definition(
        &self,
        definition_end: usize,
        assigment_start: usize,
    ) -> bool {
        debug_assert!(assigment_start > definition_end);

        for &stmt_begin in self.stmt_begins.iter() {
            if stmt_begin > definition_end {
                return true;
            } else if stmt_begin > assigment_start {
                return false;
            }
        }

        false
    }
}

fn has_filter_comment(for_loop: &ast::GenericFor) -> bool {
    let (leading_trivia, ..) = for_loop.surrounding_trivia();

    for trivia in leading_trivia {
        let comment = match trivia.token_type() {
            full_moon::tokenizer::TokenType::SingleLineComment { comment } => comment,
            full_moon::tokenizer::TokenType::MultiLineComment { comment, .. } => comment,
            _ => continue,
        };

        let filters = match crate::lint_filtering::parse_comment(comment.trim()) {
            Some(filters) => filters,
            None => continue,
        };

        if filters
            .into_iter()
            .any(|filter| filter.lint == "manual_table_clone")
        {
            return true;
        }
    }

    false
}

impl Visitor for ManualTableCloneVisitor<'_> {
    fn visit_generic_for(&mut self, node: &ast::GenericFor) {
        let (loop_type, looping_over) = match self.loop_expression(node.expressions()) {
            Some(loop_expression) => loop_expression,
            None => return,
        };

        let names = node.names().iter().collect::<Vec<_>>();
        if names.len() != 2 {
            return;
        }

        let (key, value) = (names[0].token().to_string(), names[1].token().to_string());

        let statements = node.block().stmts().collect::<Vec<_>>();

        if statements.len() != 1 {
            return;
        }

        let assignment = match statements[0] {
            ast::Stmt::Assignment(assignment) => assignment,
            _ => return,
        };

        let variables = assignment.variables();
        if variables.len() != 1 {
            return;
        }

        let variable = variables.iter().next().unwrap();

        let assigning_into = match variable {
            ast::Var::Expression(var_expression) => {
                let name = match var_expression.prefix() {
                    ast::Prefix::Expression(expression) => match expression_to_ident(expression) {
                        Some(name) => name,
                        None => return,
                    },
                    ast::Prefix::Name(name) => name,
                    _ => return,
                };

                let suffixes = var_expression.suffixes().collect::<Vec<_>>();
                if suffixes.len() != 1 {
                    return;
                }

                let index = match &suffixes[0] {
                    ast::Suffix::Index(ast::Index::Brackets { expression, .. }) => expression,
                    _ => return,
                };

                if expression_to_ident(index).map(|ident| ident.token().to_string()) != Some(key) {
                    return;
                }

                match assignment
                    .expressions()
                    .iter()
                    .next()
                    .and_then(expression_to_ident)
                {
                    Some(name) => {
                        if name.token().to_string() != value {
                            return;
                        }
                    }

                    _ => return,
                };

                name
            }

            _ => return,
        };

        let (definition_start, definition_end) = match self
            .scope_manager
            .reference_at_byte(assigning_into.token().start_position().bytes())
        {
            Some(reference) => {
                if reference.resolved.is_none() {
                    return;
                }

                let variable = &self.scope_manager.variables[reference.resolved.unwrap()];

                if !matches!(
                    variable.value,
                    Some(AssignedValue::StaticTable { has_fields: false })
                ) {
                    return;
                }

                let first_definition_range = match variable.definitions.first() {
                    Some(first_definition_range) => first_definition_range,
                    None => {
                        return;
                    }
                };

                // Make sure we haven't potentially tainted this variable before
                for reference_id in &variable.references {
                    let reference = &self.scope_manager.references[*reference_id];

                    if reference.identifier.1 > first_definition_range.1
                        && reference.identifier.0 < assigning_into.token().start_position().bytes()
                    {
                        return;
                    }
                }

                first_definition_range
            }

            _ => return,
        };

        let (position_start, position_end) = range(node);

        if self.loop_tracker.depth_at_byte(position_start)
            != self.loop_tracker.depth_at_byte(*definition_start)
        {
            return;
        }

        let only_use_loop_range = self
            .statement_in_way_of_definition(*definition_end, position_start)
            || has_filter_comment(node);

        self.matches.push(ManualTableCloneMatch {
            range: if only_use_loop_range {
                (position_start, position_end)
            } else {
                (*definition_start, position_end)
            },
            assigning_into: assigning_into.token().to_string(),
            looping_over: looping_over.to_string(),
            replaces_definition_range: if only_use_loop_range {
                Some((*definition_start, *definition_end))
            } else {
                None
            },
            loop_type,
        });
    }

    fn visit_stmt_end(&mut self, stmt: &ast::Stmt) {
        self.stmt_begins.push(range(stmt).0);
    }
}

#[cfg(test)]
mod tests {
    use crate::lints::test_util::{test_lint_config_with_output, TestUtilConfig};

    use super::{super::test_util::test_lint_config, *};

    #[test]
    fn test_manual_table_clone() {
        test_lint_config(
            ManualTableCloneLint::new(()).unwrap(),
            "manual_table_clone",
            "manual_table_clone",
            TestUtilConfig {
                standard_library: StandardLibrary::from_name("luau").unwrap(),
                ..Default::default()
            },
        );
    }

    #[test]
    fn test_no_table_clone() {
        test_lint_config_with_output(
            ManualTableCloneLint::new(()).unwrap(),
            "manual_table_clone",
            "manual_table_clone",
            TestUtilConfig::default(),
            "no_table_clone.stderr",
        );
    }

    #[test]
    fn test_false_positive() {
        test_lint_config(
            ManualTableCloneLint::new(()).unwrap(),
            "manual_table_clone",
            "false_positive",
            TestUtilConfig {
                standard_library: StandardLibrary::from_name("luau").unwrap(),
                ..Default::default()
            },
        )
    }
}
