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
}
