import * as roblox from "./roblox"
import * as selene from "./selene"
import * as timers from "timers"
import * as util from "./util"
import * as vscode from "vscode"
import * as path from "path"
import * as fs from "fs"
import * as toml from "toml"
import * as micromatch from "micromatch"
import { Diagnostic, Severity, Label } from "./structures/diagnostic"
import { Output } from "./structures/output"
import { lintConfig } from "./configLint"
import { byteToCharMap } from "./byteToCharMap"
import { Capabilities } from "./structures/capabilities"

let trySelene: Promise<boolean>

enum RunType {
    OnSave = "onSave",
    OnType = "onType",
    OnNewLine = "onNewLine",
    OnIdle = "onIdle",
}

function labelToRange(
    document: vscode.TextDocument,
    label: Label,
    byteOffsetMap: Map<number, number>,
): vscode.Range {
    return new vscode.Range(
        document.positionAt(
            byteOffsetMap.get(label.span.start) ?? label.span.start,
        ),
        document.positionAt(
            byteOffsetMap.get(label.span.end) ?? label.span.end,
        ),
    )
}

export async function activate(
    context: vscode.ExtensionContext,
): Promise<void> {
    console.log("selene-vscode activated")

    let capabilities: Capabilities = {}

    trySelene = util
        .ensureSeleneExists(context.globalStorageUri)
        .then(() => {
            selene
                .seleneCommand(
                    context.globalStorageUri,
                    "capabilities --display-style=json2",
                    selene.Expectation.Stdout,
                )
                .then((output) => {
                    if (output === null) {
                        return
                    }

                    capabilities = JSON.parse(output.toString())
                })
                .catch(() => {
                    // selene version is too old
                    return
                })
        })
        .then(() => {
            return true
        })
        .catch((error) => {
            vscode.window.showErrorMessage(
                `An error occurred when finding selene:\n${error}`,
            )
            return false
        })

    await trySelene

    context.subscriptions.push(
        vscode.commands.registerCommand("selene.reinstall", () => {
            trySelene = util
                .downloadSelene(context.globalStorageUri)
                .then(() => true)
                .catch(() => false)
            return trySelene
        }),
        vscode.commands.registerCommand(
            "selene.update-roblox-std",
            async () => {
                const output = await selene
                    .seleneCommand(
                        context.globalStorageUri,
                        "update-roblox-std",
                        selene.Expectation.Stdout,
                    )
                    .catch((error) => {
                        vscode.window.showErrorMessage(
                            `Couldn't update Roblox standard library: \n${error}`,
                        )
                        return false
                    })
                vscode.window.showInformationMessage(output?.toString() || "")
            },
        ),
        vscode.commands.registerCommand(
            "selene.generate-roblox-std",
            async () => {
                const output = await selene
                    .seleneCommand(
                        context.globalStorageUri,
                        "generate-roblox-std",
                        selene.Expectation.Stdout,
                    )
                    .catch((error) => {
                        vscode.window.showErrorMessage(
                            `Couldn't create Roblox standard library: \n${error}`,
                        )
                        return false
                    })
                vscode.window.showInformationMessage(output?.toString() || "")
            },
        ),
    )

    const diagnosticsCollection =
        vscode.languages.createDiagnosticCollection("selene")
    context.subscriptions.push(diagnosticsCollection)

    let hasWarnedAboutRoblox = false

    async function lint(document: vscode.TextDocument) {
        if (!(await trySelene)) {
            return
        }

        switch (document.languageId) {
            case "lua":
            case "luau":
                break
            case "toml":
            case "yaml":
                await lintConfig(
                    capabilities,
                    context,
                    document,
                    diagnosticsCollection,
                )
                return
            default:
                return
        }

        const workspaceFolder = vscode.workspace.getWorkspaceFolder(
            document.uri,
        )
        const workspaceRoot = workspaceFolder?.uri.fsPath
        if (!workspaceRoot) {
            console.error("Failed to find workspace root")
            return
        }

        const configPath = path.join(workspaceRoot, "selene.toml")
        let config: selene.Config = {}
        try {
            const configFileContent = fs.readFileSync(configPath, "utf-8")
            config = toml.parse(configFileContent)
        } catch (error) {
            console.error(`Error parsing config file: ${error}`)
            return
        }

        // We don't invoke selene on the files directly as it won't work on unsaved changes, so we
        // need to check for exclude paths separately
        const shouldExclude = (config.exclude || []).some((pattern: string) => {
            // Document path given is absolute so the patterns should be as well
            // If multiple `selene.toml` becomes supported, this will likely need to be changed to support it.
            const excludeGlobAbsolute = path.isAbsolute(pattern)
                ? pattern
                : path.join(workspaceRoot, pattern)

            return micromatch.isMatch(
                document.uri.fsPath.replace(/\\/g, "/"),
                excludeGlobAbsolute.replace(/\\/g, "/"),
                {
                    bash: true,
                },
            )
        })

        if (shouldExclude) {
            diagnosticsCollection.delete(document.uri)
            return
        }

        const output = await selene.seleneCommand(
            context.globalStorageUri,
            "--display-style=json2 --no-summary -",
            selene.Expectation.Stderr,
            workspaceFolder,
            document.getText(),
        )

        if (!output) {
            diagnosticsCollection.delete(document.uri)
            return
        }

        const diagnostics: vscode.Diagnostic[] = []
        const dataToAdd: Diagnostic[] = []
        const byteOffsets = new Set<number>()

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

            switch (output.type) {
                case "Diagnostic":
                    dataToAdd.push(output)
                    byteOffsets.add(output.primary_label.span.start)
                    byteOffsets.add(output.primary_label.span.end)
                    for (const label of output.secondary_labels) {
                        byteOffsets.add(label.span.start)
                        byteOffsets.add(label.span.end)
                    }
                    break
                case "InvalidConfig":
                    break
            }
        }

        const byteOffsetMap = byteToCharMap(document, byteOffsets)

        for (const data of dataToAdd) {
            let message = data.message
            if (data.primary_label.message.length > 0) {
                message += `\n${data.primary_label.message}`
            }

            if (data.notes.length > 0) {
                message += `\n${data.notes.map((note) => `note: ${note}\n`)}`
            }

            const diagnostic = new vscode.Diagnostic(
                labelToRange(document, data.primary_label, byteOffsetMap),
                message,
                data.severity === Severity.Error
                    ? vscode.DiagnosticSeverity.Error
                    : vscode.DiagnosticSeverity.Warning,
            )

            diagnostic.source = `selene::${data.code}`

            if (data.code === "unused_variable") {
                diagnostic.tags = [vscode.DiagnosticTag.Unnecessary]
            }

            diagnostic.relatedInformation = data.secondary_labels.map(
                (label) => {
                    return {
                        message: label.message,
                        location: {
                            uri: document.uri,
                            range: labelToRange(document, label, byteOffsetMap),
                        },
                    }
                },
            )

            if (
                vscode.workspace
                    .getConfiguration("selene")
                    .get<boolean>("warnRoblox")
            ) {
                if (
                    !hasWarnedAboutRoblox &&
                    roblox.processDiagnostic(data, document)
                ) {
                    hasWarnedAboutRoblox = true
                }
            }

            diagnostics.push(diagnostic)
        }

        diagnosticsCollection.set(document.uri, diagnostics)
    }

    let lastTimeout: NodeJS.Timeout
    function listenToChange() {
        switch (
            vscode.workspace.getConfiguration("selene").get<RunType>("run")
        ) {
            case RunType.OnSave:
                return vscode.workspace.onDidSaveTextDocument(lint)
            case RunType.OnType:
                return vscode.workspace.onDidChangeTextDocument((event) =>
                    lint(event.document),
                )
            case RunType.OnNewLine:
                return vscode.workspace.onDidChangeTextDocument((event) => {
                    // Contrary to removing lines, adding new lines will leave the range at the same value hence the string comparisons
                    if (
                        event.contentChanges.some(
                            (content) =>
                                !content.range.isSingleLine ||
                                content.text === "\n" ||
                                content.text === "\r\n",
                        )
                    ) {
                        lint(event.document)
                    }
                })
            case RunType.OnIdle: {
                const idleDelay = vscode.workspace
                    .getConfiguration("selene")
                    .get<number>("idleDelay") as number

                return vscode.workspace.onDidChangeTextDocument((event) => {
                    timers.clearTimeout(lastTimeout)
                    lastTimeout = timers.setTimeout(
                        lint,
                        idleDelay,
                        event.document,
                    )
                })
            }
        }
    }

    let disposable = listenToChange()
    vscode.workspace.onDidChangeConfiguration((event) => {
        if (
            event.affectsConfiguration("selene.run") ||
            event.affectsConfiguration("selene.idleDelay")
        ) {
            disposable?.dispose()
            disposable = listenToChange()
        }
    })

    vscode.workspace.onDidOpenTextDocument(lint)
    vscode.workspace.onWillDeleteFiles((event) => {
        for (const documentUri of event.files) {
            diagnosticsCollection.set(documentUri, [])
        }
    })
    vscode.window.onDidChangeActiveTextEditor((editor) => {
        if (editor !== undefined) {
            lint(editor.document)
        }
    })
}

// this method is called when your extension is deactivated
export function deactivate(): void {
    return
}
