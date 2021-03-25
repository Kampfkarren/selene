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
    cwd?: string,
    stdin?: string,
): Promise<string | null> {
    return new Promise(async (resolve, reject) => {
        const child = childProcess.exec(`"${await util.getSelenePath(storagePath).then(path => {
            if (path === undefined) {
                return Promise.reject("Could not find selene.")
            }

            return path
        })}" ${command}`, {
            cwd,
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
