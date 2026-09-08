import { resolve } from "node:path";

export async function readWorkspaceVersion(root: string): Promise<string> {
    const parsed = Bun.TOML.parse(await Bun.file(resolve(root, "Cargo.toml")).text()) as {
        workspace?: { package?: { version?: unknown } };
    };
    const version = parsed.workspace?.package?.version;
    if (
        typeof version !== "string" ||
        !version ||
        version.length > 64 ||
        /[\r\n]/.test(version)
    )
        throw new Error("workspace releaseVersion is unavailable");
    return version;
}
