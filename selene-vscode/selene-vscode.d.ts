declare module "fs-write-stream-atomic" {
    function fsWriteFileAtomic(
        path: string,
        options?: { mode: number },
    ): NodeJS.WritableStream

    export = fsWriteFileAtomic
}
