import * as selene from "./selene"
import * as util from "./util"
import * as vscode from "vscode"

let trySelene: Promise<boolean>

interface Diagnostic {
    code: string,
    message: string,
    severity: Severity,
    notes: string[],
    primary_label: Label,
    secondary_labels: Label[],
}

interface Label {
    message: string,
    span: {
        start: number,
        end: number,
    }
}

enum Severity {
    Error = "Error",
    Warning = "Warning",
}

function labelToRange(document: vscode.TextDocument, label: Label): vscode.Range {
    return new vscode.Range(
        document.positionAt(label.span.start),
        document.positionAt(label.span.end),
    )
}

export async function activate(context: vscode.ExtensionContext) {
    console.log("selene-vscode activated")

    trySelene = util.ensureSeleneExists(context.globalStoragePath).then(() => {
        return true
    }).catch(error => {
        vscode.window.showErrorMessage(`An error occurred when finding Selene:\n${error}`)
        return false
    })

    await trySelene

    console.log("selene path", await util.getSelenePath(context.globalStoragePath))

    context.subscriptions.push(vscode.commands.registerCommand("selene.reinstall", () => {
        trySelene = util.downloadSelene(context.globalStoragePath).then(() => true).catch(() => false)
        return trySelene
    }))

    const diagnosticsCollection = vscode.languages.createDiagnosticCollection("selene")
    context.subscriptions.push(diagnosticsCollection)

    async function lint(document: vscode.TextDocument) {
        if (document.languageId !== "lua") {
            return
        }

        if (!await trySelene) {
            return
        }

        const output = await selene.seleneCommand(
            context.globalStoragePath,
            "--display-style=json -",
            selene.Expectation.Stderr,
            vscode.workspace.getWorkspaceFolder(
                vscode.Uri.file(document.uri.fsPath),
            )?.uri?.fsPath,
            document.getText(),
        )

        if (!output) {
            diagnosticsCollection.delete(document.uri)
            return
        }

        const diagnostics: vscode.Diagnostic[] = []

        for (const line of output.split("\n")) {
            if (line === "Results:") {
                break
            }

            const data: Diagnostic = JSON.parse(line)

            let message = data.message
            if (data.notes.length > 0) {
                message += `\n${data.notes.map(note => `note: ${note}\n`)}`
            }

            const diagnostic = new vscode.Diagnostic(
                labelToRange(document, data.primary_label),
                message,
                data.severity === Severity.Error ? vscode.DiagnosticSeverity.Error : vscode.DiagnosticSeverity.Warning,
            )

            diagnostic.source = `selene::${data.code}`

            if (data.code === "unused_variable") {
                diagnostic.tags = [vscode.DiagnosticTag.Unnecessary]
            }

            diagnostic.relatedInformation = data.secondary_labels.map(label => {
                return {
                    message: label.message,
                    location: {
                        uri: document.uri,
                        range: labelToRange(document, label),
                    },
                }
            })

            diagnostics.push(diagnostic)
        }

        diagnosticsCollection.set(document.uri, diagnostics)
    }

    vscode.workspace.onDidSaveTextDocument(lint)
    vscode.workspace.onDidOpenTextDocument(lint)
    vscode.workspace.onDidChangeTextDocument(event => lint(event.document))
    vscode.workspace.onWillDeleteFiles(event => {
        for (const documentUri of event.files) {
            diagnosticsCollection.set(documentUri, [])
        }
    })
    vscode.window.onDidChangeActiveTextEditor(editor => {
        if (editor !== undefined) {
            lint(editor.document)
        }
    })
}

// this method is called when your extension is deactivated
export function deactivate() { }
