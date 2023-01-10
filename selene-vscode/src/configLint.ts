import * as path from "path"
import * as vscode from "vscode"
import * as selene from "./selene"
import { Output } from "./structures/output"

export async function lintConfig(
    context: vscode.ExtensionContext,
    document: vscode.TextDocument,
    diagnosticsCollection: vscode.DiagnosticCollection,
) {
    // TODO: Check version of selene
    if (
        document.languageId === "toml" &&
        !document.uri.path.endsWith("selene.toml")
    ) {
        return
    }

    const workspaceFolder = vscode.workspace.getWorkspaceFolder(document.uri)

    // TODO: Use stdin
    const output = await selene.seleneCommand(
        context.globalStorageUri,
        "validate-config --display-style=json2",
        selene.Expectation.Stderr,
        workspaceFolder,
    )

    if (output === null) {
        diagnosticsCollection.delete(document.uri)
        return
    }

    const diagnostics: vscode.Diagnostic[] = []

    for (const line of output.split("\n")) {
        if (!line) {
            continue
        }

        let output: Output

        try {
            output = JSON.parse(line)
        } catch {
            console.error(`Couldn't parse output: ${line}`)
            continue
        }

        if (output.type !== "InvalidConfig") {
            continue
        }

        const workspacePath = workspaceFolder?.uri.fsPath
        if (workspacePath === undefined) {
            continue
        }

        const relativePath = path.relative(workspacePath, document.uri.fsPath)

        if (relativePath !== output.source) {
            continue
        }

        const range = output.location
            ? document.lineAt(output.location.line).range
            : new vscode.Range(
                  document.lineAt(0).range.start,
                  document.lineAt(document.lineCount - 1).range.end,
              )

        diagnostics.push(
            new vscode.Diagnostic(
                range,
                output.error,
                vscode.DiagnosticSeverity.Error,
            ),
        )
    }

    diagnosticsCollection.set(document.uri, diagnostics)
}
