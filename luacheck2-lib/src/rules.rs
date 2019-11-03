use crate::standard_library::StandardLibrary;
use std::convert::TryInto;

use codespan_reporting::diagnostic::{
    Diagnostic as CodespanDiagnostic, Label as CodespanLabel, Severity as CodespanSeverity,
};
use full_moon::node::Node;
use serde::de::DeserializeOwned;

pub mod almost_swapped;
pub mod divide_by_zero;
pub mod empty_if;
pub mod global_usage;
pub mod if_same_then_else;
pub mod ifs_same_cond;
pub mod multiple_statements;
pub mod parenthese_conditions;
pub mod shadowing;
pub mod standard_library;
pub mod suspicious_reverse_loop;
pub mod unbalanced_assignments;
pub mod undefined_variable;
pub mod unused_variable;

#[cfg(test)]
mod test_util;

pub trait Rule {
    type Config: DeserializeOwned;
    type Error: std::error::Error;

    fn new(config: Self::Config) -> Result<Self, Self::Error>
    where
        Self: Sized;
    fn pass(&self, ast: &full_moon::ast::Ast<'static>, context: &Context) -> Vec<Diagnostic>;

    fn allow(&self) -> bool {
        false
    }

    fn severity(&self) -> Severity;
    fn rule_type(&self) -> RuleType;
}

pub enum RuleType {
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
    Error,
    Warning,
}

pub struct Diagnostic {
    code: &'static str,
    message: String,
    notes: Vec<String>,
    primary_label: Label,
    secondary_labels: Vec<Label>,
}

impl Diagnostic {
    pub fn new(code: &'static str, message: String, primary_label: Label) -> Self {
        Self {
            code,
            message,
            primary_label,

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
    ) -> Self {
        Self {
            code,
            message,
            notes,
            primary_label,
            secondary_labels,
        }
    }

    pub fn into_codespan_diagnostic(
        self,
        file_id: codespan::FileId,
        severity: CodespanSeverity,
    ) -> CodespanDiagnostic {
        CodespanDiagnostic {
            severity,
            code: Some(self.code.to_owned()),
            message: self.message.to_owned(),
            primary_label: self.primary_label.codespan_label(file_id),
            notes: self.notes.to_owned(),
            secondary_labels: self
                .secondary_labels
                .iter()
                .map(|label| label.codespan_label(file_id))
                .collect(),
        }
    }

    pub fn start_position(&self) -> u32 {
        self.primary_label.range.0
    }
}

pub struct Label {
    message: Option<String>,
    range: (u32, u32),
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

    pub fn new_with_message(range: (u32, u32), message: String) -> Label {
        Label {
            range,
            message: Some(message),
        }
    }

    pub fn codespan_label(&self, file_id: codespan::FileId) -> CodespanLabel {
        CodespanLabel::new(
            file_id.to_owned(),
            codespan::Span::new(self.range.0, self.range.1),
            self.message.as_ref().unwrap_or(&"".to_owned()).to_owned(),
        )
    }
}

#[derive(Clone, Debug)]
pub struct Context {
    pub standard_library: StandardLibrary,
}

impl Context {
    #[cfg(feature = "roblox")]
    pub fn is_roblox(&self) -> bool {
        if let Some(meta) = self.standard_library.meta {
            meta.name == Some("roblox")
        } else {
            false
        }
    }

    #[cfg(not(feature = "roblox"))]
    pub fn is_roblox(&self) -> bool {
        false
    }
}
