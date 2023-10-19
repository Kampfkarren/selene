import * as vscode from "vscode"

export interface Label {
    message: string
    span: {
        start: number
        end: number
    }
}

export enum Severity {
    Error = "Error",
    Warning = "Warning",
}

export interface Diagnostic {
    code: string
    message: string
    severity: Severity
    notes: string[]
    primary_label: Label
    secondary_labels: Label[]
    suggestion: string
}

export class SeleneDiagnostic extends vscode.Diagnostic {
    suggestion?: string

    constructor(
        range: vscode.Range,
        message: string,
        severity: vscode.DiagnosticSeverity,
        suggestion?: string,
    ) {
        super(range, message, severity)
        this.suggestion = suggestion
    }
}
