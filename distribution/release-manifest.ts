import { resolveSelection } from "./resolver.ts";
import { parseReleaseManifest } from "./release-manifest-validator.ts";
import {
    canonicalJson,
    deriveBuildId,
    nonempty,
    selectionDigest,
    sha256,
    sortedStrings,
    validHash,
} from "./release-manifest-core.ts";
import {
    readArtifactFiles,
    readWebDigest,
} from "./release-manifest-artifacts.ts";
import type {
    AgentManifest,
    BinaryDigest,
    ManifestBase,
    BuildInputs,
    Digest,
    DigestSource,
    FileEntry,
    ProduceInput,
    ReleaseManifest,
    ServerManifest,
} from "./release-manifest-types.ts";
export type {
    AgentManifest,
    BinaryDigest,
    ManifestBase,
    BuildInputs,
    Digest,
    DigestSource,
    FileEntry,
    ProduceInput,
    ReleaseManifest,
    ServerManifest,
} from "./release-manifest-types.ts";
export {
    canonicalJson,
    deriveBuildId,
    filesDigest,
    selectionDigest,
    sha256,
} from "./release-manifest-core.ts";

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
    "apiDigest",
    "configDigest",
    "configOwners",
    "binaryDigests",
    "files",
    "agentProtocolContractId",
];
const serverKeys = [
    ...baseKeys,
    "schemaFingerprints",
    "dataContractIds",
    "webDigest",
];

export async function produceReleaseManifest(
    input: ProduceInput,
): Promise<ReleaseManifest> {
    const plan = resolveSelection(input.selection);
    const files = await readArtifactFiles(input.artifactRoot);
    const binaries = expectedBinaries(plan);
    const actualBinaries = files
        .filter((file) => file.path.startsWith("bin/"))
        .map((file) => file.path)
        .sort();
    if (canonicalJson(actualBinaries) !== canonicalJson(binaries))
        throw new Error(
            "artifact binary inventory does not match selected class",
        );
    const binaryDigests = binaries.map((path) => ({
        path,
        sha256: files.find((file) => file.path === path)!.sha256,
        source: "binary-file" as const,
    }));
    const buildInputs: BuildInputs = input;
    validateInputOwners(input, plan);
    const base: ManifestBase = {
        manifestVersion: 1,
        releaseClass: plan.releaseClass,
        releaseVersion: nonempty(input.releaseVersion, "releaseVersion"),
        target: plan.target,
        artifactClass: plan.artifactClass as "server" | "node-agent",
        preset: plan.preset,
        capabilities: plan.capabilities,
        services: plan.services,
        compositionId: plan.compositionId,
        selectionDigest: selectionDigest(input.selection),
        buildId: deriveBuildId(input.selection, buildInputs),
        sourceIdentity: nonempty(input.sourceIdentity, "sourceIdentity"),
        apiDigest: validHash(input.apiDigest),
        configDigest: validHash(input.configDigest),
        configOwners: plan.configOwners,
        binaryDigests,
        files,
    };
    if (plan.artifactClass === "node-agent") {
        const manifest = {
            ...base,
            artifactClass: "node-agent",
            agentProtocolContractId: validHash(
                required(input.protocolId, "protocolId"),
            ),
        } as ReleaseManifest;
        parseReleaseManifest(manifest, input.selection);
        return manifest;
    }
    if (!input.webRoot || !input.schemaFingerprints || !input.dataContractIds)
        throw new Error(
            "server producer requires selected Web, schema and data contracts",
        );
    const manifest: ReleaseManifest = {
        ...base,
        artifactClass: "server",
        agentProtocolContractId: plan.capabilities.includes("monitor")
            ? validHash(required(input.protocolId, "protocolId"))
            : undefined,
        schemaFingerprints: hashMap(
            input.schemaFingerprints,
            "schemaFingerprints",
        ),
        dataContractIds: hashMap(input.dataContractIds, "dataContractIds"),
        webDigest: await readWebDigest(input.webRoot),
    };
    parseReleaseManifest(manifest, input.selection);
    return manifest;
}

export {
    canonicalManifestBytes,
    parseReleaseManifest,
    validateServerAgentPair,
} from "./release-manifest-validator.ts";

type Plan = ReturnType<typeof resolveSelection>;
function expectedBinaries(plan: Plan): string[] {
    if (plan.artifactClass === "node-agent") return ["bin/rz-monitor-agent"];
    if (plan.preset === "monitor") return ["bin/rz-admin", "bin/rz-monitor"];
    throw new Error("producer supports only monitor server or node-agent");
}
function required(value: unknown, label: string): string {
    if (value === undefined) throw new Error(`${label} is required`);
    return nonempty(value, label);
}
function hashMap(value: unknown, label: string): Record<string, string> {
    if (!value || typeof value !== "object" || Array.isArray(value))
        throw new Error(`${label} must be an object`);
    const record = value as Record<string, unknown>;
    const keys = Object.keys(record).sort();
    if (!keys.length) throw new Error(`${label} must not be empty`);
    for (const key of keys) {
        if (!key) throw new Error(`${label} has empty owner`);
        validHash(nonempty(record[key], `${label}.${key}`));
    }
    return Object.fromEntries(keys.map((key) => [key, record[key] as string]));
}

function validateInputOwners(input: ProduceInput, plan: Plan): void {
    if (plan.artifactClass !== "server") return;
    const exact = (
        value: Record<string, string> | undefined,
        owners: string[],
        label: string,
    ) => {
        if (
            !value ||
            canonicalJson(Object.keys(value).sort()) !== canonicalJson(owners)
        )
            throw new Error(`${label} owners differ from resolved selection`);
    };
    exact(input.schemaFingerprints, plan.schemaOwners, "schema");
    exact(input.dataContractIds, plan.schemaOwners, "data");
}
