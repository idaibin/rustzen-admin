import { resolveSelection } from "./resolver.ts";
import {
    canonicalJson,
    nonempty,
    sha256,
    sortedStrings,
    validHash,
} from "./release-manifest-core.ts";
import type {
    AgentManifest,
    BinaryDigest,
    Digest,
    DigestSource,
    FileEntry,
    ManifestBase,
    ReleaseManifest,
    ServerManifest,
} from "./release-manifest-types.ts";
type Plan = ReturnType<typeof resolveSelection>;

const baseKeys = [
    "manifestVersion",
    "releaseClass",
    "releaseVersion",
    "target",
    "artifactClass",
    "preset",
    "capabilities",
    "services",
    "compositionId",
    "selectionDigest",
    "buildId",
    "sourceIdentity",
    "configDigest",
    "nativeLayoutDigest",
    "protocolArtifactDigest",
    "configOwners",
    "binaryDigests",
    "files",
    "agentProtocolContractId",
];
const serverKeys = [
    ...baseKeys,
    "apiDigest",
    "schemaFingerprints",
    "dataContractIds",
    "webDigest",
];

export function parseReleaseManifest(
    value: unknown,
    expectedSelection: unknown,
): ReleaseManifest {
    const record = object(value, "manifest");
    const artifactClass = string(record.artifactClass, "artifactClass");
    onlyKeys(
        record,
        artifactClass === "server"
            ? serverKeys
            : artifactClass === "node-agent"
              ? baseKeys
              : [],
    );
    const base = parseBase(record, artifactClass);
    const manifest: ReleaseManifest =
        artifactClass === "server"
            ? {
                  ...base,
                  artifactClass,
                  schemaFingerprints: hashMap(
                      record.schemaFingerprints,
                      "schemaFingerprints",
                  ),
                  dataContractIds: hashMap(
                      record.dataContractIds,
                      "dataContractIds",
                  ),
                  webDigest: digestRecord(
                      record.webDigest,
                      "selected-web-files",
                      "webDigest",
                  ),
                  apiDigest: validHash(string(record.apiDigest, "apiDigest")),
              }
            : {
                  ...base,
                  artifactClass: "node-agent",
                  agentProtocolContractId: validHash(
                      required(
                          record.agentProtocolContractId,
                          "agentProtocolContractId",
                      ),
                  ),
              };
    validateClass(manifest);
    validatePlan(manifest, resolveSelection(expectedSelection));
    return manifest;
}
export function validateServerAgentPair(
    server: ServerManifest,
    agent: AgentManifest,
): void {
    if (server.agentProtocolContractId !== agent.agentProtocolContractId)
        throw new Error("server-Agent protocol IDs do not match");
}

function expectedBinaries(plan: Plan): string[] {
    return plan.artifactClass === "node-agent"
        ? ["bin/rz-monitor-agent"]
        : plan.preset === "monitor"
          ? ["bin/rz-admin", "bin/rz-monitor"]
          : (() => {
                throw new Error(
                    "producer supports only monitor server or node-agent",
                );
            })();
}
function parseBase(
    record: Record<string, unknown>,
    artifactClass: string,
): ManifestBase {
    if (record.manifestVersion !== 1)
        throw new Error("manifestVersion must be 1");
    const base: ManifestBase = {
        manifestVersion: 1,
        releaseClass: string(
            record.releaseClass,
            "releaseClass",
        ) as ManifestBase["releaseClass"],
        releaseVersion: nonempty(record.releaseVersion, "releaseVersion"),
        target: nonempty(record.target, "target"),
        artifactClass: artifactClass as ManifestBase["artifactClass"],
        preset: nonempty(record.preset, "preset"),
        capabilities: sortedStrings(record.capabilities, "capabilities"),
        services: sortedStrings(record.services, "services"),
        compositionId: validHash(string(record.compositionId, "compositionId")),
        selectionDigest: digestRecord(
            record.selectionDigest,
            "resolved-selection",
            "selectionDigest",
        ),
        buildId: validHash(string(record.buildId, "buildId")),
        sourceIdentity: nonempty(record.sourceIdentity, "sourceIdentity"),
        configDigest: validHash(string(record.configDigest, "configDigest")),
        nativeLayoutDigest: validHash(
            string(record.nativeLayoutDigest, "nativeLayoutDigest"),
        ),
        protocolArtifactDigest: validHash(
            string(record.protocolArtifactDigest, "protocolArtifactDigest"),
        ),
        configOwners: sortedStrings(record.configOwners, "configOwners"),
        binaryDigests: binaryDigests(record.binaryDigests),
        files: fileEntries(record.files),
        agentProtocolContractId:
            record.agentProtocolContractId === undefined
                ? undefined
                : validHash(
                      string(
                          record.agentProtocolContractId,
                          "agentProtocolContractId",
                      ),
                  ),
    };
    if (base.releaseClass !== "production" && base.releaseClass !== "test")
        throw new Error("releaseClass is invalid");
    return base;
}
function validateClass(manifest: ReleaseManifest) {
    const paths = manifest.files.map((file) => file.path);
    const binaries = manifest.binaryDigests.map((digest) => digest.path);
    if (
        canonicalJson(
            paths.filter((path) => path.startsWith("bin/")).sort(),
        ) !== canonicalJson(binaries.slice().sort())
    )
        throw new Error("binary digests must exactly name binary files");
    for (const digest of manifest.binaryDigests) {
        const file = manifest.files.find((entry) => entry.path === digest.path);
        if (!file || file.sha256 !== digest.sha256)
            throw new Error("binary digest must match the named file bytes");
    }
    if (manifest.artifactClass === "node-agent") {
        if (
            canonicalJson(manifest.capabilities) !==
                canonicalJson(["monitor-agent"]) ||
            canonicalJson(manifest.services) !==
                canonicalJson(["monitor-agent"]) ||
            canonicalJson(binaries) !== canonicalJson(["bin/rz-monitor-agent"])
        )
            throw new Error(
                "node-agent manifest has server fields or binaries",
            );
        return;
    }
    if (
        !manifest.capabilities.includes("access") ||
        manifest.services.includes("monitor-agent") ||
        canonicalJson(binaries) !==
            canonicalJson(["bin/rz-admin", "bin/rz-monitor"])
    )
        throw new Error("server manifest has invalid selected inventory");
    if (
        manifest.capabilities.includes("monitor") !==
        Boolean(manifest.agentProtocolContractId)
    )
        throw new Error("server Monitor protocol pairing is invalid");
}
function validatePlan(manifest: ReleaseManifest, plan: Plan) {
    for (const key of [
        "preset",
        "artifactClass",
        "releaseClass",
        "target",
        "compositionId",
    ] as const)
        if (manifest[key] !== plan[key])
            throw new Error(`manifest ${key} differs from resolved selection`);
    if (
        canonicalJson(manifest.capabilities) !==
            canonicalJson(plan.capabilities) ||
        canonicalJson(manifest.services) !== canonicalJson(plan.services)
    )
        throw new Error(
            "manifest capabilities/services differ from resolved selection",
        );
    if (manifest.selectionDigest.sha256 !== sha256(canonicalJson(plan)))
        throw new Error(
            "manifest selectionDigest differs from resolved selection",
        );
    if (
        manifest.artifactClass === "server" &&
        (canonicalJson(Object.keys(manifest.schemaFingerprints).sort()) !==
            canonicalJson(plan.schemaOwners) ||
            canonicalJson(Object.keys(manifest.dataContractIds).sort()) !==
                canonicalJson(plan.schemaOwners) ||
            canonicalJson(manifest.configOwners) !==
                canonicalJson(plan.configOwners))
    )
        throw new Error(
            "manifest schema/data owners differ from resolved selection",
        );
}
function object(value: unknown, label: string): Record<string, unknown> {
    if (!value || typeof value !== "object" || Array.isArray(value))
        throw new Error(`${label} must be an object`);
    return value as Record<string, unknown>;
}
function onlyKeys(record: Record<string, unknown>, keys: string[]) {
    const allowed = new Set(keys);
    for (const key of Object.keys(record))
        if (!allowed.has(key))
            throw new Error(`unknown manifest field: ${key}`);
}
function required(value: unknown, label: string): string {
    if (value === undefined) throw new Error(`${label} is required`);
    return nonempty(value, label);
}
function digestRecord(
    value: unknown,
    source: DigestSource,
    label: string,
): Digest {
    const record = object(value, label);
    onlyKeys(
        record,
        source === "binary-file"
            ? ["path", "sha256", "source"]
            : ["sha256", "source"],
    );
    if (record.source !== source)
        throw new Error(`${label} requires ${source} source`);
    const path =
        source === "binary-file"
            ? checkedPath(nonempty(record.path, `${label}.path`))
            : undefined;
    return {
        sha256: validHash(nonempty(record.sha256, `${label}.sha256`)),
        source,
        ...(path ? { path } : {}),
    };
}
function binaryDigests(value: unknown): BinaryDigest[] {
    if (!Array.isArray(value) || !value.length)
        throw new Error("binaryDigests must be nonempty");
    const result = value.map((entry) =>
        digestRecord(entry, "binary-file", "binaryDigests"),
    );
    if (new Set(result.map((item) => item.path)).size !== result.length)
        throw new Error("binaryDigests repeat paths");
    return result as BinaryDigest[];
}
function hashMap(value: unknown, label: string): Record<string, string> {
    const record = object(value, label);
    const keys = Object.keys(record).sort();
    if (!keys.length) throw new Error(`${label} must not be empty`);
    for (const key of keys) {
        if (!key) throw new Error(`${label} has empty owner`);
        validHash(nonempty(record[key], `${label}.${key}`));
    }
    return Object.fromEntries(keys.map((key) => [key, record[key] as string]));
}
function fileEntries(value: unknown): FileEntry[] {
    if (!Array.isArray(value) || !value.length)
        throw new Error("files must be nonempty");
    const result = value
        .map((entry) => {
            const record = object(entry, "file");
            onlyKeys(record, ["path", "type", "mode", "size", "sha256"]);
            const path = checkedPath(nonempty(record.path, "file.path"));
            if (record.type !== "file")
                throw new Error("file type must be file");
            const mode = record.mode;
            if (path.startsWith("bin/") ? mode !== "0755" : mode !== "0644")
                throw new Error("file mode is invalid");
            if (
                typeof record.size !== "number" ||
                !Number.isSafeInteger(record.size) ||
                record.size < 0
            )
                throw new Error("file size is invalid");
            return {
                path,
                type: "file" as const,
                mode,
                size: record.size,
                sha256: validHash(nonempty(record.sha256, "file.sha256")),
            } as FileEntry;
        })
        .sort((left, right) =>
            left.path < right.path ? -1 : left.path > right.path ? 1 : 0,
        );
    if (
        new Set(result.map((entry) => entry.path)).size !== result.length ||
        canonicalJson(result) !== canonicalJson(value)
    )
        throw new Error("files must be sorted and unique");
    return result;
}

export const canonicalManifestBytes = (
    manifest: ReleaseManifest,
    expectedSelection: unknown,
): Uint8Array =>
    new TextEncoder().encode(
        canonicalJson(parseReleaseManifest(manifest, expectedSelection)),
    );

function string(value: unknown, label: string): string {
    if (typeof value !== "string") throw new Error(`${label} must be a string`);
    return value;
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
        path.split("/").some((part) => !part || part === "." || part === "..")
    )
        throw new Error("file path is not canonical POSIX");
    return path;
}
