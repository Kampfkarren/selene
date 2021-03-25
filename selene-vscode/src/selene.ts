import * as childProcess from "child_process"
import * as process from "process"
import * as vscode from "vscode"
import * as util from "./util"

export enum Expectation {
    Stderr,
    Stdout,
}

export async function seleneCommand(
    storagePath: vscode.Uri,
    command: string,
    expectation: Expectation,
    workspace?: vscode.WorkspaceFolder,
    stdin?: string,
): Promise<string | null> {
    return new Promise(async (resolve, reject) => {
        const selenePath = await util.getSelenePath(storagePath)
        if (selenePath === undefined) {
            return reject("Could not find selene.")
        }

        const child = childProcess.exec(`"${selenePath.fsPath}" ${command}`, {
            cwd: workspace?.uri.fsPath,
        }, (error, stdout) => {
            if (expectation === Expectation.Stderr) {
                resolve(error && stdout)
            } else {
                if (error) {
                    reject(error)
                } else {
                    resolve(stdout)
                }
            }
        })

        if (stdin !== undefined) {
            child.stdin?.write(stdin)
            child.stdin?.end()
        }
    })
}
