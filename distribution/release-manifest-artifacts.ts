import { constants } from "node:fs";
import { lstat, open, readdir, realpath } from "node:fs/promises";
import { join, relative, resolve, sep } from "node:path";
import { filesDigest, sha256 } from "./release-manifest-core.ts";
import type { Digest, FileEntry } from "./release-manifest-types.ts";
type Identity = {
    dev: bigint;
    ino: bigint;
    mode: bigint;
    size?: bigint;
    mtimeNs: bigint;
    ctimeNs: bigint;
};
type ScannedFile = { entry: FileEntry; bytes: Uint8Array };
let beforeDirectoryHook: ((path: string) => Promise<void> | void) | undefined;
export const setArtifactDirectoryHookForTest = (
    hook?: (path: string) => Promise<void> | void,
) => {
    beforeDirectoryHook = hook;
};
let beforeOpenHook: ((path: string) => Promise<void> | void) | undefined;
let afterOpenHook: ((path: string) => Promise<void> | void) | undefined;
export const setArtifactAfterOpenHookForTest = (
    hook?: (path: string) => Promise<void> | void,
) => {
    afterOpenHook = hook;
};
/** Test-only race seam; production leaves it undefined. */
export const setArtifactReadHookForTest = (
    hook?: (path: string) => Promise<void> | void,
) => {
    beforeOpenHook = hook;
};
export async function readArtifactFiles(root: string): Promise<FileEntry[]> {
    return (await scanArtifactFiles(root)).map((file) => file.entry);
}
export async function readSingleArtifactFile(
    root: string,
    expectedPath: string,
): Promise<ScannedFile> {
    const files = await scanArtifactFiles(root);
    if (files.length !== 1 || files[0].entry.path !== expectedPath)
        throw new Error(`artifact must contain exactly ${expectedPath}`);
    return files[0];
}
async function scanArtifactFiles(root: string): Promise<ScannedFile[]> {
    const base = resolve(root);
    const parents = await directoryIdentities(base);
    try {
        return (await walk(base, base, parents)).sort((a, b) =>
            a.entry.path < b.entry.path
                ? -1
                : a.entry.path > b.entry.path
                  ? 1
                  : 0,
        );
    } finally {
        await verifyDirectories(parents);
    }
}
export async function readWebDigest(root: string): Promise<Digest> {
    return {
        sha256: filesDigest(await readArtifactFiles(root)),
        source: "selected-web-files",
    };
}
async function walk(
    root: string,
    directory: string,
    parents: Map<string, Identity & { real: string }>,
): Promise<ScannedFile[]> {
    const result: ScannedFile[] = [];
    await beforeDirectoryHook?.(directory);
    await verifyDirectory(directory, parents);
    for (const entry of await readdir(directory, { withFileTypes: true })) {
        const path = join(directory, entry.name);
        const before = await lstat(path, { bigint: true });
        if (before.isSymbolicLink())
            throw new Error(`artifact contains symlink: ${path}`);
        if (entry.isDirectory()) {
            parents.set(path, {
                ...identity(before),
                real: await realpath(path),
            });
            result.push(...(await walk(root, path, parents)));
            continue;
        }
        if (!before.isFile())
            throw new Error(`artifact contains non-file: ${path}`);
        await beforeOpenHook?.(path);
        await verifyDirectory(directory, parents);
        const handle = await open(
            path,
            constants.O_RDONLY | constants.O_NOFOLLOW,
        );
        try {
            await afterOpenHook?.(path);
            const opened = await handle.stat({ bigint: true });
            const bytes = await handle.readFile();
            const after = await handle.stat({ bigint: true });
            const pathAfter = await lstat(path, { bigint: true });
            if (
                !same(before, opened) ||
                !same(opened, after) ||
                !same(before, pathAfter)
            )
                throw new Error(`artifact changed while being read: ${path}`);
            const mode = opened.mode & 0o777n;
            if (mode !== 0o644n && mode !== 0o755n)
                throw new Error(`artifact file has invalid mode: ${path}`);
            result.push({
                entry: {
                    path: checkedPath(
                        relative(root, path).split(sep).join("/"),
                    ),
                    type: "file",
                    mode: mode === 0o755n ? "0755" : "0644",
                    size: Number(opened.size),
                    sha256: sha256(bytes),
                },
                bytes,
            });
        } finally {
            await handle.close();
        }
    }
    return result;
}
async function directoryIdentities(
    root: string,
): Promise<Map<string, Identity & { real: string }>> {
    const stat = await lstat(root, { bigint: true });
    if (stat.isSymbolicLink() || !stat.isDirectory())
        throw new Error("artifact root must be a non-link directory");
    return new Map([[root, { ...identity(stat), real: await realpath(root) }]]);
}
async function verifyDirectory(
    path: string,
    entries: Map<string, Identity & { real: string }>,
) {
    const before = entries.get(path);
    if (!before) return;
    const after = await lstat(path, { bigint: true });
    if (
        after.isSymbolicLink() ||
        !same(before, after) ||
        before.real !== (await realpath(path))
    )
        throw new Error(`artifact directory changed while being read: ${path}`);
}
async function verifyDirectories(
    entries: Map<string, Identity & { real: string }>,
) {
    for (const [path, before] of entries) {
        const after = await lstat(path, { bigint: true });
        if (
            after.isSymbolicLink() ||
            !same(before, after) ||
            before.real !== (await realpath(path))
        )
            throw new Error(
                `artifact directory changed while being read: ${path}`,
            );
    }
}
function identity(s: any): Identity {
    return {
        dev: s.dev,
        ino: s.ino,
        mode: s.mode & 0o777n,
        size: s.size,
        mtimeNs: s.mtimeNs,
        ctimeNs: s.ctimeNs,
    };
}
function same(a: any, b: any) {
    const x = identity(a),
        y = identity(b);
    return (
        x.dev === y.dev &&
        x.ino === y.ino &&
        x.mode === y.mode &&
        x.size === y.size &&
        x.mtimeNs === y.mtimeNs &&
        x.ctimeNs === y.ctimeNs
    );
}
function checkedPath(path: string): string {
    if (
        path.includes("\\") ||
        path.includes("\0") ||
        path.startsWith("/") ||
        [
            "manifest.json",
            "release-manifest.json",
            "signature.json",
            "envelope.json",
            "signature-envelope.json",
        ].includes(path) ||
        path.split("/").some((x) => !x || x === "." || x === "..")
    )
        throw new Error("file path is not canonical POSIX");
    return path;
}
