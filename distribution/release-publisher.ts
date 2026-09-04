import { constants } from "node:fs";
import { lstat, mkdir, open, realpath, rm } from "node:fs/promises";
import { dirname, join } from "node:path";
import { randomUUID } from "node:crypto";
import {
    createCanonicalArchive,
    readCanonicalArchive,
} from "./canonical-archive.ts";
import { readReleaseFiles } from "./release-publisher-files.ts";
import { atomicRenameNoReplace } from "./atomic-rename.ts";
import { canonicalJson, sha256 } from "./release-manifest-core.ts";
import {
    canonicalManifestBytes,
    parseReleaseManifest,
} from "./release-manifest-validator.ts";
import {
    releaseEnvelopePayload,
    signReleaseEnvelope,
    verifyReleaseEnvelope,
    type TrustedReleaseKey,
} from "./release-envelope.ts";
import type { CanonicalArchiveInput } from "./canonical-archive.ts";

export type ReleasePublicationInput = CanonicalArchiveInput & {
    privateKey: string;
    trusted: TrustedReleaseKey;
};
export type ReleasePublication = {
    root: string;
    archiveSha256: string;
    manifestSha256: string;
    envelopeSha256: string;
};
let beforePublishHook: (() => Promise<void> | void) | undefined;
export const setReleaseBeforePublishHookForTest = (
    hook?: () => Promise<void> | void,
) => {
    beforePublishHook = hook;
};

export async function publishSelectedRelease(
    input: ReleasePublicationInput,
): Promise<ReleasePublication> {
    const manifest = parseReleaseManifest(input.manifest, input.selection);
    if (manifest.releaseClass !== "production")
        throw new Error("production signing requires a production manifest");
    const archive = await createCanonicalArchive(input);
    const manifestBytes = canonicalManifestBytes(manifest, input.selection);
    const payload = releaseEnvelopePayload(
        manifest,
        input.trusted.keyId,
        archive,
        manifestBytes,
    );
    const envelope = signReleaseEnvelope(payload, input.privateKey);
    const parent = dirname(input.staging.root);
    const identity = await privateDirectory(parent);
    const final = join(parent, "release");
    const lock = join(parent, ".release.lock");
    const lockHandle = await open(lock, "wx", 0o600).catch(() => {
        throw new Error(
            "release publish is already in progress or output exists",
        );
    });
    const temp = join(parent, `.release-${randomUUID()}.tmp`);
    try {
        await absent(final);
        await mkdir(temp, { mode: 0o700 });
        await write(temp, "archive.tar", archive);
        await write(temp, "release-manifest.json", manifestBytes);
        await write(temp, "signature-envelope.json", envelope);
        await verifyReleaseDirectory(
            temp,
            input.selection,
            input.trusted,
            payload,
        );
        await beforePublishHook?.();
        await verifyIdentity(identity);
        await absent(final);
        await syncDirectory(temp);
        atomicRenameNoReplace(temp, final);
        await syncDirectory(parent);
        return {
            root: final,
            archiveSha256: sha256(archive),
            manifestSha256: sha256(manifestBytes),
            envelopeSha256: sha256(envelope),
        };
    } finally {
        await lockHandle.close();
        await rm(temp, { recursive: true, force: true });
        await rm(lock, { force: true });
    }
}

export async function verifyReleaseDirectory(
    root: string,
    selection: unknown,
    trusted: TrustedReleaseKey,
    expected?: ReturnType<typeof releaseEnvelopePayload>,
): Promise<ReleasePublication> {
    const files = await readReleaseFiles(root);
    const required = [
        "archive.tar",
        "release-manifest.json",
        "signature-envelope.json",
    ];
    if (canonicalJson([...files.keys()].sort()) !== canonicalJson(required))
        throw new Error("release directory inventory is invalid");
    const archive = requiredFile(files, "archive.tar");
    const manifestBytes = requiredFile(files, "release-manifest.json");
    const envelope = requiredFile(files, "signature-envelope.json");
    const read = readCanonicalArchive(archive, selection);
    const manifest = parseManifestBytes(manifestBytes, selection);
    if (canonicalJson(read.manifest) !== canonicalJson(manifest))
        throw new Error("release archive and manifest differ");
    const payload = verifyReleaseEnvelope(envelope, trusted);
    const derived = releaseEnvelopePayload(
        manifest,
        trusted.keyId,
        archive,
        manifestBytes,
    );
    if (canonicalJson(payload) !== canonicalJson(derived))
        throw new Error("release envelope tuple differs from artifacts");
    if (expected && canonicalJson(payload) !== canonicalJson(expected))
        throw new Error("release reread differs from signed tuple");
    return {
        root,
        archiveSha256: sha256(archive),
        manifestSha256: sha256(manifestBytes),
        envelopeSha256: sha256(envelope),
    };
}
async function write(root: string, name: string, bytes: Uint8Array) {
    const path = join(root, name);
    const handle = await open(path, "wx", 0o600);
    try {
        await handle.writeFile(bytes);
        await handle.chmod(0o644);
        await handle.sync();
    } finally {
        await handle.close();
    }
}
async function syncDirectory(path: string) {
    const handle = await open(path, constants.O_RDONLY);
    try {
        await handle.sync();
    } finally {
        await handle.close();
    }
}
function parseManifestBytes(bytes: Uint8Array, selection: unknown) {
    const source = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
    let value: unknown;
    try {
        value = JSON.parse(source);
    } catch {
        throw new Error("release manifest JSON is invalid");
    }
    const manifest = parseReleaseManifest(value, selection);
    if (source !== canonicalJson(manifest))
        throw new Error("release manifest bytes are not canonical");
    return manifest;
}
function requiredFile(
    files: Map<string, Uint8Array>,
    path: string,
): Uint8Array {
    const value = files.get(path);
    if (!value) throw new Error(`release file is missing: ${path}`);
    return value;
}
async function absent(path: string) {
    try {
        await lstat(path);
        throw new Error("release final output already exists");
    } catch (error: unknown) {
        if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
    }
}
type DirectoryIdentity = {
    path: string;
    dev: number;
    ino: number;
    mode: number;
    uid: number;
    real: string;
};
async function privateDirectory(path: string): Promise<DirectoryIdentity> {
    const stat = await lstat(path);
    if (
        !stat.isDirectory() ||
        stat.isSymbolicLink() ||
        stat.uid !== process.getuid() ||
        (stat.mode & 0o077) !== 0
    )
        throw new Error(
            "release parent must be a private owned non-link directory",
        );
    return {
        path,
        dev: stat.dev,
        ino: stat.ino,
        mode: stat.mode,
        uid: stat.uid,
        real: await realpath(path),
    };
}
async function verifyIdentity(before: DirectoryIdentity) {
    const after = await privateDirectory(before.path);
    if (
        after.dev !== before.dev ||
        after.ino !== before.ino ||
        after.mode !== before.mode ||
        after.uid !== before.uid ||
        after.real !== before.real
    )
        throw new Error("release parent changed before publish");
}
