use crate::{
    ast_util::scopes::AssignedValue,
    standard_library::{Field, FieldKind, Observes},
};

use super::*;

use full_moon::ast::Ast;
use regex::Regex;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
#[serde(default)]
pub struct UnusedVariableConfig {
    allow_unused_self: bool,
    ignore_pattern: String,
}

impl Default for UnusedVariableConfig {
    fn default() -> Self {
        Self {
            allow_unused_self: true,
            ignore_pattern: "^_".to_owned(),
        }
    }
}

pub struct UnusedVariableLint {
    allow_unused_self: bool,
    ignore_pattern: Regex,
}

#[derive(Debug, PartialEq, Eq)]
pub enum AnalyzedReference {
    Read,
    PlainWrite,
    ObservedWrite(Label),
}

impl Lint for UnusedVariableLint {
    type Config = UnusedVariableConfig;
    type Error = regex::Error;

    const SEVERITY: Severity = Severity::Warning;
    const LINT_TYPE: LintType = LintType::Style;

    fn new(config: Self::Config) -> Result<Self, Self::Error> {
        Ok(Self {
            allow_unused_self: config.allow_unused_self,
            ignore_pattern: Regex::new(&config.ignore_pattern)?,
        })
    }

    fn pass(&self, _: &Ast, context: &Context, ast_context: &AstContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (_, variable) in ast_context
            .scope_manager
            .variables
            .iter()
            .filter(|(_, variable)| !self.ignore_pattern.is_match(&variable.name))
        {
            if context.standard_library.global_has_fields(&variable.name) {
                continue;
            }

            let references = variable
                .references
                .iter()
                .copied()
                .map(|id| &ast_context.scope_manager.references[id]);

            // We need to make sure that references that are marked as "read" aren't only being read in an "observes: write" context.
            let analyzed_references = references
                .map(|reference| {
                    let is_static_table =
                        matches!(variable.value, Some(AssignedValue::StaticTable { .. }));

                    if reference.write.is_some() {
                        if let Some(indexing) = &reference.indexing {
                            if is_static_table
                                && indexing.len() == 1 // This restriction can be lifted someday, but only once we can verify that the value has no side effects/is its own static table
                                && indexing.iter().any(|index| index.static_name.is_some())
                            {
                                return AnalyzedReference::ObservedWrite(Label::new_with_message(
                                    reference.identifier,
                                    format!("`{}` is only getting written to", variable.name),
                                ));
                            }
                        }

                        if !reference.read {
                            return AnalyzedReference::PlainWrite;
                        }
                    }

                    if !is_static_table {
                        return AnalyzedReference::Read;
                    }

                    let within_function_stmt = match &reference.within_function_stmt {
                        Some(within_function_stmt) => within_function_stmt,
                        None => return AnalyzedReference::Read,
                    };

                    let function_call_stmt = &ast_context.scope_manager.function_calls
                        [within_function_stmt.function_call_stmt_id];

                    // The function call it's within is script defined, we can't assume anything
                    if ast_context.scope_manager.references[function_call_stmt.initial_reference]
                        .resolved
                        .is_some()
                    {
                        return AnalyzedReference::Read;
                    }

                    let function_behavior = match context
                        .standard_library
                        .find_global(&function_call_stmt.call_name_path)
                    {
                        Some(Field {
                            field_kind: FieldKind::Function(function_behavior),
                            ..
                        }) => function_behavior,
                        _ => return AnalyzedReference::Read,
                    };

                    let argument = match function_behavior
                        .arguments
                        .get(within_function_stmt.argument_index)
                    {
                        Some(argument) => argument,
                        None => return AnalyzedReference::Read,
                    };

                    let write_only = argument.observes == Observes::Write;

                    if !write_only {
                        return AnalyzedReference::Read;
                    }

                    AnalyzedReference::ObservedWrite(Label::new_with_message(
                        reference.identifier,
                        format!(
                            "`{}` only writes to `{}`",
                            // TODO: This is a typo if this is a method call
                            function_call_stmt.call_name_path.join("."),
                            variable.name
                        ),
                    ))
                })
                .collect::<Vec<_>>();

            if !analyzed_references
                .iter()
                .any(|reference| reference == &AnalyzedReference::Read)
            {
                let mut notes = Vec::new();

                if variable.is_self {
                    if self.allow_unused_self {
                        continue;
                    }

                    notes.push("`self` is implicitly defined when defining a method".to_owned());
                    notes
                        .push("if you don't need it, consider using `.` instead of `:`".to_owned());
                }

                let write_only = !analyzed_references.is_empty();

                diagnostics.push(Diagnostic::new_complete(
                    "unused_variable",
                    if write_only {
                        format!("{} is assigned a value, but never used", variable.name)
                    } else {
                        format!("{} is defined, but never used", variable.name)
                    },
                    Label::new(variable.identifiers[0]),
                    notes,
                    analyzed_references
                        .into_iter()
                        .filter_map(|reference| {
                            if let AnalyzedReference::ObservedWrite(label) = reference {
                                Some(label)
                            } else {
                                None
                            }
                        })
                        .collect(),
                ));
            };
        }

        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::{super::test_util::test_lint, *};

    #[test]
    fn test_blocks() {
        test_lint(
            UnusedVariableLint::new(UnusedVariableConfig::default()).unwrap(),
            "unused_variable",
            "blocks",
        );
    }

    #[test]
    fn test_locals() {
        test_lint(
            UnusedVariableLint::new(UnusedVariableConfig::default()).unwrap(),
            "unused_variable",
            "locals",
        );
    }

    #[test]
    fn test_edge_cases() {
        test_lint(
            UnusedVariableLint::new(UnusedVariableConfig::default()).unwrap(),
            "unused_variable",
            "edge_cases",
        );
    }

    #[test]
    fn test_explicit_self() {
        test_lint(
            UnusedVariableLint::new(UnusedVariableConfig::default()).unwrap(),
            "unused_variable",
            "explicit_self",
        );
    }

    #[test]
    fn test_function_overriding() {
        test_lint(
            UnusedVariableLint::new(UnusedVariableConfig::default()).unwrap(),
            "unused_variable",
            "function_overriding",
        );
    }

    #[test]
    fn test_generic_for_shadowing() {
        test_lint(
            UnusedVariableLint::new(UnusedVariableConfig::default()).unwrap(),
            "unused_variable",
            "generic_for_shadowing",
        );
    }

    #[test]
    fn test_if() {
        test_lint(
            UnusedVariableLint::new(UnusedVariableConfig::default()).unwrap(),
            "unused_variable",
            "if",
        );
    }

    #[test]
    fn test_ignore() {
        test_lint(
            UnusedVariableLint::new(UnusedVariableConfig::default()).unwrap(),
            "unused_variable",
            "ignore",
        );
    }

    #[test]
    fn test_invalid_regex() {
        assert!(UnusedVariableLint::new(UnusedVariableConfig {
            ignore_pattern: "(".to_owned(),
            ..UnusedVariableConfig::default()
        })
        .is_err());
    }

    #[test]
    fn test_mutating_functions() {
        test_lint(
            UnusedVariableLint::new(UnusedVariableConfig::default()).unwrap(),
            "unused_variable",
            "mutating_functions",
        );
    }

    #[test]
    fn test_observes() {
        test_lint(
            UnusedVariableLint::new(UnusedVariableConfig::default()).unwrap(),
            "unused_variable",
            "observes",
        );
    }

    #[test]
    fn test_overriding() {
        test_lint(
            UnusedVariableLint::new(UnusedVariableConfig::default()).unwrap(),
            "unused_variable",
            "overriding",
        );
    }

    #[test]
    fn test_self() {
        test_lint(
            UnusedVariableLint::new(UnusedVariableConfig {
                allow_unused_self: false,
                ..UnusedVariableConfig::default()
            })
            .unwrap(),
            "unused_variable",
            "self",
        );
    }

    #[test]
    fn test_self_ignored() {
        test_lint(
            UnusedVariableLint::new(UnusedVariableConfig::default()).unwrap(),
            "unused_variable",
            "self_ignored",
        );
    }

    #[test]
    fn test_shadowing() {
        test_lint(
            UnusedVariableLint::new(UnusedVariableConfig::default()).unwrap(),
            "unused_variable",
            "shadowing",
        );
    }

    #[test]
    fn test_varargs() {
        test_lint(
            UnusedVariableLint::new(UnusedVariableConfig::default()).unwrap(),
            "unused_variable",
            "varargs",
        );
    }

    #[cfg(feature = "roblox")]
    #[test]
    fn test_types() {
        test_lint(
            UnusedVariableLint::new(UnusedVariableConfig::default()).unwrap(),
            "unused_variable",
            "types",
        );
    }

    #[test]
    fn test_write_only() {
        test_lint(
            UnusedVariableLint::new(UnusedVariableConfig::default()).unwrap(),
            "unused_variable",
            "write_only",
        );
    }
}
