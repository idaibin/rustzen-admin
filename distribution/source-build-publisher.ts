import { constants, type BigIntStats } from "node:fs";
import { basename, dirname, isAbsolute, join, relative, sep } from "node:path";
import { lstat, mkdir, open, realpath, readdir, rm } from "node:fs/promises";
import { randomUUID } from "node:crypto";
import { atomicRenameNoReplace } from "./atomic-rename.ts";
import {
    parseSourceBuildCertificate,
    sourceBuildCertificateBytes,
    type SourceBuildCertificate,
} from "./source-build-certificate.ts";
import {
    readIssuedSourceBuildCertificate,
    type IssuedSourceBuildCertificate,
} from "./source-build-issuer.ts";

const name = "source-build-manifest.json";
const maxCertificateBytes = 64 * 1024;
let beforeRenameHook: ((root: string) => Promise<void> | void) | undefined;
let directorySyncHook: ((path: string) => Promise<void> | void) | undefined;

/** Test-only seam immediately before the final publication checks. */
export const setSourceBuildBeforeRenameHookForTest = (
    hook?: (root: string) => Promise<void> | void,
) => { beforeRenameHook = hook; };
export const setSourceBuildDirectorySyncHookForTest = (
    hook?: (path: string) => Promise<void> | void,
) => { directorySyncHook = hook; };

/** Publishes one certificate beside a release through a private atomic boundary. */
export async function publishSourceBuildCertificate(input: {
    issued: IssuedSourceBuildCertificate;
}): Promise<{ path: string; certificate: SourceBuildCertificate }> {
    if (canonicalKeys(input) !== '["issued"]')
        throw new Error("certificate publication input fields are invalid");
    const issued = readIssuedSourceBuildCertificate(input.issued);
    const parent = dirname(issued.releaseRoot);
    const realRoot = issued.trustedRoot;
    const identity = await privateDirectory(parent);
    assertContained(realRoot, identity.real);
    const certificateRoot = join(parent, "source-build-certificate");
    let createdRoot = false;
    await mkdir(certificateRoot, { mode: 0o700 })
        .then(() => { createdRoot = true; })
        .catch((error: NodeJS.ErrnoException) => {
            if (error.code !== "EEXIST") throw error;
        });
    const certificateIdentity = await privateDirectory(certificateRoot);
    assertContained(realRoot, certificateIdentity.real);
    await unchanged(identity);
    if (createdRoot) await syncDirectory(parent);
    const final = join(certificateRoot, name);
    const lock = join(certificateRoot, ".source-build.lock");
    const certificate = parseSourceBuildCertificate(issued.certificate, issued.selection);
    const bytes = sourceBuildCertificateBytes(certificate, issued.selection);
    const handle = await open(lock, "wx", 0o600).catch(() => {
        throw new Error("certificate publication is in progress or output exists");
    });
    const temp = join(certificateRoot, `.${name}-${randomUUID()}.tmp`);
    let handleClosed = false;
    let cleaned = false;
    try {
        await exactInventory(certificateRoot, [basename(lock)]);
        await write(temp, bytes);
        await reread(temp, issued.selection, certificate);
        await beforeRenameHook?.(certificateRoot);
        await unchanged(identity);
        await unchanged(certificateIdentity);
        await exactInventory(certificateRoot, [basename(lock), basename(temp)]);
        await syncDirectory(certificateRoot);
        atomicRenameNoReplace(temp, final);
        await syncDirectory(certificateRoot);
        const rereadCertificate = await reread(final, issued.selection, certificate);
        await handle.close();
        handleClosed = true;
        await rm(lock, { force: true });
        await syncDirectory(certificateRoot);
        cleaned = true;
        assertContained(realRoot, await realpath(certificateRoot));
        await exactInventory(certificateRoot, [name]);
        return { path: final, certificate: rereadCertificate };
    } finally {
        if (!handleClosed) await handle.close();
        await rm(temp, { force: true });
        if (!cleaned) {
            await rm(lock, { force: true });
            await syncDirectory(certificateRoot).catch(() => undefined);
        }
    }
}

async function write(path: string, bytes: Uint8Array) {
    const handle = await open(path, "wx", 0o600);
    try { await handle.writeFile(bytes); await handle.chmod(0o644); await handle.sync(); }
    finally { await handle.close(); }
}
async function reread(path: string, selection: unknown, expected: SourceBuildCertificate) {
    const before = await lstat(path, { bigint: true });
    if (!before.isFile() || before.isSymbolicLink() || (before.mode & 0o777n) !== 0o644n)
        throw new Error("certificate file mode or identity is invalid");
    const handle = await open(path, constants.O_RDONLY | constants.O_NOFOLLOW);
    try {
        const opened = await handle.stat({ bigint: true });
        const bytes = await boundedRead(handle);
        const after = await handle.stat({ bigint: true });
        const pathAfter = await lstat(path, { bigint: true });
        if (!same(before, opened) || !same(before, after) || !same(before, pathAfter))
            throw new Error("certificate changed while being read");
        const value = parseSourceBuildCertificate(JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(bytes)), selection);
        if (new TextDecoder().decode(bytes) !== new TextDecoder().decode(sourceBuildCertificateBytes(value, selection)) || JSON.stringify(value) !== JSON.stringify(expected))
            throw new Error("certificate reread differs from published value");
        return value;
    } finally { await handle.close(); }
}
async function privateDirectory(path: string) {
    const stat = await lstat(path, { bigint: true });
    if (!stat.isDirectory() || stat.isSymbolicLink() || stat.uid !== BigInt(process.getuid()) || (stat.mode & 0o077n) !== 0n)
        throw new Error("certificate parent must be private owned non-link directory");
    return { stat, real: await realpath(path), path };
}
async function unchanged(before: Awaited<ReturnType<typeof privateDirectory>>) {
    const after = await privateDirectory(before.path);
    if (!sameDirectory(before.stat, after.stat) || before.real !== after.real)
        throw new Error("certificate parent changed before publication");
}
async function exactInventory(root: string, expected: string[]) {
    const entries = await readdir(root, { withFileTypes: true });
    const names = entries.map((entry) => entry.name).sort();
    if (!expected.includes(name) && names.includes(name))
        throw new Error("certificate final output already exists");
    if (
        JSON.stringify(names) !== JSON.stringify([...expected].sort()) ||
        entries.some((entry) => !entry.isFile() || entry.isSymbolicLink())
    ) throw new Error("certificate directory inventory is invalid");
}
async function syncDirectory(path: string) {
    const handle = await open(path, constants.O_RDONLY);
    try { await handle.sync(); } finally { await handle.close(); }
    await directorySyncHook?.(path);
}
async function boundedRead(handle: Awaited<ReturnType<typeof open>>) {
    const bytes = Buffer.allocUnsafe(maxCertificateBytes + 1);
    let offset = 0;
    while (offset < bytes.length) {
        const read = await handle.read(bytes, offset, bytes.length - offset, null);
        if (!read.bytesRead) return bytes.subarray(0, offset);
        offset += read.bytesRead;
    }
    throw new Error("certificate exceeds size limit");
}
function same(left: BigIntStats, right: BigIntStats) { return left.dev === right.dev && left.ino === right.ino && left.mode === right.mode && left.mtimeNs === right.mtimeNs && left.ctimeNs === right.ctimeNs; }
function sameDirectory(left: BigIntStats, right: BigIntStats) { return left.dev === right.dev && left.ino === right.ino && left.mode === right.mode && left.uid === right.uid; }
function assertContained(root: string, path: string) {
    const value = relative(root, path);
    if (value === ".." || value.startsWith(`..${sep}`) || isAbsolute(value))
        throw new Error("certificate parent escaped trusted root");
}
function canonicalKeys(value: object) {
    return JSON.stringify(Object.keys(value).sort());
}
