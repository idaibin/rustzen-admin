import {
    lstat,
    mkdir,
    open,
    realpath,
    rename,
    rm,
    writeFile,
} from "node:fs/promises";
import { join, relative, resolve } from "node:path";
import { randomUUID } from "node:crypto";
import { nativeUnitBytes, parseNativeLayoutBytes } from "./native-layout.ts";
import {
    readArtifactFileTree,
    readSingleArtifactFile,
    type ArtifactFile,
} from "./release-manifest-artifacts.ts";
import {
    canonicalJson,
    deriveBuildId,
    sha256,
} from "./release-manifest-core.ts";
import { parseSelectedApiBytes } from "./selected-contract-validator.ts";
import { parseSelectedConfigBytes } from "./selected-config.ts";
import { parseSelectedProtocolBytes } from "./selected-protocol.ts";
import { resolveSelection } from "./resolver.ts";
import { parseSchemaBytes } from "./schema-contract.ts";
import type { BuildInputs, FileEntry } from "./release-manifest-types.ts";

type Roots = {
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
    const source = await sources(input, plan.artifactClass === "server");
    const buildId = deriveBuildId(input.selection, input, source.digests);
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
        await rename(temp, final);
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

async function sources(input: StagingInput, server: boolean) {
    const binary = await readArtifactFileTree(input.binaryRoot);
    const expected = server
        ? ["bin/rz-admin", "bin/rz-monitor"]
        : ["bin/rz-monitor-agent"];
    if (
        canonicalJson(binary.map((x) => x.entry.path)) !==
        canonicalJson(expected)
    )
        throw new Error("staging binary root inventory differs from selection");
    const config = await readSingleArtifactFile(
        input.configRoot,
        "config.json",
    );
    const native = await readSingleArtifactFile(
        input.nativeRoot,
        "native-layout.json",
    );
    const protocol = await readSingleArtifactFile(
        input.protocolRoot,
        "protocol.json",
    );
    parseSelectedConfigBytes(config.bytes, input.selection);
    const layout = parseNativeLayoutBytes(native.bytes, input.selection);
    parseSelectedProtocolBytes(protocol.bytes, input.selection);
    const contracts: ArtifactFile[] = [
        copy(config, "contracts/config/config.json"),
        copy(native, "contracts/native/native-layout.json"),
        copy(protocol, "contracts/protocol/protocol.json"),
    ];
    let apiDigest: string | undefined, schemaDigest: string | undefined;
    if (server) {
        const api = await readSingleArtifactFile(
            required(input.apiRoot, "apiRoot"),
            "api.json",
        );
        const schema = await readSingleArtifactFile(
            required(input.schemaRoot, "schemaRoot"),
            "schema.json",
        );
        parseSelectedApiBytes(api.bytes, input.selection);
        await parseSchemaBytes(schema.bytes, input.selection);
        apiDigest = api.entry.sha256;
        schemaDigest = schema.entry.sha256;
        contracts.unshift(
            copy(api, "contracts/api/api.json"),
            copy(schema, "contracts/schema/schema.json"),
        );
    }
    const units = Object.entries(nativeUnitBytes(input.selection))
        .map(([path, text]) => {
            const bytes = new TextEncoder().encode(text);
            return {
                entry: {
                    path,
                    type: "file" as const,
                    mode: "0644" as const,
                    size: bytes.length,
                    sha256: sha256(bytes),
                },
                bytes,
            };
        })
        .sort((a, b) => a.entry.path.localeCompare(b.entry.path));
    if (
        canonicalJson(
            units.map((x) => ({ path: x.entry.path, sha256: x.entry.sha256 })),
        ) !== canonicalJson(layout.layout.units)
    )
        throw new Error("staging units differ from native layout");
    const web = server
        ? await readArtifactFileTree(required(input.webRoot, "webRoot"))
        : [];
    if (server && !web.length)
        throw new Error("staging Web root must not be empty");
    const files = [
        ...binary,
        ...contracts,
        ...units,
        ...web.map((x) => copy(x, `web/${x.entry.path}`)),
    ].sort((a, b) => a.entry.path.localeCompare(b.entry.path));
    return {
        files,
        digests: {
            configDigest: config.entry.sha256,
            nativeLayoutDigest: native.entry.sha256,
            protocolArtifactDigest: protocol.entry.sha256,
            ...(apiDigest ? { apiDigest, schemaDigest } : {}),
        },
    };
}
function copy(file: ArtifactFile, path: string): ArtifactFile {
    return { bytes: file.bytes, entry: { ...file.entry, path } };
}
async function write(
    root: string,
    path: string,
    bytes: Uint8Array,
    mode: number,
) {
    const output = join(root, path);
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
