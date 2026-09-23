import { constants } from "node:fs";
import { lstat, open, readdir, realpath } from "node:fs/promises";
import { join } from "node:path";

let afterOpenHook: ((path: string) => Promise<void> | void) | undefined;
export const setReleaseReadHookForTest = (
    hook?: (path: string) => Promise<void> | void,
) => {
    afterOpenHook = hook;
};
export async function readReleaseFiles(
    root: string,
): Promise<Map<string, Uint8Array>> {
    const rootIdentity = await directory(root);
    const result = new Map<string, Uint8Array>();
    try {
        for (const entry of await readdir(root, { withFileTypes: true })) {
            if (!entry.isFile() || entry.isSymbolicLink())
                throw new Error("release directory contains a non-file");
            const path = join(root, entry.name);
            const before = await lstat(path, { bigint: true });
            if (before.isSymbolicLink() || (before.mode & 0o777n) !== 0o644n)
                throw new Error("release file mode or identity is invalid");
            const handle = await open(
                path,
                constants.O_RDONLY | constants.O_NOFOLLOW,
            );
            try {
                const opened = await handle.stat({ bigint: true });
                await afterOpenHook?.(path);
                const bytes = await handle.readFile();
                const after = await handle.stat({ bigint: true });
                const pathAfter = await lstat(path, { bigint: true });
                if (
                    !same(identity(path, before), identity(path, opened)) ||
                    !same(identity(path, opened), identity(path, after)) ||
                    !same(identity(path, before), identity(path, pathAfter))
                )
                    throw new Error("release file changed while being read");
                result.set(entry.name, bytes);
            } finally {
                await handle.close();
            }
        }
        return result;
    } finally {
        await verifyDirectory(rootIdentity);
    }
}
type Identity = {
    path: string;
    dev: bigint;
    ino: bigint;
    mode: bigint;
    mtimeNs: bigint;
    ctimeNs: bigint;
    real?: string;
};
async function directory(path: string): Promise<Identity> {
    const stat = await lstat(path, { bigint: true });
    if (stat.isSymbolicLink() || !stat.isDirectory())
        throw new Error("release root must be a non-link directory");
    return { ...identity(path, stat), real: await realpath(path) };
}
async function verifyDirectory(before: Identity) {
    const after = await directory(before.path);
    if (!same(before, after) || before.real !== after.real)
        throw new Error("release directory changed while being read");
}
function identity(path: string, stat: any): Identity {
    return {
        path,
        dev: stat.dev,
        ino: stat.ino,
        mode: stat.mode & 0o777n,
        mtimeNs: stat.mtimeNs,
        ctimeNs: stat.ctimeNs,
    };
}
function same(left: Identity, right: Identity) {
    return (
        left.dev === right.dev &&
        left.ino === right.ino &&
        left.mode === right.mode &&
        left.mtimeNs === right.mtimeNs &&
        left.ctimeNs === right.ctimeNs
    );
}
