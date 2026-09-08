import { nativeUnitBytes, parseNativeLayoutBytes } from "./native-layout.ts";
import {
    readArtifactFileTree,
    readSingleArtifactFile,
    type ArtifactFile,
} from "./release-manifest-artifacts.ts";
import { canonicalJson, sha256 } from "./release-manifest-core.ts";
import { parseSelectedApiBytes } from "./selected-contract-validator.ts";
import { parseSelectedConfigBytes } from "./selected-config.ts";
import { parseSelectedProtocolBytes } from "./selected-protocol.ts";
import { parseSchemaBytes } from "./schema-contract.ts";
import type { StagingInput } from "./native-staging.ts";
import { VerifiedContainerExportSnapshot } from "./container-export-validator.ts";
import { parseInventory } from "../scripts/distribution-web-inventory-policy.ts";
import type { BuildInputs } from "./release-manifest-types.ts";

const sourceToken = Symbol("verified native source");
const sourceStates = new WeakMap<VerifiedNativeSource, VerifiedNativeSourceState>();
let createVerifiedNativeSource: (value: {
    files: ArtifactFile[];
    digests: NativeSourceDigests;
    selection: unknown;
    buildInputs: BuildInputs;
}) => VerifiedNativeSource;
export class VerifiedNativeSource {
    private constructor(
        token: typeof sourceToken,
        value: VerifiedNativeSourceState,
    ) {
        if (token !== sourceToken) throw new Error("verified native source construction is internal");
        sourceStates.set(this, structuredClone(value));
        Object.freeze(this);
    }
    static {
        createVerifiedNativeSource = (value) =>
            new VerifiedNativeSource(sourceToken, value);
    }
}
Object.freeze(VerifiedNativeSource.prototype);
Object.freeze(VerifiedNativeSource);
export type NativeSourceDigests = {
    configDigest: string;
    nativeLayoutDigest: string;
    protocolArtifactDigest: string;
    apiDigest?: string;
    schemaDigest?: string;
};
export type VerifiedNativeSourceState = {
    files: ArtifactFile[];
    digests: NativeSourceDigests;
    selection: unknown;
    buildInputs: BuildInputs;
};
export function readVerifiedNativeSource(source: unknown): VerifiedNativeSourceState {
    if (
        !(source instanceof VerifiedNativeSource) ||
        Reflect.ownKeys(source).length !== 0
    )
        throw new Error("staging verified source capability is invalid");
    const state = sourceStates.get(source);
    if (!state) throw new Error("staging verified source capability is invalid");
    return structuredClone(state);
}
function verified(value: VerifiedNativeSourceState) {
    return createVerifiedNativeSource(value);
}
export async function fromPathStagingInput(
    input: StagingInput,
    server: boolean,
): Promise<VerifiedNativeSource> {
    const binary = await readArtifactFileTree(input.binaryRoot);
    const expected = server
        ? ["bin/rz-admin", "bin/rz-monitor"]
        : ["bin/rz-monitor-agent"];
    if (canonicalJson(binary.map((file) => file.entry.path)) !== canonicalJson(expected))
        throw new Error("staging binary root inventory differs from selection");

    const config = await readSingleArtifactFile(input.configRoot, "config.json");
    const native = await readSingleArtifactFile(input.nativeRoot, "native-layout.json");
    const protocol = await readSingleArtifactFile(input.protocolRoot, "protocol.json");
    parseSelectedConfigBytes(config.bytes, input.selection);
    const layout = parseNativeLayoutBytes(native.bytes, input.selection);
    parseSelectedProtocolBytes(protocol.bytes, input.selection);

    const contracts = [
        copy(config, "contracts/config/config.json"),
        copy(native, "contracts/native/native-layout.json"),
        copy(protocol, "contracts/protocol/protocol.json"),
    ];
    let apiDigest: string | undefined;
    let schemaDigest: string | undefined;
    if (server) {
        const api = await readSingleArtifactFile(required(input.apiRoot, "apiRoot"), "api.json");
        const schema = await readSingleArtifactFile(required(input.schemaRoot, "schemaRoot"), "schema.json");
        parseSelectedApiBytes(api.bytes, input.selection);
        await parseSchemaBytes(schema.bytes, input.selection);
        apiDigest = api.entry.sha256;
        schemaDigest = schema.entry.sha256;
        contracts.unshift(copy(api, "contracts/api/api.json"), copy(schema, "contracts/schema/schema.json"));
    }

    const units = Object.entries(nativeUnitBytes(input.selection))
        .map(([path, text]) => unit(path, text))
        .sort((left, right) => left.entry.path.localeCompare(right.entry.path));
    if (canonicalJson(units.map(unitIdentity)) !== canonicalJson(layout.layout.units))
        throw new Error("staging units differ from native layout");

    const web = server ? await readArtifactFileTree(required(input.webRoot, "webRoot")) : [];
    if (server && !web.length) throw new Error("staging Web root must not be empty");
    return verified({
        files: [...binary, ...contracts, ...units, ...web.map((file) => copy(file, `web/${file.entry.path}`))]
            .sort((left, right) => left.entry.path.localeCompare(right.entry.path)),
        digests: {
            configDigest: config.entry.sha256,
            nativeLayoutDigest: native.entry.sha256,
            protocolArtifactDigest: protocol.entry.sha256,
            ...(apiDigest ? { apiDigest, schemaDigest } : {}),
        },
        selection: input.selection,
        buildInputs: buildInputs(input),
    });
}

/** Maps only bytes retained by the unforgeable host-verified export snapshot. */
export function fromContainerSnapshot(
    snapshot: VerifiedContainerExportSnapshot,
): VerifiedNativeSource {
    if (!(snapshot instanceof VerifiedContainerExportSnapshot))
        throw new Error("container snapshot capability is invalid");
    const selection = snapshot.selection();
    const inventory = parseInventory(json(snapshot.artifact("release/web/inventory.json").bytes));
    const native = snapshot.artifact("release/contracts/native/native-layout.json").bytes;
    const layout = parseNativeLayoutBytes(native, selection);
    const binaries = ["release/server/bin/rz-admin", "release/server/bin/rz-monitor"];
    const contracts = [
        "api/api.json",
        "config/config.json",
        "native/native-layout.json",
        "protocol/protocol.json",
        "schema/schema.json",
    ];
    const files = [
        ...binaries.map((path) =>
            snapshotFile(snapshot, path, path.replace("release/server/", ""), "0755"),
        ),
        ...contracts.map((path) =>
            snapshotFile(snapshot, `release/contracts/${path}`, `contracts/${path}`, "0644"),
        ),
        ...Object.entries(nativeUnitBytes(selection)).map(([path, text]) => unit(path, text)),
        ...snapshot
            .artifacts("release/web/dist/")
            .map((artifact) => copiedBytes(artifact.bytes, `web/${artifact.entry.path.slice("release/web/dist/".length)}`, "0644")),
    ].sort((left, right) => comparePath(left.entry.path, right.entry.path));
    const units = files
        .filter((file) => file.entry.path.startsWith("systemd/"))
        .map(unitIdentity)
        .sort((left, right) => comparePath(left.path, right.path));
    if (canonicalJson(units) !== canonicalJson(layout.layout.units))
        throw new Error("container staging units differ from snapshot native layout");
    if (!inventory.selectedRoutes.length)
        throw new Error("container staging Web inventory has no selected routes");
    const get = (path: string) => requiredFile(files, path);
    return verified({
        files,
        digests: {
            apiDigest: get("contracts/api/api.json").entry.sha256,
            configDigest: get("contracts/config/config.json").entry.sha256,
            nativeLayoutDigest: sha256(native),
            protocolArtifactDigest: get("contracts/protocol/protocol.json").entry.sha256,
            schemaDigest: get("contracts/schema/schema.json").entry.sha256,
        },
        selection,
        buildInputs: {
            releaseVersion: snapshot.recordedProvenance().releaseVersion,
            sourceIdentity: snapshot.recordedProvenance().sourceIdentityInput,
            toolchain: snapshot.recordedProvenance().rustcVv,
            selectedRoutes: [...inventory.selectedRoutes].sort(),
        },
    });
}

function buildInputs(input: StagingInput): BuildInputs {
    return {
        releaseVersion: input.releaseVersion,
        sourceIdentity: input.sourceIdentity,
        toolchain: input.toolchain,
        selectedRoutes: [...input.selectedRoutes],
    };
}

function unit(path: string, text: string): ArtifactFile {
    const bytes = new TextEncoder().encode(text);
    return { entry: { path, type: "file", mode: "0644", size: bytes.length, sha256: sha256(bytes) }, bytes };
}
function snapshotFile(
    snapshot: VerifiedContainerExportSnapshot,
    source: string,
    path: string,
    mode: "0644" | "0755",
) {
    return copiedBytes(snapshot.artifact(source).bytes, path, mode);
}
function copiedBytes(bytes: Uint8Array, path: string, mode: "0644" | "0755"): ArtifactFile {
    return {
        entry: { path, type: "file", mode, size: bytes.byteLength, sha256: sha256(bytes) },
        bytes,
    };
}
function unitIdentity(file: ArtifactFile) { return { path: file.entry.path, sha256: file.entry.sha256 }; }
function copy(file: ArtifactFile, path: string): ArtifactFile { return { bytes: file.bytes, entry: { ...file.entry, path } }; }
function required(value: string | undefined, label: string) { if (!value) throw new Error(`${label} is required`); return value; }
function requiredFile(files: ArtifactFile[], path: string) {
    const file = files.find((candidate) => candidate.entry.path === path);
    if (!file) throw new Error(`container staging is missing ${path}`);
    return file;
}
function comparePath(left: string, right: string) { return left < right ? -1 : left > right ? 1 : 0; }
function json(bytes: Uint8Array): unknown {
    try {
        return JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(bytes));
    } catch {
        throw new Error("container staging Web inventory is invalid JSON");
    }
}
