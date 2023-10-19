use crate::{ast_util::scopes::ScopeManager, standard_library::StandardLibrary};
use std::convert::TryInto;

use codespan_reporting::diagnostic::{
    Diagnostic as CodespanDiagnostic, Label as CodespanLabel, Severity as CodespanSeverity,
};
use full_moon::{ast::Ast, node::Node};
use serde::de::DeserializeOwned;

pub mod almost_swapped;
pub mod bad_string_escape;
pub mod compare_nan;
pub mod constant_table_comparison;
pub mod deprecated;
pub mod divide_by_zero;
pub mod duplicate_keys;
pub mod empty_if;
pub mod empty_loop;
pub mod global_usage;
pub mod high_cyclomatic_complexity;
pub mod if_same_then_else;
pub mod ifs_same_cond;
pub mod invalid_lint_filter;
pub mod manual_table_clone;
pub mod mismatched_arg_count;
pub mod mixed_table;
pub mod multiple_statements;
pub mod must_use;
pub mod parenthese_conditions;
pub mod shadowing;
pub mod standard_library;
pub mod suspicious_reverse_loop;
pub mod type_check_inside_call;
pub mod unbalanced_assignments;
pub mod undefined_variable;
pub mod unscoped_variables;
pub mod unused_variable;

#[cfg(feature = "roblox")]
pub mod roblox_incorrect_color3_new_bounds;

#[cfg(feature = "roblox")]
pub mod roblox_incorrect_roact_usage;

#[cfg(feature = "roblox")]
pub mod roblox_suspicious_udim2_new;

#[cfg(test)]
mod test_util;

pub trait Lint {
    type Config: DeserializeOwned;
    type Error: std::error::Error;

    const SEVERITY: Severity;
    const LINT_TYPE: LintType;

    fn new(config: Self::Config) -> Result<Self, Self::Error>
    where
        Self: Sized;

    fn pass(
        &self,
        ast: &full_moon::ast::Ast,
        context: &Context,
        ast_context: &AstContext,
    ) -> Vec<Diagnostic>;
}

pub enum LintType {
    /// Code that does something simple but in a complex way
    Complexity,

    /// Code that is outright wrong or very very useless
    /// Should have severity "Error"
    Correctness,

    /// Code that can be written in a faster way
    Performance,

    /// Code that should be written in a more idiomatic way
    Style,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Allow,
    Error,
    Warning,
}

/// Indicates the confidence in the correctness of a suggestion
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Applicability {
    /// The suggestion is definitely what the user intended, or maintains the exact meaning of the code
    ///
    /// The suggestion is safe to be automatically applied
    MachineApplicable,

    /// The suggestion is probably what the user intended, but may not maintain the exact meaning of the code
    ///
    /// The suggestion generates valid code when applied
    MaybeIncorrect,

    /// The suggestion is probably what the user intended, but may not maintain the exact meaning of the code
    ///
    /// The suggestion does not generate valid code when applied
    HasPlaceholders,

    Unspecified,
}

#[derive(Debug)]
pub struct Diagnostic {
    pub code: &'static str,
    pub message: String,
    pub notes: Vec<String>,
    pub primary_label: Label,
    pub secondary_labels: Vec<Label>,
    pub suggestion: Option<String>,
    pub applicability: Applicability,
}

impl Diagnostic {
    pub fn new(
        code: &'static str,
        message: String,
        primary_label: Label,
        suggestion: Option<String>,
        applicability: Applicability,
    ) -> Self {
        let mut label = primary_label;
        if let Some(ref suggestion_str) = suggestion {
            if label.message.is_none() {
                label = Label::new_with_message(
                    label.range,
                    if suggestion_str.is_empty() {
                        "consider removing this".to_string()
                    } else {
                        format!("try: `{}`", suggestion_str)
                    },
                );
            }
        };

        Self {
            code,
            message,
            primary_label: label,
            suggestion,
            applicability,

            notes: Vec::new(),
            secondary_labels: Vec::new(),
        }
    }

    pub fn new_complete(
        code: &'static str,
        message: String,
        primary_label: Label,
        notes: Vec<String>,
        secondary_labels: Vec<Label>,
        suggestion: Option<String>,
        applicability: Applicability,
    ) -> Self {
        Self {
            code,
            message,
            notes,
            primary_label,
            secondary_labels,
            suggestion,
            applicability,
        }
    }

    pub fn into_codespan_diagnostic(
        self,
        file_id: codespan::FileId,
        severity: CodespanSeverity,
    ) -> CodespanDiagnostic<codespan::FileId> {
        let mut labels = Vec::with_capacity(1 + self.secondary_labels.len());
        labels.push(self.primary_label.codespan_label(file_id));
        labels.extend(&mut self.secondary_labels.iter().map(|label| {
            CodespanLabel::secondary(file_id, codespan::Span::new(label.range.0, label.range.1))
                .with_message(label.message.as_ref().unwrap_or(&"".to_owned()).to_owned())
        }));

        CodespanDiagnostic {
            code: Some(self.code.to_owned()),
            labels,
            message: self.message.to_owned(),
            notes: self.notes,
            severity,
        }
    }

    pub fn start_position(&self) -> u32 {
        self.primary_label.range.0
    }

    pub fn has_machine_applicable_fix(&self) -> bool {
        self.suggestion.is_some() && self.applicability == Applicability::MachineApplicable
    }

    pub fn has_maybe_incorrect_fix(&self) -> bool {
        self.suggestion.is_some() && self.applicability == Applicability::MaybeIncorrect
    }

    /// After applying suggestions, calls `get_new_diagnostics` and reruns to ensure fixes didn't produce new errors
    pub fn get_applied_suggestions_code<F>(
        code: &str,
        diagnostics: Vec<&Diagnostic>,
        get_new_diagnostics: F,
    ) -> String
    where
        F: Fn(&str) -> Vec<Diagnostic>,
    {
        let mut chosen_diagnostics = Self::get_independent_suggestions(&diagnostics);
        let mut owned_diagnostics_ref;
        let mut owned_diagnostics;

        let mut fixed_code = code.to_string();

        while !chosen_diagnostics.is_empty() {
            let mut bytes_offset = 0;

            for diagnostic in chosen_diagnostics {
                if let Some(suggestion) = &diagnostic.suggestion {
                    let (start, end) = diagnostic.primary_label.range;

                    // This conversion can theoretically overflow, but it's tied to string length so the user
                    // would face memory constraints of such large strings long before this is an issue
                    let start_with_offset = start as isize + bytes_offset;
                    let end_with_offset = end as isize + bytes_offset;

                    fixed_code = format!(
                        "{}{}{}",
                        // Conversion is safe as this algorithm guarantees range + offset will never be negative
                        &fixed_code[..start_with_offset as usize],
                        suggestion.as_str(),
                        &fixed_code[(end_with_offset as usize)..]
                    );

                    bytes_offset +=
                        suggestion.len() as isize - (end_with_offset - start_with_offset);
                }
            }

            owned_diagnostics = get_new_diagnostics(&fixed_code);
            owned_diagnostics_ref = owned_diagnostics.iter().collect::<Vec<_>>();
            chosen_diagnostics = Self::get_independent_suggestions(&owned_diagnostics_ref);
        }

        fixed_code
    }

    /// * Chooses all disjoint suggestions
    /// * If a suggestion is completely contained inside others, uses the innermost one and discards the outer ones
    /// * If suggestions partially overlap, chooses the one that starts first and discard the other ones
    fn get_independent_suggestions<'a>(diagnostics: &'a [&'a Diagnostic]) -> Vec<&'a Diagnostic> {
        let mut sorted_diagnostics = diagnostics
            .iter()
            .filter(|&&diagnostic| {
                diagnostic.has_machine_applicable_fix() || diagnostic.has_maybe_incorrect_fix()
            })
            .copied()
            .collect::<Vec<_>>();
        sorted_diagnostics.sort_by_key(|diagnostic| diagnostic.start_position());

        let mut chosen_diagnostics = Vec::new();

        let mut candidate = match sorted_diagnostics.first() {
            Some(first) => *first,
            None => return Vec::new(),
        };

        for &diagnostic in sorted_diagnostics.iter().skip(1) {
            let this_range = diagnostic.primary_label.range;

            if this_range.1 <= candidate.primary_label.range.1 {
                // Completely contained inside
                candidate = diagnostic;
            } else if this_range.0 <= candidate.primary_label.range.1 {
                // Partially overlapping
                continue;
            } else {
                chosen_diagnostics.push(candidate);
                candidate = diagnostic;
            }
        }
        chosen_diagnostics.push(candidate);

        chosen_diagnostics
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Label {
    pub message: Option<String>,
    pub range: (u32, u32),
}

impl Label {
    pub fn new<P: TryInto<u32>>(range: (P, P)) -> Label {
        let range = (
            range
                .0
                .try_into()
                .unwrap_or_else(|_| panic!("TryInto failed for Label::new range")),
            range
                .1
                .try_into()
                .unwrap_or_else(|_| panic!("TryInto failed for Label::new range")),
        );

        Label {
            range,
            message: None,
        }
    }

    pub fn from_node<N: Node>(node: N, message: Option<String>) -> Label {
        let (start, end) = node.range().expect("node passed returned a None range");

        Label {
            message,
            range: (start.bytes() as u32, end.bytes() as u32),
        }
    }

    pub fn new_with_message<P: TryInto<u32>>(range: (P, P), message: String) -> Label {
        let range = (
            range
                .0
                .try_into()
                .unwrap_or_else(|_| panic!("TryInto failed for Label::new range")),
            range
                .1
                .try_into()
                .unwrap_or_else(|_| panic!("TryInto failed for Label::new range")),
        );

        Label {
            range,
            message: Some(message),
        }
    }

    pub fn codespan_label(&self, file_id: codespan::FileId) -> CodespanLabel<codespan::FileId> {
        CodespanLabel::primary(
            file_id.to_owned(),
            codespan::Span::new(self.range.0, self.range.1),
        )
        .with_message(self.message.as_ref().unwrap_or(&"".to_owned()).to_owned())
    }
}

#[derive(Clone, Debug)]
pub struct Context {
    pub standard_library: StandardLibrary,
    pub user_set_standard_library: Option<Vec<String>>,
}

impl Context {
    #[cfg(feature = "roblox")]
    pub fn is_roblox(&self) -> bool {
        self.standard_library.name.as_deref() == Some("roblox")
    }

    #[cfg(not(feature = "roblox"))]
    pub fn is_roblox(&self) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct AstContext {
    pub scope_manager: ScopeManager,

    // Converting ast -> code should probably come from the parser, but this should be ok as a temporary solution
    pub code: String,
}

impl AstContext {
    pub fn from_ast(ast: &Ast, code: &String) -> Self {
        Self {
            scope_manager: ScopeManager::new(ast),
            code: code.to_string(),
        }
    }
}
