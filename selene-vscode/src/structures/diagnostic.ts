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
    fixed_code: string
}

export class SeleneDiagnostic extends vscode.Diagnostic {
    fixed_code?: string

    constructor(
        range: vscode.Range,
        message: string,
        severity: vscode.DiagnosticSeverity,
        fixed_code?: string,
    ) {
        super(range, message, severity)
        this.fixed_code = fixed_code
    }
}
