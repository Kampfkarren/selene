import * as os from "os"
import * as selene from "./selene"
import * as requestNative from "request"
import * as request from "request-promise-native"
import * as unzip from "unzipper"
import * as vscode from "vscode"

const fsWriteFileAtomic = require("fs-write-stream-atomic")

const GITHUB_RELEASES = "https://summer-bonus-a893.boyned.workers.dev"

interface GithubRelease {
    assets: {
        name: string,
        browser_download_url: string,
    }[],
    html_url: string,
    tag_name: string,
}

let getLatestSeleneReleasePromise: Promise<GithubRelease>

export async function getLatestSeleneRelease(): Promise<GithubRelease> {
    if (getLatestSeleneReleasePromise) {
        return getLatestSeleneReleasePromise
    }

    // getLatestSeleneReleasePromise = new Promise(async () => {
    //     return JSON.parse(await ) as GithubRelease
    // })

    getLatestSeleneReleasePromise = request(
        GITHUB_RELEASES,
        {
            headers: {
                "User-Agent": "selene-vscode",
            }
        },
    ).then(body => {
        return JSON.parse(body) as GithubRelease
    }).catch((error) => {
        vscode.window.showErrorMessage(`Error downloading selene.\n${error.toString()}`)
        return Promise.reject(error)
    })

    return getLatestSeleneReleasePromise
}

export function platformIsSupported(): boolean {
    switch (os.platform()) {
        case "darwin":
        case "linux":
        case "win32":
            return true
        default:
            return false
    }
}

function getSeleneFilename(): string {
    switch (os.platform()) {
        case "win32":
            return "selene.exe"
        case "linux":
        case "darwin":
            return "selene"
        default:
            throw new Error("Platform not supported")
    }
}

function getSeleneFilenamePattern(): RegExp {
    switch (os.platform()) {
        case "win32":
            return /selene-[^-]+-windows.zip/
        case "linux":
            return /selene-[^-]+-linux.zip/
        case "darwin":
            return /selene-[^-]+-macos.zip/
        default:
            throw new Error("Platform not supported")
    }
}

async function fileExists(filename: vscode.Uri): Promise<boolean> {
    try {
        await vscode.workspace.fs.stat(filename);
        return true;
    } catch (err) {
        // If an error was thrown, the file was not found
        return false;
    }
}

export async function downloadSelene(directory: vscode.Uri) {
    vscode.window.showInformationMessage("Downloading Selene...")

    const filename = getSeleneFilename()
    const filenamePattern = getSeleneFilenamePattern()
    const release = await getLatestSeleneRelease()

    for (const asset of release.assets) {
        if (filenamePattern.test(asset.name)) {
            const file = fsWriteFileAtomic(vscode.Uri.joinPath(directory, filename).fsPath, {
                mode: 0o755,
            })

            return new Promise((resolve, reject) => {
                requestNative(asset.browser_download_url, {
                    headers: {
                        "User-Agent": "selene-vscode",
                    }
                })
                    .pipe(unzip.Parse())
                    .on("entry", (entry: unzip.Entry) => {
                        if (entry.path !== filename) {
                            entry.autodrain()
                            return
                        }

                        entry.pipe(file)
                            .on("finish", resolve)
                            .on("error", reject)
                    })
            })
        }
    }
}

export async function getSelenePath(storagePath: vscode.Uri): Promise<vscode.Uri | undefined> {
    const settingPath = vscode.workspace.getConfiguration("selene").get<string | null>("selenePath")
    if (settingPath) {
        return vscode.Uri.file(settingPath)
    }

    const downloadPath = vscode.Uri.joinPath(storagePath, getSeleneFilename());
    if (await fileExists(downloadPath)) {
        return downloadPath
    }
}

export async function ensureSeleneExists(storagePath: vscode.Uri) {
    const path = await getSelenePath(storagePath)

    if (path === undefined) {
        await vscode.workspace.fs.createDirectory(storagePath);
        return downloadSelene(storagePath)
    } else {
        if (!await fileExists(path)) {
            return Promise.reject("Path given for selene does not exist")
        }

        const version = (await selene.seleneCommand(storagePath, "--version", selene.Expectation.Stdout))?.trim()
        const release = await getLatestSeleneRelease()
        if (version !== `selene ${release.tag_name}`) {
            openUpdatePrompt(storagePath, release)
        }
    }
}

function openUpdatePrompt(directory: vscode.Uri, release: GithubRelease) {
    vscode.window.showInformationMessage(
        `There's an update available for selene: ${release.tag_name}`,
        "Install Update",
        "Later",
        "Release Notes",
    ).then(option => {
        switch (option) {
            case "Install Update":
                downloadSelene(directory).then(() => vscode.window.showInformationMessage("Update succeeded."))
                break
            case "Release Notes":
                vscode.env.openExternal(vscode.Uri.parse(release.html_url))
                openUpdatePrompt(directory, release)
                break
        }
    })
}
