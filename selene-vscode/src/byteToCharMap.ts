import * as vscode from "vscode"

export function byteToCharMap(
    document: vscode.TextDocument,
    byteOffsets: Set<number>,
): Map<number, number> {
    const text = document.getText()
    const byteOffsetMap = new Map<number, number>()
    let currentOffset = 0

    // Iterate through each character in the string
    for (let charOffset = 0; charOffset < text.length; charOffset++) {
        // Calculate the current byte offset we have reached so far
        currentOffset += Buffer.byteLength(text[charOffset], "utf-8")
        for (const offset of byteOffsets) {
            if (currentOffset >= offset) {
                byteOffsetMap.set(offset, charOffset + 1)
                byteOffsets.delete(offset)

                if (byteOffsets.size === 0) {
                    return byteOffsetMap
                }
            }
        }
    }

    return byteOffsetMap
}
