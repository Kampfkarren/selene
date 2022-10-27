import * as selene from "./selene"
import * as vscode from "vscode"
import { spawn } from "child_process"
import { getSelenePath } from "./util"
import { lint } from "./extension"

const alreadyKnownPaths = new Set<string>()

const OPTION_YES = "Yes"
const OPTION_NOT_THIS_TIME = "Not this time"
const OPTION_NEVER = "Never for this project"

export async function pluginsNotLoadedFor(
    context: vscode.ExtensionContext,
    path: string,
): Promise<void> {
    if (alreadyKnownPaths.has(path)) {
        return
    }

    alreadyKnownPaths.add(path)

    const option = await vscode.window.showInformationMessage(
        "This project is trying to load plugins. Do you want to enable them?",
        OPTION_YES,
        OPTION_NOT_THIS_TIME,
        OPTION_NEVER,
    )

    const command = ["plugin-authorization", path]
    let response: string

    switch (option) {
        case OPTION_YES:
            response = "Plugins successfully enabled."
            break
        case OPTION_NEVER:
            command.push("--block")
            response = "Plugins will not be enabled for this project."
            break
        case OPTION_NOT_THIS_TIME:
        case undefined:
            return
    }

    const selenePath = await getSelenePath(context.globalStorageUri)
    if (!selenePath) {
        throw new Error("Could not find selene.")
    }

    const seleneExec = spawn(selenePath.fsPath, command)

    let stderr = ""

    seleneExec.stderr.on("data", (data) => {
        stderr += data.toString()
    })

    seleneExec.on("close", (code) => {
        if (code !== 0) {
            vscode.window.showErrorMessage(
                `An error occurred while trying to enable plugins.\n\n${stderr}`,
            )

            return
        }

        vscode.window.showInformationMessage(response)

        if (option === OPTION_YES) {
            for (const document of vscode.workspace.textDocuments) {
                lint(document)
            }
        }
    })
}
