use std::io;
use codespan_reporting::diagnostic::{
    Diagnostic as CodespanDiagnostic, Label as CodespanLabel, LabelStyle, Severity,
};
use serde::Serialize;
use termcolor::StandardStream;
use std::io::Write;

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum JsonOutput {
    Diagnostic(JsonDiagnostic),
    Summary(JsonSummary),
}

#[derive(Serialize)]
pub struct JsonSummary {
    pub(crate) errors: usize,
    pub(crate) warnings: usize,
    pub(crate) parse_errors: usize,
}

#[derive(Serialize)]
pub struct JsonDiagnostic {
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
    start_line: usize,
    start_column: usize,
    end: usize,
    end_line: usize,
    end_column: usize,
}

fn label_to_serializable(
    label: &CodespanLabel<codespan::FileId>,
    files: &codespan::Files<&str>,
) -> Label {
    let start_location = files
        .location(label.file_id, label.range.start as u32)
        .expect("unable to determine start location for label");
    let end_location = files
        .location(label.file_id, label.range.end as u32)
        .expect("unable to determine end location for label");
    Label {
        message: label.message.to_owned(),
        span: Span {
            start: label.range.start,
            start_line: start_location.line.into(),
            start_column: start_location.column.into(),
            end: label.range.end,
            end_line: end_location.line.into(),
            end_column: end_location.column.into(),
        },
    }
}

pub fn diagnostic_to_json(
    diagnostic: &CodespanDiagnostic<codespan::FileId>,
    files: &codespan::Files<&str>,
) -> JsonDiagnostic {
    JsonDiagnostic {
        code: diagnostic.code.to_owned(),
        message: diagnostic.message.to_owned(),
        severity: diagnostic.severity.to_owned(),
        notes: diagnostic.notes.to_owned(),
        primary_label: label_to_serializable(
            diagnostic.labels.first().expect("no labels passed"),
            files,
        ),
        secondary_labels: diagnostic
            .labels
            .iter()
            .filter(|label| label.style == LabelStyle::Secondary)
            .map(|label| label_to_serializable(label, files))
            .collect(),
    }
}

pub fn log_total_json(
    mut stdout: StandardStream,
    parse_errors: usize,
    lint_errors: usize,
    lint_warnings: usize,
) -> io::Result<()> {
    writeln!(
        stdout,
        "{}",
        serde_json::to_string(&JsonOutput::Summary(JsonSummary {
            errors: lint_errors,
            warnings: lint_warnings,
            parse_errors
        }))
            .unwrap()
    )
        .unwrap();

    Ok(())
}
