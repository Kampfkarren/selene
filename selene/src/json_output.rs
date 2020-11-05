use codespan_reporting::diagnostic::{
    Diagnostic as CodespanDiagnostic, Label as CodespanLabel, LabelStyle, Severity,
};
use serde::Serialize;

#[derive(Serialize)]
struct JsonDiagnostic {
    severity: Severity,
    code: Option<String>,
    message: String,
    primary_label: Label,
    notes: Vec<String>,
    secondary_labels: Vec<Label>,
}

#[derive(Serialize)]
struct Label {
    span: Span,
    message: String,
}

#[derive(Serialize)]
struct Span {
    start: usize,
    end: usize,
}

fn label_to_serializable<T>(label: &CodespanLabel<T>) -> Label {
    Label {
        message: label.message.to_owned(),
        span: Span {
            start: label.range.start,
            end: label.range.end,
        },
    }
}

pub fn diagnostic_to_json(
    diagnostic: &CodespanDiagnostic<codespan::FileId>,
) -> serde_json::Result<String> {
    serde_json::to_string(&JsonDiagnostic {
        code: diagnostic.code.to_owned(),
        message: diagnostic.message.to_owned(),
        severity: diagnostic.severity.to_owned(),
        notes: diagnostic.notes.to_owned(),
        primary_label: label_to_serializable(diagnostic.labels.first().expect("no labels passed")),
        secondary_labels: diagnostic
            .labels
            .iter()
            .filter(|label| label.style == LabelStyle::Secondary)
            .map(label_to_serializable)
            .collect(),
    })
}
