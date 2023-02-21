import * as semver from "semver"

export type Capabilities = {
    validateConfig?: {
        version: string
    }
}

export function capability(
    capabilities: Capabilities,
    key: keyof Capabilities,
    supportedVersion: string,
): Capabilities[typeof key] {
    const capability = capabilities[key]

    if (capability === undefined) {
        return undefined
    }

    if (!semver.satisfies(capability.version, supportedVersion)) {
        return undefined
    }

    return capability
}
