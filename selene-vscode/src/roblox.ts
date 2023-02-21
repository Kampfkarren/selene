import * as vscode from "vscode"
import { Diagnostic } from "./structures/diagnostic"
import { TextDecoder, TextEncoder } from "util"

const SETUP_CONFIGURATION = "Setup Configuration"

const ROBLOX_PROBLEMS: Set<string> = new Set(
    ["game", "workspace", "UDim2", "Vector2", "Vector3", "Enum"].map(
        (name) => `\`${name}\` is not defined`,
    ),
)

export function processDiagnostic(
    diagnostic: Diagnostic,
    document: vscode.TextDocument,
): boolean {
    if (ROBLOX_PROBLEMS.has(diagnostic.message)) {
        const workspace = vscode.workspace.getWorkspaceFolder(document.uri)
        if (workspace === undefined) {
            return false
        }

        if (!vscode.workspace.fs.isWritableFileSystem(workspace.uri.scheme)) {
            return false
        }

        vscode.window
            .showWarningMessage(
                "It looks like you're trying to lint a Roblox codebase without proper configuration.",
                SETUP_CONFIGURATION,
                "Ignore",
            )
            .then(async (answer) => {
                if (answer !== SETUP_CONFIGURATION) {
                    return
                }

                const configFilename = vscode.Uri.joinPath(
                    workspace.uri,
                    "selene.toml",
                )

                let configContents: Uint8Array

                try {
                    configContents = await vscode.workspace.fs.readFile(
                        configFilename,
                    )
                } catch (error) {
                    if (
                        error instanceof vscode.FileSystemError &&
                        error.code === "FileNotFound"
                    ) {
                        configContents = new Uint8Array()
                    } else {
                        vscode.window.showErrorMessage(
                            `Couldn't read existing config, if there was one.\n\n${
                                typeof error === "object" && error !== null
                                    ? error.toString()
                                    : error
                            }`,
                        )

                        return
                    }
                }

                const contents = new TextDecoder().decode(configContents)

                vscode.workspace.fs.writeFile(
                    configFilename,
                    new TextEncoder().encode(addRobloxLibrary(contents)),
                )
            })

        return true
    }

    return false
}

// This is a heuristic, but if you're the type of person to know how to break this heuristic
// you're also the type of person to not need this feature
function addRobloxLibrary(contents: string): string {
    let standardLibrarySet = false

    const lines = contents.split("\n").map((line) => {
        if (!line.startsWith("std")) {
            return line
        }

        standardLibrarySet = true

        const match = line.match(/std\s*=\s*"(.+)"/)
        if (match === null) {
            // You are doing something dumb.
            return line
        }

        return `std = "roblox+${match[1]}"`
    })

    return standardLibrarySet
        ? lines.join("\n")
        : `std = "roblox"\n${lines.join("\n")}`
}
