import {
    lstat,
    mkdir,
    open,
    realpath,
    rm,
    writeFile,
} from "node:fs/promises";
import { join, relative, resolve } from "node:path";
import { randomUUID } from "node:crypto";
import { readArtifactFileTree, type ArtifactFile } from "./release-manifest-artifacts.ts";
import {
    canonicalJson,
    deriveBuildId,
    sha256,
} from "./release-manifest-core.ts";
import { resolveSelection } from "./resolver.ts";
import type { BuildInputs, FileEntry } from "./release-manifest-types.ts";
import { fromPathStagingInput } from "./native-staging-source.ts";
import {
    canonicalStagingPath,
    verifyPublishedSource,
} from "./native-staging-validation.ts";

export type Roots = {
    binaryRoot: string;
    webRoot?: string;
    apiRoot?: string;
    schemaRoot?: string;
    configRoot: string;
    nativeRoot: string;
    protocolRoot: string;
};
export type StagingInput = Roots &
    BuildInputs & {
        selection: unknown;
        outputParent: string;
        trustedRoot: string;
    };
export type StagingResult = {
    root: string;
    files: FileEntry[];
    sha256: string;
    buildId: string;
    target: string;
    artifactClass: "server" | "node-agent";
};
import { VerifiedNativeSource } from "./native-staging-source.ts";
import { atomicRenameNoReplace } from "./atomic-rename.ts";
let beforePublishHook: (() => Promise<void> | void) | undefined;
/** Test-only seam; production never installs it. */
export const setNativeStagingBeforePublishHookForTest = (
    hook?: () => Promise<void> | void,
) => {
    beforePublishHook = hook;
};

export async function produceNativeStaging(
    input: StagingInput,
): Promise<StagingResult> {
    const plan = resolveSelection(input.selection);
    const source = await fromPathStagingInput(input, plan.artifactClass === "server");
    return publishNativeStagingBytes({
        outputParent: input.outputParent,
        trustedRoot: input.trustedRoot,
        source,
    });
}

/** Publishes already-verified bytes without reopening their producer roots. */
export async function publishNativeStagingBytes(input: {
    outputParent: string;
    trustedRoot: string;
    source: VerifiedNativeSource;
}): Promise<StagingResult> {
    if (canonicalJson(Object.keys(input).sort()) !== canonicalJson(["outputParent", "source", "trustedRoot"]))
        throw new Error("staging publisher inputs are invalid");
    const source = verifyPublishedSource(input.source);
    const plan = resolveSelection(source.selection);
    const buildId = deriveBuildId(
        source.selection,
        source.buildInputs,
        source.digests,
    );
    const identities = await outputBase(input.outputParent, input.trustedRoot);
    const privateRoot = join(input.outputParent, ".native-staging");
    identities.push(await privateDirectory(privateRoot, true));
    const final = join(
        privateRoot,
        buildId,
        plan.target,
        plan.artifactClass,
        "payload",
    );
    const lock = join(
        privateRoot,
        `.${buildId}-${plan.target}-${plan.artifactClass}.lock`,
    );
    const lockHandle = await open(lock, "wx", 0o600).catch(() => {
        throw new Error(
            "staging publish is already in progress or output exists",
        );
    });
    const temp = join(privateRoot, `.${buildId}-${randomUUID()}.tmp`);
    try {
        await rejectExisting(final);
        await mkdir(temp, { mode: 0o700 });
        for (const file of source.files)
            await write(
                temp,
                file.entry.path,
                file.bytes,
                file.entry.mode === "0755" ? 0o755 : 0o644,
            );
        const reread = await readArtifactFileTree(temp);
        if (
            canonicalJson(reread.map((x) => x.entry)) !==
            canonicalJson(source.files.map((x) => x.entry))
        )
            throw new Error("staged payload differs from verified bytes");
        identities.push(await privateChild(privateRoot, buildId));
        identities.push(
            await privateChild(join(privateRoot, buildId), plan.target),
        );
        identities.push(
            await privateChild(
                join(privateRoot, buildId, plan.target),
                plan.artifactClass,
            ),
        );
        await beforePublishHook?.();
        await verifyIdentities(identities);
        await rejectExisting(final);
        atomicRenameNoReplace(temp, final);
        return {
            root: final,
            files: source.files.map((x) => x.entry),
            sha256: sha256(canonicalJson(source.files.map((x) => x.entry))),
            buildId,
            target: plan.target,
            artifactClass: plan.artifactClass,
        };
    } finally {
        await lockHandle.close();
        await rm(temp, { recursive: true, force: true });
        await rm(lock, { force: true });
    }
}

async function write(
    root: string,
    path: string,
    bytes: Uint8Array,
    mode: number,
) {
    if (!canonicalStagingPath(path)) throw new Error("staging output path is invalid");
    const output = join(root, path);
    if (relative(root, output).startsWith("..")) throw new Error("staging output path escapes temporary root");
    await mkdir(join(output, ".."), { recursive: true, mode: 0o700 });
    await writeFile(output, bytes, { flag: "wx", mode });
}
async function rejectExisting(path: string) {
    try {
        await lstat(path);
        throw new Error("staging final output already exists");
    } catch (error: unknown) {
        if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
    }
}
async function privateChild(
    parent: string,
    child: string,
): Promise<DirectoryIdentity> {
    const path = join(parent, child);
    try {
        await mkdir(path, { mode: 0o700 });
    } catch (error: unknown) {
        if ((error as NodeJS.ErrnoException).code !== "EEXIST") throw error;
    }
    return privateDirectory(path, false);
}
async function privateDirectory(
    path: string,
    create: boolean,
): Promise<DirectoryIdentity> {
    if (create) await mkdir(path, { recursive: true, mode: 0o700 });
    const stat = await lstat(path);
    if (
        !stat.isDirectory() ||
        stat.isSymbolicLink() ||
        stat.uid !== process.getuid() ||
        (stat.mode & 0o077) !== 0
    )
        throw new Error(
            "staging output directory must be private owned non-link directory",
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
type DirectoryIdentity = {
    path: string;
    dev: number;
    ino: number;
    mode: number;
    uid: number;
    real: string;
};
async function outputBase(
    path: string,
    trustedRoot: string,
): Promise<DirectoryIdentity[]> {
    const root = resolve(trustedRoot),
        base = resolve(path),
        rel = relative(root, base);
    if (rel === ".." || rel.startsWith("../") || rel.startsWith("/"))
        throw new Error("staging output base must be beneath trusted root");
    const result: DirectoryIdentity[] = [];
    let current = root;
    for (const part of ["", ...rel.split("/").filter(Boolean)]) {
        if (part) {
            current = join(current, part);
            try {
                await mkdir(current, { mode: 0o755 });
            } catch (error: unknown) {
                if ((error as NodeJS.ErrnoException).code !== "EEXIST")
                    throw error;
            }
        }
        const stat = await lstat(current),
            real = await realpath(current);
        if (
            !stat.isDirectory() ||
            stat.isSymbolicLink() ||
            stat.uid !== process.getuid() ||
            (stat.mode & 0o022) !== 0
        )
            throw new Error(
                "staging output base must be owned non-link and non-writable by others",
            );
        result.push({
            path: current,
            dev: stat.dev,
            ino: stat.ino,
            mode: stat.mode,
            uid: stat.uid,
            real,
        });
    }
    return result;
}
async function verifyIdentities(entries: DirectoryIdentity[]) {
    for (const before of entries) {
        const after = await lstat(before.path),
            real = await realpath(before.path);
        if (
            after.dev !== before.dev ||
            after.ino !== before.ino ||
            after.mode !== before.mode ||
            after.uid !== before.uid ||
            real !== before.real
        )
            throw new Error("staging output directory changed before publish");
    }
}
function required(value: string | undefined, label: string) {
    if (!value) throw new Error(`${label} is required`);
    return value;
}
